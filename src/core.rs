// Pure Rust core for Stylus/Orbit compatibility
// Place in src/core.rs

pub fn calc_spread(bid: f64, ask: f64) -> f64 {
    if ask > 0.0 { (bid - ask) / ask } else { 0.0 }
}

pub fn detect_arbitrage(bid: f64, ask: f64, threshold: f64) -> bool {
    calc_spread(bid, ask) > threshold
}

pub fn expected_profit(size: f64, bid: f64, ask: f64, fee_bps: f64) -> f64 {
    let gross = (bid - ask) * size;
    let fee = (bid * size + ask * size) * fee_bps / 10000.0;
    gross - fee
}
