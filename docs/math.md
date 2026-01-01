# 📐 PolyShark — Mathematical Foundations & Formulas

All math concepts and formulas used in the PolyShark arbitrage bot.

---

## 1. Core Arbitrage Principle

In prediction markets, binary outcomes must satisfy:

```
P(YES) + P(NO) = 1
```

When this constraint is violated, an **arbitrage opportunity** exists.

### Example Mispricing

| Outcome | Price |
|---------|-------|
| YES | 0.62 |
| NO | 0.43 |
| **Sum** | **1.05** |

The 5% excess represents a **logical arbitrage spread**.

### Why This Works
- One outcome MUST happen (YES or NO)
- Winner pays out $1.00
- If you buy both for $0.95, you profit $0.05 guaranteed

---

## 2. Multi-Outcome Constraints

For markets with multiple mutually exclusive outcomes:

```
P(A) + P(B) + P(C) + ... + P(N) = 1
```

**Arbitrage condition:**
```
Σ P(i) ≠ 1  →  opportunity exists
```

| Scenario | Sum | Action |
|----------|-----|--------|
| Sum > 1 | Overpriced | Sell all sides |
| Sum < 1 | Underpriced | Buy all sides |

---

## 3. Order Book Formulas

### Best Bid & Best Ask
```
Best Bid = MAX(all bid prices)     // Highest price someone will BUY at
Best Ask = MIN(all ask prices)     // Lowest price someone will SELL at
```

### Midpoint Price
```
Midpoint = (Best_Bid + Best_Ask) / 2
```

### Bid-Ask Spread
```
Spread = Best_Ask - Best_Bid
```

### Arbitrage Spread
```
Spread = |1.0 - (YES_price + NO_price)|
```

---

## 4. Execution Price (VWAP)

Volume-Weighted Average Price — what you actually pay when buying.

```
For buying SIZE tokens:
    remaining = SIZE
    total_cost = 0
    
    For each ask level (lowest price first):
        fill = min(remaining, level.size)
        total_cost += fill × level.price
        remaining -= fill
        if remaining == 0: break
    
    VWAP = total_cost / SIZE
```

**Example:** Buy 600 tokens
```
Asks:
  0.51 × 400 tokens = $204
  0.52 × 200 tokens = $104 (only need 200 more)
  
Total Cost = $308
VWAP = 308 / 600 = $0.5133
```

---

## 5. Fee Model

Fees are modeled as a percentage of notional value:

### Convert Basis Points to Decimal
```
Fee_Rate = taker_base_fee / 10000
```

**Example:**
```
200 basis points = 200 / 10000 = 0.02 = 2%
```

### Calculate Fee
```
Fee = Notional × Fee_Rate
```

Use **95th percentile** of observed fee rates to model worst-case scenarios.

---

## 6. Slippage Model

Slippage is **non-linear** and punishes large orders:

```
effective_price = price × (1 + k × (size / liquidity)^α)
```

### Parameters

| Parameter | Typical Value | Meaning |
|-----------|---------------|---------|
| `k` | 1.0 | Base impact coefficient |
| `α` | 1.3 – 1.8 | Impact exponent (>1 means superlinear) |

### Slippage Calculation
```
Slippage = |Execution_Price - Midpoint| / Midpoint
```

**Example:**
```
Midpoint = 0.50
Execution = 0.52

Slippage = |0.52 - 0.50| / 0.50 = 0.04 = 4%
```

---

## 7. Partial Fill Model

Not all orders fill completely:

```
fill_ratio = min(1.0, liquidity / (liquidity + size × β))
filled_size = requested_size × fill_ratio
```

Where `β` controls fill aggressiveness.

Unfilled portions represent **dead opportunity cost**.

---

## 8. Latency & Adverse Selection

The price you see is **never** the price you get.

```
observed_price → wait Δt → execute at drifted_price
```

### Adverse Move Calculation
```
latency = t_exec - t_signal
adverse_move = price_exec - price_signal
```

Fast markets and thin liquidity produce **worse adverse moves**.

---

