use std::str::FromStr;

use clap::{Args, Parser, Subcommand};
use poke_common::message::Recipients;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Subcommand)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    /// Send a message to a set of recipients
    Send {
        #[command(flatten)]
        args: SendArgs,
    },

    /// List all clients currently connected
    List,

    /// Register an account on the server
    Register,
}

#[derive(Debug, Args, Clone)]
pub struct SendArgs {
    #[arg(value_parser = Recipients::from_str)]
    pub recipients: Recipients,

    pub message: String,
}
