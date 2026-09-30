mod cli;
mod commands;
mod core;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::{
    cli::{Cli, Command},
    commands::{jcx, qcgen},
    core::config::Config,
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
        Command::Jcx(args) => jcx::run(args, &config),
    }
}
