# 📡 PolyShark — Polymarket API Reference

Complete documentation for integrating with the Polymarket API to build a data-driven arbitrage bot.

---

## Overview

Polymarket provides **three API layers** for accessing market data and executing trades:

| API Layer | Base URL | Purpose |
|-----------|----------|---------|
| **GAMMA API** | `https://gamma-api.polymarket.com` | Market discovery, events, metadata (GraphQL) |
| **CLOB API** | `https://clob.polymarket.com` | Order book, pricing, trading |
| **WebSocket** | `wss://ws-subscriptions-clob.polymarket.com/ws` | Real-time streaming |

**Blockchain**: Polygon (Chain ID: `137`)

**Status Page**: https://status-clob.polymarket.com/

---

## Rate Limits

### Data API
| Endpoint | Rate Limit |
|----------|------------|
| `/trades` | Throttled |
| `/positions` | Throttled |
| `/closed-positions` | Throttled |

### GAMMA API
| Endpoint | Rate Limit |
|----------|------------|
| `/events` | Throttled |
| `/markets` | Throttled |

### CLOB API — Market Data
| Endpoint | Description |
|----------|-------------|
| `/book` | Single order book |
| `/books` | Multiple order books |
| `/price` | Single token price |
| `/prices` | Multiple token prices |
| `/midprice` | Single midpoint |
| `/midprices` | Multiple midpoints |

### CLOB API — Ledger Endpoints
| Endpoint | Description |
|----------|-------------|
| `/trades` | Trade history |
| `/orders` | Order history |
| `/order` | Single order details |
| `/data/orders` | Historical orders |
| `/data/trades` | Historical trades |
| `/notifications` | User notifications |

### CLOB API — Trading Endpoints
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/order` | POST | Place single order |
| `/order` | DELETE | Cancel single order |
| `/orders` | POST | Place batch orders |
| `/orders` | DELETE | Cancel batch orders |
| `/cancel-all` | POST/DELETE | Cancel all orders |
| `/cancel-market-orders` | POST/DELETE | Cancel market-specific orders |

**Rate Limit Behavior:**
- **Throttling**: Requests over the limit are delayed/queued rather than dropped
- **Burst Allowances**: Some endpoints allow short bursts above the sustained rate
- **Time Windows**: Limits reset based on sliding time windows (per 10 seconds, per minute)

---

## GAMMA API — Market Discovery

### GET /markets

Retrieves a list of markets with filtering and sorting options.

```bash
curl --request GET \
  --url https://gamma-api.polymarket.com/markets
```

#### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `limit` | integer | Number of results (≥ 0) |
| `offset` | integer | Pagination offset (≥ 0) |
| `order` | string | Comma-separated fields to order by |
| `ascending` | boolean | Sort order |
| `id` | string | Filter by market ID |
| `slug` | string | Filter by market slug |
| `clob_token_ids` | string | Filter by CLOB token IDs |
| `condition_ids` | string | Filter by condition IDs |
| `liquidity_num_min` | number | Minimum liquidity |
| `liquidity_num_max` | number | Maximum liquidity |
| `volume_num_min` | number | Minimum volume |
| `volume_num_max` | number | Maximum volume |
| `start_date_min` | ISO date | Start date range (min) |
| `start_date_max` | ISO date | Start date range (max) |
| `end_date_min` | ISO date | End date range (min) |
| `end_date_max` | ISO date | End date range (max) |
| `tag_id` | string | Filter by tag |
| `closed` | boolean | Filter closed markets |

#### Response Schema (Key Fields)

```json
[
  {
    "id": "<market_id>",
    "question": "Will Bitcoin reach $100k by Dec 2024?",
    "conditionId": "<condition_id>",
    "slug": "bitcoin-100k-dec-2024",
    "outcomes": "[\"Yes\", \"No\"]",
    "outcomePrices": "[0.65, 0.35]",
    "clobTokenIds": "[\"token_yes_id\", \"token_no_id\"]",
    "volume": "1250000",
    "volumeNum": 1250000,
    "volume24hr": 45000,
    "volume1wk": 180000,
    "liquidity": "85000",
    "liquidityNum": 85000,
    "liquidityClob": 85000,
    "active": true,
    "closed": false,
    "acceptingOrders": true,
    "enableOrderBook": true,
    "makerBaseFee": 0,
    "takerBaseFee": 200,
    "orderPriceMinTickSize": 0.01,
    "orderMinSize": 5,
    "bestBid": 0.64,
    "bestAsk": 0.66,
    "lastTradePrice": 0.65,
    "spread": 0.02,
    "oneDayPriceChange": 0.03,
    "oneHourPriceChange": 0.01,
    "negRisk": false
  }
]
```

**Important Fields for Arbitrage:**

| Field | Description |
|-------|-------------|
| `clobTokenIds` | Token IDs for CLOB trading |
| `outcomePrices` | Current YES/NO prices |
| `bestBid` / `bestAsk` | Current bid/ask spread |
| `makerBaseFee` / `takerBaseFee` | Fee rates (in basis points) |
| `liquidityClob` | Available liquidity on CLOB |
| `spread` | Current bid-ask spread |

---

### GET /events

Fetches a list of events (groups of related markets).

```bash
curl --request GET \
  --url https://gamma-api.polymarket.com/events
