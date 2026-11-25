use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use poke_common::auth::{Auth, Intent};
use thiserror::Error;

use crate::config::{self, Config};

#[derive(Debug, Error)]
pub enum AuthenticateError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("couldn't serialize: {0}")]
    SerializationError(#[from] postcard::Error),

    #[error("config write error: {0}")]
    ConfigWriteError(#[from] config::Error),
}

pub fn authenticate(
    mut config: Config,
    stream: &mut TcpStream,
    intent: Intent,
) -> Result<(), AuthenticateError> {
    if config.token.is_none() {
        stream.write_all(&postcard::to_allocvec_cobs(&Auth {
            user: config.username.clone(),
            token: None,
            intent,
        })?)?;

        let token = postcard::from_bytes_cobs(
            &mut BufReader::new(&mut *stream).split(0x00).next().unwrap()?,
        )?;

        config.token = Some(token);

        config.write_to_disk()?;
    };

    stream.write_all(&postcard::to_allocvec_cobs(&Auth {
        user: config.username.clone(),
        token: config.token,
        intent,
    })?)?;

    stream.flush()?;

    Ok(())
}
