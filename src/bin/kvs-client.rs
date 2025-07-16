use clap::{Command, arg, value_parser};
use kvs::{Client, Request, Response, Result};
use std::net::SocketAddr;
use std::process::exit;
use tracing::Level;

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
                        .default_value(DEFAULT_ADDRESS),
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
                        .default_value(DEFAULT_ADDRESS),
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
                        .default_value(DEFAULT_ADDRESS),
                )
                .arg_required_else_help(true),
        )
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .pretty()
        .with_max_level(Level::DEBUG)
        .init();

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("set", matches)) => {
            let key = matches.get_one::<String>("KEY").expect("Required");
            let value = matches.get_one::<String>("VALUE").expect("Required");
            let addr = matches.get_one::<SocketAddr>("addr").expect("Required");

            let mut client = Client::connect(*addr)?;
            let request: Request = Request::Set {
                key: key.clone(),
                value: value.clone(),
            };
            let response = client.send(request)?;

            match response {
                Response::Value(s) => println!("{}", s),
                Response::Ok => {}
                Response::NotFound => {
                    eprintln!("Key not found");
                    exit(1);
                }
                Response::Error(err) => {
                    eprintln!("Error: {}", err);
                    exit(1);
                }
            }
        }
        Some(("get", matches)) => {
            let key = matches.get_one::<String>("KEY").expect("Required");
            let addr = matches.get_one::<SocketAddr>("addr").expect("Required");

            let mut client = Client::connect(*addr)?;
            let request: Request = Request::Get { key: key.clone() };

            let response = client.send(request)?;

            match response {
                Response::Value(s) => println!("{}", s),
                Response::Ok => {}
                Response::NotFound => {
                    println!("Key not found");
                }
                Response::Error(err) => {
                    eprintln!("Error: {}", err);
                    exit(1);
                }
            }
        }
        Some(("rm", matches)) => {
            let key = matches.get_one::<String>("KEY").expect("Required");
            let addr = matches.get_one::<SocketAddr>("addr").expect("Required");

            let mut client = Client::connect(*addr)?;
            let request: Request = Request::Remove { key: key.clone() };

            let response = client.send(request)?;

            match response {
                Response::Value(s) => println!("{}", s),
                Response::Ok => {}
                Response::NotFound => {
                    eprintln!("Key not found");
                    exit(1);
                }
                Response::Error(err) => {
                    eprintln!("Error: {}", err);
                    exit(1);
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
