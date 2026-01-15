# ArbiShark Architecture

## System Overview

ArbiShark is a permission-safe autonomous trading agent for Arbitrum that uses ERC-7715 Advanced Permissions and Envio HyperIndex for safe, automated arbitrage trading.

```mermaid
graph TB
    User[User] -->|1. Grant Permission Once| MetaMask[MetaMask Smart Account<br/>ERC-7715 Permissions]
    MetaMask -->|2. Enforced Daily Limit| Agent[ArbiShark Agent<br/>Rust Backend]
    Agent -->|3. Query Markets| Envio[Envio HyperIndex<br/>Arbitrum Data]
    Agent -->|4. Execute Trades| Contracts[Arbitrum Contracts<br/>DemoMarket]
    Envio -->|5. Index Events| Contracts
    Agent -->|6. Real-time Updates| Dashboard[Dashboard<br/>WebSocket + REST]
    
    subgraph "Agent Components"
        Constraint[Constraint Engine]
        Arbitrage[Arbitrage Detector]
        Execution[Execution Engine]
        Guard[Permission Guard]
    end
    
    Agent --> Constraint
    Agent --> Arbitrage
    Agent --> Execution
    Agent --> Guard
    
    style MetaMask fill:#f9a825
    style Envio fill:#00bcd4
    style Agent fill:#4caf50
    style Dashboard fill:#9c27b0
```

## Data Flow

```mermaid
sequenceDiagram
    participant U as User
    participant M as MetaMask
    participant A as Agent
    participant E as Envio
    participant C as Contract
    participant D as Dashboard
    
    U->>M: Grant ERC-7715 Permission
    M->>A: Permission Token
    
    loop Every 5 seconds
        A->>E: Health Check
        E-->>A: Block Height, Latency
        A->>E: Query Markets
        E-->>A: Market Data
        A->>A: Detect Arbitrage
        alt Has Opportunity
            A->>M: Check Allowance
            M-->>A: Remaining Limit
            A->>C: Execute Trade
            C-->>A: Trade Result
            A->>D: Update Dashboard
        end
    end
```

## Permission System

```mermaid
graph LR
    User[User Sets Limit<br/>10 USDC/day] --> Config[ERC-7715 Config]
    Config --> MetaMask[MetaMask Validates]
    MetaMask --> Guard[Permission Guard]
    
    Guard --> Check{Can Spend?}
    Check -->|Yes| Execute[Execute Trade]
    Check -->|No| Skip[Skip Trade]
    
    Execute --> Record[Record Spend]
    Record --> Guard
    
    style Guard fill:#ff9800
    style Execute fill:#4caf50
    style Skip fill:#f44336
```

## Component Architecture

```mermaid
graph TD
    Main[main.rs] --> Config[config.rs<br/>Load Settings]
    Main --> Engine[engine.rs<br/>Main Loop]
    
    Engine --> MarketClient[market_client.rs]
    MarketClient --> Polymarket[PolymarketClient]
    MarketClient --> Arbitrum[ArbitrumMarketClient]
    
    Engine --> Arb[arb.rs<br/>Arbitrage Detector]
    Arb --> Constraint[constraint.rs<br/>Logic Checker]
    
    Engine --> Execution[execution.rs<br/>Trade Executor]
    Execution --> Fees[fees.rs]
    Execution --> Slippage[slippage.rs]
    Execution --> Fills[fills.rs]
    
    Engine --> Wallet[wallet.rs<br/>Balance Tracker]
    Engine --> Guard[permission_guard.rs<br/>ERC-7715 Enforcer]
    
    Engine --> API[api.rs<br/>REST + WebSocket]
    API --> Dashboard[Dashboard UI]
    
    style Engine fill:#2196f3
    style Guard fill:#ff9800
    style API fill:#9c27b0
```

## Safety Mechanisms

```mermaid
graph TD
    Start[Agent Start] --> HealthCheck{Envio<br/>Healthy?}
    HealthCheck -->|No| SafeMode[Enter Safe Mode]
    HealthCheck -->|Yes| CheckData{Data Fresh?<br/>\u003c5s delay}
    
    CheckData -->|No| SafeMode
    CheckData -->|Yes| CheckPerm{Permission<br/>Valid?}
    
    CheckPerm -->|No| Stop[Stop Trading]
    CheckPerm -->|Yes| Trade[Execute Trade]
    
    Trade --> Success{Success?}
    Success -->|No| FailCount[Increment Fail Count]
    Success -->|Yes| ResetFail[Reset Fail Count]
    
    FailCount --> TooMany{Fails \u003e 3?}
    TooMany -->|Yes| SafeMode
    TooMany -->|No| Start
    
    ResetFail --> Start
    SafeMode --> Cooldown[Wait 5 min]
    Cooldown --> Start
    
    style SafeMode fill:#f44336
    style Trade fill:#4caf50
    style Stop fill:#ff5722
```

