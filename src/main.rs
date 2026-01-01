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
mod solana;
mod metamask;
mod config;
mod websocket;
mod positions;

use crate::wallet::Wallet;
use crate::market::MarketDataProvider;
use crate::arb::ArbitrageDetector;
use crate::execution::ExecutionEngine;
use crate::fees::FeeModel;
use crate::solana::SolanaManager;
use crate::latency::LatencyModel;
use crate::types::Side;
use crate::config::Config;
use crate::metamask::MetaMaskClient;
use crate::positions::{Position, PositionManager};
use std::time::Duration;
use colored::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load().unwrap_or_else(|e| {
        println!("⚠️ Config load failed ({}), using defaults", e);
        Config::default_config()
    });

    println!("\n{}", "=======================================================".bright_blue());
    println!(" {} {}", "🦈".cyan(), "PolyShark v2.0 (Hackathon Release)".bold().cyan());
    println!("   - {}", "Permissioned Autonomous Agent".white());
    println!("   - Powered by {}", "MetaMask Advanced Permissions (ERC-7715)".yellow());
    println!("   - Multi-Chain Ready: {} + {}", "Polymarket".purple(), "Solana".green());
    println!("{}", "=======================================================\n".bright_blue());

    // Initialize MetaMask client
    let metamask = MetaMaskClient::new();
    
    // Connect to MetaMask
    print!("{} MetaMask:      Connecting... ", "🦊 [Init]".bold().yellow());
    match metamask.connect().await {
        Ok(addr) => println!("{}", format!("Connected ({}...)", &addr[..10]).green()),
        Err(e) => println!("{}", format!("Failed: {}", e).red()),
    }

    // Request permission
    print!("{} ERC-7715:      Requesting permission... ", "🔐 [Init]".bold().yellow());
    match metamask.request_permission(
        &config.permission.token,
        config.permission.daily_limit_usdc,
        config.permission.duration_days,
    ).await {
        Ok(perm) => println!("{}", format!("Granted (ID: {})", &perm.permission_id[..20]).green()),
        Err(e) => println!("{}", format!("Failed: {}", e).red()),
    }

    println!("{} Market Data:   Envio Indexer...           {}", "📡 [Init]".bold().yellow(), "Connected.".green());

    // Solana Check
    print!("{} Solana Devnet:  Connecting... ", "☀️ [Init]".bold().yellow());
    let sol_manager = SolanaManager::new();
    match sol_manager.check_connection() {
        Ok(v) => println!("{}", format!("Connected! (v{})", v).green()),
        Err(_) => println!("{}", "Skipped (Offline)".red()),
    }

    // Initialize components from config
    let fee_model = FeeModel { maker_fee_bps: 0, taker_fee_bps: 200 };
    let mut wallet = Wallet::new(config.permission.daily_limit_usdc);
    let market_provider = MarketDataProvider::new(&config.api.gamma_url);
    let detector = ArbitrageDetector::new(
        config.trading.min_spread_threshold,
        config.trading.min_profit_threshold,
    );
    let latency_model = LatencyModel::new(
        config.timing.latency_base_ms,
        config.timing.adverse_selection_std,
    );
    let execution_engine = ExecutionEngine::new(fee_model.clone(), latency_model);
    
    // Position manager for exit logic
    let mut position_manager = PositionManager::new(
        0.005,  // 0.5% profit target spread
        0.02,   // 2% stop loss spread
        config.timing.position_timeout_secs,
    );

    println!("{} Daily Allowance: ${:.2} USDC (Enforced by ERC-7715)", "💸 [Init]".bold().yellow(), wallet.daily_limit);
    println!("{} Trade Size: ${:.2} per leg", "📊 [Init]".bold().yellow(), config.trading.trade_size);
    println!();

    loop {
        println!("\n{}", "📡 Fetching markets from Gamma API...".cyan());
        let mut markets = match market_provider.fetch_markets().await {
            Ok(m) => m,
            Err(e) => {
                println!("⚠️ Failed to fetch markets: {}", e);
                tokio::time::sleep(Duration::from_secs(config.timing.poll_interval_secs)).await;
                continue;
            }
        };
        println!("   Found {} active markets (Limit {})", markets.len(), config.api.market_limit);

        // Hydrate prices
        market_provider.hydrate_market_prices(&mut markets).await;

        // Check for position exits FIRST
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let exits = position_manager.check_exits(&markets, current_time, fee_model.taker_rate());
        if !exits.is_empty() {
            println!("📤 Closed {} positions:", exits.len());
            for exit in &exits {
                println!("   {} | {:?} | PnL: ${:.4}", 
                    exit.position.token_id, exit.reason, exit.pnl);
            }
        }

        // Scan for new signals
        let signals = detector.scan(&markets);
        if signals.is_empty() {
            println!("   No arbitrage signals found.");
        } else {
            println!("⚡ Detected {} arbitrage signals!", signals.len());
            
            for signal in signals {
                println!("   Signal on Market {}: Spread {:.2}%, Edge ${:.2}", 
                    signal.market_id, signal.spread * 100.0, signal.edge);

                if let Some(market) = markets.iter().find(|m| m.id == signal.market_id) {
                    if signal.recommended_side == Side::Buy {
                        let size_per_leg = config.trading.trade_size;
                        
                        // Check MetaMask permission before trading
                        let remaining = metamask.get_remaining_allowance().await;
                        let required = size_per_leg * 2.0; // Both legs
                        
                        if remaining < required {
                            println!("   ⚠️ Insufficient permission allowance (${:.2} < ${:.2})", 
                                remaining, required);
                            continue;
                        }

                        println!("   Attempting to execute arb strategy...");

                        // Execute both legs
                        for (idx, token_id) in market.clob_token_ids.iter().enumerate() {
                            if let Ok(book) = market_provider.fetch_order_book(token_id).await {
                                if let Some(result) = execution_engine.execute(
                                    &book, size_per_leg, Side::Buy, &mut wallet
                                ) {
                                    // Record in MetaMask
                                    let _ = metamask.record_spend(result.total_cost).await;
                                    
                                    // Track position for exit
                                    position_manager.open_position(Position {
                                        market_id: market.id.clone(),
                                        token_id: token_id.clone(),
                                        side: Side::Buy,
                                        size: result.filed_size,
                                        entry_price: result.execution_price,
                                        entry_time: current_time,
                                        entry_spread: signal.spread,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Show stats
        println!("\n📊 Stats: {} trades | Win rate: {:.0}% | PnL: ${:.2} | Open: {}", 
            position_manager.trade_count(),
            position_manager.win_rate() * 100.0,
            position_manager.total_pnl(),
            position_manager.get_positions().len(),
        );

        println!("💤 Sleeping {}s...", config.timing.poll_interval_secs);
        tokio::time::sleep(Duration::from_secs(config.timing.poll_interval_secs)).await;
    }
}
