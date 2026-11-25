use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Recipients {
    All,
    List(Vec<String>),
}

impl FromStr for Recipients {
    type Err = !;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "@all" => Ok(Self::All),
            _ => Ok(Self::List(
                s.split(',').map(|r| r.trim().to_owned()).collect(),
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub sender: String,
    pub recipients: Recipients,
    pub content: String,
}
