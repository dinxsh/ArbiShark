# Stylus Integration for ArbiShark

ArbiShark's core math and arbitrage logic is designed to be Stylus-ready. The following pure Rust functions can be compiled to WASM and used in Stylus smart contracts for on-chain arbitrage verification:

## Core Functions

- Spread calculation
- Arbitrage detection
- Expected profit calculation

## Example (Rust, Stylus-compatible)

```rust
pub fn calc_spread(bid: f64, ask: f64) -> f64 {
    if ask > 0.0 { (bid - ask) / ask } else { 0.0 }
}

pub fn detect_arbitrage(prices: &[f64]) -> bool {
    let sum: f64 = prices.iter().sum();
    sum > 1.0
}

pub fn expected_profit(size: f64, spread: f64) -> f64 {
    size * spread
}
```

## How to Compile for Stylus

- Use `no_std` and avoid async/network code.
- Export functions as WASM for Stylus deployment.

See [docs/arbitrum/doc.md] for more details.