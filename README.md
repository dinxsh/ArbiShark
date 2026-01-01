# 🦈 PolyShark

> *"If markets contradict themselves, eat the contradiction."*

**PolyShark** is a **permission-safe arbitrage agent** for Polymarket, built for the MetaMask Hackathon. It detects logical mispricing between linked prediction markets and executes trades automatically within user-defined limits using **ERC-7715 Advanced Permissions**.

---

## 🏆 Hackathon Highlights

| Feature | Implementation |
|---------|----------------|
| **Smart Accounts** | MetaMask Smart Account with ERC-7715 |
| **Advanced Permissions** | Daily USDC spend limits (10 USDC/day) |
| **Automation** | Zero-popup trading after permission grant |
| **On-Chain Integration** | Polymarket via Envio indexer |

> 📘 **Full Architecture:** [metamask/v1.md](./metamask/v1.md)

---

## 🎯 What It Does

1. **Detects** logical arbitrage (when YES + NO ≠ 1)
2. **Validates** against ERC-7715 permission allowance
3. **Executes** trades automatically (no wallet popups)
4. **Tracks** realistic PnL with fees, slippage, and execution costs

---

## 🧠 Architecture

```
MetaMask Smart Account (ERC-7715)
         ↓
Advanced Permission (Daily USDC Limit)
         ↓
   PolyShark Agent (Rust)
         ↓
  Polymarket Contracts
         ↑
  Envio Indexer (Market State)
```

### Module Structure

```
src/
├── wallet.rs        → Permission-aware adapter
├── market.rs        → Envio-sourced market data
├── constraint.rs    → Logical relationships
├── arb.rs           → Arbitrage detection
└── engine.rs        → Main trading loop
```

---

## 📊 Permission Specification

PolyShark requests the following permission:

| Property | Value |
|----------|-------|
| **Type** | Spend permission |
| **Token** | USDC |
| **Limit** | 10 USDC per day |
| **Scope** | Polymarket trading adapter |
| **Duration** | 30 days |

> *"PolyShark may automatically trade up to 10 USDC per day on your behalf. You can revoke this permission at any time."*

---

## 🔧 Execution Realism

| Parameter | Description |
|-----------|-------------|
| **Fees** | Taker/maker fees from Polymarket API |
| **Slippage** | Non-linear price impact from order book |
| **Partial Fills** | Orders may not fully execute |
| **Latency** | Delay between signal and execution |
| **Position Sizing** | Dynamic sizing based on risk & liquidity |

---

## 📚 Documentation

| Doc | Purpose |
|-----|---------|
| [**DOCUMENTATION.md**](./DOCUMENTATION.md) | Navigation index |
| [**metamask/v1.md**](./metamask/v1.md) | ERC-7715 architecture |
| [**spec.md**](./spec.md) | Technical specification |
| [**math.md**](./math.md) | Mathematical foundations |
| [**polymarket.md**](./polymarket.md) | API reference |
| [**implementation.md**](./implementation.md) | Build guide |
| [**context.md**](./context.md) | Project background |

---

## 📈 Roadmap

| Feature | Status |
|---------|--------|
| Multi-market constraints | ✅ Done |
| Random latency injection | ✅ Done |
| Fee modeling refinement | ✅ Done |
| Monte Carlo simulation | ✅ Done |
| **ERC-7715 Integration** | ✅ Done |
| **Smart Account Support** | ✅ Done |
| Solana devnet deployment | ⏳ Future |

---

## 🛠️ Tech Stack

- **Language:** Rust
- **Wallet:** MetaMask Smart Account
- **Permissions:** ERC-7715
- **Market Data:** Polymarket CLOB API + Envio
- **Target:** Polygon (Chain ID: 137)

---

## 📄 License

MIT License — See [LICENSE](./LICENSE) for details.
