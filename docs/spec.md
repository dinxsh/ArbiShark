# 🦈 PolyShark — Technical Specification

Detailed breakdown of all structs, traits, implementations, and functions for each module.

---

## 📁 `src/types.rs` — Core Data Types

### Structs

```rust
use serde::{Deserialize, Serialize};

/// Represents a Polymarket prediction market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: String,
    pub question: String,
    pub slug: String,
    pub outcomes: Vec<String>,           // ["Yes", "No"]
    pub outcome_prices: Vec<f64>,        // [0.65, 0.35]
    pub clob_token_ids: Vec<String>,     // Token IDs for trading
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub maker_base_fee: u32,             // In basis points (e.g., 0)
    pub taker_base_fee: u32,             // In basis points (e.g., 200 = 2%)
    pub liquidity: f64,
    pub volume_24hr: f64,
    pub active: bool,
    pub accepting_orders: bool,
}

/// Single price level in order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}

/// Order book for a single token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub token_id: String,
    pub bids: Vec<PriceLevel>,           // Sorted highest first
    pub asks: Vec<PriceLevel>,           // Sorted lowest first
    pub timestamp: u64,
}

/// Executed trade record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub token_id: String,
    pub price: f64,
    pub size: f64,
    pub side: Side,
    pub timestamp: u64,
}

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

/// Arbitrage signal
#[derive(Debug, Clone)]
pub struct ArbitrageSignal {
    pub market_id: String,
    pub spread: f64,                     // How much prices deviate from 1.0
    pub edge: f64,                       // Expected profit per unit
    pub recommended_side: Side,
    pub yes_price: f64,
    pub no_price: f64,
}

/// Execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub filled_size: f64,
    pub execution_price: f64,
    pub fee_paid: f64,
    pub slippage: f64,
    pub total_cost: f64,
    pub success: bool,
}
```

### Implementations for `Market`

```rust
impl Market {
    /// Check if prices sum to exactly 1.0 (no arbitrage)
    pub fn is_balanced(&self) -> bool {
        let sum: f64 = self.outcome_prices.iter().sum();
        (sum - 1.0).abs() < 0.001
    }

    /// Get the spread (deviation from balanced)
    pub fn get_spread(&self) -> f64 {
        let sum: f64 = self.outcome_prices.iter().sum();
        (sum - 1.0).abs()
    }

    /// Get YES token price (assumes binary market)
    pub fn yes_price(&self) -> f64 {
        self.outcome_prices.get(0).copied().unwrap_or(0.0)
    }

    /// Get NO token price (assumes binary market)
    pub fn no_price(&self) -> f64 {
        self.outcome_prices.get(1).copied().unwrap_or(0.0)
    }

    /// Get taker fee as decimal (e.g., 0.02 for 2%)
    pub fn taker_fee_rate(&self) -> f64 {
        self.taker_base_fee as f64 / 10000.0
    }
}
```

### Implementations for `OrderBook`

```rust
impl OrderBook {
    /// Get best bid price
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|l| l.price)
    }

    /// Get best ask price
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|l| l.price)
    }

    /// Get midpoint price
    pub fn midpoint(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    /// Get bid-ask spread
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Total liquidity on bid side
    pub fn total_bid_liquidity(&self) -> f64 {
        self.bids.iter().map(|l| l.size).sum()
    }

    /// Total liquidity on ask side
    pub fn total_ask_liquidity(&self) -> f64 {
        self.asks.iter().map(|l| l.size).sum()
    }

    /// Calculate execution price for a given size (walks the book)
    pub fn execution_price(&self, size: f64, side: Side) -> Option<f64> {
        let levels = match side {
            Side::Buy => &self.asks,
            Side::Sell => &self.bids,
        };

        let mut remaining = size;
        let mut total_cost = 0.0;

        for level in levels {
            let fill = remaining.min(level.size);
            total_cost += fill * level.price;
            remaining -= fill;
            if remaining <= 0.0 {
                break;
            }
        }

        if remaining > 0.0 {
            None // Not enough liquidity
        } else {
            Some(total_cost / size) // Volume-weighted average price
        }
    }
}
```

---

## 📁 `src/wallet.rs` — Wallet Simulation

### Structs

