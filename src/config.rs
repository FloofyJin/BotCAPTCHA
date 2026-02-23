use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub challenge: ChallengeConfig,
    pub validation: ValidationConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// HMAC-SHA256 secret for signing tokens. Change in production.
    pub token_secret: String,
    /// Signed token lifetime in seconds.
    pub token_ttl_secs: u64,
    /// Registered site keys allowed to use this service.
    /// If empty, any sitekey (including none) is accepted (open/demo mode).
    pub site_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeConfig {
    pub word_count_min: usize,
    pub word_count_max: usize,
    pub duration_ms_min: u64,
    pub duration_ms_max: u64,
    pub word_pool: Vec<String>,
    pub grid_size: usize,
    pub grid_coords_min: usize,
    pub grid_coords_max: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub min_time_ms: u64,
    pub max_time_ms: u64,
    pub success_threshold: f32,
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

pub type SharedConfig = Arc<Config>;
