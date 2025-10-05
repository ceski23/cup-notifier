#![warn(clippy::pedantic)]

mod cup;
mod discord;
mod scheduler;
mod setup;

use anyhow::Result;
use clap::{Parser, command};
use std::path::PathBuf;
use std::process;
use tracing::{error, info};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long, value_name = "FILE", default_value = "config.yaml")]
    config: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup::setup_logging()?;

    let config = setup::setup_config(&args.config).unwrap_or_else(|err| {
        error!("Failed to load config. Error: {err}");
        process::exit(1);
    });

    info!("Resolved config: {:#?}", config);

    if let Err(err) = scheduler::start_scheduler(&config).await {
        error!("Failed to start notifications system. Error: {err}");
        process::exit(1)
    }

    Ok(())
}
