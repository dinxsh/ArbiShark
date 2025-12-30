# 🦈 PolyShark — Implementation Steps

Step-by-step guide to implement PolyShark yourself. All execution parameters (fees, slippage, fills) come from Polymarket API. Only wallet balance is simulated locally.

---

## Phase 1: Project Setup

### Step 1.1: Initialize Rust Project
```bash
cargo new polyshark
cd polyshark
```

### Step 1.2: Add Dependencies
Add to `Cargo.toml`:
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio-tungstenite = { version = "0.20", features = ["native-tls"] }
futures-util = "0.3"
```

---

## Phase 2: Polymarket API Client

### Step 2.1: Create API Types
Create `src/types.rs`:
- [ ] Define `Market` struct (id, question, outcomes, outcomePrices, clobTokenIds, bestBid, bestAsk, makerBaseFee, takerBaseFee)
- [ ] Define `OrderBook` struct (bids, asks, timestamp)
- [ ] Define `Trade` struct (price, size, side, timestamp)
- [ ] Define `PriceLevel` struct (price, size)

### Step 2.2: Create Gamma API Client
Create `src/gamma.rs`:
- [ ] Implement `get_markets()` → fetch from `https://gamma-api.polymarket.com/markets`
- [ ] Parse `clobTokenIds` and `outcomePrices` for each market
- [ ] Extract `makerBaseFee`, `takerBaseFee` for fee calculation

### Step 2.3: Create CLOB API Client
Create `src/clob.rs`:
- [ ] Implement `get_book(token_id)` → fetch order book
- [ ] Implement `get_midpoint(token_id)` → fetch midpoint price
- [ ] Implement `get_price(token_id, side)` → fetch BUY/SELL price
- [ ] Handle rate limiting with exponential backoff

---

## Phase 3: Wallet Simulation

### Step 3.1: Create Wallet Module
Create `src/wallet.rs`:
- [ ] Define `Wallet` struct with `usdc: f64` (starting at 1000.0)
- [ ] Define `Position` struct (token_id, side, size, entry_price)
- [ ] Implement `can_afford(amount)` → check balance
- [ ] Implement `deduct(amount)` → subtract from usdc
- [ ] Implement `credit(amount)` → add to usdc
- [ ] Implement `get_equity()` → cash + position value

---

## Phase 4: Fee & Slippage Models (From API)

### Step 4.1: Create Fee Model
Create `src/fees.rs`:
- [ ] Extract `takerBaseFee` from market data (in basis points)
- [ ] Implement `calculate_fee(notional, fee_bps)` → `notional * fee_bps / 10000`
- [ ] Apply fee on every trade

### Step 4.2: Create Slippage Model
Create `src/slippage.rs`:
- [ ] Fetch order book depth from `/book` endpoint
- [ ] Walk the book to calculate execution price for given size
- [ ] Implement `calculate_slippage(book, size, side)` → actual execution price
- [ ] Slippage = difference between midpoint and execution price

### Step 4.3: Create Fill Model
Create `src/fills.rs`:
- [ ] Check available liquidity from order book
- [ ] Implement `estimate_fill_ratio(book, size)` → how much can actually fill
- [ ] Apply partial fill logic

---

## Phase 5: Arbitrage Detection

### Step 5.1: Create Constraint Checker
Create `src/constraint.rs`:
- [ ] For binary markets: check if `P(YES) + P(NO) = 1`
- [ ] Calculate spread: `spread = |1 - (yes_price + no_price)|`
- [ ] Define `MIN_SPREAD_THRESHOLD` (e.g., 0.02 for 2%)
- [ ] Return arbitrage signal if spread > threshold

### Step 5.2: Create Arbitrage Logic
Create `src/arb.rs`:
- [ ] Scan all active markets for constraint violations
- [ ] Calculate expected profit: `edge * size - fees - slippage`
- [ ] Only signal if `expected_profit > 0`
- [ ] Implement position sizing based on edge and liquidity