```

#### Response Schema (Key Fields)

```json
[
  {
    "id": "<event_id>",
    "ticker": "BTC-100K",
    "slug": "bitcoin-price-2024",
    "title": "Bitcoin Price Milestones",
    "description": "Markets on Bitcoin reaching key price levels",
    "active": true,
    "closed": false,
    "liquidity": 500000,
    "volume": 2500000,
    "openInterest": 150000,
    "category": "Crypto",
    "negRisk": true,
    "negRiskMarketID": "<neg_risk_market_id>",
    "negRiskFeeBips": 50,
    "markets": [
      { /* nested market objects */ }
    ]
  }
]
```

---

## CLOB API — Order Book & Pricing

### GET /book

Get order book for a single token.

```bash
curl "https://clob.polymarket.com/book?token_id=<token_id>"
```

**Response:**
```json
{
  "market": "<market_id>",
  "asset_id": "<token_id>",
  "bids": [
    { "price": "0.64", "size": "1500" },
    { "price": "0.63", "size": "3200" }
  ],
  "asks": [
    { "price": "0.65", "size": "800" },
    { "price": "0.66", "size": "2100" }
  ],
  "hash": "<book_hash>",
  "timestamp": "1703424000000"
}
```

---

### GET /books

Get order books for multiple tokens (batched).

```bash
curl "https://clob.polymarket.com/books?token_ids=<id1>,<id2>,<id3>"
```

---

### GET /price

Get price for a single token.

```bash
curl "https://clob.polymarket.com/price?token_id=<token_id>&side=BUY"
```

**Parameters:**
- `token_id`: Token identifier
- `side`: `BUY` or `SELL`

**Response:**
```json
{
  "price": "0.65"
}
```

---

### GET /prices

Get prices for multiple tokens.

```bash
curl "https://clob.polymarket.com/prices?token_ids=<id1>,<id2>&side=BUY"
```

---

### GET /midpoint

Get midpoint price for a single token.

```bash
curl "https://clob.polymarket.com/midpoint?token_id=<token_id>"
```

**Response:**
```json
{
  "mid": "0.645"
}
```

---

### GET /midpoints

Get midpoint prices for multiple tokens.

```bash
curl "https://clob.polymarket.com/midpoints?token_ids=<id1>,<id2>"
```

---

## CLOB API — Trading (Authenticated)

### Authentication

Trading requires API credentials. Generate them using the SDK:

```python
from py_clob_client.client import ClobClient

client = ClobClient(
    "https://clob.polymarket.com",
    key=PRIVATE_KEY,
    chain_id=137,
    signature_type=1,  # 0=EOA, 1=Magic/Email, 2=Proxy
    funder=FUNDER_ADDRESS
)

