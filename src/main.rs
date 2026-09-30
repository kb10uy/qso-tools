mod cli;
mod config;
mod qcgen;
mod qso;
mod schope;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::{
    cli::{Cli, Command},
    config::Config,
};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;
    match cli.command {
        Command::Qcgen(args) => qcgen::run(args, &config),
    }
}
