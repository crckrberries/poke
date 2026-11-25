use std::{
    env,
    error::Error,
    io::{BufRead, BufReader, Write, stdin},
    net::TcpStream,
};

use clap::Parser;
use poke_client::{config::Config, creds::Credentials};
use poke_common::comm::{ClientPacket, Query, QueryResponse};

use crate::arg::Cli;

mod arg;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let maybe_config = Credentials::read_from_disk()?;

    let host = Config::read_from_disk()?.map(|c| c.server).flatten();

    let mut stream = TcpStream::connect(format!(
        "{}:61601",
        host.unwrap_or_else(|| env::var("POKE_SERVER")
            .expect("set `POKE_SERVER` or specify a server in config.toml before using"))
    ))?;

    let config = maybe_config.unwrap_or_else(|| {
        eprint!("username: ");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        Credentials {
            username: input.trim_suffix('\n').to_owned(),
            token: None,
        }
    });

    poke_client::creds::authenticate(config, &mut stream, poke_common::auth::Intent::Cli)?;

    let outgoing_packet = match cli.command {
        arg::Commands::Send { args } => &ClientPacket::Message(args.recipients, args.message),
        arg::Commands::List => &ClientPacket::Query(Query::ListClients),
        arg::Commands::Register => return Ok(()),
    };

    stream.write_all(&postcard::to_allocvec_cobs(outgoing_packet)?)?;
    stream.flush()?;

    if let ClientPacket::Query(_) = outgoing_packet {
        let res: QueryResponse =
            postcard::from_bytes_cobs(&mut BufReader::new(stream).split(0x00).next().unwrap()?)?;

        match res {
            QueryResponse::Clients(clients) => {
                println!("{}", clients.join(", "))
            }
        }
    }

    Ok(())
}
