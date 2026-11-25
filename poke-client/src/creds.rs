use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpStream;
use std::{
    fs::{OpenOptions, create_dir_all},
    io::{Read, Write},
    path::PathBuf,
};

use poke_common::auth::Auth;
use poke_common::auth::Intent;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub token: Option<String>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("couldn't deserialize file: {0}")]
    FileDeserialize(#[from] toml::de::Error),

    #[error("couldn't serialize config: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),
}

impl Credentials {
    fn path() -> PathBuf {
        let config_dir: PathBuf = dirs::config_dir().unwrap().join("poke");
        config_dir.join("creds.toml")
    }

    pub fn read_from_disk() -> Result<Option<Self>, Error> {
        let Ok(mut file) = OpenOptions::new().read(true).open(Self::path()) else {
            return Ok(None);
        };

        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents)?;

        Ok(Some(toml::from_str::<Self>(&file_contents)?))
    }

    pub fn write_to_disk(&self) -> Result<(), Error> {
        create_dir_all(Self::path().parent().unwrap())?;
        let mut file = OpenOptions::new()
            .truncate(false)
            .create(true)
            .read(true)
            .write(true)
            .open(Self::path())?;

        file.write_all(toml::to_string_pretty(self)?.as_bytes())?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum AuthenticateError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("couldn't serialize: {0}")]
    SerializationError(#[from] postcard::Error),

    #[error("config write error: {0}")]
    ConfigWriteError(#[from] Error),
}

pub fn authenticate(
    mut config: Credentials,
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
