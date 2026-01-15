use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use warp::Filter;
use serde::Serialize;
use crate::metamask::{MetaMaskClient, PermissionGrant};
use crate::positions::PositionManager;
use tokio::sync::RwLock;
use crate::types::ArbitrageSignal;

// Global log buffer for dashboard
static LOGS: Lazy<Arc<Mutex<Vec<String>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

// Helper to push logs to buffer
pub fn push_log(msg: &str) {
    let mut logs = LOGS.lock().unwrap();
    if logs.last().map_or(true, |last| last != msg) {
        logs.push(msg.to_string());
        if logs.len() > 100 {
            let len = logs.len();
            logs.drain(0..(len - 100));
        }
    }
}

/// API Server State
#[derive(Clone)]
pub struct ApiState {
    pub metamask: Arc<MetaMaskClient>,
    pub position_manager: Arc<RwLock<PositionManager>>,
}

#[derive(Serialize)]
pub struct StatsResponse {
    connected: bool, // Agent is running
    permission_active: bool,
    daily_limit: f64,
    spent_today: f64,
    total_trades: usize,
    win_rate: f64,
    total_pnl: f64,
    open_positions: usize,
}

#[derive(Serialize)]
pub struct TradeResponse {
    market_id: String,
    token_id: String,
    side: String,
    size: f64,
    entry_price: f64,
    entry_time: u64,
}

/// Start the API server
pub async fn start_server(state: ApiState) {
    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    // POST /api/permission
    // Receives permission grant from frontend (MetaMask)
    let permission_route = warp::path!("api" / "permission")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_permission);

    // GET /api/stats
    // Returns live stats for dashboard
    let stats_route = warp::path!("api" / "stats")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_stats);

    // GET /api/trades
    let trades_route = warp::path!("api" / "trades")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_trades);

    // GET /api/signals
    let signals_route = warp::path!("api" / "signals")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_signals);

    // GET /api/status
    let status_route = warp::path!("api" / "status")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_status);

    // Serve static dashboard files at /
    let dashboard_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("dashboard");
    let static_files = warp::fs::dir(dashboard_dir.clone());

    // If root path, serve index.html
    let index_html = warp::path::end().and(warp::fs::file(dashboard_dir.join("index.html")));

    // GET /api/logs
    let logs_route = warp::path!("api" / "logs")
        .and(warp::get())
        .map(|| {
            let logs = LOGS.lock().unwrap();
            warp::reply::json(&*logs)
        });

    let routes = permission_route
        .or(stats_route)
        .or(trades_route)
        .or(signals_route)
        .or(status_route)
        .or(logs_route)
        .or(index_html)
        .or(static_files)
        .with(cors);

    println!("🌍 [API] Server starting on http://localhost:3030");
    push_log("🌍 [API] Server started");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_state(state: ApiState) -> impl Filter<Extract = (ApiState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle permission update from frontend
async fn handle_permission(
    grant: PermissionGrant, // Frontend sends the grant object directly
    state: ApiState,
) -> Result<impl warp::Reply, warp::Rejection> {
    let msg = format!("📥 [API] Received permission grant from Dashboard: {}", grant.permission_id);
    println!("{}", msg);
    push_log(&msg);
    // Update the MetaMask client
    state.metamask.set_permission(grant).await;
    Ok(warp::reply::json(&serde_json::json!({ "status": "ok" })))
}

/// Handle stats request
async fn handle_stats(state: ApiState) -> Result<impl warp::Reply, warp::Rejection> {
    let perm = state.metamask.get_permission().await;
    let pm = state.position_manager.read().await;

    let (active, limit, spent) = match perm {
        Some(p) => (!p.revoked, p.daily_limit, p.spent_today),
        None => (false, 0.0, 0.0),
    };

    let stats = StatsResponse {
        connected: true,
        permission_active: active,
        daily_limit: limit,
        spent_today: spent,
        total_trades: pm.trade_count(),
        win_rate: pm.win_rate() * 100.0,
        total_pnl: pm.total_pnl(),
        open_positions: pm.get_positions().len(),
    };

    Ok(warp::reply::json(&stats))
}

async fn handle_trades(state: ApiState) -> Result<impl warp::Reply, warp::Rejection> {
    let pm = state.position_manager.read().await;
    let mut trades = Vec::new();
    for pos in pm.get_positions() {
        trades.push(TradeResponse {
            market_id: pos.market_id.clone(),
            token_id: pos.token_id.clone(),
            side: format!("{:?}", pos.side),
            size: pos.size,
            entry_price: pos.entry_price,
            entry_time: pos.entry_time,
        });
    }
    Ok(warp::reply::json(&trades))
}

async fn handle_signals(state: ApiState) -> Result<impl warp::Reply, warp::Rejection> {
    // TODO: Connect to live signals from detector/engine
    Ok(warp::reply::json(&serde_json::json!([])))
}

async fn handle_status(state: ApiState) -> Result<impl warp::Reply, warp::Rejection> {
    // TODO: Connect to engine status/errors
    Ok(warp::reply::json(&serde_json::json!({"status": "ok"})))
}
