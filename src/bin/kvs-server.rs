use clap::{Command, arg, value_parser};
use kvs::{Engine, Result, Server};
use std::net::SocketAddr;
use tracing::{Level, info};

const DEFAULT_ADDRESS: &str = "127.0.0.1:4000";

fn cli() -> Command {
    Command::new("kvs-server")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            arg!(--addr <ADDR> "The server address")
                .value_parser(value_parser!(SocketAddr))
                .default_value(DEFAULT_ADDRESS),
        )
        .arg(
            arg!(--engine <ENGINE> "The storage engine")
                .value_parser(value_parser!(Engine))
                .default_value("kvs"),
        )
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .pretty()
        .with_max_level(Level::DEBUG)
        .init();

    let matches = cli().get_matches();
    let addr = matches.get_one::<SocketAddr>("addr").expect("Required");
    let engine = matches.get_one::<Engine>("engine").expect("Required");

    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {}", engine.to_string());
    info!("Listening on {}", addr);

    Server::build(*addr, *engine)?.run()?;

    Ok(())
}
