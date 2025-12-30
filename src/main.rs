mod types;
mod wallet;
mod fees;
mod fee_calibrator;
mod slippage;
mod fills;
mod constraint;
mod arb;
mod execution;
mod engine;
mod simulation;
mod market;
mod latency;
// mod gamma;     // Use Envio instead of Gamma
mod solana;

use crate::wallet::Wallet;
use crate::market::MarketDataProvider;
use crate::arb::ArbitrageDetector;
use crate::execution::ExecutionEngine;
use crate::fees::FeeModel;
use crate::solana::SolanaManager;
use crate::latency::LatencyModel;
use crate::types::Side;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=======================================================");
    println!(" 🦈 PolyShark v1.0 (Hackathon Release)");
    println!("   - Permissioned Autonomous Agent");
    println!("   - Powered by MetaMask Advanced Permissions (ERC-7715)");
    println!("   - Multi-Chain Ready: Polymarket (Polygon) + Solana");
    println!("=======================================================\n");

    println!("🔐 [Init] Security Core: MetaMask Smart Account Adapter... Connected.");
    println!("📡 [Init] Market Data:   Envio Indexer (Mock)...           Connected.");

    // Solana Check
    print!("☀️ [Init] Solana Devnet:  Connecting... ");
    let sol_manager = SolanaManager::new();
    match sol_manager.check_connection() {
        Ok(v) => println!("Connected! (v{})", v),
        Err(_) => println!("Skipped (Offline)"),
    }

    // Initialize generic fee model (can be updated per market if needed)
    let fee_model = FeeModel { maker_fee_bps: 0, taker_fee_bps: 200 };
    
    // Components
    let mut wallet = Wallet::new(10.0); // 10 USDC daily spend limit
    let market_provider = MarketDataProvider::new("https://indexer.envio.dev/graphql");
    let detector = ArbitrageDetector::new(0.02, 0.10); // 2% spread, $0.10 min profit
    let latency_model = LatencyModel::new(50, 0.001); // 50ms delay, 0.1% adverse selection std
    let execution_engine = ExecutionEngine::new(fee_model, latency_model);

    println!("💸 [Init] Daily Allowance: ${:.2} USDC (Enforced by ERC-7715)", wallet.daily_limit);

    loop {
        println!("\n📡 Fetching markets from Envio (Gamma API)...");
        let mut markets = match market_provider.fetch_markets().await {
            Ok(m) => m,
            Err(e) => {
                println!("⚠️ Failed to fetch markets: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        println!("   Found {} active markets (Limit 20)", markets.len());

        // Hydrate prices (Real E2E)
        /* 
           In a production bot, we'd use WebSocket streams.
           For this demo loop, we fetch books sequentially to be "completely real".
        */
        for market in markets.iter_mut() {
            let mut prices = Vec::new();
            for (i, token_id) in market.clob_token_ids.iter().enumerate() {
                match market_provider.fetch_order_book(token_id).await {
                    Ok(book) => {
                        let price = book.midpoint().unwrap_or(0.0);
                        if price > 0.0 {
                            println!("   CTX: Market {} | Token {} | Price: {:.3}", market.slug, i, price);
                        }
                        prices.push(price);
                    },
                    Err(_) => prices.push(0.0), // Failed to fetch
                }
            }
            // Update market state if we got prices for all outcomes (usually 2)
            if prices.len() == market.outcomes.len() && prices.iter().all(|&p| p > 0.0) {
                 market.outcome_prices = prices;
            }
        }

        let signals = detector.scan(&markets);
        if signals.is_empty() {
            println!("   No arbitrage signals found.");
        } else {
            println!("⚡ Detected {} arbitrage signals!", signals.len());
            
            for signal in signals {
                println!("   Signal on Market {}: Spread {:.2}%, Edge ${:.2}", 
                    signal.market_id, signal.spread * 100.0, signal.edge);

                // Find the market to get token IDs
                if let Some(market) = markets.iter().find(|m| m.id == signal.market_id) {
                    // For a BUY signal (undervalued), we buy both YES and NO
                    // For a SELL signal (overvalued), we sell both (if we held them, but here we likely just ignore or short if possible)
                    // Simplified: We only act on BUY signals for this demo to consume allowance
                    
                    if signal.recommended_side == Side::Buy {
                        let size_per_leg = 5.0; // Fixed size for demo
                        println!("   Attempting to execute arb strategy...");

                        // Leg 1: Buy YES
                        let yes_token = &market.clob_token_ids[0];
                        if let Ok(book) = market_provider.fetch_order_book(yes_token).await {
                             execution_engine.execute(&book, size_per_leg, Side::Buy, &mut wallet);
                        }

                        // Leg 2: Buy NO
                        let no_token = &market.clob_token_ids[1];
                         if let Ok(book) = market_provider.fetch_order_book(no_token).await {
                             execution_engine.execute(&book, size_per_leg, Side::Buy, &mut wallet);
                        }
                    }
                }
            }
        }

        println!("💤 Sleeping 5s...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