# Create or derive API credentials
creds = client.create_or_derive_api_creds()
client.set_api_creds(creds)
```

**Signature Types:**
| Value | Type | Description |
|-------|------|-------------|
| `0` | EOA | MetaMask, hardware wallets, direct private key |
| `1` | Magic/Email | Email login with Magic Link |
| `2` | Proxy | Browser wallet proxy contracts |

---

### Order Types

| Type | Description |
|------|-------------|
| `GTC` | Good Till Cancelled — remains active until filled or cancelled |
| `GTD` | Good Till Date — expires at specified time |
| `FOK` | Fill Or Kill — must fill completely or cancel |
| `IOC` | Immediate Or Cancel — fill what's possible, cancel rest |

---

### Place Limit Order

```python
from py_clob_client.clob_types import OrderArgs, OrderType
from py_clob_client.order_builder.constants import BUY, SELL

# Create limit order
order = OrderArgs(
    token_id="<token_id>",
    price=0.64,
    size=100.0,  # Number of shares
    side=BUY
)

signed_order = client.create_order(order)
response = client.post_order(signed_order, OrderType.GTC)
print(response)
```

---

### Place Market Order

```python
from py_clob_client.clob_types import MarketOrderArgs, OrderType
from py_clob_client.order_builder.constants import BUY

# Create market order (by $ amount)
market_order = MarketOrderArgs(
    token_id="<token_id>",
    amount=25.0,  # Dollar amount to spend
    side=BUY,
    order_type=OrderType.FOK
)

signed_order = client.create_market_order(market_order)
response = client.post_order(signed_order, OrderType.FOK)
print(response)
```

---

### Get Open Orders

```python
from py_clob_client.clob_types import OpenOrderParams

open_orders = client.get_orders(OpenOrderParams())
print(open_orders)
```

---

### Cancel Order

```python
# Cancel single order
client.cancel(order_id)

# Cancel all orders
client.cancel_all()
```

---

### Get Trades

```python
# Get last trade price
last_price = client.get_last_trade_price("<token_id>")

# Get trade history (requires auth)
trades = client.get_trades()
print(trades)
```

---

## WebSocket API — Real-Time Streaming

### Connection

```javascript
const ws = new WebSocket('wss://ws-subscriptions-clob.polymarket.com/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({
    type: 'subscribe',
    channel: 'market',
    markets: ['<market_id_1>', '<market_id_2>']
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(data);
};
```

### Available Channels

| Channel | Description | Auth Required |
|---------|-------------|---------------|
| `market` | Price updates | No |
| `book` | Order book changes | No |
| `trades` | Executed trades | No |
| `user` | Your orders/fills | Yes |

### Message Types

**Price Update:**
```json
{
  "type": "price_update",
  "market_id": "<market_id>",
  "token_id": "<token_id>",
  "price": 0.652,
  "timestamp": 1703424000123
}
```

**Trade:**
```json
{
  "type": "trade",
  "market_id": "<market_id>",
  "price": 0.65,
  "size": 250,
  "side": "BUY",
  "timestamp": 1703424000456
}
```

**Order Book Update:**
```json
{
  "type": "book_update",
  "market_id": "<market_id>",
  "bids": [...],
  "asks": [...],
  "timestamp": 1703424000789
}
```

---

## Token Allowances (EOA Users)

If using MetaMask or hardware wallet, you must set token allowances before trading:

```python
# Check if allowances are needed
needs_allowance = client.needs_allowance()

# Set allowances
if needs_allowance:
    client.set_allowances()
```

---

## Python Client Reference

### Installation

```bash
pip install py-clob-client
```

### Full Client Example

```python
from py_clob_client.client import ClobClient
from py_clob_client.clob_types import (
    OrderArgs, MarketOrderArgs, OrderType, 
    OpenOrderParams, BookParams
)
from py_clob_client.order_builder.constants import BUY, SELL

# Initialize client
HOST = "https://clob.polymarket.com"
CHAIN_ID = 137
PRIVATE_KEY = "<your-private-key>"
FUNDER = "<your-funder-address>"

client = ClobClient(
    HOST,
    key=PRIVATE_KEY,
    chain_id=CHAIN_ID,
    signature_type=1,
    funder=FUNDER
)
client.set_api_creds(client.create_or_derive_api_creds())

