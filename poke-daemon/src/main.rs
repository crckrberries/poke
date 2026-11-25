use std::{
    env,
    io::{BufRead, BufReader},
    net::TcpStream,
};

use notify_rust::Notification;
use poke_client::config::Config;
use poke_common::comm::ServerPacket;
use poke_common::message::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_config = Config::read_from_disk()?;

    let Some(config) = maybe_config else {
        eprintln!("run `poke register` to create an account");
        return Ok(());
    };

    let mut stream = TcpStream::connect(format!(
        "{}:61601",
        env::var("POKE_SERVER").expect("set `POKE_SERVER` before using")
    ))?;

    poke_client::auth::authenticate(config, &mut stream, poke_common::auth::Intent::Daemon)?;

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
