ArbiShark should be a robust, Arbitrum‑native, ERC‑7715‑powered agent template that other builders can fork to build safe autonomous trading/market agents on Arbitrum, with Polymarket used only as a reference backend and calibration source.[1][2][3][4]

Below is a concise but complete PRD you can drop into the repo (`docs/PRD-arbishark.md`) and implement against.

***

## 1. Product Overview

**Product Name:** ArbiShark  
**Status:** Rebrand and extension of PolyShark  
**Primary Chain:** Arbitrum One (demo on Arbitrum Sepolia)  
**Core Concept:**  
A permission‑safe **agent template** for Arbitrum that:

- Uses **ERC‑7715** MetaMask Delegation Toolkit for cryptographic daily spend limits and scoped permissions.[4][5]
- Consumes **Arbitrum market data** via Envio HyperIndex (or equivalent indexers) with Polymarket kept as a “CLOB pattern” example.[3][6]
- Implements a battle‑tested **execution simulator** (fees, slippage, partial fills, latency) and arbitrage logic.
- Provides a **dashboard** that demonstrates “one popup → many safe trades” UX for Arbitrum users.

**Primary Hackathon Track:** Arbitrum Open Track  
**Secondary Narratives:** Stylus‑ready Rust core, Orbit‑ready architecture.[2][7]

***

## 2. Goals & Non‑Goals

### 2.1 Goals

1. **Rebrand & reposition**
   - Rename PolyShark to ArbiShark in user‑facing materials.
   - Make all top‑level docs Arbitrum‑first; Polymarket is an example backend, not the core identity.[1][2]

2. **Demonstrate best use of Arbitrum stack**
   - ERC‑7715 permission integration via MetaMask Delegation Toolkit.[5][4]
   - Envio HyperIndex for Arbitrum data ingestion.[6][3]
   - Rust core designed to be Stylus‑compatible and Orbit‑deployable.

3. **Deliver a working agent demo**
   - Agent running on Arbitrum Sepolia against a simple demo market (on‑chain or simulated via Envio).
   - Dashboard showing permissions, trades, PnL.

4. **Provide a reusable template**
   - Pluggable `MarketClient` trait so any Arbitrum protocol can integrate.
   - Documented patterns for fees, slippage, risk, and permissioning.

### 2.2 Non‑Goals

- Full production deployment with real capital on Arbitrum.
- Full Stylus/Orbit deployment (only partial prototypes and clear docs).
- Supporting every possible market type; focus on binary/multi‑outcome and simple CLOB‑like markets.

***

## 3. User Personas & Use Cases

### 3.1 Personas

1. **Power user on Arbitrum**
   - Wants bots/agents but hates unlimited approvals and blind trust.
   - Uses MetaMask, willing to grant **policy‑level** permissions.

2. **Arbitrum protocol builder**
   - Has a markets protocol (perps, predictions, games).
   - Wants an off‑the‑shelf agent template for their users.

3. **Hackathon judge / reviewer**
   - Evaluates **Arbitrum‑native innovation**, not general bots.
   - Cares about:
     - Arbitrum stack integration.
     - UX, safety, creativity, extensibility.[7][2][1]

### 3.2 Key Use Cases

- User grants a 10 USDC.e/day spend limit and sees ArbiShark run dozens of bounded trades with no further popups.
- Protocol dev forks ArbiShark, implements their own `MarketClient` for an Arbitrum market, and instantly gets a permission‑safe agent.
- Judge reviews repo and sees:
  - Clear Arbitrum dependency graph (Envio, ERC‑7715).
  - Strong docs (math, spec, implementation) wired to Arbitrum.

***

## 4. Functional Requirements

### 4.1 Rebranding & Documentation

**FR‑1: Naming & branding**

- Change project name in:
  - Repo name or description (ArbiShark).
  - README title, logos, and badges.
  - `Project Context`, `spec.md`, `math.md`, `implementation guide`, `HACKQUEST_SUBMISSION.md`.

**FR‑2: Arbitrum‑first framing**