## Deployment Architecture

```mermaid
graph TB
    subgraph "Arbitrum Sepolia"
        Contract[DemoMarket.sol<br/>Prediction Market]
    end
    
    subgraph "Envio Infrastructure"
        Indexer[HyperIndex<br/>Event Indexer]
        GraphQL[GraphQL API]
    end
    
    subgraph "Agent Server"
        Backend[Rust Backend<br/>ArbiShark]
        API[REST API<br/>Port 3000]
        WS[WebSocket<br/>Real-time Updates]
    end
    
    subgraph "Frontend"
        Dashboard[Dashboard<br/>index.html]
    end
    
    Contract -->|Events| Indexer
    Indexer --> GraphQL
    GraphQL -->|Query| Backend
    Backend --> API
    Backend --> WS
    API --> Dashboard
    WS --> Dashboard
    
    style Contract fill:#ff9800
    style Indexer fill:#00bcd4
    style Backend fill:#4caf50
    style Dashboard fill:#9c27b0
```

## State Management

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Running: Start Agent
    Running --> Trading: Opportunity Found
    Trading --> Running: Trade Complete
    Trading --> SafeMode: Trade Failed
    Running --> SafeMode: Health Check Failed
    SafeMode --> Cooldown: Enter Cooldown
    Cooldown --> Idle: Cooldown Complete
    Running --> Stopped: User Revokes Permission
    Stopped --> [*]
    
    note right of SafeMode
        Triggered by:
        - Data delay \u003e 5s
        - 3+ consecutive failures
        - Envio unreachable
    end note
```

---

## Key Design Decisions

### 1. Permission-First Architecture
- **Why**: Solves the trust problem in autonomous agents
- **How**: ERC-7715 cryptographic enforcement, not trust-based limits
- **Benefit**: Users maintain control without popup fatigue

### 2. Envio as Safety Gate
- **Why**: Stale data leads to bad trades
- **How**: Health checks every query, automatic halt on delay
- **Benefit**: Safe automation with real-time data guarantees

### 3. Pluggable MarketClient
- **Why**: Template for any Arbitrum protocol
- **How**: Trait-based abstraction, easy to swap implementations
- **Benefit**: Reusable for DEXs, NFTs, games, etc.

### 4. Adaptive Strategy
- **Why**: Intelligent budgeting within limits
- **How**: Conservative/Normal/Aggressive modes based on remaining allowance
- **Benefit**: Maximizes opportunities while respecting constraints

---

## Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Language** | Rust | Performance, safety, WASM-ready for Stylus |
| **Permissions** | ERC-7715 | Standard for advanced permissions |
| **Indexing** | Envio HyperIndex | Low-latency, Arbitrum-native |
| **API** | Warp (Rust) | Fast, async, WebSocket support |
| **Frontend** | Vanilla HTML/JS | Simple, no build step needed |
| **Deployment** | Arbitrum Sepolia | Testnet for demo, mainnet-ready |

---

## Security Considerations

1. **Permission Enforcement**: All trades go through `PermissionGuard`
2. **Data Validation**: GraphQL errors checked, timeouts enforced
3. **Fail-Safe Defaults**: Agent halts on any uncertainty
4. **No Private Keys in Code**: Environment variables only
5. **Rate Limiting**: Configurable poll intervals
6. **Audit Trail**: All trades logged with timestamps

---

## Scalability

- **Horizontal**: Multiple agents can run independently
- **Vertical**: Single agent handles dozens of markets
- **Data**: Envio scales with Arbitrum network
- **API**: WebSocket for efficient real-time updates

---

## Future Enhancements

1. **Stylus Integration**: On-chain arbitrage verification
2. **Orbit Support**: Multi-chain agent deployment
3. **Advanced Strategies**: ML-based opportunity detection
4. **MEV Protection**: Flashbots-style private transactions
5. **Governance**: DAO-controlled parameter updates
