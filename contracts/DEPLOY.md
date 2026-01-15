# DemoMarket Deployment Guide

## Prerequisites

- Foundry installed (`curl -L https://foundry.paradigm.xyz | bash`)
- Arbitrum Sepolia testnet ETH
- Private key with funds

## Quick Deploy

```bash
# 1. Set environment variables
export PRIVATE_KEY="your_private_key_here"
export ARBITRUM_SEPOLIA_RPC="https://sepolia-rollup.arbitrum.io/rpc"

# 2. Deploy contract
forge create --rpc-url $ARBITRUM_SEPOLIA_RPC \
  --private-key $PRIVATE_KEY \
  contracts/DemoMarket.sol:DemoMarket \
  --constructor-args "Will Bitcoin reach $100k by end of 2026?"

# 3. Note the deployed contract address
# Update config.toml and envio/config.yaml with the address
```

## Verify on Arbiscan

```bash
forge verify-contract \
  --rpc-url $ARBITRUM_SEPOLIA_RPC \
  --etherscan-api-key $ARBISCAN_API_KEY \
  <CONTRACT_ADDRESS> \
  contracts/DemoMarket.sol:DemoMarket \
  --constructor-args $(cast abi-encode "constructor(string)" "Will Bitcoin reach $100k by end of 2026?")
```

## Interact with Contract

```bash
# Get current market state
cast call <CONTRACT_ADDRESS> "getMarket()" --rpc-url $ARBITRUM_SEPOLIA_RPC

# Update prices (owner only)
cast send <CONTRACT_ADDRESS> \
  "updatePrices(uint256,uint256)" 5500 4500 \
  --private-key $PRIVATE_KEY \
  --rpc-url $ARBITRUM_SEPOLIA_RPC

# Simulate price movement
cast send <CONTRACT_ADDRESS> \
  "simulatePriceMovement()" \
  --private-key $PRIVATE_KEY \
  --rpc-url $ARBITRUM_SEPOLIA_RPC

# Check for arbitrage
cast call <CONTRACT_ADDRESS> "hasArbitrage()" --rpc-url $ARBITRUM_SEPOLIA_RPC
```

## Setup Envio HyperIndex

1. Update `envio/config.yaml` with deployed contract address
2. Update start_block with deployment block number
3. Deploy to Envio:

```bash
cd envio
envio dev  # Test locally first
envio deploy  # Deploy to production
```

4. Update `config.toml` with Envio GraphQL endpoint

## Testing

```bash
# Test health check
curl -X POST <ENVIO_ENDPOINT> \
  -H "Content-Type: application/json" \
  -d '{"query": "{ _meta { block { number timestamp } } }"}'

# Test market query
curl -X POST <ENVIO_ENDPOINT> \
  -H "Content-Type: application/json" \
  -d '{"query": "{ quotes(limit: 10, orderBy: timestamp, orderDirection: desc) { yesPrice noPrice liquidity timestamp } }"}'
```
