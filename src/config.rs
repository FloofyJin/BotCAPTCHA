use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub challenge: ChallengeConfig,
    pub validation: ValidationConfig,
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
    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Create default configuration
    pub fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            challenge: ChallengeConfig {
                word_count_min: 150,
                word_count_max: 250,
                duration_ms_min: 4000,
                duration_ms_max: 6000,
                word_pool: vec![
                    "the", "and", "for", "are", "but", "not", "you", "all", "can",
                    "was", "one", "our", "out", "day", "get", "has", "him", "how",
                    "man", "new", "now", "old", "see", "two", "way", "who", "its",
                    "let", "put", "say", "she", "too", "use", "set", "run",
                    "data", "code", "node", "port", "host", "byte", "file", "path",
                    "loop", "list", "hash", "tree", "link", "type", "flag",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                grid_size: 10,
                grid_coords_min: 10,
                grid_coords_max: 15,
            },
            validation: ValidationConfig {
                min_time_ms: 0,
                max_time_ms: 10000,
                success_threshold: 0.8,
            },
        }
    }

    /// Save configuration to a TOML file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }
}

pub type SharedConfig = Arc<Config>;