---

## Phase 6: Execution Engine

### Step 6.1: Create Execution Simulator
Create `src/execution.rs`:
- [ ] Combine fee, slippage, and fill models
- [ ] Simulate order execution with realistic costs
- [ ] Update wallet after each trade
- [ ] Log all execution details

### Step 6.2: Implement Decision Gate
In execution logic:
```rust
if expected_profit <= 0.0 {
    return None; // Skip trade
}
```

---

## Phase 7: Main Trading Loop

### Step 7.1: Create Engine Module
Create `src/engine.rs`:
- [ ] Initialize wallet with starting balance
- [ ] Poll markets at regular interval
- [ ] Detect arbitrage opportunities
- [ ] Execute trades through simulation
- [ ] Track and log PnL

### Step 7.2: Implement Main Loop
```rust
loop {
    // 1. Fetch latest prices from Polymarket
    // 2. Check for arbitrage opportunities
    // 3. Calculate expected costs (fees + slippage from API data)
    // 4. Execute if profitable
    // 5. Update wallet
    // 6. Log equity
    // 7. Sleep (respect rate limits)
}
```

---

## Phase 8: WebSocket Streaming (Optional)

### Step 8.1: Add Real-Time Prices
Create `src/websocket.rs`:
- [ ] Connect to `wss://ws-subscriptions-clob.polymarket.com/ws`
- [ ] Subscribe to `market` channel for price updates
- [ ] Update local price cache on each message
- [ ] Replace polling with streaming for faster signals

---

## Phase 9: Testing & Validation

### Step 9.1: Unit Tests
- [ ] Test fee calculation
- [ ] Test slippage calculation from order book
- [ ] Test constraint checking (YES + NO = 1)
- [ ] Test wallet updates

### Step 9.2: Integration Tests
- [ ] Test API client against live Polymarket
- [ ] Verify order book parsing
- [ ] Run paper trading for 1000+ simulated trades

### Step 9.3: Validation Criteria
- [ ] Win rate > 50% after all costs
- [ ] Positive PnL after 1000 trades
- [ ] No unexpected drawdowns

---

## Quick Reference: What Comes From Where

| Component | Source |
|-----------|--------|
| Wallet balance | **Local simulation** (starts at $1000) |
| Market prices | Polymarket GAMMA API |
| Order book depth | Polymarket CLOB `/book` |
| Fees (maker/taker) | Market `makerBaseFee`, `takerBaseFee` |
| Slippage | Calculated from order book |
| Fill rates | Calculated from order book liquidity |
| Latency | Measured from API response times |

---

## File Structure

```
polyshark/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point
│   ├── types.rs         # API types & structs
│   ├── gamma.rs         # GAMMA API client
│   ├── clob.rs          # CLOB API client
│   ├── wallet.rs        # Wallet simulation
│   ├── fees.rs          # Fee model (from API)
│   ├── slippage.rs      # Slippage model (from order book)
│   ├── fills.rs         # Fill model (from order book)
│   ├── constraint.rs    # Logical constraints
│   ├── arb.rs           # Arbitrage detection
│   ├── execution.rs     # Execution simulator
│   ├── engine.rs        # Main trading loop
│   └── websocket.rs     # Optional: real-time streaming
└── tests/
    └── integration.rs   # Integration tests
```

---

## Progress Tracker

- [ ] Phase 1: Project Setup
- [ ] Phase 2: Polymarket API Client
- [ ] Phase 3: Wallet Simulation
- [ ] Phase 4: Fee & Slippage Models
- [ ] Phase 5: Arbitrage Detection
- [ ] Phase 6: Execution Engine
- [ ] Phase 7: Main Trading Loop
- [ ] Phase 8: WebSocket Streaming
- [ ] Phase 9: Testing & Validation

---

Good luck! Check off each step as you complete it. 🦈
