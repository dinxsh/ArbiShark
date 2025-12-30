use crate::types::Side;
use crate::wallet::Wallet;
use crate::market::MarketDataProvider;
use crate::arb::ArbitrageDetector;
use crate::execution::ExecutionEngine;
use std::time::Duration;

#[allow(dead_code)]
pub struct TradingEngine {
    pub wallet: Wallet,
    pub market_provider: MarketDataProvider,
    pub detector: ArbitrageDetector,
    pub execution_engine: ExecutionEngine,
}

impl TradingEngine {
    pub fn new(
        wallet: Wallet,
        market_provider: MarketDataProvider,
        detector: ArbitrageDetector,
        execution_engine: ExecutionEngine,
    ) -> Self {
        Self {
            wallet,
            market_provider,
            detector,
            execution_engine,
        }
    }

    /// Run a single tick of the trading loop
    pub async fn tick(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch markets
        let markets = self.market_provider.fetch_markets().await?;

        // Scan for signals
        let signals = self.detector.scan(&markets);
        
        for signal in signals {
            // Simplified execution logic from main.rs
            if signal.recommended_side == Side::Buy {
               // Find market
               if let Some(market) = markets.iter().find(|m| m.id == signal.market_id) {
                    let size_per_leg = 5.0; // Fixed for now

                    // Execute on all outcomes (Buy Bundle behavior)
                    for token_id in &market.clob_token_ids {
                        if let Ok(book) = self.market_provider.fetch_order_book(token_id).await {
                             self.execution_engine.execute(&book, size_per_leg, Side::Buy, &mut self.wallet);
                        }
                    }
               }
            }
        }
        Ok(())
    }

    /// Run the loop for a specific duration or number of ticks
    pub async fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            if let Err(e) = self.tick().await {
                eprintln!("Error in tick: {}", e);
            }
            // In simulation we might not want to sleep strictly, or sleep 0 for speed
            // simulating "ticks"
            tokio::time::sleep(Duration::from_millis(100)).await; 
        }
    }
}
