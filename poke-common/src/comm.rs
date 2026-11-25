use serde::{Deserialize, Serialize};

use crate::message::{Message, Recipients};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientPacket {
    Message(Recipients, String),
    Query(Query),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Query {
    ListClients,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerPacket {
    Message(Message),
    Heartbeat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QueryResponse {
    Clients(Vec<String>),
}

// pub fn parse<T: for<'a> Deserialize<'a>>(buf: Vec<u8>) -> Result<T, Error> {
// 	let mut packet_data = buf;
// 	packet_data.push(0);
// 	dbg!(&packet_data);
// 	postcard::from_bytes_cobs(&mut packet_data)
// }
