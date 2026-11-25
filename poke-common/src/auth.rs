use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Auth {
    pub user: String,
    pub token: Option<String>,
    pub intent: Intent,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Intent {
    Cli,
    Daemon,
}
