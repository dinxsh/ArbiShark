// Risk Management System
// Prevents losses and manages trading risk

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_drawdown: f64,           // Max % loss from peak (e.g., 0.20 = 20%)
    pub max_daily_loss: f64,         // Max $ loss per day
    pub max_consecutive_losses: u32, // Stop after N losses
    pub volatility_threshold: f64,   // Pause if volatility > threshold
    pub min_liquidity: f64,          // Min market liquidity required
    pub max_position_size: f64,      // Max $ per position
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_drawdown: 0.20,      // 20% max drawdown
            max_daily_loss: 50.0,    // $50 max daily loss
            max_consecutive_losses: 5,
            volatility_threshold: 0.15, // 15% volatility
            min_liquidity: 1000.0,   // $1000 min liquidity
            max_position_size: 100.0, // $100 max position
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskManager {
    config: RiskConfig,
    peak_balance: f64,
    current_balance: f64,
    daily_loss: f64,
    consecutive_losses: u32,
    recent_trades: Vec<TradeResult>,
    circuit_breaker: bool,
}

#[derive(Debug, Clone)]
struct TradeResult {
    pnl: f64,
    timestamp: DateTime<Utc>,
}

impl RiskManager {
    pub fn new(config: RiskConfig, initial_balance: f64) -> Self {
        Self {
            config,
            peak_balance: initial_balance,
            current_balance: initial_balance,
            daily_loss: 0.0,
            consecutive_losses: 0,
            recent_trades: Vec::new(),
            circuit_breaker: false,
        }
    }

    /// Check if trading should be halted
    pub fn should_halt(&self) -> (bool, Option<String>) {
        // Circuit breaker activated
        if self.circuit_breaker {
            return (true, Some("Circuit breaker activated".to_string()));
        }

        // Check drawdown
        let drawdown = (self.peak_balance - self.current_balance) / self.peak_balance;
        if drawdown > self.config.max_drawdown {
            return (true, Some(format!(
                "Max drawdown exceeded: {:.1}% (limit: {:.1}%)",
                drawdown * 100.0,
                self.config.max_drawdown * 100.0
            )));
        }

        // Check daily loss
        if self.daily_loss > self.config.max_daily_loss {
            return (true, Some(format!(
                "Daily loss limit hit: ${:.2} (limit: ${:.2})",
                self.daily_loss,
                self.config.max_daily_loss
            )));
        }

        // Check consecutive losses
        if self.consecutive_losses >= self.config.max_consecutive_losses {
            return (true, Some(format!(
                "Too many consecutive losses: {} (limit: {})",
                self.consecutive_losses,
                self.config.max_consecutive_losses
            )));
        }

        // Check volatility
        let volatility = self.calculate_volatility();
        if volatility > self.config.volatility_threshold {
            return (true, Some(format!(
                "Market too volatile: {:.1}% (limit: {:.1}%)",
                volatility * 100.0,
                self.config.volatility_threshold * 100.0
            )));
        }

        (false, None)
    }

    /// Validate if a trade is allowed
    pub fn validate_trade(&self, trade_size: f64, market_liquidity: f64) -> Result<(), String> {
        // Check position size
        if trade_size > self.config.max_position_size {
            return Err(format!(
                "Trade size ${:.2} exceeds max ${:.2}",
                trade_size,
                self.config.max_position_size
            ));
        }

        // Check liquidity
        if market_liquidity < self.config.min_liquidity {
            return Err(format!(
                "Insufficient liquidity: ${:.2} (min: ${:.2})",
                market_liquidity,
                self.config.min_liquidity
            ));
        }

        // Check if halted
        if let (true, Some(reason)) = self.should_halt() {
            return Err(format!("Trading halted: {}", reason));
        }

        Ok(())
    }

    /// Record trade result
    pub fn record_trade(&mut self, pnl: f64) {
        self.current_balance += pnl;
        
        // Update peak
        if self.current_balance > self.peak_balance {
            self.peak_balance = self.current_balance;
        }

        // Update daily loss
        if pnl < 0.0 {
            self.daily_loss += pnl.abs();
            self.consecutive_losses += 1;
        } else {
            self.consecutive_losses = 0;
        }

        // Store trade result
        self.recent_trades.push(TradeResult {
            pnl,
            timestamp: Utc::now(),
        });

        // Keep only last 100 trades
        if self.recent_trades.len() > 100 {
            self.recent_trades.remove(0);
        }
    }

    /// Calculate recent volatility
    fn calculate_volatility(&self) -> f64 {
        if self.recent_trades.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = self.recent_trades
            .iter()
            .map(|t| t.pnl / self.current_balance)
            .collect();

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;

        variance.sqrt()
    }

    /// Reset daily counters
    pub fn reset_daily(&mut self) {
        self.daily_loss = 0.0;
    }

    /// Activate emergency circuit breaker
    pub fn activate_circuit_breaker(&mut self) {
        self.circuit_breaker = true;
        tracing::error!("🚨 Circuit breaker activated!");
    }

    /// Deactivate circuit breaker
    pub fn deactivate_circuit_breaker(&mut self) {
        self.circuit_breaker = false;
        tracing::info!("✅ Circuit breaker deactivated");
    }

    /// Get risk status
    pub fn get_status(&self) -> RiskStatus {
        let drawdown = (self.peak_balance - self.current_balance) / self.peak_balance;
        let volatility = self.calculate_volatility();
        let (is_halted, halt_reason) = self.should_halt();

        RiskStatus {
            current_balance: self.current_balance,
            peak_balance: self.peak_balance,
            drawdown_percent: drawdown * 100.0,
            daily_loss: self.daily_loss,
            consecutive_losses: self.consecutive_losses,
            volatility_percent: volatility * 100.0,
            is_halted,
            halt_reason,
            circuit_breaker: self.circuit_breaker,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskStatus {
    pub current_balance: f64,
    pub peak_balance: f64,
    pub drawdown_percent: f64,
    pub daily_loss: f64,
    pub consecutive_losses: u32,
    pub volatility_percent: f64,
    pub is_halted: bool,
    pub halt_reason: Option<String>,
    pub circuit_breaker: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawdown_limit() {
        let config = RiskConfig {
            max_drawdown: 0.10, // 10%
            ..Default::default()
        };
        let mut manager = RiskManager::new(config, 100.0);

        // Lose 11% - should halt
        manager.record_trade(-11.0);
        
        let (should_halt, reason) = manager.should_halt();
        assert!(should_halt);
        assert!(reason.unwrap().contains("drawdown"));
    }

    #[test]
    fn test_consecutive_losses() {
        let config = RiskConfig {
            max_consecutive_losses: 3,
            ..Default::default()
        };
        let mut manager = RiskManager::new(config, 100.0);

        // 3 losses in a row
        manager.record_trade(-1.0);
        manager.record_trade(-1.0);
        manager.record_trade(-1.0);

        let (should_halt, _) = manager.should_halt();
        assert!(should_halt);
    }

    #[test]
    fn test_trade_validation() {
        let manager = RiskManager::new(RiskConfig::default(), 100.0);

        // Valid trade
        assert!(manager.validate_trade(50.0, 2000.0).is_ok());

        // Trade too large
        assert!(manager.validate_trade(150.0, 2000.0).is_err());

        // Insufficient liquidity
        assert!(manager.validate_trade(50.0, 500.0).is_err());
    }
}
