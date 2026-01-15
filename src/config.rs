//! Configuration module for ArbiShark
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
    #[serde(default)]
    pub strategy: StrategyConfig,
    #[serde(default)]
    pub safety: SafetyConfig,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub arbitrum: Option<ArbitrumConfig>,
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

/// Strategy configuration for adaptive trading
#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    /// Threshold for conservative mode (below this %)
    pub conservative_threshold: f64,
    /// Threshold for aggressive mode (above this %)
    pub aggressive_threshold: f64,
    /// Minimum edge required in conservative mode
    pub conservative_min_edge: f64,
    /// Minimum edge required in normal mode
    pub normal_min_edge: f64,
    /// Minimum edge required in aggressive mode
    pub aggressive_min_edge: f64,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            conservative_threshold: 0.30,
            aggressive_threshold: 0.70,
            conservative_min_edge: 0.05,
            normal_min_edge: 0.02,
            aggressive_min_edge: 0.01,
        }
    }
}

/// Safety configuration for failure handling
#[derive(Debug, Deserialize, Clone)]
pub struct SafetyConfig {
    /// Maximum data delay (ms) before suspending trading
    pub max_data_delay_ms: u64,
    /// Maximum consecutive API failures before safe mode
    pub max_consecutive_failures: u32,
    /// Cooldown period (seconds) in safe mode
    pub safe_mode_cooldown_secs: u64,
    /// Assume zero allowance if permission query fails
    pub assume_zero_on_perm_error: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            max_data_delay_ms: 5000,
            max_consecutive_failures: 3,
            safe_mode_cooldown_secs: 300,
            assume_zero_on_perm_error: true,
        }
    }
}

/// Arbitrum network configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ArbitrumConfig {
    pub sepolia_rpc: String,
    pub mainnet_rpc: String,
    pub sepolia_chain_id: u64,
    pub mainnet_chain_id: u64,
    pub envio_endpoint: String,
    pub usdc_e_address: String,
    pub demo_contract_address: String,
}

impl Default for ArbitrumConfig {
    fn default() -> Self {
        Self {
            sepolia_rpc: "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
            mainnet_rpc: "https://arb1.arbitrum.io/rpc".to_string(),
            sepolia_chain_id: 421614,
            mainnet_chain_id: 42161,
            envio_endpoint: "https://indexer.bigdevenergy.link/your-project/v1/graphql".to_string(),
            usdc_e_address: "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8".to_string(),
            demo_contract_address: "0x0000000000000000000000000000000000000000".to_string(),
        }
    }
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
            strategy: StrategyConfig::default(),
            safety: SafetyConfig::default(),
            mode: Some("arbitrum_demo".to_string()),
            arbitrum: Some(ArbitrumConfig::default()),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.permission.daily_limit_usdc <= 0.0 {
            return Err("daily_limit_usdc must be positive".to_string());
        }
        
        if let Some(mode) = &self.mode {
            if mode == "arbitrum_demo" && self.arbitrum.is_none() {
                return Err("arbitrum config required for arbitrum_demo mode".to_string());
            }
        }
        
        Ok(())
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
