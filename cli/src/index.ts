#!/usr/bin/env node

import { Command } from 'commander';
import inquirer from 'inquirer';
import chalk from 'chalk';
import ora from 'ora';
import fs from 'fs-extra';
import path from 'path';

const program = new Command();

program
    .name('arbishark')
    .description('CLI tool to scaffold ArbiShark agents')
    .version('1.0.0');

program
    .command('create <project-name>')
    .description('Create a new ArbiShark agent')
    .option('-p, --protocol <type>', 'Protocol type (dex, nft, lending, prediction)')
    .option('-c, --chain <chain>', 'Chain (arbitrum-one, arbitrum-sepolia, orbit)')
    .action(async (projectName, options) => {
        console.log(chalk.cyan.bold('\n🦈 ArbiShark Agent Generator\n'));

        // Interactive prompts if options not provided
        const answers = await inquirer.prompt([
            {
                type: 'list',
                name: 'protocol',
                message: 'Select protocol type:',
                choices: [
                    { name: '🔄 DEX Arbitrage (Uniswap, Camelot, SushiSwap)', value: 'dex' },
                    { name: '🖼️  NFT Sniping (OpenSea, Blur)', value: 'nft' },
                    { name: '💰 Lending Optimizer (Aave, Compound)', value: 'lending' },
                    { name: '🎲 Prediction Markets (Custom)', value: 'prediction' },
                ],
                when: !options.protocol,
            },
            {
                type: 'list',
                name: 'chain',
                message: 'Select target chain:',
                choices: [
                    { name: 'Arbitrum One (Mainnet)', value: 'arbitrum-one' },
                    { name: 'Arbitrum Sepolia (Testnet)', value: 'arbitrum-sepolia' },
                    { name: 'Orbit Chain', value: 'orbit' },
                ],
                when: !options.chain,
            },
            {
                type: 'input',
                name: 'dailyLimit',
                message: 'Daily spending limit (USDC):',
                default: '10',
                validate: (input) => !isNaN(input) && parseFloat(input) > 0,
            },
            {
                type: 'list',
                name: 'strategy',
                message: 'Default strategy:',
                choices: ['Conservative', 'Balanced', 'Aggressive'],
            },
        ]);

        const config = {
            protocol: options.protocol || answers.protocol,
            chain: options.chain || answers.chain,
            dailyLimit: answers.dailyLimit,
            strategy: answers.strategy,
        };

        const spinner = ora('Creating project structure...').start();

        try {
            await createProject(projectName, config);
            spinner.succeed(chalk.green('Project created successfully!'));

            console.log(chalk.cyan('\n📋 Next steps:\n'));
            console.log(chalk.white(`  cd ${projectName}`));
            console.log(chalk.white(`  cp .env.example .env`));
            console.log(chalk.white(`  # Edit .env with your keys`));
            console.log(chalk.white(`  cargo build --release`));
            console.log(chalk.white(`  cargo run\n`));

            console.log(chalk.gray('📚 Documentation: docs/README.md'));
            console.log(chalk.gray('🚀 Deploy: ./deploy.sh\n'));
        } catch (error) {
            spinner.fail(chalk.red('Failed to create project'));
            console.error(error);
            process.exit(1);
        }
    });

async function createProject(projectName: string, config: any) {
    const projectPath = path.join(process.cwd(), projectName);

    // Create directory structure
    await fs.ensureDir(projectPath);
    await fs.ensureDir(path.join(projectPath, 'src'));
    await fs.ensureDir(path.join(projectPath, 'contracts'));
    await fs.ensureDir(path.join(projectPath, 'docs'));
    await fs.ensureDir(path.join(projectPath, 'dashboard'));

    // Copy template files based on protocol
    const templateDir = path.join(__dirname, '..', 'templates', config.protocol);
    await fs.copy(templateDir, projectPath);

    // Generate config.toml
    const configToml = generateConfig(config);
    await fs.writeFile(path.join(projectPath, 'config.toml'), configToml);

    // Generate .env.example
    const envExample = generateEnvExample(config);
    await fs.writeFile(path.join(projectPath, '.env.example'), envExample);

    // Generate README
    const readme = generateReadme(projectName, config);
    await fs.writeFile(path.join(projectPath, 'README.md'), readme);

    // Copy Cargo.toml
    const cargoToml = generateCargoToml(projectName, config);
    await fs.writeFile(path.join(projectPath, 'Cargo.toml'), cargoToml);
}

