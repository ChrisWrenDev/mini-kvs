use clap::{Command, arg, value_parser};
use kvs::{KvStore, KvsError, Result, init_logging};
use std::env::current_dir;
use std::net::SocketAddr;
use std::process;

const DEFAULT_ADDRESS: &str = "127.0.0.1:4000";

fn cli() -> Command {
    Command::new("kvs-client")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("set")
                .about("Add a key/value to the store")
                .arg(arg!(<KEY> "The key added to store"))
                .arg(arg!(<VALUE> "The value added to store"))
                .arg(
                    arg!(--addr <ADDR> "The server address")
                        .value_parser(value_parser!(SocketAddr))
                        .num_args(1)
                        .require_equals(true)
                        .default_missing_value(DEFAULT_ADDRESS),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("get")
                .about("Retrieve a value from the store using a key")
                .arg(arg!(<KEY> "The key in the store"))
                .arg(
                    arg!(--addr <ADDR> "The server address")
                        .value_parser(value_parser!(SocketAddr))
                        .num_args(1)
                        .require_equals(true)
                        .default_missing_value(DEFAULT_ADDRESS),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("rm")
                .about("Remove a key/value pair from store")
                .arg(arg!(<KEY> "The key in the store"))
                .arg(
                    arg!(--addr <ADDR> "The server address")
                        .value_parser(value_parser!(SocketAddr))
                        .num_args(1)
                        .require_equals(true)
                        .default_missing_value(DEFAULT_ADDRESS),
                )
                .arg_required_else_help(true),
        )
}

fn main() -> Result<()> {
    init_logging();

    // Build log
    let path_dir = current_dir().map_err(|_| KvsError::FileNotFound)?;
    let mut store = KvStore::open(&path_dir)?;

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("set", matches)) => {
            let key = matches.get_one::<String>("KEY").expect("Required");
            let value = matches.get_one::<String>("VALUE").expect("Required");
            let addr = matches.get_one::<SocketAddr>("ADDR").expect("Required");

            store.set(key.to_owned(), value.to_owned())?;
        }
        Some(("get", matches)) => {
            let key = matches.get_one::<String>("KEY").expect("Required");
            let addr = matches.get_one::<SocketAddr>("ADDR").expect("Required");

            let value = store.get(key.to_string());
            match value {
                Ok(Some(val)) => println!("{}", val),
                Ok(None) | Err(_) => {
                    println!("Key not found");
                }
            }
        }
        Some(("rm", matches)) => {
            let key = matches.get_one::<String>("KEY").expect("Required");
            let addr = matches.get_one::<SocketAddr>("ADDR").expect("Required");

            match store.remove(key.clone()) {
                Ok(()) => {}
                Err(KvsError::KeyNotFound) => {
                    println!("Key not found");
                    process::exit(1);
                }
                Err(e) => return Err(e),
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
