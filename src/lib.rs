use schemars::JsonSchema;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(JsonSchema, Deserialize, Debug, Clone)]
pub struct Config {
    /// Discord's webhook URL to which send request
    pub webhook_url: String,

    /// Base URL of Cup's instance
    pub cup_base_url: String,

    /// Cron pattern to use
    #[serde_inline_default("0 0 0 * * *".to_string())]
    pub cron: String,
}