function generateConfig(config: any): string {
    const chainConfig = {
        'arbitrum-one': {
            rpc: 'https://arb1.arbitrum.io/rpc',
            chainId: 42161,
        },
        'arbitrum-sepolia': {
            rpc: 'https://sepolia-rollup.arbitrum.io/rpc',
            chainId: 421614,
        },
        orbit: {
            rpc: 'https://your-orbit-rpc.com',
            chainId: 1234567,
        },
    };

    const chain = chainConfig[config.chain];

    return `# ${config.protocol.toUpperCase()} Agent Configuration
mode = "${config.protocol}_arbitrage"

[permission]
daily_limit_usdc = ${config.dailyLimit}
duration_days = 30
token = "USDC"

[trading]
min_spread_threshold = 0.02
min_profit_threshold = 0.10
trade_size = 5.0
max_position_value = 50.0

[strategy]
mode = "${config.strategy.toLowerCase()}"
conservative_threshold = 0.30
aggressive_threshold = 0.70

[arbitrum]
rpc = "${chain.rpc}"
chain_id = ${chain.chainId}
envio_endpoint = "https://your-envio-endpoint.com/graphql"

[safety]
max_data_delay_ms = 5000
max_consecutive_failures = 3
safe_mode_cooldown_secs = 300
`;
}

function generateEnvExample(config: any): string {
    return `# Arbitrum Configuration
PRIVATE_KEY=your_private_key_here
ARBITRUM_RPC=${config.chain === 'arbitrum-one' ? 'https://arb1.arbitrum.io/rpc' : 'https://sepolia-rollup.arbitrum.io/rpc'}

# Envio HyperIndex
ENVIO_ENDPOINT=https://your-envio-endpoint.com/graphql

# Protocol-specific (${config.protocol})
${config.protocol === 'dex' ? 'UNISWAP_POOL=0x...\nCAMELOT_POOL=0x...' : ''}
${config.protocol === 'nft' ? 'OPENSEA_API_KEY=your_key\nBLUR_API_KEY=your_key' : ''}
${config.protocol === 'lending' ? 'AAVE_POOL=0x...\nCOMPOUND_POOL=0x...' : ''}

# Optional
TELEGRAM_BOT_TOKEN=your_token
DISCORD_WEBHOOK=your_webhook
`;
}

function generateReadme(projectName: string, config: any): string {
    return `# ${projectName}

ArbiShark agent for ${config.protocol} arbitrage on ${config.chain}.

## Quick Start

\`\`\`bash
# 1. Setup environment
cp .env.example .env
# Edit .env with your keys

# 2. Build
cargo build --release

# 3. Run
cargo run --release
\`\`\`

## Configuration

- **Protocol**: ${config.protocol}
- **Chain**: ${config.chain}
- **Daily Limit**: $${config.dailyLimit} USDC
- **Strategy**: ${config.strategy}

## Features

- ✅ ERC-7715 Permission System
- ✅ Envio HyperIndex Integration
- ✅ Real-time ${config.protocol} monitoring
- ✅ Adaptive strategy modes
- ✅ Risk management

## Documentation

See \`docs/\` for detailed guides.

---

Generated with ArbiShark CLI
`;
}

function generateCargoToml(projectName: string, config: any): string {
    return `[package]
name = "${projectName}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
chrono = "0.4"
thiserror = "1.0"
`;
}

program.parse();
