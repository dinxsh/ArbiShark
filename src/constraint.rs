use crate::types::{ArbitrageSignal, Market, Side};

/// Binary market constraint checker
#[derive(Debug, Clone)]
pub struct ConstraintChecker {
    pub min_spread_threshold: f64,  // e.g., 0.02 for 2%
}

impl ConstraintChecker {
    pub fn new(min_spread_threshold: f64) -> Self {
        Self { min_spread_threshold }
    }

    /// Check if market has arbitrage opportunity
    pub fn check_violation(&self, market: &Market) -> Option<ArbitrageSignal> {
        // Calculate sum of all outcome prices
        let sum: f64 = market.outcome_prices.iter().sum();
        let spread = (sum - 1.0).abs();
        
        if spread <= self.min_spread_threshold {
            return None; // No opportunity
        }

        let recommended_side = if sum > 1.0 {
            Side::Sell // Prices are overvalued (Sum > 1), Sell the bundle? (Selling all outcomes is complex, usually implies minting)
                       // In Polymarket, you can Sell if you hold, or you Mint sets and Sell.
                       // For simple arb, we usually look for Sum < 1 (buying the bundle for < $1).
        } else {
            Side::Buy  // Prices are undervalued (Sum < 1), Buy all outcomes for guaranteed payout of $1
        };

        Some(ArbitrageSignal {
            market_id: market.id.clone(),
            spread,
            edge: spread, // Gross edge before costs
            recommended_side,
            yes_price: market.yes_price(), // Legacy field, might need updating in ArbitrageSignal struct to be generic
            no_price: market.no_price(),   // Legacy field
        })
    }
}