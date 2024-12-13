use crate::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct RawArgs {
    /// Subcommand to run
    #[clap(subcommand)]
    command: Option<Command>,
    /// Enable debug logging
    #[clap(short, long, default_value = "false")]
    debug: bool,
}

#[derive(Debug)]
pub(crate) struct ParsedArgs {
    pub command: Command,
    pub log_level: tracing::Level,
}

impl ParsedArgs {
    pub fn parse_args() -> Self {
        let args: RawArgs = clap::Parser::parse();
        let log_level = if args.debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        };
        ParsedArgs {
            command: args.command.unwrap_or_default(),
            log_level,
        }
    }
}

#[derive(Subcommand, Default, Clone, Debug)]
pub enum Command {
    /// Start the application
    #[default]
    Start,
}