```rust
use std::collections::HashMap;
use crate::types::Side;

/// Open position in a market
#[derive(Debug, Clone)]
pub struct Position {
    pub token_id: String,
    pub side: Side,
    pub size: f64,
    pub entry_price: f64,
    pub entry_time: u64,
}

/// Simulated wallet
#[derive(Debug, Clone)]
pub struct Wallet {
    pub usdc: f64,                              // Cash balance
    pub positions: HashMap<String, Position>,   // token_id -> Position
    pub starting_balance: f64,
    pub total_fees_paid: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
}
```

### Implementations for `Wallet`

```rust
impl Wallet {
    /// Create new wallet with starting balance
    pub fn new(starting_balance: f64) -> Self {
        Self {
            usdc: starting_balance,
            positions: HashMap::new(),
            starting_balance,
            total_fees_paid: 0.0,
            total_trades: 0,
            winning_trades: 0,
        }
    }

    /// Check if wallet can afford a purchase
    pub fn can_afford(&self, amount: f64) -> bool {
        self.usdc >= amount
    }

    /// Deduct amount from wallet
    pub fn deduct(&mut self, amount: f64) -> bool {
        if self.can_afford(amount) {
            self.usdc -= amount;
            true
        } else {
            false
        }
    }

    /// Credit amount to wallet
    pub fn credit(&mut self, amount: f64) {
        self.usdc += amount;
    }

    /// Add fee to tracking
    pub fn record_fee(&mut self, fee: f64) {
        self.total_fees_paid += fee;
    }

    /// Record a trade result
    pub fn record_trade(&mut self, is_winner: bool) {
        self.total_trades += 1;
        if is_winner {
            self.winning_trades += 1;
        }
    }

    /// Get current equity (cash + position value)
    pub fn equity(&self, current_prices: &HashMap<String, f64>) -> f64 {
        let position_value: f64 = self.positions.iter()
            .map(|(token_id, pos)| {
                let current_price = current_prices.get(token_id).unwrap_or(&0.0);
                pos.size * current_price
            })
            .sum();
        self.usdc + position_value
    }

    /// Get profit/loss from starting balance
    pub fn pnl(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.equity(current_prices) - self.starting_balance
    }

    /// Get win rate
    pub fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            self.winning_trades as f64 / self.total_trades as f64
        }
    }

    /// Open a new position
    pub fn open_position(&mut self, token_id: String, side: Side, size: f64, price: f64, timestamp: u64) {
        self.positions.insert(token_id.clone(), Position {
            token_id,
            side,
            size,
            entry_price: price,
            entry_time: timestamp,
        });
    }

    /// Close a position and return PnL
    pub fn close_position(&mut self, token_id: &str, exit_price: f64) -> Option<f64> {
        if let Some(pos) = self.positions.remove(token_id) {
            let pnl = match pos.side {
                Side::Buy => (exit_price - pos.entry_price) * pos.size,
                Side::Sell => (pos.entry_price - exit_price) * pos.size,
            };
            self.credit(pos.size * exit_price);
            Some(pnl)
        } else {
            None
        }
    }
}
```

---

## 📁 `src/fees.rs` — Fee Model

### Struct

```rust
/// Fee model based on Polymarket fee structure
#[derive(Debug, Clone)]
pub struct FeeModel {
    pub maker_fee_bps: u32,   // Basis points (usually 0)
    pub taker_fee_bps: u32,   // Basis points (usually ~200)
}
```

### Implementations

```rust
impl FeeModel {
    /// Create from market data
    pub fn from_market(market: &Market) -> Self {
        Self {
            maker_fee_bps: market.maker_base_fee,
            taker_fee_bps: market.taker_base_fee,
        }
    }

    /// Calculate fee for a trade
    pub fn calculate(&self, notional: f64, is_maker: bool) -> f64 {
        let bps = if is_maker { self.maker_fee_bps } else { self.taker_fee_bps };
        notional * (bps as f64 / 10000.0)
    }

    /// Get taker fee as decimal
    pub fn taker_rate(&self) -> f64 {
        self.taker_fee_bps as f64 / 10000.0
    }
}
```

---

## 📁 `src/slippage.rs` — Slippage Model

### Struct

```rust
/// Slippage calculator using order book
#[derive(Debug, Clone)]
pub struct SlippageModel;
```

### Implementations

