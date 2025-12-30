# 🛠️ PolyShark — Implementation Guide

A step-by-step guide to building the PolyShark arbitrage bot from scratch.

---

## Phase 1: Core Infrastructure

### 1.1 Wallet Module (`wallet.rs`)

```rust
struct Wallet {
    usdc: f64,           // Starting balance: 1000.00
    positions: HashMap<MarketId, Position>,
}

struct Position {
    outcome: Outcome,    // YES or NO
    size: f64,
    entry_price: f64,
    entry_time: Timestamp,
}
```

**Key Functions:**
- `new(initial_balance)` — Initialize wallet
- `can_afford(amount)` — Check available balance
- `deduct(amount)` — Remove USDC for trade
- `credit(amount)` — Add USDC from sale
- `get_equity()` — Total value (cash + positions)

---

### 1.2 Market Module (`market.rs`)

```rust
struct Market {
    id: MarketId,
    yes_price: f64,
    no_price: f64,
    liquidity: f64,
    volume_24h: f64,
}
```

**Key Functions:**
- `fetch_from_api()` — Pull live prices
- `apply_drift()` — Simulate price movement
- `get_midprice()` — Calculate mid-market price
- `get_spread()` — YES + NO - 1

---

### 1.3 Constraint Module (`constraint.rs`)

```rust
enum Constraint {
    Binary { yes_market: MarketId, no_market: MarketId },
    MultiOutcome { markets: Vec<MarketId> },
}
```

**Key Functions:**
- `check_violation()` — Returns spread if sum ≠ 1
- `get_mispriced_side()` — Which side to trade
- `compute_theoretical_edge()` — Raw arbitrage size

---

## Phase 2: Execution Simulation

### 2.1 Fee Model

```rust
struct FeeModel {
    taker_fee: f64,   // 0.02 = 2%
    maker_fee: f64,   // Often 0
}

impl FeeModel {
    fn apply(&self, notional: f64, is_taker: bool) -> f64 {
        let rate = if is_taker { self.taker_fee } else { self.maker_fee };
        notional * rate
    }
}
```

---

### 2.2 Slippage Model

```rust
struct SlippageModel {
    k: f64,     // ~1.0
    alpha: f64, // 1.3 - 1.8
}

impl SlippageModel {
    fn effective_price(&self, price: f64, size: f64, liquidity: f64) -> f64 {
        let impact = self.k * (size / liquidity).powf(self.alpha);
        price * (1.0 + impact)
    }
}
```

---

### 2.3 Fill Model

```rust
struct FillModel {
    beta: f64,  // Aggressiveness factor
}

impl FillModel {
    fn fill_ratio(&self, size: f64, liquidity: f64) -> f64 {
        (liquidity / (liquidity + size * self.beta)).min(1.0)
    }
}
```

---

### 2.4 Latency Model

```rust
struct LatencyModel {
    mean_delay_ms: u64,
    adverse_move_std: f64,
}

impl LatencyModel {
    fn apply(&self, signal_price: f64) -> (f64, Duration) {
        let delay = sample_delay(self.mean_delay_ms);
        let adverse = sample_normal(0.0, self.adverse_move_std);
        (signal_price * (1.0 + adverse), delay)
    }
}
```

---

### 2.5 Execution Engine

```rust
struct ExecutionEngine {
    fee_model: FeeModel,
    slippage_model: SlippageModel,
    fill_model: FillModel,
    latency_model: LatencyModel,
}

impl ExecutionEngine {
    fn execute(&self, order: Order, wallet: &mut Wallet) -> TradeResult {
        // 1. Apply latency
        let (exec_price, delay) = self.latency_model.apply(order.signal_price);
        
        // 2. Calculate slippage
        let slipped_price = self.slippage_model.effective_price(
            exec_price, order.size, order.market.liquidity
        );
        
        // 3. Determine fill
        let fill_ratio = self.fill_model.fill_ratio(order.size, order.market.liquidity);
        let filled_size = order.size * fill_ratio;
        
        // 4. Calculate costs
        let notional = slipped_price * filled_size;
        let fee = self.fee_model.apply(notional, true);
        let total_cost = notional + fee;
        
        // 5. Execute if profitable
        if wallet.can_afford(total_cost) {
            wallet.deduct(total_cost);
            // Record position...
        }
        
        TradeResult { filled_size, exec_price: slipped_price, fee, delay }
    }
}
```

---

## Phase 3: Arbitrage Logic

### 3.1 Signal Detection (`arb.rs`)

