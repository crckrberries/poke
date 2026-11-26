use std::{
    env,
    error::Error,
    io::{BufRead, BufReader, Write, stdin},
    net::TcpStream,
};

use clap::Parser;
use poke_client::{
    config::Config,
    creds::{self, AuthenticateError, Credentials},
};
use poke_common::{
    auth::Intent,
    comm::{ClientPacket, Query, QueryResponse},
};

use crate::arg::Cli;

mod arg;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let host = Config::read_from_disk()?.map(|c| c.server).flatten();

    let mut stream = TcpStream::connect(format!(
        "{}:61601",
        env::var("POKE_SERVER")
            .unwrap_or_else(|_| host
                .expect("set `POKE_SERVER` or specify a server in config.toml before using"))
    ))?;

    let maybe_creds = Credentials::read_from_disk()?;

    let creds = maybe_creds.unwrap_or_else(|| {
        eprint!("username: ");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        Credentials {
            username: input.trim_suffix('\n').to_owned(),
            token: None,
        }
    });

    if let Err(AuthenticateError::InvalidCredentials) =
        creds::authenticate(creds, &mut stream, Intent::Cli)
    {
        eprintln!("invalid credentials; contact the server owner for help");
        return Ok(());
    };

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
