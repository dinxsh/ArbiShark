// Plugin System for ArbiShark
// Extensible architecture for custom strategies and integrations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin decision for trade signals
#[derive(Debug, Clone)]
pub enum PluginDecision {
    Continue,
    Skip(String),
    ModifySize(f64),
    ModifySpread(f64),
}

/// Plugin action for errors
#[derive(Debug, Clone)]
pub enum PluginAction {
    Retry,
    Skip,
    Halt,
}

/// Core plugin trait
#[async_trait]
pub trait AgentPlugin: Send + Sync {
    /// Plugin metadata
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;

    /// Lifecycle hooks
    async fn on_start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    async fn on_stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// Trade hooks
    async fn on_trade_signal(
        &self,
        signal: &ArbitrageSignal,
    ) -> PluginDecision {
        PluginDecision::Continue
    }

    async fn on_trade_complete(
        &self,
        trade: &TradeResult,
    ) {
        // Default: do nothing
    }

    async fn on_error(
        &self,
        error: &str,
    ) -> PluginAction {
        PluginAction::Skip
    }
}

/// Example: Sentiment Analysis Plugin
pub struct SentimentPlugin {
    api_key: String,
    threshold: f64,
}

impl SentimentPlugin {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            threshold: -0.5,
        }
    }

    async fn get_sentiment(&self, market_id: &str) -> f64 {
        // Simulate API call to sentiment analysis service
        // In production: call Twitter API, Reddit API, etc.
        0.0
    }
}

#[async_trait]
impl AgentPlugin for SentimentPlugin {
    fn name(&self) -> &str {
        "sentiment-analyzer"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Analyzes social sentiment before executing trades"
    }

    async fn on_trade_signal(&self, signal: &ArbitrageSignal) -> PluginDecision {
        let sentiment = self.get_sentiment(&signal.market_id).await;

        if sentiment < self.threshold {
            PluginDecision::Skip(format!(
                "Negative sentiment: {:.2}",
                sentiment
            ))
        } else {
            PluginDecision::Continue
        }
    }
}

/// Example: Notification Plugin
pub struct NotificationPlugin {
    telegram_token: Option<String>,
    discord_webhook: Option<String>,
}

impl NotificationPlugin {
    pub fn new(telegram_token: Option<String>, discord_webhook: Option<String>) -> Self {
        Self {
            telegram_token,
            discord_webhook,
        }
    }

    async fn send_telegram(&self, message: &str) {
        if let Some(token) = &self.telegram_token {
            // Send to Telegram
            tracing::info!("📱 Telegram: {}", message);
        }
    }

    async fn send_discord(&self, message: &str) {
        if let Some(webhook) = &self.discord_webhook {
            // Send to Discord
            tracing::info!("💬 Discord: {}", message);
        }
    }
}

#[async_trait]
impl AgentPlugin for NotificationPlugin {
    fn name(&self) -> &str {
        "notifications"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Sends notifications via Telegram and Discord"
    }

    async fn on_trade_complete(&self, trade: &TradeResult) {
        let message = format!(
            "🦈 Trade Complete\nMarket: {}\nPnL: ${:.2}\nStatus: {}",
            trade.market_id,
            trade.pnl,
            if trade.pnl > 0.0 { "✅ Profit" } else { "❌ Loss" }
        );

        self.send_telegram(&message).await;
        self.send_discord(&message).await;
    }

    async fn on_error(&self, error: &str) -> PluginAction {
        let message = format!("⚠️ Error: {}", error);
        self.send_telegram(&message).await;
        self.send_discord(&message).await;
        PluginAction::Skip
    }
}

/// Plugin Manager
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn AgentPlugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn AgentPlugin>) {
        let name = plugin.name().to_string();
        tracing::info!("📦 Registered plugin: {} v{}", name, plugin.version());
        self.plugins.insert(name, plugin);
    }

    pub async fn start_all(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (name, plugin) in &mut self.plugins {
            plugin.on_start().await?;
            tracing::info!("✅ Started plugin: {}", name);
        }
        Ok(())
    }

    pub async fn stop_all(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (name, plugin) in &mut self.plugins {
            plugin.on_stop().await?;
            tracing::info!("🛑 Stopped plugin: {}", name);
        }
        Ok(())
    }

    pub async fn process_signal(&self, signal: &ArbitrageSignal) -> PluginDecision {
        for plugin in self.plugins.values() {
            match plugin.on_trade_signal(signal).await {
                PluginDecision::Continue => continue,
                decision => return decision,
            }
        }
        PluginDecision::Continue
    }

    pub async fn notify_trade(&self, trade: &TradeResult) {
        for plugin in self.plugins.values() {
            plugin.on_trade_complete(trade).await;
        }
    }

    pub async fn handle_error(&self, error: &str) -> PluginAction {
        for plugin in self.plugins.values() {
            match plugin.on_error(error).await {
                PluginAction::Halt => return PluginAction::Halt,
                _ => continue,
            }
        }
        PluginAction::Skip
    }
}

// Placeholder types (should match your existing types)
#[derive(Debug, Clone)]
pub struct ArbitrageSignal {
    pub market_id: String,
    pub spread: f64,
    pub edge: f64,
}

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub market_id: String,
    pub pnl: f64,
    pub gas_cost: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_manager() {
        let mut manager = PluginManager::new();

        // Register notification plugin
        let notif_plugin = NotificationPlugin::new(
            Some("test_token".to_string()),
            None,
        );
        manager.register(Box::new(notif_plugin));

        // Start all plugins
        assert!(manager.start_all().await.is_ok());

        // Test trade notification
        let trade = TradeResult {
            market_id: "test_market".to_string(),
            pnl: 1.5,
            gas_cost: 0.001,
        };
        manager.notify_trade(&trade).await;

        // Stop all plugins
        assert!(manager.stop_all().await.is_ok());
    }
}
