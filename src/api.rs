//! HTTP API module for Frontend <-> Agent communication
//!
//! Exposes endpoints for the dashboard to control the agent and view stats.

use warp::Filter;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::metamask::{MetaMaskClient, PermissionGrant};
use crate::positions::PositionManager;
use tokio::sync::RwLock;

/// API Server State
#[derive(Clone)]
pub struct ApiState {
    pub metamask: Arc<MetaMaskClient>,
    pub position_manager: Arc<RwLock<PositionManager>>,
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

    let routes = permission_route
        .or(stats_route)
        .with(cors);

    println!("🌍 [API] Server starting on http://localhost:3030");
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
    println!("📥 [API] Received permission grant from Dashboard: {}", grant.permission_id);
    
    // Update the MetaMask client
    state.metamask.set_permission(grant).await;
    
    Ok(warp::reply::json(&serde_json::json!({ "status": "ok" })))
}

#[derive(Serialize)]
struct StatsResponse {
    connected: bool, // Agent is running
    permission_active: bool,
    daily_limit: f64,
    spent_today: f64,
    total_trades: usize,
    win_rate: f64,
    total_pnl: f64,
    open_positions: usize,
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