- Every top‑level doc must:
  - Mention Arbitrum One / Arbitrum Sepolia as primary deployment.
  - Reference Envio HyperIndex for Arbitrum as a data source.[3][6]
  - Reference ERC‑7715 as the permission system.[4]
- Polymarket docs must:
  - Be renamed to “CLOB API Pattern (Polymarket Example)” and explicitly describe Arbitrum mapping.

**FR‑3: Track mapping section**

- README and submission doc include a section:
  - “Why this wins the Arbitrum Open Track” with bullet points for Technical Completeness, UX, Creativity, Wow Factor.[7]

***

### 4.2 Core Agent Architecture

**FR‑4: MarketClient abstraction**

Define a trait:

```rust
#[async_trait::async_trait]
pub trait MarketClient {
    async fn get_markets(&self) -> Result<Vec<Market>>;
    async fn get_order_book(&self, token_id: &str) -> Result<OrderBook>;
    async fn stream_quotes(&self) -> Result<QuoteStream>;
}
```

Implement:

- `PolymarketClient` (already exists) adapted to this trait.
- `ArbitrumMarketClient` that:
  - Reads quotes from an Envio HyperIndex endpoint on Arbitrum.[6][3]
  - Maps GraphQL response to `Market` and `OrderBook`.

**FR‑5: PermissionGuard and ERC‑7715 mapping**

- Add `PermissionGuard`:

```rust
pub struct PermissionGuard {
    pub daily_limit: f64,
    pub spent_today: f64,
}
```

- Execution flow:
  - `preview = engine.preview(order)` → compute total_cost.
  - If `!guard.can_spend(preview.total_cost)`, skip.
  - After success: `guard.record_spend(result.total_cost)`.
- Document mapping between `PermissionGuard` fields and ERC‑7715 JSON (`limit.amount`, `period`, `scope`).[4]

**FR‑6: Main loop aware of Arbitrum**

- `engine.rs` loop must:
  - Take `MarketClient` and `PermissionGuard` as dependencies.
  - Log total spend vs ERC‑7715 limit in each tick.
  - Be network‑agnostic but annotated in docs as “Arbitrum agent loop”.

***

### 4.3 Arbitrum Demo Integration

**FR‑7: Demo market on Arbitrum Sepolia**

- A simple Solidity contract:
  - Maintains YES/NO prices for a single market.
  - Emits `QuoteUpdated` events with prices and liquidity.
- Either:
  - Deploy it on Arbitrum Sepolia (ideal).
  - Or simulate it with local events but keep interfaces ready for real deployment.

**FR‑8: Envio HyperIndex integration**

- Create a HyperIndex subgraph / config for the demo contract on Arbitrum.[3][6]
- Implement `ArbitrumMarketClient` that queries this endpoint for:
  - Latest quotes (YES/NO, liquidity).
- Wire into ArbiShark’s main loop for demo mode.

**FR‑9: Demo mode toggle**

- Add a simple config (`config.toml` or CLI flag):
  - `mode = "polymarket"` or `mode = "arbitrum_demo"`.
- In `arbitrum_demo` mode:
  - Use `ArbitrumMarketClient`.
  - Use Arbitrum RPC/Envio URLs.
  - Use ERC‑7715 permission specs for USDC.e.

***

### 4.4 UI / UX

**FR‑10: Dashboard fields**

Dashboard must show:

- Network (Arbitrum One / Sepolia).
- ERC‑7715 daily limit and remaining spend.
- Number of trades executed in current day.
- Current PnL and equity.
- Safety status:
  - Normal / Safe mode (e.g., after N failures, cooldown).
- Last N signals and expected vs realized PnL.

**FR‑11: UX flow**

- User journey:
  1. Connect wallet on Arbitrum.
  2. Grant ERC‑7715 permission via MetaMask Delegation Toolkit.
  3. See “Autonomous mode ON” with daily cap visible.
  4. Watch trades execute within limit, with clear safety indicators.

***

### 4.5 Stylus & Orbit Story (Docs + Minimal Code)

**FR‑12: Stylus‑ready core**

