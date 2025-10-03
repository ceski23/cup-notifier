use anyhow::{Result, bail};
use cup_notifier::Config;
use reqwest::Url;
use serde::Deserialize;
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Root {
    pub images: Vec<Image>,
    pub last_updated: String,
    pub metrics: Metrics,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Image {
    pub in_use: bool,
    pub parts: Parts,
    pub reference: String,
    pub result: ImageResult,
    pub server: Option<String>,
    pub time: i64,
    pub url: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Parts {
    pub registry: String,
    pub repository: String,
    pub tag: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ImageResult {
    pub error: Option<String>,
    pub has_update: bool,
    pub info: Option<Info>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Info {
    #[serde(rename = "version")]
    Version {
        current_version: String,
        new_tag: String,
        new_version: String,
        version_update_type: String,
    },
    #[serde(rename = "digest")]
    Digest {
        local_digests: Vec<String>,
        remote_digest: String,
    },
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Metrics {
    pub major_updates: u32,
    pub minor_updates: u32,
    pub monitored_images: u32,
    pub other_updates: u32,
    pub patch_updates: u32,
    pub unknown: u32,
    pub up_to_date: u32,
    pub updates_available: u32,
}

pub async fn fetch_fresh_data(config: &Config) -> Result<Root> {
    info!("Fetching fresh images data...");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    let base_url = Url::parse(&config.cup_base_url)?;

    let refresh_response = client
        .get(base_url.join("api/v3/refresh")?)
        .send()
        .await?
        .text()
        .await?;

    if refresh_response != "OK" {
        bail!("Refresh failed")
    }

    let root = client
        .get(base_url.join("api/v3/json")?)
        .send()
        .await?
        .json::<Root>()
        .await?;

    Ok(root)
}
