use crate::api::push_log;
mod market_client;
mod permission_guard;
use crate::market_client::{MarketClient, ArbitrumMarketClient};
use crate::market_client::PolymarketClient;
use crate::permission_guard::PermissionGuard;
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
mod api;

use crate::wallet::Wallet;
// ...existing code...
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
use std::sync::Arc;
use tokio::sync::RwLock;
use colored::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load().unwrap_or_else(|e| {
        println!("⚠️ Config load failed ({}), using defaults", e);
        Config::default_config()
    });

    println!("\n{}", "=======================================================".bright_blue());
    println!(" {} {}", "🦈".cyan(), "ArbiShark v1.0 (Hackathon Release)".bold().cyan());
    println!("   - {}", "Arbitrum-First Permissioned Agent".white());
    println!("   - Powered by {}", "MetaMask Delegation Toolkit (ERC-7715)".yellow());
    println!("   - Arbitrum + Polymarket CLOB Pattern".purple());
    println!("   - Hybrid DApp: {}", "Enabled (API Port 3030)".purple());
    println!("{}", "=======================================================\n".bright_blue());

    // Initialize Components (Shared State)
    let metamask = Arc::new(MetaMaskClient::new());
    // Read mode from config.toml (default: polymarket)
    let mode: String = config.mode.clone();
    println!("Running in mode: {}", mode);

    // PermissionGuard setup (ERC-7715 mapping)
    let mut guard = PermissionGuard { daily_limit: config.permission.daily_limit_usdc, spent_today: 0.0 };

    // MarketClient selection
    let market_client: Box<dyn MarketClient + Send + Sync> = match mode.as_str() {
        "arbitrum_demo" => {
            println!("Using ArbitrumMarketClient (Envio HyperIndex)");
            Box::new(ArbitrumMarketClient {
                endpoint: "https://envio-arbitrum-hyperindex.example/graphql".to_string(),
            })
        },
        _ => {
            println!("Using PolymarketClient (CLOB Pattern Example)");
            Box::new(PolymarketClient {
                gamma_url: "https://gamma-api.polymarket.com/events?limit=20&active=true&closed=false".to_string(),
                clob_url: "https://clob.polymarket.com/book".to_string(),
                client: reqwest::Client::new(),
            })
        }
    };
    
    // Position manager for exit logic (Shared)
    let position_manager = Arc::new(RwLock::new(PositionManager::new(
        0.005,  // 0.5% profit target spread
        0.02,   // 2% stop loss spread
        config.timing.position_timeout_secs,
    )));

    // 🚀 Start API Server
    let api_state = api::ApiState {
        metamask: metamask.clone(),
        position_manager: position_manager.clone(),
    };
    
    tokio::spawn(async move {
        api::start_server(api_state).await;
    });

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
    // Use the selected market_client for all market data
    let detector = ArbitrageDetector::new(
        config.trading.min_spread_threshold,
        config.trading.min_profit_threshold,
    );
    let latency_model = LatencyModel::new(
        config.timing.latency_base_ms,
        config.timing.adverse_selection_std,
    );
    let execution_engine = ExecutionEngine::new(fee_model.clone(), latency_model);
    
    println!("{} Daily Allowance: ${:.2} USDC (Enforced by ERC-7715)", "💸 [Init]".bold().yellow(), wallet.daily_limit);
    println!("{} Trade Size: ${:.2} per leg", "📊 [Init]".bold().yellow(), config.trading.trade_size);
    println!();
    println!("⏳ Waiting for MetaMask permission via Dashboard...");

    loop {
        // Wait for active permission if not present
        if !metamask.has_valid_permission().await {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        let log_msg = format!("📡 Fetching markets...");
        println!("\n{}", log_msg.cyan());
        push_log(&log_msg);
        let mut markets = match market_client.get_markets().await {
            Ok(m) => m,
            Err(e) => {
                println!("⚠️ Failed to fetch markets: {}", e);
                tokio::time::sleep(Duration::from_secs(config.timing.poll_interval_secs)).await;
                continue;
            }
        };
        let found_msg = format!("   Found {} active markets", markets.len());
        println!("{}", found_msg);
        push_log(&found_msg);

        // Check for position exits FIRST
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Lock position manager for updates
        let mut exits = Vec::new(); // Placeholder to avoid holding lock too long if logic was complex
        {
            let mut pm = position_manager.write().await;
            exits = pm.check_exits(&markets, current_time, fee_model.taker_rate());
        }

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
            let msg = "   No arbitrage signals found.";
            println!("{}", msg);
            push_log(msg);
        } else {
            let msg = format!("⚡ Detected {} arbitrage signals!", signals.len());
            println!("{}", msg);
            push_log(&msg);
            for signal in signals {
                let sig_msg = format!("   Signal on Market {}: Spread {:.2}%, Edge ${:.2}",
                    signal.market_id, signal.spread * 100.0, signal.edge);
                println!("{}", sig_msg);
                push_log(&sig_msg);
                if let Some(market) = markets.iter().find(|m| m.id == signal.market_id) {
                    if signal.recommended_side == Side::Buy {
                        let size_per_leg = config.trading.trade_size;
                        let remaining = metamask.get_remaining_allowance().await;
                        let required = size_per_leg * 2.0;
                        if remaining < required {
                            let warn_msg = format!("   ⚠️ Insufficient permission allowance (${:.2} < ${:.2})", remaining, required);
                            println!("{}", warn_msg);
                            push_log(&warn_msg);
                            continue;
                        }
                        let exec_msg = "   Attempting to execute arb strategy...";
                        println!("{}", exec_msg);
                        push_log(exec_msg);
                        for token_id in &market.clob_token_ids {
                            if let Ok(book) = market_client.get_order_book(token_id).await {
                                if let Some(result) = execution_engine.execute(
                                    &book, size_per_leg, Side::Buy, &mut wallet
                                ) {
                                    let _ = metamask.record_spend(result.total_cost).await;
                                    let mut pm = position_manager.write().await;
                                    pm.open_position(Position {
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
        {
            let pm = position_manager.read().await;
            let stats_msg = format!("📊 Stats: {} trades | Win rate: {:.0}% | PnL: ${:.2} | Open: {}",
                pm.trade_count(),
                pm.win_rate() * 100.0,
                pm.total_pnl(),
                pm.get_positions().len(),
            );
            println!("\n{}", stats_msg);
            push_log(&stats_msg);
        }

        let sleep_msg = format!("💤 Sleeping {}s...", config.timing.poll_interval_secs);
        println!("{}", sleep_msg);
        push_log(&sleep_msg);
        tokio::time::sleep(Duration::from_secs(config.timing.poll_interval_secs)).await;
    }
}