# === READ-ONLY OPERATIONS ===

# Get server status
print(client.get_ok())
print(client.get_server_time())

# Get markets
markets = client.get_simplified_markets()
print(markets["data"][:5])

# Get prices
token_id = "<token_id>"
mid = client.get_midpoint(token_id)
price = client.get_price(token_id, side="BUY")
book = client.get_order_book(token_id)
books = client.get_order_books([BookParams(token_id=token_id)])

print(f"Midpoint: {mid}")
print(f"Buy Price: {price}")
print(f"Book: {book.bids[:3]}, {book.asks[:3]}")

# === TRADING OPERATIONS ===

# Place limit order
order = OrderArgs(
    token_id=token_id,
    price=0.50,
    size=10.0,
    side=BUY
)
signed = client.create_order(order)
resp = client.post_order(signed, OrderType.GTC)
print(f"Order placed: {resp}")

# Get open orders
open_orders = client.get_orders(OpenOrderParams())
print(f"Open orders: {len(open_orders)}")

# Cancel all orders
client.cancel_all()
```

---

## TypeScript Client Reference

### Installation

```bash
npm install @polymarket/clob-client ethers
```

### Full Client Example

```typescript
import { ClobClient, OrderType, Side } from "@polymarket/clob-client";
import { Wallet } from "@ethersproject/wallet";

const HOST = "https://clob.polymarket.com";
const CHAIN_ID = 137;
const PRIVATE_KEY = "<your-private-key>";
const FUNDER = "<your-funder-address>";

const signer = new Wallet(PRIVATE_KEY);

(async () => {
  // Get API credentials
  const creds = await new ClobClient(HOST, CHAIN_ID, signer).createOrDeriveApiKey();
  
  // Initialize authenticated client
  const client = new ClobClient(
    HOST,
    CHAIN_ID,
    signer,
    creds,
    1,        // signature_type: 0=EOA, 1=Magic, 2=Proxy
    FUNDER
  );

  // Place order
  const resp = await client.createAndPostOrder(
    {
      tokenID: "<token_id>",
      price: 0.50,
      side: Side.BUY,
      size: 10
    },
    { tickSize: "0.01", negRisk: false },
    OrderType.GTC
  );
  
  console.log(resp);
})();
```

---

## Parameter Extraction for Simulation

### Measuring Real Fees

```python
def measure_fees(trades: list) -> float:
    """Calculate P95 fee rate from historical trades."""
    fee_rates = []
    for trade in trades:
        expected = trade["size"] * trade["quoted_price"]
        actual = trade["net_cost"]
        rate = (expected - actual) / expected
        fee_rates.append(rate)
    
    return sorted(fee_rates)[int(len(fee_rates) * 0.95)]
```

### Measuring Slippage

```python
def measure_slippage(books: list, trades: list) -> tuple:
    """Fit slippage curve: slippage = k * (size/liquidity)^alpha"""
    data_points = []
    for book, trade in zip(books, trades):
        mid = (book["best_bid"] + book["best_ask"]) / 2
        slippage = (trade["exec_price"] - mid) / mid
        size_ratio = trade["size"] / book["total_liquidity"]
        data_points.append((size_ratio, slippage))
    
    # Fit power law
    return fit_power_law(data_points)  # Returns (k, alpha)
```

### Measuring Fill Rates

```python
def measure_fill_rates(orders: list) -> dict:
    """Calculate fill rate by size bucket."""
    buckets = defaultdict(list)
    for order in orders:
        bucket = size_bucket(order["requested"])
        fill_ratio = order["filled"] / order["requested"]
        buckets[bucket].append(fill_ratio)
    
    return {k: mean(v) for k, v in buckets.items()}
```

### Measuring Latency

```python
def measure_latency(events: list) -> dict:
    """Calculate latency distribution."""
    latencies = []
    for i in range(len(events) - 1):
        if events[i]["type"] == "signal" and events[i+1]["type"] == "execution":
            latency = events[i+1]["timestamp"] - events[i]["timestamp"]
            latencies.append(latency)
    
    return {
        "mean": mean(latencies),
        "p50": percentile(latencies, 0.50),
        "p95": percentile(latencies, 0.95),
        "p99": percentile(latencies, 0.99)
    }