## 9. Expected Profit Formula

When a signal fires:

```
raw_edge = |1 - Σ prices|

expected_costs = fee_estimate 
               + slippage_estimate(size) 
               + adverse_selection_estimate

expected_profit = raw_edge × size - expected_costs
```

### Decision Gate (Critical)

```
if expected_profit ≤ 0:
    skip trade
```

This single rule eliminates **80% of unprofitable trades**.

---

## 10. Position Sizing

Dynamic sizing prevents oversizing disasters:

```
size = min(
    wallet_equity × risk_pct,        // max 2% of capital
    liquidity × liquidity_pct,       // max 1% of book
    (edge / max_edge) × confidence_cap  // confidence scaling
)
```

### Data-Driven Sizing

Plot `Expected PnL vs trade size` from historical data to find:

```
max_size = argmax(ExpectedPnL(size))
```

---

## 11. Profit/Loss (PnL)

### For a Long (Buy) Position
```
PnL = (Exit_Price - Entry_Price) × Size
```

### For a Short (Sell) Position
```
PnL = (Entry_Price - Exit_Price) × Size
```

---

## 12. Win Rate & Equity

### Win Rate
```
Win_Rate = Winning_Trades / Total_Trades
```

**Target:** Win Rate > 50% after all costs

### Equity
```
Equity = Cash + Σ(Position_Size × Current_Price)
```

---

## 13. Mean Reversion Model

After entering a position, profit is realized when prices **revert to fair value**:

```
closing_spread = entry_spread - exit_spread
profit = closing_spread × position_size - round_trip_costs
```

### Exit Conditions

1. Spread narrows to threshold (profit target)
2. Time limit exceeded (cut losses)
3. Constraint re-established (`P(YES) + P(NO) = 1`)

---

## 14. Statistical Validation

To confirm real edge exists:

| Metric | Requirement |
|--------|-------------|
| **Trade count** | ≥ 1,000 trades |
| **Win rate** | > 50% after costs |
| **Sharpe ratio** | > 1.0 annualized |
| **Max drawdown** | < 15% of peak |

---

## Quick Reference Card

| Formula | Equation |
|---------|----------|
| Midpoint | `(bid + ask) / 2` |
| Spread | `ask - bid` |
| Arb Spread | `\|1 - (YES + NO)\|` |
| Fee | `notional × (bps / 10000)` |
| VWAP | `Σ(fill × price) / total_size` |
| Slippage | `\|exec - mid\| / mid` |
| Expected Profit | `edge - fees - slippage` |
| Fill Ratio | `liquidity / size` |
| Equity | `cash + Σ(pos × price)` |

---

## Code Mapping

### `types.rs`

| Formula | Function | Code |
|---------|----------|------|
| Best Bid | `OrderBook::best_bid()` | `bids.first().map(\|l\| l.price)` |
| Midpoint | `OrderBook::midpoint()` | `(bid + ask) / 2.0` |
| VWAP | `OrderBook::execution_price()` | Walk book, `total_cost / size` |
| Arb Spread | `Market::get_spread()` | `\|sum - 1.0\|` |

### `fees.rs`

| Formula | Function | Code |
|---------|----------|------|
| Fee | `FeeModel::calculate()` | `notional × (bps / 10000.0)` |

### `slippage.rs`

| Formula | Function | Code |
|---------|----------|------|
| Slippage | `SlippageModel::calculate()` | `(exec_price - midpoint) / midpoint` |

### `arb.rs`

| Formula | Function | Code |
|---------|----------|------|
| Expected Profit | `ArbitrageDetector::expected_profit()` | `gross - fee_cost - slippage_cost` |
| Should Trade | `ArbitrageDetector::should_trade()` | `expected_profit > min_threshold` |

### `wallet.rs`

| Formula | Function | Code |
|---------|----------|------|
| Equity | `Wallet::equity()` | `usdc + Σ(pos.size × current_price)` |
| PnL | `Wallet::pnl()` | `equity - starting_balance` |
| Win Rate | `Wallet::win_rate()` | `winning_trades / total_trades` |
