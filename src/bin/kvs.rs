use clap::{Command, arg};
use kvs::{KvStore, KvsError, Result};
use std::env::current_dir;
use std::process;

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
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
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("get")
                .about("Retrieve a value from the store using a key")
                .arg(arg!(<KEY> "The key in the store"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("rm")
                .about("Remove a key/value pair from store")
                .arg(arg!(<KEY> "The key in the store"))
                .arg_required_else_help(true),
        )
}

fn main() -> Result<()> {
    // Build log
    let path_dir = current_dir().unwrap();
    let mut store = KvStore::open(&path_dir)?;

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("set", matches)) => {
            let key = matches.get_one::<String>("KEY").expect("Required");
            let value = matches.get_one::<String>("VALUE").expect("Required");

            store.set(key.to_owned(), value.to_owned())?;
        }
        Some(("get", matches)) => {
            let key = matches.get_one::<String>("KEY").expect("Required");
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
