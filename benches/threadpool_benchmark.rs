use clap::ValueEnum;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use kvs::{
    ClientSync, ClientTraitSync, Engine, PoolType, RayonThreadPool, Request, Response, Server,
    ServerTrait, ThreadPoolTrait,
};
use once_cell::sync::Lazy;
use rand::{Rng, thread_rng};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

const NUM_VALS: usize = 1000;

const ASCII_START: u8 = 33;
const ASCII_END: u8 = 127;

// NOTE: benchmark failing at 16 threads
// fn generate_sequence() -> Vec<usize> {
//     // Change to num_cpus::get_physical() for physical cores
//     let max = 2 * num_cpus::get();
//     let mut sequence = Vec::new();
//     let mut val = 1;
//
//     while val <= max {
//         sequence.push(val);
//         val *= 2;
//     }
//
//     sequence
// }

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
    let thread_counts = [1, 2, 4, 8];
    for engine in Engine::value_variants() {
        for pool in PoolType::value_variants() {
            let mut bench_write = c.benchmark_group("bench_write");
            for num_threads in thread_counts.iter() {
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

                // Wait for server
                loop {
                    if TcpStream::connect(addr).is_ok() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }

                // Setup Clients
                let client_pool = RayonThreadPool::new(NUM_VALS as u32).unwrap();

                let write_id = format!("{}-{}-{}-write", engine, pool, num_threads);

                let total_bytes = NUM_VALS * (KEYS[0].len() + VALS[0].len());
                bench_write.throughput(Throughput::Bytes(total_bytes as u64));
                bench_write.sample_size(10);
                // bench_write.measurement_time(Duration::from_secs(500));
                bench_write.bench_with_input(&write_id, &num_threads, |b, _num_threads| {
                    b.iter(|| {
                        // Measured work goes here
                        let (done_tx, done_rx) = std::sync::mpsc::channel();

                        for i in 0..NUM_VALS {
                            let key = KEYS[i].clone();
                            let value = VALS[i].clone();
                            let done_tx = done_tx.clone();

                            client_pool.spawn(move || {
                                let mut client = ClientSync::connect(addr).unwrap();
                                let request = Request::Set { key, value };
                                let res = client.send(request);
                                if let Err(e) = &res {
                                    eprintln!("Client send failed: {:?}", e);
                                }
                                assert!(res.is_ok());

                                // Notify completion
                                done_tx.send(()).unwrap()
                            });
                        }

                        // Wait for all jobs to finish
                        for _ in 0..NUM_VALS {
                            // for _ in 0..clients.len() {
                            done_rx.recv().unwrap();
                        }
                    });
                });
                // Server shutdown
                server_shutdown.send(()).unwrap();
                ClientSync::connect(addr).unwrap();
                server_handle.join().expect("Server thread panicked");
                println!("Finished: {}", &write_id);
            }
        }
    }
}

fn bench_read(c: &mut Criterion) {
    println!("Running bench_read");
    let thread_counts = [1, 2, 4, 8];
    for engine in Engine::value_variants() {
        for pool in PoolType::value_variants() {
            let mut bench_read = c.benchmark_group("bench_read");
            for num_threads in thread_counts.iter() {
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

                // Prepolulate storage
                let mut clients = Vec::with_capacity(NUM_VALS);
                for i in 0..NUM_VALS {
                    let mut client = ClientSync::connect(addr).unwrap();
                    let key = KEYS[i].clone();
                    let value = VALS[i].clone();
                    let request: Request = Request::Set { key, value };
                    let res = client.send(request);
                    assert!(res.is_ok());

                    clients.push(client);
                }

                // Setup Clients
                let client_pool = RayonThreadPool::new(NUM_VALS as u32).unwrap();

                let read_id = format!("{}-{}-{}-read", engine, pool, num_threads);

                let total_bytes = NUM_VALS * KEYS[0].len();
                bench_read.throughput(Throughput::Bytes(total_bytes as u64));
                bench_read.sample_size(10);
                // bench_read.measurement_time(Duration::from_secs(60));
                bench_read.bench_with_input(&read_id, num_threads, |b, _num_threads| {
                    b.iter(|| {
                        // Measured work goes here
                        let (done_tx, done_rx) = std::sync::mpsc::channel();

                        for i in 0..NUM_VALS {
                            let key = KEYS[i].clone();
                            let value = VALS[i].clone();
                            let done_tx = done_tx.clone();

                            client_pool.spawn(move || {
                                let mut client = ClientSync::connect(addr).unwrap();
                                let request = Request::Get { key };
                                let res = client.send(request);

                                match res {
                                    Ok(val) => {
                                        if let Response::Value(v) = val {
                                            assert_eq!(value, v);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Client send failed: {:?}", e);
                                    }
                                }

                                // Notify completion
                                done_tx.send(()).unwrap()
                            });
                        }
                        // Wait for all jobs to finish
                        for _ in 0..NUM_VALS {
                            // for _ in 0..clients.len() {
                            done_rx.recv().unwrap();
                        }
                    });
                });

                // Server shutdown
                server_shutdown.send(()).unwrap();
                ClientSync::connect(addr).unwrap();
                server_handle.join().unwrap();
                println!("Finished: {}", &read_id);
            }
        }
    }
}

criterion_group!(threadpool, bench_write, bench_read);
criterion_main!(threadpool);
