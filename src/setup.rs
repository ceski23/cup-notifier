use std::path::PathBuf;

use anyhow::Result;
use cup_notifier::Config;
use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use figment_file_provider_adapter::FileAdapter;
use tracing::{Level, info};
use tracing_subscriber::{EnvFilter, filter, fmt, prelude::*};

pub fn setup_logging() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?)
        .with(fmt::layer().compact())
        .with(filter::Targets::new().with_target("cup_notifier", Level::INFO))
        .init();

    info!("Logger setup");

    Ok(())
}

pub fn setup_config(config_path: &PathBuf) -> Result<Config> {
    let config: Config = Figment::new()
        .merge(FileAdapter::wrap(Yaml::file(config_path)))
        .merge(FileAdapter::wrap(Env::prefixed("CUP_NOTIFIER_")))
        .extract()?;

    Ok(config)
}
