# 📚 PolyShark Documentation

> Navigation guide for all project documentation.

---

## Quick Links

| Doc | Purpose | Audience |
|-----|---------|----------|
| [**README**](./README.md) | Project overview & status | Everyone |
| [**MetaMask Architecture**](./metamask/v1.md) | ERC-7715 integration | Judges, Developers |
| [**Technical Spec**](./spec.md) | Structs, traits, implementations | Developers |
| [**Math Foundations**](./math.md) | Formulas & arbitrage logic | Developers |
| [**API Reference**](./polymarket.md) | Polymarket API integration | Developers |
| [**Implementation Guide**](./implementation.md) | Step-by-step build guide | Contributors |
| [**Project Context**](./context.md) | Background & philosophy | Everyone |

---

## Recommended Reading Order

### For Hackathon Judges
1. [README.md](./README.md) — What PolyShark does
2. [metamask/v1.md](./metamask/v1.md) — ERC-7715 architecture (the core hackathon work)
3. [spec.md](./spec.md) — Technical implementation

### For Developers
1. [README.md](./README.md) — Overview
2. [context.md](./context.md) — Why this project exists
3. [math.md](./math.md) — Mathematical foundations
4. [spec.md](./spec.md) — Code structure
5. [polymarket.md](./polymarket.md) — API integration
6. [implementation.md](./implementation.md) — Build it yourself

---

## Architecture Overview

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

See [metamask/v1.md](./metamask/v1.md) for full architecture details.

---

## File Summary

| File | Lines | Size | Description |
|------|-------|------|-------------|
| `README.md` | 108 | 3.4 KB | Project overview |
| `context.md` | 300 | ~8 KB | Project background |
| `math.md` | 400 | ~10 KB | Math & formulas |
| `spec.md` | 694 | 19 KB | Technical specification |
| `implementation.md` | 362 | 9 KB | Implementation guide |
| `polymarket.md` | 816 | 23 KB | API reference |
| `metamask/v1.md` | 234 | 5.4 KB | Hackathon architecture |
