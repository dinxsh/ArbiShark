# ArbiShark Deployment Script for Windows
# Run with: .\deploy.ps1

Write-Host "🦈 ArbiShark Deployment Script" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
if (!(Get-Command forge -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Foundry not installed. Visit: https://book.getfoundry.sh/getting-started/installation" -ForegroundColor Red
    exit 1
}

# Load environment variables
if (Test-Path .env) {
    Get-Content .env | ForEach-Object {
        if ($_ -match '^([^#].+?)=(.+)$') {
            [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2])
        }
    }
} else {
    Write-Host "❌ .env file not found. Copy .env.example and fill in your values." -ForegroundColor Red
    exit 1
}

# Validate environment variables
$PRIVATE_KEY = $env:PRIVATE_KEY
$ARBITRUM_SEPOLIA_RPC = $env:ARBITRUM_SEPOLIA_RPC

if (!$PRIVATE_KEY) {
    Write-Host "❌ PRIVATE_KEY not set in .env" -ForegroundColor Red
    exit 1
}

if (!$ARBITRUM_SEPOLIA_RPC) {
    Write-Host "❌ ARBITRUM_SEPOLIA_RPC not set in .env" -ForegroundColor Red
    exit 1
}

# Set default question
$QUESTION = if ($env:DEMO_QUESTION) { $env:DEMO_QUESTION } else { "Will Bitcoin reach 100k by end of 2026?" }

Write-Host "📋 Deployment Configuration:" -ForegroundColor Yellow
Write-Host "   RPC: $ARBITRUM_SEPOLIA_RPC"
Write-Host "   Question: $QUESTION"
Write-Host ""

# Deploy contract
Write-Host "🚀 Deploying DemoMarket contract..." -ForegroundColor Green
$DEPLOY_OUTPUT = forge create --rpc-url $ARBITRUM_SEPOLIA_RPC --private-key $PRIVATE_KEY contracts/DemoMarket.sol:DemoMarket --constructor-args $QUESTION 2>&1

# Extract contract address
$CONTRACT_ADDRESS = ($DEPLOY_OUTPUT | Select-String "Deployed to: (.+)" | ForEach-Object { $_.Matches.Groups[1].Value }).Trim()

if (!$CONTRACT_ADDRESS) {
    Write-Host "❌ Deployment failed!" -ForegroundColor Red
    Write-Host $DEPLOY_OUTPUT
    exit 1
}

Write-Host "✅ Contract deployed successfully!" -ForegroundColor Green
Write-Host "   Address: $CONTRACT_ADDRESS"
Write-Host ""

# Update config.toml
Write-Host "📝 Updating config.toml..." -ForegroundColor Yellow
(Get-Content config.toml) -replace 'demo_contract_address = "0x0000000000000000000000000000000000000000"', "demo_contract_address = `"$CONTRACT_ADDRESS`"" | Set-Content config.toml

# Update envio/config.yaml
Write-Host "📝 Updating envio/config.yaml..." -ForegroundColor Yellow
(Get-Content envio/config.yaml) -replace '- 0x0000000000000000000000000000000000000000', "- $CONTRACT_ADDRESS" | Set-Content envio/config.yaml

Write-Host "✅ Configuration files updated!" -ForegroundColor Green
Write-Host ""

# Get deployment block number
Write-Host "🔍 Getting deployment block number..." -ForegroundColor Yellow
$BLOCK_NUMBER = cast block-number --rpc-url $ARBITRUM_SEPOLIA_RPC

Write-Host "   Block: $BLOCK_NUMBER"

# Update envio start_block
(Get-Content envio/config.yaml) -replace 'start_block: 0', "start_block: $BLOCK_NUMBER" | Set-Content envio/config.yaml

Write-Host ""
Write-Host "🎉 Deployment Complete!" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "📋 Next Steps:" -ForegroundColor Yellow
Write-Host "   1. Verify contract on Arbiscan:"
Write-Host "      https://sepolia.arbiscan.io/address/$CONTRACT_ADDRESS"
Write-Host ""
Write-Host "   2. Deploy Envio HyperIndex:"
Write-Host "      cd envio"
Write-Host "      envio deploy"
Write-Host ""
Write-Host "   3. Update .env with Envio endpoint after deployment"
Write-Host ""
Write-Host "   4. Test the agent:"
Write-Host "      cargo run --release"
Write-Host ""
Write-Host "Contract Address: $CONTRACT_ADDRESS" -ForegroundColor Cyan
Write-Host "Deployment Block: $BLOCK_NUMBER" -ForegroundColor Cyan
