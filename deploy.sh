#!/bin/bash
# Deployment script for ArbiShark Demo Contract on Arbitrum Sepolia

set -e  # Exit on error

echo "🦈 ArbiShark Deployment Script"
echo "================================"
echo ""

# Check prerequisites
command -v forge >/dev/null 2>&1 || { echo "❌ Foundry not installed. Run: curl -L https://foundry.paradigm.xyz | bash"; exit 1; }

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "❌ .env file not found. Copy .env.example and fill in your values."
    exit 1
fi

# Validate environment variables
if [ -z "$PRIVATE_KEY" ]; then
    echo "❌ PRIVATE_KEY not set in .env"
    exit 1
fi

if [ -z "$ARBITRUM_SEPOLIA_RPC" ]; then
    echo "❌ ARBITRUM_SEPOLIA_RPC not set in .env"
    exit 1
fi

# Set default question if not provided
QUESTION="${DEMO_QUESTION:-Will Bitcoin reach \$100k by end of 2026?}"

echo "📋 Deployment Configuration:"
echo "   RPC: $ARBITRUM_SEPOLIA_RPC"
echo "   Question: $QUESTION"
echo ""

# Deploy contract
echo "🚀 Deploying DemoMarket contract..."
DEPLOY_OUTPUT=$(forge create --rpc-url "$ARBITRUM_SEPOLIA_RPC" \
  --private-key "$PRIVATE_KEY" \
  contracts/DemoMarket.sol:DemoMarket \
  --constructor-args "$QUESTION" \
  2>&1)

# Extract contract address
CONTRACT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | grep "Deployed to:" | awk '{print $3}')

if [ -z "$CONTRACT_ADDRESS" ]; then
    echo "❌ Deployment failed!"
    echo "$DEPLOY_OUTPUT"
    exit 1
fi

echo "✅ Contract deployed successfully!"
echo "   Address: $CONTRACT_ADDRESS"
echo ""

# Update config.toml
echo "📝 Updating config.toml..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/demo_contract_address = \"0x0000000000000000000000000000000000000000\"/demo_contract_address = \"$CONTRACT_ADDRESS\"/" config.toml
else
    # Linux
    sed -i "s/demo_contract_address = \"0x0000000000000000000000000000000000000000\"/demo_contract_address = \"$CONTRACT_ADDRESS\"/" config.toml
fi

# Update envio/config.yaml
echo "📝 Updating envio/config.yaml..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/- 0x0000000000000000000000000000000000000000/- $CONTRACT_ADDRESS/" envio/config.yaml
else
    sed -i "s/- 0x0000000000000000000000000000000000000000/- $CONTRACT_ADDRESS/" envio/config.yaml
fi

echo "✅ Configuration files updated!"
echo ""

# Get deployment block number
echo "🔍 Getting deployment block number..."
BLOCK_NUMBER=$(cast block-number --rpc-url "$ARBITRUM_SEPOLIA_RPC")
echo "   Block: $BLOCK_NUMBER"

# Update envio start_block
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/start_block: 0/start_block: $BLOCK_NUMBER/" envio/config.yaml
else
    sed -i "s/start_block: 0/start_block: $BLOCK_NUMBER/" envio/config.yaml
fi

echo ""
echo "🎉 Deployment Complete!"
echo "================================"
echo ""
echo "📋 Next Steps:"
echo "   1. Verify contract on Arbiscan:"
echo "      https://sepolia.arbiscan.io/address/$CONTRACT_ADDRESS"
echo ""
echo "   2. Deploy Envio HyperIndex:"
echo "      cd envio && envio deploy"
echo ""
echo "   3. Update .env with Envio endpoint after deployment"
echo ""
echo "   4. Test the agent:"
echo "      cargo run --release"
echo ""
echo "Contract Address: $CONTRACT_ADDRESS"
echo "Deployment Block: $BLOCK_NUMBER"
