# 📐 PolyShark — Mathematical Foundations

This document explains the mathematical models that power PolyShark's arbitrage detection and execution simulation.

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

## 3. Slippage Model

Slippage is **non-linear** and punishes large orders:

```
effective_price = price × (1 + k × (size / liquidity)^α)
```

### Parameters

| Parameter | Typical Value | Meaning |
|-----------|---------------|---------|
| `k` | 1.0 | Base impact coefficient |
| `α` | 1.3 – 1.8 | Impact exponent (>1 means superlinear) |

### Empirical Derivation

From real order book data:

```
slippage = (exec_price - mid_price) / mid_price
```

Fit `k` and `α` using least squares on observed trades, grouped by `size / liquidity`.

---

## 4. Fee Model

Fees are modeled as a percentage of notional value:

```rust
struct FeeModel {
    taker_fee: f64,   // e.g., 0.02 = 2%
    maker_fee: f64,   // often 0
}
```

### Application

```
fee = notional × fee_rate
net_cost = notional + fee
```

Use **95th percentile** of observed fee rates to model worst-case scenarios.

---

## 5. Partial Fill Model

Not all orders fill completely, especially when chasing mispricing:

```
fill_ratio = min(1.0, liquidity / (liquidity + size × β))
```

Where `β` controls fill aggressiveness.

### Practical Application

```
filled_size = requested_size × fill_ratio
unfilled = requested_size - filled_size
```

Unfilled portions represent **dead opportunity cost**.

---

## 6. Latency & Adverse Selection

The price you see is **never** the price you get.

### Latency Model

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

## 7. Expected Profit Formula

When a signal fires, compute:

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

## 8. Position Sizing

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

Use this as the upper bound.

---

## 9. Mean Reversion Model

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

## 10. Statistical Validation

To confirm real edge exists:

| Metric | Requirement |
|--------|-------------|
| **Trade count** | ≥ 1,000 trades |
| **Win rate** | > 50% after costs |
| **Sharpe ratio** | > 1.0 annualized |
| **Max drawdown** | < 15% of peak |

If PnL is positive after 1,000+ trades with realistic execution:

✅ **Real edge confirmed**  
❌ If not, it was **fake alpha**

---

## Summary

PolyShark uses these mathematical models to:

1. Detect constraint violations (Sum ≠ 1)
2. Estimate true execution costs
3. Filter unprofitable trades
4. Size positions appropriately
5. Validate edge statistically

This transforms paper profits into **credible, data-driven trading logic**.
