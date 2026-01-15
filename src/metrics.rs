// ArbiShark Monitoring Dashboard
// Real-time metrics and health monitoring

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    // Performance
    pub trades_today: u32,
    pub trades_total: u64,
    pub win_rate: f64,
    pub avg_profit_per_trade: f64,
    pub total_pnl: f64,
    pub daily_pnl: f64,
    pub sharpe_ratio: f64,
    
    // Health
    pub envio_latency_ms: u64,
    pub envio_uptime_percent: f64,
    pub last_trade_time: Option<DateTime<Utc>>,
    pub consecutive_failures: u32,
    pub is_safe_mode: bool,
    
    // Usage
    pub daily_spent: f64,
    pub daily_limit: f64,
    pub remaining_allowance: f64,
    pub strategy_mode: String,
    
    // Costs
    pub gas_spent_eth: f64,
    pub gas_saved_vs_l1: f64,
    
    // System
    pub uptime_seconds: u64,
    pub version: String,
    pub last_updated: DateTime<Utc>,
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            trades_today: 0,
            trades_total: 0,
            win_rate: 0.0,
            avg_profit_per_trade: 0.0,
            total_pnl: 0.0,
            daily_pnl: 0.0,
            sharpe_ratio: 0.0,
            envio_latency_ms: 0,
            envio_uptime_percent: 100.0,
            last_trade_time: None,
            consecutive_failures: 0,
            is_safe_mode: false,
            daily_spent: 0.0,
            daily_limit: 10.0,
            remaining_allowance: 10.0,
            strategy_mode: "Normal".to_string(),
            gas_spent_eth: 0.0,
            gas_saved_vs_l1: 0.0,
            uptime_seconds: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_updated: Utc::now(),
        }
    }
}

pub struct MetricsCollector {
    metrics: Arc<RwLock<AgentMetrics>>,
    start_time: DateTime<Utc>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(AgentMetrics::default())),
            start_time: Utc::now(),
        }
    }

    pub async fn record_trade(&self, profit: f64, gas_cost: f64) {
        let mut metrics = self.metrics.write().await;
        
        metrics.trades_today += 1;
        metrics.trades_total += 1;
        metrics.total_pnl += profit;
        metrics.daily_pnl += profit;
        metrics.gas_spent_eth += gas_cost;
        metrics.last_trade_time = Some(Utc::now());
        
        // Calculate win rate
        if profit > 0.0 {
            metrics.win_rate = (metrics.win_rate * (metrics.trades_total - 1) as f64 + 1.0) 
                / metrics.trades_total as f64;
        } else {
            metrics.win_rate = (metrics.win_rate * (metrics.trades_total - 1) as f64) 
                / metrics.trades_total as f64;
        }
        
        // Calculate average profit
        metrics.avg_profit_per_trade = metrics.total_pnl / metrics.trades_total as f64;
        
        // Estimate L1 gas cost (10x higher)
        metrics.gas_saved_vs_l1 += gas_cost * 9.0;
        
        metrics.last_updated = Utc::now();
    }

    pub async fn update_envio_health(&self, latency_ms: u64, is_healthy: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.envio_latency_ms = latency_ms;
        
        if !is_healthy {
            metrics.consecutive_failures += 1;
        } else {
            metrics.consecutive_failures = 0;
        }
        
        metrics.last_updated = Utc::now();
    }

    pub async fn update_spending(&self, amount: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.daily_spent += amount;
        metrics.remaining_allowance = metrics.daily_limit - metrics.daily_spent;
        
        // Update strategy mode based on remaining allowance
        let allowance_pct = metrics.remaining_allowance / metrics.daily_limit;
        metrics.strategy_mode = if allowance_pct < 0.3 {
            "Conservative".to_string()
        } else if allowance_pct > 0.7 {
            "Aggressive".to_string()
        } else {
            "Normal".to_string()
        };
        
        metrics.last_updated = Utc::now();
    }

    pub async fn set_safe_mode(&self, enabled: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.is_safe_mode = enabled;
        metrics.last_updated = Utc::now();
    }

    pub async fn get_metrics(&self) -> AgentMetrics {
        let mut metrics = self.metrics.read().await.clone();
        metrics.uptime_seconds = (Utc::now() - self.start_time).num_seconds() as u64;
        metrics
    }

    pub async fn reset_daily(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.trades_today = 0;
        metrics.daily_pnl = 0.0;
        metrics.daily_spent = 0.0;
        metrics.remaining_allowance = metrics.daily_limit;
        metrics.last_updated = Utc::now();
    }

    // Export metrics for Prometheus
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.get_metrics().await;
        
        format!(
            "# HELP arbishark_trades_total Total number of trades\n\
             # TYPE arbishark_trades_total counter\n\
             arbishark_trades_total {}\n\
             \n\
             # HELP arbishark_win_rate Win rate percentage\n\
             # TYPE arbishark_win_rate gauge\n\
             arbishark_win_rate {}\n\
             \n\
             # HELP arbishark_pnl_total Total PnL in USDC\n\
             # TYPE arbishark_pnl_total gauge\n\
             arbishark_pnl_total {}\n\
             \n\
             # HELP arbishark_envio_latency_ms Envio latency in milliseconds\n\
             # TYPE arbishark_envio_latency_ms gauge\n\
             arbishark_envio_latency_ms {}\n\
             \n\
             # HELP arbishark_gas_saved_eth Gas saved vs L1 in ETH\n\
             # TYPE arbishark_gas_saved_eth gauge\n\
             arbishark_gas_saved_eth {}\n\
             \n\
             # HELP arbishark_safe_mode Safe mode status (1=enabled, 0=disabled)\n\
             # TYPE arbishark_safe_mode gauge\n\
             arbishark_safe_mode {}\n",
            metrics.trades_total,
            metrics.win_rate,
            metrics.total_pnl,
            metrics.envio_latency_ms,
            metrics.gas_saved_vs_l1,
            if metrics.is_safe_mode { 1 } else { 0 }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();
        
        // Record profitable trade
        collector.record_trade(1.5, 0.001).await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.trades_total, 1);
        assert_eq!(metrics.total_pnl, 1.5);
        assert!(metrics.win_rate > 0.0);
    }

    #[tokio::test]
    async fn test_spending_tracking() {
        let collector = MetricsCollector::new();
        
        collector.update_spending(3.0).await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.daily_spent, 3.0);
        assert_eq!(metrics.remaining_allowance, 7.0);
    }
}
