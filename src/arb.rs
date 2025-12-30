#![allow(dead_code)]
use crate::constraint::ConstraintChecker;
use crate::types::{ArbitrageSignal, Market};

/// Arbitrage detector
#[derive(Debug)]
pub struct ArbitrageDetector {
    pub constraint_checker: ConstraintChecker,
    pub min_profit_threshold: f64,  // Minimum expected profit to trade
}

impl ArbitrageDetector {
    pub fn new(min_spread: f64, min_profit: f64) -> Self {
        Self {
            constraint_checker: ConstraintChecker::new(min_spread),
            min_profit_threshold: min_profit,
        }
    }

    /// Scan markets for arbitrage opportunities
    pub fn scan(&self, markets: &[Market]) -> Vec<ArbitrageSignal> {
        markets.iter()
            .filter(|m| m.active && m.accepting_orders)
            .filter_map(|m| self.constraint_checker.check_violation(m))
            .collect()
    }

    /// Calculate expected profit after costs
    pub fn expected_profit(
        &self,
        signal: &ArbitrageSignal,
        size: f64,
        fee_rate: f64,
        slippage: f64,
    ) -> f64 {
        let gross = signal.edge * size;
        let fee_cost = size * signal.yes_price * fee_rate * 2.0; // Both legs
        let slippage_cost = size * slippage;
        
        gross - fee_cost - slippage_cost
    }

    /// Decide if trade is worth taking
    pub fn should_trade(
        &self,
        signal: &ArbitrageSignal,
        size: f64,
        fee_rate: f64,
        slippage: f64,
    ) -> bool {
        self.expected_profit(signal, size, fee_rate, slippage) > self.min_profit_threshold
    }
}