```rust
impl SlippageModel {
    /// Calculate slippage from order book
    pub fn calculate(book: &OrderBook, size: f64, side: Side) -> Option<f64> {
        let midpoint = book.midpoint()?;
        let exec_price = book.execution_price(size, side)?;
        
        let slippage = match side {
            Side::Buy => (exec_price - midpoint) / midpoint,
            Side::Sell => (midpoint - exec_price) / midpoint,
        };
        
        Some(slippage)
    }

    /// Estimate execution cost including slippage
    pub fn execution_cost(book: &OrderBook, size: f64, side: Side) -> Option<f64> {
        let exec_price = book.execution_price(size, side)?;
        Some(exec_price * size)
    }
}
```

---

## 📁 `src/fills.rs` — Fill Model

### Struct

```rust
/// Fill rate estimator
#[derive(Debug, Clone)]
pub struct FillModel;
```

### Implementations

```rust
impl FillModel {
    /// Estimate how much of the order can fill
    pub fn estimate_fill_ratio(book: &OrderBook, size: f64, side: Side) -> f64 {
        let available = match side {
            Side::Buy => book.total_ask_liquidity(),
            Side::Sell => book.total_bid_liquidity(),
        };

        if available >= size {
            1.0
        } else {
            available / size
        }
    }

    /// Get filled size based on available liquidity
    pub fn filled_size(book: &OrderBook, requested_size: f64, side: Side) -> f64 {
        let ratio = Self::estimate_fill_ratio(book, requested_size, side);
        requested_size * ratio
    }
}
```

---

## 📁 `src/constraint.rs` — Arbitrage Constraints

### Struct

```rust
/// Binary market constraint checker
#[derive(Debug, Clone)]
pub struct ConstraintChecker {
    pub min_spread_threshold: f64,  // e.g., 0.02 for 2%
}
```

### Implementations

```rust
impl ConstraintChecker {
    pub fn new(min_spread_threshold: f64) -> Self {
        Self { min_spread_threshold }
    }

    /// Check if market has arbitrage opportunity
    pub fn check_violation(&self, market: &Market) -> Option<ArbitrageSignal> {
        let spread = market.get_spread();
        
        if spread <= self.min_spread_threshold {
            return None; // No opportunity
        }

        let sum = market.yes_price() + market.no_price();
        let recommended_side = if sum > 1.0 {
            Side::Sell // Prices are overvalued
        } else {
            Side::Buy  // Prices are undervalued
        };

        Some(ArbitrageSignal {
            market_id: market.id.clone(),
            spread,
            edge: spread, // Gross edge before costs
            recommended_side,
            yes_price: market.yes_price(),
            no_price: market.no_price(),
        })
    }
}
```

---

## 📁 `src/arb.rs` — Arbitrage Logic

### Struct

```rust
/// Arbitrage detector
#[derive(Debug)]
pub struct ArbitrageDetector {
    pub constraint_checker: ConstraintChecker,
    pub min_profit_threshold: f64,  // Minimum expected profit to trade
}
```

### Implementations

```rust
impl ArbitrageDetector {
    pub fn new(min_spread: f64, min_profit: f64) -> Self {
        Self {
            constraint_checker: ConstraintChecker::new(min_spread),
            min_profit_threshold: min_profit,
        }
    }

    /// Scan markets for arbitrage opportunities
    pub fn scan(&self, markets: &[Market]) -> Vec<ArbitrageSignal> {
        markets.iter()
            .filter(|m| m.active && m.accepting_orders)
            .filter_map(|m| self.constraint_checker.check_violation(m))
            .collect()
    }

    /// Calculate expected profit after costs
    pub fn expected_profit(
        &self,
        signal: &ArbitrageSignal,
        size: f64,
        fee_rate: f64,
        slippage: f64,
    ) -> f64 {
        let gross = signal.edge * size;
        let fee_cost = size * signal.yes_price * fee_rate * 2.0; // Both legs
        let slippage_cost = size * slippage;
        
        gross - fee_cost - slippage_cost
    }

    /// Decide if trade is worth taking
    pub fn should_trade(
        &self,
        signal: &ArbitrageSignal,
        size: f64,
        fee_rate: f64,
        slippage: f64,
    ) -> bool {
        self.expected_profit(signal, size, fee_rate, slippage) > self.min_profit_threshold
    }
}
```

---

## 📁 `src/execution.rs` — Execution Engine

### Struct

```rust
/// Execution simulator
#[derive(Debug)]
pub struct ExecutionEngine {
    pub fee_model: FeeModel,
}
```

### Implementations

