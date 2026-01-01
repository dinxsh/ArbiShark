//! Configuration module for PolyShark
//! 
//! Loads settings from config.toml instead of hardcoded values.

use serde::Deserialize;
use std::fs;

/// Root configuration structure
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub permission: PermissionConfig,
    pub trading: TradingConfig,
    pub timing: TimingConfig,
    pub api: ApiConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PermissionConfig {
    pub daily_limit_usdc: f64,
    pub duration_days: u32,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TradingConfig {
    pub min_spread_threshold: f64,
    pub min_profit_threshold: f64,
    pub trade_size: f64,
    pub max_position_value: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TimingConfig {
    pub poll_interval_secs: u64,
    pub position_timeout_secs: u64,
    pub latency_base_ms: u64,
    pub adverse_selection_std: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub gamma_url: String,
    pub clob_url: String,
    pub websocket_url: String,
    pub market_limit: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub colorize: bool,
}

impl Config {
    /// Load configuration from config.toml
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from("config.toml")
    }

    /// Load configuration from a specific file
    pub fn load_from(path: &str) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError::FileNotFound(path.to_string(), e.to_string()))?;
        
        toml::from_str(&contents)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Create default configuration
    pub fn default_config() -> Self {
        Self {
            permission: PermissionConfig {
                daily_limit_usdc: 10.0,
                duration_days: 30,
                token: "USDC".to_string(),
            },
            trading: TradingConfig {
                min_spread_threshold: 0.02,
                min_profit_threshold: 0.10,
                trade_size: 5.0,
                max_position_value: 50.0,
            },
            timing: TimingConfig {
                poll_interval_secs: 5,
                position_timeout_secs: 3600,
                latency_base_ms: 50,
                adverse_selection_std: 0.001,
            },
            api: ApiConfig {
                gamma_url: "https://gamma-api.polymarket.com/events".to_string(),
                clob_url: "https://clob.polymarket.com".to_string(),
                websocket_url: "wss://ws-subscriptions-clob.polymarket.com/ws".to_string(),
                market_limit: 20,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                colorize: true,
            },
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => write!(f, "Config file not found: {} ({})", path, err),
            Self::ParseError(err) => write!(f, "Config parse error: {}", err),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default_config();
        assert_eq!(config.permission.daily_limit_usdc, 10.0);
        assert_eq!(config.trading.min_spread_threshold, 0.02);
    }
}