- Extract a `core` Rust module:
  - Pure functions for:
    - Spread calculation.
    - Arbitrage detection.
    - Simple expected profit calculation.
  - No async, no network.
- Add `docs/stylus.md`:
  - Describe how these functions could be compiled to Stylus and used in a smart contract to verify arbitrage conditions on‑chain.[2]

**FR‑13: Orbit config**

- Add `orbit.toml`:
  - RPC URL, indexer URL, gas token, etc.
- Add `docs/orbit.md`:
  - Explain how ArbiShark could run as an agent against Orbit markets and still be permission‑bounded by an ERC‑7715 account on Arbitrum.[2]

***

## 5. Non‑Functional Requirements

- **Performance:**  
  - Must be able to handle at least dozens of markets and 1–2 updates per second without lag on a typical dev machine.
- **Reliability:**
  - Safe defaults: If data becomes stale or Envio is unreachable, agent halts trading.
- **Safety:**
  - All trade execution paths must go through `PermissionGuard` and `Wallet`.
  - No direct calls that bypass limits.

***

## 6. Milestones & Implementation Plan

### Milestone 1: Rebrand & Docs (Day 1–2)

- Rename project to ArbiShark in README, logos, main docs.
- Add Arbitrum‑first narrative; Polymarket becomes “example backend”.
- Add track mapping and judging criteria section.

### Milestone 2: Core Abstractions (Day 2–3)

- Implement `MarketClient` trait.
- Adapt existing Polymarket code to `PolymarketClient`.
- Add `PermissionGuard` and integrate into execution flow.
- Update tests to cover permission gating.

### Milestone 3: Arbitrum Demo Wiring (Day 3–4)

- Ship minimal `ArbitrumMarketClient` that:
  - Either queries a real Envio indexer on Arbitrum Sepolia.
  - Or uses a stubbed JSON/GraphQL endpoint that mimics Envio’s shape.
- Add config for `mode = arbitrum_demo`.
- Confirm main loop runs end‑to‑end in demo mode.

### Milestone 4: Dashboard & UX (Day 4–5)

- Update dashboard:
  - Arbitrum network indicator.
  - ERC‑7715 daily limit and usage.
  - Safety state.
- Add demo script (`docs/demo-script-arbitrum.md`) for hackathon presentation.

### Milestone 5: Stylus & Orbit Docs (Day 5)

- Extract core math functions into a Stylus‑friendly module.
- Draft `stylus.md` and `orbit.md` describing how to extend beyond the hackathon.

***

## 7. Acceptance Criteria

The rebrand from PolyShark to ArbiShark is “done” when:

- All user‑visible materials are Arbitrum‑first and consistently use “ArbiShark”.
- Codebase:
  - Has `MarketClient` abstraction with both Polymarket and Arbitrum demo implementations.
  - Uses `PermissionGuard` in all execution paths.
- Demo:
  - Runs in `arbitrum_demo` mode, showing:
    - Arbitrum network.
    - ERC‑7715 daily cap & usage.
    - Autonomous trades or simulated trades within limits.
- Docs:
  - Include clear sections for:
    - Open Track fit (judging criteria).
    - Stylus future path.
    - Orbit future path.

This PRD gives you a clear blueprint to refactor PolyShark into a robust ArbiShark that showcases Arbitrum’s best hackathon use cases while staying realistic to implement within a short timeframe.[1][6][2][3][4]

[1](https://hackquest.io/en/hackathons/Arbitrum-APAC-Mini-Hackathon)
[2](https://phemex.com/news/article/arbitrum-and-hackquest-launch-asiafocused-mini-hackathon-with-4000-prize-pool-39696)
[3](https://docs.arbitrum.io/for-devs/third-party-docs/Envio/)
[4](https://docs.metamask.io/delegation-toolkit/concepts/erc7715/)
[5](https://metamask.io/news/hacker-guide-metamask-delegation-toolkit-erc-7715-actions)
[6](https://docs.envio.dev/docs/HyperIndex/arbitrum)
[7](https://www.panewslab.com/en/articles/27b5b3fb-238f-4987-8a7d-53fb30db3184)