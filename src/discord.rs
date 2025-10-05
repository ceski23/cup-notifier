use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub embeds: Vec<Embed>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Embed {
    pub title: String,
    pub description: String,
    pub color: u32,
    pub url: Option<String>,
    pub thumbnail: Thumbnail,
}

#[derive(Debug, Serialize, Clone)]
pub struct Thumbnail {
    pub url: String,
}

pub async fn send_webhook(webhook_url: &String, payload: WebhookPayload) -> Result<()> {
    let client = reqwest::Client::new();

    client.post(webhook_url).json(&payload).send().await?;

    Ok(())
}