```

---

## Integration Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     DISCOVERY PHASE                             │
├─────────────────────────────────────────────────────────────────┤
│  GET gamma-api.polymarket.com/markets   → Find active markets   │
│  GET gamma-api.polymarket.com/events    → Get related markets   │
│  Extract clobTokenIds, outcomePrices    → Identify tokens       │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                     CALIBRATION PHASE                           │
├─────────────────────────────────────────────────────────────────┤
│  GET /book (×100 samples)               → Order book snapshots  │
│  GET /data/trades                       → Historical executions │
│  WebSocket stream                       → Latency measurement   │
│  Compute fee_rate, k, α, fills          → Parameter estimation  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                     SIMULATION PHASE                            │
├─────────────────────────────────────────────────────────────────┤
│  ExecutionSimulator::new(                                       │
│      fee_model: FeeModel { rate: derived_fee },                 │
│      slippage_model: SlippageModel { k, alpha },                │
│      fill_model: FillModel { rates: derived_fills },            │
│      latency_model: LatencyModel { distribution }               │
│  )                                                              │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                     LIVE TRADING PHASE                          │
├─────────────────────────────────────────────────────────────────┤
│  WebSocket stream        → Real-time price updates              │
│  detect_arbitrage()      → Signal generation                    │
│  should_trade()          → Cost-aware filtering                 │
│  POST /order             → Order execution                      │
│  GET /orders             → Confirm fills                        │
└─────────────────────────────────────────────────────────────────┘
```

---

## Error Handling

| Status Code | Meaning | Action |
|-------------|---------|--------|
| 200 | Success | Process response |
| 400 | Bad Request | Check parameters |
| 401 | Unauthorized | Verify API credentials |
| 429 | Rate Limited | Back off exponentially |
| 500 | Server Error | Retry with delay |

```python
import time

def with_retry(func, max_retries=5):
    """Execute function with exponential backoff."""
    delay = 0.1
    for attempt in range(max_retries):
        try:
            return func()
        except RateLimitError:
            time.sleep(delay)
            delay *= 2
        except ServerError:
            time.sleep(delay)
            delay *= 2
    raise MaxRetriesExceededError()
```

---

## Resources

| Resource | URL |
|----------|-----|
| **Official Docs** | https://docs.polymarket.com |
| **CLOB Introduction** | https://docs.polymarket.com/developers/CLOB/introduction |
| **Rate Limits** | https://docs.polymarket.com/quickstart/introduction/rate-limits |
| **Status Page** | https://status-clob.polymarket.com |
| **Python Client** | https://github.com/Polymarket/py-clob-client |
| **TypeScript Client** | https://github.com/Polymarket/clob-client |
| **Exchange Contracts** | https://github.com/Polymarket/ctf-exchange |
| **Audit Report** | https://github.com/Polymarket/ctf-exchange/blob/main/audit/ChainSecurity_Polymarket_Exchange_audit.pdf |

---

## Fee Structure

Polymarket uses **basis points** for fees:

| Fee Type | Typical Value | Calculation |
|----------|---------------|-------------|
| Maker Fee | 0 bps | Free for liquidity providers |
| Taker Fee | ~200 bps (2%) | Applied to market orders |

```python
def calculate_fee(notional: float, is_maker: bool, market: dict) -> float:
    """Calculate trading fee."""
    fee_bps = market["makerBaseFee"] if is_maker else market["takerBaseFee"]
    return notional * (fee_bps / 10000)
```

---

## Neg Risk Markets

Some markets use **Negative Risk** structure where:
- Multiple outcomes share a single collateral pool
- Prices across all outcomes sum to 1.00
- Special fee structure applies via `negRiskFeeBips`

Check for neg risk markets:
```python
if market["negRisk"]:
    neg_risk_market_id = market["negRiskMarketID"]
    neg_risk_fee = market["negRiskFeeBips"]
```
