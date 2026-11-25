use std::{io::Write, net::TcpStream};

use serde::Serialize;
use thiserror::Error;

pub struct Client {
    pub username: String,
    pub stream: TcpStream,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            username: self.username.clone(),
            stream: self.stream.try_clone().unwrap(),
        }
    }
}

#[derive(Debug, Error)]
pub enum BroadcastError {
    #[error("couldn't serialize: {0}")]
    SerializationError(#[from] postcard::Error),
}

pub fn broadcast_to<T: Serialize>(
    usernames: Vec<String>,
    clients: &mut Vec<Client>,
    content: T,
) -> Result<(), BroadcastError> {
    let buf = &postcard::to_allocvec_cobs(&content)?;

    clients.retain_mut(|c| {
        !usernames.contains(&c.username)
            || (usernames.contains(&c.username) && c.stream.write(buf).is_ok())
    });
    Ok(())
}

pub fn broadcast<T: Serialize>(
    clients: &mut Vec<Client>,
    content: T,
) -> Result<(), BroadcastError> {
    let buf = &postcard::to_allocvec_cobs(&content)?;

    clients.retain_mut(|c| c.stream.write(buf).is_ok());
    Ok(())
}
