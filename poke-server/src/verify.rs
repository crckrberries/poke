use std::{
    collections::HashMap,
    fs::{OpenOptions, create_dir_all},
    io::{BufRead, BufReader, Read, Write},
    mem::MaybeUninit,
    net::TcpStream,
    path::PathBuf,
};

use poke_common::auth::Auth;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisteredUsers {
    pub users: HashMap<String, String>,
}

impl RegisteredUsers {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn is_registered(&self, auth: &Auth) -> bool {
        self.users.contains_key(&auth.user)
    }

    pub fn check_auth(&self, auth: &Auth) -> bool {
        self.users
            .get(&auth.user)
            .is_some_and(|t| Some(t) == auth.token.as_ref())
    }

    fn path() -> PathBuf {
        let config_dir: PathBuf = dirs::config_dir().unwrap().join("poke-server");
        config_dir.join("registered_users.toml")
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

// TODO: This error is Big and Fatt and needs to be split up
#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("couldn't serialize user registry: {0}")]
    SerializeRegistry(#[from] toml::ser::Error),

    #[error("couldn't deserialize user registry: {0}")]
    DeserializeRegistry(#[from] toml::de::Error),

    #[error("couldn't deserialize auth packet: {0}")]
    Deserialize(#[from] postcard::Error),

    #[error("connection dropped while authenticating")]
    LostConnection,
}

pub fn verify(
    users: &mut RegisteredUsers,
    stream: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
) -> Result<Auth, Error> {
    for packet in reader.split(0x00) {
        let auth: Auth = postcard::from_bytes_cobs(&mut packet?)?;

        if users.check_auth(&auth) {
            return Ok(auth);
        }

        if !users.is_registered(&auth) {
            let mut token_buf: MaybeUninit<[u8; 36]> = MaybeUninit::uninit();
            let token = Uuid::new_v4()
                .hyphenated()
                .encode_lower_uninit(token_buf.as_mut())
                .to_owned();

            users.users.insert(auth.user, token.clone());
            users.write_to_disk()?;
            stream.write_all(&postcard::to_allocvec_cobs(&token)?)?;
        }
    }

    // TODO: this doesn't get called
    Err(Error::LostConnection)
}
