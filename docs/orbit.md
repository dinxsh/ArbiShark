# Orbit Integration for ArbiShark

ArbiShark can be extended to run as an agent against Orbit markets, using ERC-7715 permissioning on Arbitrum.

## How it Works

- Orbit config in `orbit.toml` sets RPC/indexer URLs and daily spend limit.
- Agent logic is identical to Arbitrum demo, but targets Orbit markets.
- PermissionGuard maps to ERC-7715 account on Arbitrum.

## Future Path

- Deploy core logic as Stylus contract for on-chain arbitrage.
- Use Orbit's permissioned agent model for safe, autonomous trading.

See [docs/arbitrum/doc.md] for more details.