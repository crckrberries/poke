use std::{
    fs::{OpenOptions, create_dir_all},
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

// TODO: this entire thing has like a lot in common with `Credentials`

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("couldn't deserialize file: {0}")]
    FileDeserialize(#[from] toml::de::Error),

    #[error("couldn't serialize config: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: Option<String>,
}

impl Config {
    fn path() -> PathBuf {
        let config_dir: PathBuf = dirs::config_dir().unwrap().join("poke");
        config_dir.join("config.toml")
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
