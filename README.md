# 🦈 PolyShark

> *"If markets contradict themselves, eat the contradiction."*

**PolyShark** is a high-performance arbitrage agent for Polymarket that detects and exploits logical mispricing between linked prediction markets. It features **Live Data Hydration**, **Concurrent Order Book Fetching**, and **Solana Devnet Integration**.

---

## 🎯 What It Does

- **Live Data:** Streams real-time market data from **Polymarket Gamma** and **CLOB** APIs.
- **Concurrent Execution:** Hydrates order books in parallel (High-Frequency).
- **Permissioned Actions:** Enforces **ERC-7715** style permissions (Daily USDC Limits).
- **Multi-Chain Ready:** Connected to **Solana Devnet** for future on-chain execution.
- **Logical Arbitrage:** Detects when `YES + NO < 1.0` (Risk-free profit).

---

## 🚀 Quick Start

Ensure you have Rust installed.

```bash
# Run the bot (Live Mode)
cargo run
```

---

## 🧠 Architecture

```
main.rs          → Agent loop & orchestration
wallet.rs        → Permissioned Wallet (ERC-7715 logic)
market.rs        → Live Data Provider (Gamma/CLOB) [Concurrent]
solana.rs        → Solana Devnet Connection
constraint.rs    → Logical relationships (A + B = 1, etc.)
arb.rs           → Arbitrage detection & signal logic
```

---

## 📊 Performance

| Metric | Value |
|--------|-------|
| **Refresh Rate** | < 2.0s (70+ Tokens) |
| **Concurrency** | 50 Parallel Streams |
| **Data Source** | Live Polymarket API |

---

## 🔧 Execution Realism

PolyShark models **real execution dynamics**:

| Parameter | Description |
|-----------|-------------|
| **Fees** | Taker/maker fees applied per trade |
| **Slippage** | Non-linear price impact based on order size |
| **Permissions** | Strictly enforced daily spend limits (simulated smart account) |

---

## 🛠️ Stack Status

| Component | Status | Implementation |
|-----------|--------|----------------|
| **Market Data** | ✅ Live | `market.rs` (Gamma + CLOB) |
| **Concurrency** | ✅ Done | `futures::stream` (Buffer 50) |
| **Solana** | ✅ Connected | `solana.rs` (Devnet RPC) |
| **Logic** | ✅ Done | `arb.rs` + `constraint.rs` |
| **On-Chain Exec** | ⏳ Planned | Solana Transaction Building |

---

## 📚 Documentation

### Core Concepts
- [**context.md**](docs/context.md) — Project background & "Why PolyShark?"
- [**maths.md**](docs/maths.md) — Mathematical foundations of arbitrage detection
- [**polymarket.md**](docs/polymarket.md) — Polymarket API reference

### Technical Specifications
- [**spec.md**](docs/spec.md) — Full generic system specification
- [**metamask/v1.md**](docs/metamask/v1.md) — **Hackathon Architecture: ERC-7715 Integration**
- [**implementation.md**](docs/implementation.md) — Implementation log

---

## 📄 License

MIT License — See [LICENSE](./LICENSE) for details.
