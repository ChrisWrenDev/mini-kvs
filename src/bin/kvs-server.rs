use clap::{Command, ValueEnum, arg, value_parser};
use kvs::{KvStore, KvsError, Result};
use std::env::current_dir;
use std::net::SocketAddr;
use std::process;
use std::str::FromStr;

const DEFAULT_ADDRESS: &str = "127.0.0.1:4000";

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
enum Engine {
    Kvs,
    Sled,
    Memory,
}

fn cli() -> Command {
    Command::new("kvs-servr")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            arg!(--addr <ADDR> "The server address")
                .value_parser(value_parser!(SocketAddr))
                .default_missing_value(DEFAULT_ADDRESS),
        )
        .arg(
            arg!(--engine <ENGINE> "The storage engine")
                .value_parser(value_parser!(Engine))
                .default_missing_value("kvs"),
        )
}

fn main() -> Result<()> {
    let matches = cli().get_matches();
    let addr = matches.get_one::<SocketAddr>("addr").expect("Required");
    let engine = matches.get_one::<Engine>("engine").expect("Required");

    println!("Using address: {}", addr);
    println!("Using engine: {:?}", engine);

    Ok(())
}