```rust
impl ExecutionEngine {
    pub fn new(fee_model: FeeModel) -> Self {
        Self { fee_model }
    }

    /// Simulate order execution
    pub fn execute(
        &self,
        book: &OrderBook,
        size: f64,
        side: Side,
        wallet: &mut Wallet,
    ) -> Option<ExecutionResult> {
        // 1. Check fill ratio
        let filled_size = FillModel::filled_size(book, size, side);
        if filled_size <= 0.0 {
            return None;
        }

        // 2. Calculate execution price (with slippage)
        let exec_price = book.execution_price(filled_size, side)?;
        let midpoint = book.midpoint()?;
        let slippage = ((exec_price - midpoint) / midpoint).abs();

        // 3. Calculate costs
        let notional = exec_price * filled_size;
        let fee = self.fee_model.calculate(notional, false); // Taker
        let total_cost = notional + fee;

        // 4. Check if affordable
        if !wallet.can_afford(total_cost) {
            return None;
        }

        // 5. Execute
        wallet.deduct(total_cost);
        wallet.record_fee(fee);

        Some(ExecutionResult {
            filled_size,
            execution_price: exec_price,
            fee_paid: fee,
            slippage,
            total_cost,
            success: true,
        })
    }
}
```

---

## Quick Reference Table

| Module | Structs | Key Methods |
|--------|---------|-------------|
| `types.rs` | `Market`, `OrderBook`, `PriceLevel`, `Trade`, `Side`, `ArbitrageSignal`, `ExecutionResult` | `is_balanced()`, `get_spread()`, `execution_price()` |
| `wallet.rs` | `Wallet`, `Position` | `can_afford()`, `deduct()`, `credit()`, `equity()`, `pnl()` |
| `fees.rs` | `FeeModel` | `from_market()`, `calculate()` |
| `slippage.rs` | `SlippageModel` | `calculate()`, `execution_cost()` |
| `fills.rs` | `FillModel` | `estimate_fill_ratio()`, `filled_size()` |
| `constraint.rs` | `ConstraintChecker` | `check_violation()` |
| `arb.rs` | `ArbitrageDetector` | `scan()`, `expected_profit()`, `should_trade()` |
| `execution.rs` | `ExecutionEngine` | `execute()` |

---

No traits needed for this implementation — all functionality is via struct impls. You can add traits later for testing/mocking (e.g., `trait Exchange`, `trait WalletOps`).


## File Structure

```
polyshark/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point + trading loop
│   ├── types.rs          # Core data types (Market, OrderBook, Trade, Side, etc.)
│   ├── gamma.rs          # GAMMA API client (market discovery)
│   ├── clob.rs           # CLOB API client (order book, prices)
│   ├── wallet.rs         # Wallet simulation (local $1000)
│   ├── fees.rs           # Fee model (from API makerBaseFee/takerBaseFee)
│   ├── slippage.rs       # Slippage model (from order book depth)
│   ├── fills.rs          # Fill model (from order book liquidity)
│   ├── constraint.rs     # Arbitrage constraints (YES + NO = 1)
│   ├── arb.rs            # Arbitrage detection logic
│   ├── execution.rs      # Execution engine (combines all models)
│   └── websocket.rs      # Optional: real-time streaming
└── tests/
    └── integration.rs    # Integration tests
```

---

## Module Dependencies

```
main.rs
  └── engine (trading loop)
        ├── gamma.rs  ──────> types.rs (Market)
        ├── clob.rs   ──────> types.rs (OrderBook, PriceLevel)
        ├── wallet.rs ──────> types.rs (Side)
        ├── arb.rs    ──────> constraint.rs ──> types.rs (ArbitrageSignal)
        └── execution.rs
              ├── fees.rs      ──> types.rs (Market)
              ├── slippage.rs  ──> types.rs (OrderBook, Side)
              └── fills.rs     ──> types.rs (OrderBook, Side)
```

---

## Order of Implementation

| Step | File | Description |
|------|------|-------------|
| 1 | `types.rs` | Define all shared structs first |
| 2 | `wallet.rs` | Local simulation (no API needed) |
| 3 | `fees.rs` | Simple calculations |
| 4 | `slippage.rs` | Uses OrderBook |
| 5 | `fills.rs` | Uses OrderBook |
| 6 | `constraint.rs` | Uses Market |
| 7 | `gamma.rs` | API client for markets |
| 8 | `clob.rs` | API client for order books |
| 9 | `arb.rs` | Combines constraint + market data |
| 10 | `execution.rs` | Combines fees + slippage + fills + wallet |
| 11 | `main.rs` | Wire everything together |

