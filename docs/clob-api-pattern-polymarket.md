# CLOB API Pattern (Polymarket Example)

This document describes the CLOB (Central Limit Order Book) API pattern using Polymarket as an example, and explicitly maps it to Arbitrum integration for ArbiShark.

---

## Overview

Polymarket provides a CLOB-style API for prediction markets. ArbiShark uses this as a reference backend for its MarketClient abstraction, but is designed to be Arbitrum-first and protocol-agnostic.

## API Endpoints (Polymarket)
- Market list: `https://gamma-api.polymarket.com/events?limit=20&active=true&closed=false`
- Order book: `https://clob.polymarket.com/book`

## Mapping to Arbitrum
- The same MarketClient trait can be implemented for any Arbitrum protocol exposing CLOB-like data (e.g., via Envio HyperIndex).
- For Arbitrum, use Envio HyperIndex GraphQL endpoints to fetch market and order book data.

## Example: MarketClient for Polymarket
```rust
// See src/market_client.rs for trait definition
// PolymarketClient implements MarketClient using Polymarket endpoints
```

## Example: MarketClient for Arbitrum
```rust
// ArbitrumMarketClient implements MarketClient using Envio HyperIndex
// See src/market_client.rs for trait definition
```

---

For more details, see [docs/arbitrum/doc.md].
