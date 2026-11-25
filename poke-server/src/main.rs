use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use poke_common::{
    auth::Intent,
    comm::{ClientPacket, QueryResponse, ServerPacket},
    message::Message,
};

use crate::{
    server::{Client, broadcast, broadcast_to},
    verify::RegisteredUsers,
};

mod server;
mod verify;

struct ServerState {
    clients: Mutex<Vec<Client>>,
    registered: Mutex<RegisteredUsers>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:61601")?;

    let server_state = Arc::new(ServerState {
        clients: Mutex::new(Vec::new()),
        registered: Mutex::new(
            RegisteredUsers::read_from_disk()
                .unwrap()
                .unwrap_or_else(|| RegisteredUsers::new()),
        ),
    });

    let heartbeat_server_state = server_state.clone();

    thread::spawn(move || {
        heartbeat(heartbeat_server_state);
    });

    for incoming in listener.incoming() {
        let mut stream = incoming?;
        let thread_state = server_state.clone();

        thread::spawn(move || {
            let mut registered = thread_state.registered.lock().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());

            let Ok(auth) = verify::verify(&mut registered, &mut stream, &mut reader) else {
                eprintln!("auth failed");
                return;
            };

            drop(registered);

            let mut client = Client {
                username: auth.user,
                stream: stream.try_clone().unwrap(),
            };

            match auth.intent {
                Intent::Daemon => {
                    let mut clients = thread_state.clients.lock().unwrap();

                    clients.push(client.clone());
                }

                _ => (),
            }

            for packet_data in reader.split(0x00) {
                let packet: ClientPacket =
                    postcard::from_bytes_cobs(&mut packet_data.unwrap()).unwrap();

                match packet {
                    ClientPacket::Message(recipients, content) => {
                        let message = Message {
                            sender: client.username.clone(),
                            recipients,
                            content,
                        };

                        let mut clients = thread_state.clients.lock().unwrap();
                        match message.recipients.clone() {
                            poke_common::message::Recipients::All => {
                                broadcast(&mut clients, ServerPacket::Message(message)).unwrap();
                            }

                            poke_common::message::Recipients::List(recipients) => {
                                broadcast_to(
                                    recipients,
                                    &mut clients,
                                    ServerPacket::Message(message),
                                )
                                .unwrap();
                            }
                        }
                    }

                    ClientPacket::Query(query) => match query {
                        poke_common::comm::Query::ListClients => {
                            let clients = thread_state.clients.lock().unwrap();
                            client
                                .stream
                                .write_all(
                                    &postcard::to_allocvec_cobs(&QueryResponse::Clients(
                                        clients.iter().map(|c| c.username.clone()).collect(),
                                    ))
                                    .unwrap(),
                                )
                                .unwrap();
                        }
                    },
                }
            }
        });
    }

    Ok(())
}

fn heartbeat(server_state: Arc<ServerState>) {
    loop {
        let mut clients = server_state.clients.lock().unwrap();

        broadcast(&mut clients, ServerPacket::Heartbeat).unwrap();

        drop(clients);

        // TODO check if this keeps the tcp connection alive
        thread::sleep(Duration::from_secs(15));
    }
}