```rust
fn detect_arbitrage(markets: &[Market], constraints: &[Constraint]) -> Vec<Signal> {
    let mut signals = vec![];
    
    for constraint in constraints {
        if let Some(violation) = constraint.check_violation(markets) {
            if violation.spread > MIN_SPREAD_THRESHOLD {
                signals.push(Signal {
                    constraint: constraint.clone(),
                    spread: violation.spread,
                    action: violation.recommended_action(),
                });
            }
        }
    }
    
    signals
}
```

---

### 3.2 Trade Decision

```rust
fn should_trade(signal: &Signal, engine: &ExecutionEngine, wallet: &Wallet) -> bool {
    let raw_edge = signal.spread;
    
    let expected_costs = 
        engine.fee_model.estimate(signal.size) +
        engine.slippage_model.estimate(signal.size, signal.liquidity) +
        engine.latency_model.estimate_adverse_cost();
    
    let expected_profit = raw_edge * signal.size - expected_costs;
    
    expected_profit > 0.0 && wallet.can_afford(signal.size)
}
```

---

### 3.3 Position Sizing

```rust
fn compute_size(signal: &Signal, wallet: &Wallet, market: &Market) -> f64 {
    let risk_limit = wallet.equity() * 0.02;        // Max 2% of capital
    let liquidity_limit = market.liquidity * 0.01;  // Max 1% of book
    let confidence_size = signal.spread / MAX_SPREAD * CONFIDENCE_CAP;
    
    risk_limit.min(liquidity_limit).min(confidence_size)
}
```

---

## Phase 4: Main Loop

### 4.1 Engine Module (`engine.rs`)

```rust
fn main_loop(
    wallet: &mut Wallet,
    markets: &mut [Market],
    constraints: &[Constraint],
    engine: &ExecutionEngine,
) {
    loop {
        // 1. Update prices
        for market in markets.iter_mut() {
            market.fetch_from_api();
            market.apply_drift();
        }
        
        // 2. Detect arbitrage
        let signals = detect_arbitrage(markets, constraints);
        
        // 3. Evaluate and execute
        for signal in signals {
            if should_trade(&signal, engine, wallet) {
                let size = compute_size(&signal, wallet, &signal.market);
                let order = Order::new(signal, size);
                engine.execute(order, wallet);
            }
        }
        
        // 4. Check for mean reversion exits
        try_close_positions(wallet, markets, engine);
        
        // 5. Log PnL
        log_equity(wallet);
        
        // 6. Rate limit
        sleep(TICK_INTERVAL);
    }
}
```

---

## Phase 5: API Integration

### 5.1 Polymarket Client

```rust
struct PolymarketClient {
    base_url: String,
    api_key: Option<String>,
    rate_limiter: RateLimiter,
}

impl PolymarketClient {
    async fn get_book(&self, market_id: &str) -> Result<OrderBook>;
    async fn get_trades(&self, market_id: &str) -> Result<Vec<Trade>>;
    async fn stream_prices(&self) -> Result<WebSocketStream>;
}
```

---

## Phase 6: Parameter Calibration

### 6.1 Data Collection

```rust
async fn calibrate_parameters(client: &PolymarketClient) -> ExecutionParameters {
    // Collect 100+ samples
    let trades = client.get_trades(market_id).await?;
    let books = collect_book_snapshots(client, 100).await?;
    
    // Fit models
    let fee_rate = compute_p95_fee_rate(&trades);
    let (k, alpha) = fit_slippage_curve(&books, &trades);
    let fill_rates = compute_fill_distribution(&trades);
    let latency_dist = measure_latency_distribution().await;
    
    ExecutionParameters { fee_rate, k, alpha, fill_rates, latency_dist }
}
```

---

## Testing Checklist

| Test | Command | Expected |
|------|---------|----------|
| Unit tests | `cargo test` | All pass |
| Paper trading | `cargo run --release` | Equity tracks correctly |
| Slippage model | `cargo test test_slippage` | Non-linear impact |
| Fee deduction | `cargo test test_fees` | Correct amounts |
| Partial fills | `cargo test test_fills` | < 100% fill rates |

---

## Deployment Order

1. ✅ Implement wallet & market modules
2. ✅ Add execution simulation (fees, slippage, fills)
3. ✅ Build arbitrage detection logic
4. ✅ Create main trading loop
5. ⬜ Integrate Polymarket API
6. ⬜ Calibrate from live data
7. ⬜ Run paper trading validation
8. ⬜ Deploy to Solana devnet

---

## Next Steps

After successful paper trading with real API parameters:

1. **Multi-market constraints** — Handle A + B + C = 1 scenarios
2. **Latency injection** — Randomize execution delays
3. **Monte Carlo** — Run 10,000 simulated trade sequences
4. **Market graph** — Model constraint dependencies
5. **Solana integration** — Real on-chain execution
