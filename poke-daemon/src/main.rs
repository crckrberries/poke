use std::{
    env,
    io::{BufRead, BufReader},
    net::TcpStream,
    thread,
    time::Duration,
};

#[cfg(not(target_os = "macos"))]
use notify_rust::Notification;
use poke_client::creds::Credentials;
use poke_client::{
    config::Config,
    creds::{self, AuthenticateError},
};
use poke_common::message::Message;
use poke_common::{auth::Intent, comm::ServerPacket};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = Config::read_from_disk()?.map(|c| c.server).flatten();

    let maybe_creds = Credentials::read_from_disk()?;

    let Some(creds) = maybe_creds else {
        eprintln!("run `poke register` to create an account");
        return Ok(());
    };

    loop {
        if let Ok(mut stream) = TcpStream::connect(format!(
            "{}:61601",
            env::var("POKE_SERVER").unwrap_or_else(|_| host
                .clone()
                .expect("set `POKE_SERVER` or specify a server in config.toml before using"))
        )) {
            println!("connected to server");

            if let Err(AuthenticateError::InvalidCredentials) =
                creds::authenticate(creds.clone(), &mut stream, Intent::Daemon)
            {
                eprintln!("invalid credentials; contact the server owner for help");
                return Ok(());
            };

            println!("authenticated");

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

            println!("connection dropped, retrying in a bit...");
        } else {
            println!("couldn't connect, retrying in a bit...")
        };

        thread::sleep(Duration::from_secs(15));
    }
}

#[cfg(target_os = "macos")]
fn display_notification(msg: &Message) {
    use std::process::Command;

    let script = r#"
		on run { title, content }
			display notification content with title title
		end run
	"#;

    Command::new("osascript")
        .args(["-e", script, "--", &msg.sender, &msg.content])
        .spawn()
        .unwrap();
}

#[cfg(not(target_os = "macos"))]
fn display_notification(msg: &Message) {
    Notification::new()
        .summary(&msg.sender)
        .body(&msg.content)
        .show()
        .unwrap();
}
