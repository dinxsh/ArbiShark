# PolyShark — Project Context

> *"If markets contradict themselves, eat the contradiction."*

This document explains the philosophy and design principles behind PolyShark.

---

## What PolyShark Does

PolyShark is an **arbitrage bot** for Polymarket that:

1. Uses a **permission-safe wallet** (ERC-7715)
2. Consumes **real market data** from Polymarket APIs
3. Detects **logical arbitrage** (when YES + NO ≠ 1)
4. Executes trades with **realistic costs**
5. Tracks **PnL** through all market conditions

---

## The Core Idea: Execution Simulator

Your wallet is just a float. What matters is **how trades execute**.

```
Strategy  →  Execution Simulator  →  Wallet Update
```

The simulator models reality:
- **Fees** (taker/maker from API)
- **Slippage** (non-linear, from order book)
- **Partial fills** (not all orders complete)
- **Latency** (price moves before you execute)

---

## The Realistic Execution Model

### Fees
```rust
fee = notional * fee_rate;
wallet.usdc -= fee;
```

### Slippage (Non-Linear)
```
effective_price = price * (1 + k * (size / liquidity)^α)
```
Where `k ≈ 1.0` and `α ≈ 1.3–1.8`. This punishes oversizing hard.

### Partial Fills
```rust
fill_ratio = min(1.0, liquidity / (liquidity + size * beta))
filled_size = size * fill_ratio
```

### Latency
```rust
observed_price → wait Δt → execute at drifted_price
```

---

## The Decision Gate

When a signal fires:

```
1. observe spread
2. compute expected profit
3. subtract fees
4. subtract slippage
5. apply latency penalty
6. size position
7. simulate partial fill
8. update wallet
```

**Critical rule:**
```
if expected_profit <= 0:
    skip trade
```

This single rule saves money.

---

## Why Paper Trading Works

Because we have the Polymarket API:

- ❌ We do NOT need to guess
- ❌ We do NOT need perfect fills
- ❌ We do NOT need to trade yet

We can:
1. Extract real market behavior
2. Build a hostile simulator
3. Validate the edge properly

This is **exactly how professional arb desks operate**.

---

## Statistical Validation

If PnL is positive after **1,000+ trades** with realistic execution:

✅ **Real edge confirmed**
❌ If not, it was **fake alpha**

Most strategies die here. That's a good thing.

---

## Upgrade Path

| Paper Bot | Production Version |
|-----------|-------------------|
| `Wallet.usdc` | Smart Account balance |
| `Market.price` | Envio-indexed price |
| `try_arbitrage()` | ERC-7715 permissioned tx |
| `try_close()` | Exit transaction |

---

## Key Best Practices

1. **Use P95, not average** — Model worst-case fees and slippage
2. **Inflate parameters** — Add 20-30% buffer in simulation
3. **Use WebSockets** — Minimize latency, avoid rate limits
4. **Cache responses** — Polymarket throttles requests
5. **Size dynamically** — Never use fixed trade sizes

---

## Rate Limits

| Endpoint | Approximate Limit |
|----------|-------------------|
| Price endpoints | ~1500 requests/10s |
| Trade/order data | ~900 requests/10s |

Use WebSocket streaming to avoid hitting limits.

---

## Further Reading

- [**math.md**](./math.md) — All formulas
- [**spec.md**](./spec.md) — Technical specification
- [**polymarket.md**](./polymarket.md) — API reference
- [**metamask/v1.md**](./metamask/v1.md) — ERC-7715 architecture
