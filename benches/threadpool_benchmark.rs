use clap::ValueEnum;
use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use kvs::{Client, ClientTrait, Engine, PoolType, Request, Response, Server, ServerTrait};
use once_cell::sync::Lazy;
use rand::{Rng, thread_rng};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tempfile::TempDir;

const NUM_VALS: usize = 1000;

const ASCII_START: u8 = 33;
const ASCII_END: u8 = 127;

fn generate_sequence() -> Vec<usize> {
    // Change to num_cpus::get_physical() for physical cores
    let max = 2 * num_cpus::get();
    let mut sequence = Vec::new();
    let mut val = 1;

    while val <= max {
        sequence.push(val);
        val *= 2;
    }

    sequence
}

fn get_vals(len: usize) -> Vec<String> {
    let mut vals = Vec::with_capacity(NUM_VALS);
    for _ in 0..NUM_VALS {
        let mut rng = thread_rng();
        let val: String = (0..len)
            .map(|_| rng.gen_range(ASCII_START, ASCII_END) as char)
            .collect();

        vals.push(val);
    }

    vals
}

static KEYS: Lazy<Vec<String>> = Lazy::new(|| get_vals(10));
static VALS: Lazy<Vec<String>> = Lazy::new(|| get_vals(10));

fn bench_write(c: &mut Criterion) {
    println!("Running bench_write");
    for engine in Engine::value_variants() {
        for pool in PoolType::value_variants() {
            let mut bench_write = c.benchmark_group("bench_write");
            for num_threads in generate_sequence().iter() {
                let base_port = 8800;
                let port = base_port + (*num_threads as u16);
                let addr: SocketAddr = format!("127.0.0.1:{}", port)
                    .parse()
                    .expect("Unable to parse socket address");

                // Setup Server
                let tempdir = TempDir::new_in("/tmp").unwrap();
                let unique_path = tempdir.path().join(engine.to_string());
                std::fs::create_dir_all(&unique_path).unwrap();

                let mut server =
                    Server::build(addr, *engine, *pool, *num_threads as u32, unique_path).unwrap();
                let server_shutdown = server.shutdown();
                let server_handle = std::thread::spawn(move || {
                    server.run().unwrap();
                });

                loop {
                    if TcpStream::connect(addr).is_ok() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }

                let write_id = format!("{}-{}-{}-write", engine, pool, num_threads);

                let total_bytes = NUM_VALS * (KEYS[0].len() + VALS[0].len());
                bench_write.throughput(Throughput::Bytes(total_bytes as u64));
                bench_write.sample_size(10);
                bench_write.measurement_time(Duration::from_secs(30));
                bench_write.bench_with_input(write_id, num_threads, |b, _num_threads| {
                    b.iter_batched(
                        || {
                            // Setup work goes here
                            let mut clients = Vec::with_capacity(NUM_VALS);
                            for _ in 0..NUM_VALS {
                                clients.push(Arc::new(Mutex::new(Client::connect(addr).unwrap())));
                            }

                            clients
                        },
                        |mut clients| {
                            // Measured work goes here

                            let mut client_handles: Vec<JoinHandle<()>> = Vec::new();

                            for (i, client) in clients.iter_mut().enumerate() {
                                let key = KEYS[i].clone();
                                let value = VALS[i].clone();
                                let client = Arc::clone(client);

                                let handle = thread::spawn(move || {
                                    let request = Request::Set { key, value };
                                    let res = client.lock().unwrap().send(request);
                                    assert!(res.is_ok());
                                });

                                client_handles.push(handle);
                            }

                            for client in client_handles {
                                client.join().expect("Client thread panicked");
                            }
                        },
                        BatchSize::LargeInput,
                    );
                });
                // Server shutdown
                server_shutdown.send(()).unwrap();
                server_handle.join().expect("Server thread panicked");
            }
        }
    }
}

fn bench_read(c: &mut Criterion) {
    println!("Running bench_read");
    for engine in Engine::value_variants() {
        for pool in PoolType::value_variants() {
            let mut bench_read = c.benchmark_group("bench_read");
            for num_threads in generate_sequence().iter() {
                let base_port = 8800;
                let port = base_port + (*num_threads as u16);
                let addr: SocketAddr = format!("127.0.0.1:{}", port)
                    .parse()
                    .expect("Unable to parse socket address");

                // Setup Server
                let tempdir = TempDir::new_in("/tmp").unwrap();
                let unique_path = tempdir.path().join(engine.to_string());
                std::fs::create_dir_all(&unique_path).unwrap();

                let mut server =
                    Server::build(addr, *engine, *pool, *num_threads as u32, unique_path).unwrap();
                let server_shutdown = server.shutdown();
                let server_handle = std::thread::spawn(move || {
                    server.run().unwrap();
                });

                loop {
                    if TcpStream::connect(addr).is_ok() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }

                let read_id = format!("{}-{}-{}-read", engine, pool, num_threads);

                let total_bytes = NUM_VALS * KEYS[0].len();
                bench_read.throughput(Throughput::Bytes(total_bytes as u64));
                bench_read.sample_size(10);
                bench_read.measurement_time(Duration::from_secs(30));
                bench_read.bench_with_input(read_id, num_threads, |b, _num_threads| {
                    b.iter_batched(
                        || {
                            let mut clients = Vec::with_capacity(NUM_VALS);
                            for i in 0..NUM_VALS {
                                let mut client = Client::connect(addr).unwrap();
                                let key = KEYS[i].clone();
                                let value = VALS[i].clone();
                                let request: Request = Request::Set { key, value };
                                let res = client.send(request);
                                assert!(res.is_ok());

                                clients.push(client);
                            }

                            clients
                        },
                        |mut clients| {
                            // Measured work goes here
                            for (i, client) in clients.iter_mut().enumerate() {
                                let key = KEYS[i].clone();
                                let value = VALS[i].clone();
                                let request: Request = Request::Get { key };
                                let res = client.send(request).unwrap();

                                if let Response::Value(v) = res {
                                    assert_eq!(value, v);
                                }
                            }
                        },
                        BatchSize::LargeInput,
                    );
                });

                // Server shutdown
                server_shutdown.send(()).unwrap();
                server_handle.join().expect("Server thread panicked");
            }
        }
    }
}

criterion_group!(threadpool, bench_write, bench_read);
criterion_main!(threadpool);
