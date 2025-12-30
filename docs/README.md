# 🦈 PolyShark

> *"If markets contradict themselves, eat the contradiction."*

**PolyShark** is a paper-trading arbitrage bot for Polymarket that detects and exploits logical mispricing between linked prediction markets.

---

## 🎯 What It Does

- Uses a **simulated USDC wallet** (starting at `$10.00` daily limit)
- Simulates **linked markets** with realistic price drift
- Detects **logical arbitrage opportunities** (e.g., when YES + NO ≠ 1)
- Executes **buy + sell** pairs to capture the spread
- Waits for **mean reversion** before closing positions
- Tracks **realistic PnL** with fees, slippage, and execution costs

---

## 🧠 Architecture

```
wallet.rs        → USDC wallet & position tracking
market.rs        → Market simulation + price drift
constraint.rs    → Logical relationships (A + B = 1, etc.)
arb.rs           → Arbitrage detection & signal logic
engine.rs        → Main trading loop
```

---

## 📊 What to Expect

| Behavior | Description |
|----------|-------------|
| Equity fluctuation | Normal — reflects market noise |
| Small wins | Most trades capture modest spreads |
| Rare drawdowns | Expected from adverse moves |
| Mean reversion | Clear profit when prices correct |
| Logic breaks | Easily visible when constraints are violated |

---

## 🔧 Execution Realism

PolyShark models **real execution dynamics**:

| Parameter | Description |
|-----------|-------------|
| **Fees** | Taker/maker fees applied per trade |
| **Slippage** | Non-linear price impact based on order size |
| **Partial Fills** | Orders may not fully execute |
| **Latency** | Delay between signal and execution |
| **Position Sizing** | Dynamic sizing based on risk & liquidity |

---

## 🚀 Upgrade Path

| Paper Bot | Solana Version |
|-----------|----------------|
| `Wallet.usdc` | SPL Token balance |
| `Market.price` | On-chain price oracle |
| `try_arbitrage()` | Atomic transaction |
| `try_close()` | Exit transaction |

---

## 📚 Documentation

### Core Concepts
- [**context.md**](./context.md) — Project background & "Why PolyShark?"
- [**maths.md**](./maths.md) — Mathematical foundations of arbitrage detection
- [**formulaes.md**](./formulaes.md) — Detailed financial formulas (Kelly Criterion, etc.)
- [**polymarket.md**](./polymarket.md) — Polymarket API reference

### Technical Specifications
- [**spec.md**](./spec.md) — Full generic system specification
- [**metamask/v1.md**](./metamask/v1.md) — **Hackathon Architecture: ERC-7715 Integration**
- [**implementation.md**](./implementation.md) — Implementation log

---

## 📈 Roadmap & Status

| Feature | Status | Notes |
|---------|--------|-------|
| **Multi-market constraints** | ✅ Done | `constraint.rs` (Generalized) |
| **Random latency injection** | ✅ Done | `latency.rs` (50ms base + drift) |
| **Fee modeling refinement** | ✅ Done | `fee_calibrator.rs` (P95 logic) |
| **Monte Carlo simulation** | ✅ Done | `simulation.rs` |
| **Market dependency graph** | ✅ Done | Covered by generalized constraints |
| Solana devnet deployment | ⏳ Pending | Future work |

---

## 🛠️ Tech Stack

- **Language**: Rust
- **Target**: Polymarket CLOB API
- **Future**: Solana blockchain integration

---

## 📄 License

MIT License — See [LICENSE](./LICENSE) for details.
