use std::{
    env,
    io::{BufRead, BufReader},
    net::TcpStream,
};

use notify_rust::Notification;
use poke_client::config::{Config};
use poke_client::creds::Credentials;
use poke_common::comm::ServerPacket;
use poke_common::message::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = Config::read_from_disk()?.map(|c| c.server).flatten();

    let mut stream = TcpStream::connect(format!(
        "{}:61601",
        host.unwrap_or_else(|| env::var("POKE_SERVER").expect("set `POKE_SERVER` or specify a server in config.toml before using"))
    ))?;

    let maybe_creds = Credentials::read_from_disk()?;

    let Some(creds) = maybe_creds else {
        eprintln!("run `poke register` to create an account");
        return Ok(());
    };

    poke_client::creds::authenticate(creds, &mut stream, poke_common::auth::Intent::Daemon)?;

    let reader = BufReader::new(stream);

    for packet_data in reader.split(0x00) {
        let packet: ServerPacket = postcard::from_bytes_cobs(&mut packet_data?)?;

        match packet {
            ServerPacket::Message(message) => {
                display_notification(&message);
            }

            ServerPacket::Heartbeat => (),
        }
    }

    Ok(())
}

fn display_notification(msg: &Message) {
    Notification::new()
        .summary(&msg.sender)
        .body(&msg.content)
        .show()
        .unwrap();
}
