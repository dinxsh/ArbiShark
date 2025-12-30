
# POLYSHARK - bot for trading on polymarket

> *“If markets contradict themselves, eat the contradiction.”*

Short, memorable, fits the “sharky” arbitrage theme, and still professional.

---

# 🎯 What this bot does (recap)

* Uses a **fake USDC wallet** (`1000.00`)
* Simulates **linked markets**
* Introduces **price drift & mispricing**
* Detects **logical arbitrage**
* Executes **buy + sell**
* Waits for **mean reversion**
* Closes positions
* Tracks **realistic PnL**

---

# 🧠 Architecture (simple & correct)

```
wallet.rs        → USDC wallet & positions
market.rs        → market + price drift
constraint.rs    → logical relationships
arb.rs           → arbitrage logic
engine.rs        → main loop
```

---

# 📊 What to expect when you run it

* Equity fluctuates
* Most trades win small
* Rare drawdowns
* Clear mean reversion behavior
* You’ll **see when logic breaks**

This is **real signal validation**.

---

# 🧠 How this upgrades to Solana later

| Paper bot         | Solana version    |
| ----------------- | ----------------- |
| `Wallet.usdc`     | SPL Token balance |
| `Market.price`    | On-chain price    |
| `try_arbitrage()` | Atomic tx         |
| `try_close()`     | Exit tx           |



---

# 🚀 Next upgrades (recommended order)

1. Multi-market constraints (A+B+C=1)
2. Random latency injection
3. Fee modeling
4. Monte Carlo runs
5. Market graph
6. Then → Solana devnet

---



## 1. The key idea: “Execution Simulator”

Your wallet stays a float.
What changes is **how trades are executed**.

```
Strategy  →  Execution Simulator  →  Wallet Update
```

The simulator is where reality lives.

---

## 2. Gas / Fees (modeled correctly)

On Polymarket you pay:

* taker fee
* sometimes maker fee
* protocol fee
* (no “gas” like Solana, but execution cost exists)

Model it as:

```rust
struct FeeModel {
    taker_fee: f64,   // e.g. 0.02 = 2%
    maker_fee: f64,   // often 0
}
```

Apply on every trade:

```rust
fee = notional * fee_rate;
wallet.usdc -= fee;
```

This is **far more realistic** than ignoring gas.

---

## 3. Slippage (not linear — this matters)

Your previous assumption:

```
price += size / liquidity
```

That’s optimistic.

Realistic model (AMM-like):

```
effective_price = price * (1 + k * (size / liquidity)^α)
```

Where:

* `k ≈ 1.0`
* `α ≈ 1.3 – 1.8`

This punishes oversizing *hard*.

---

## 4. Partial fills (this kills fake alpha)

In real markets:

* your order might not fully fill
* especially when chasing mispricing

Model it as probability:

```rust
fill_ratio = min(1.0, liquidity / (liquidity + size * beta))
```

Then:

```
filled_size = size * fill_ratio
```

Unfilled portion = dead.

This alone wipes out many fake strategies.

---

## 5. Latency & adverse selection

You **never** get the price you saw.

Model this:

```rust
observed_price → wait Δt → execute at drifted_price
```

Implementation:

* detect signal at `t`
* wait 1–3 ticks
* execute at updated price

If the opportunity disappears → trade loses.

This is CRITICAL.

---

## 6. Position sizing (fixing your oversizing concern)

Never use fixed `trade_size`.

Use **dynamic sizing**:

```
size = min(
  wallet.usdc * 0.02,        // max 2% of capital
  liquidity * 0.01,          // max 1% of book
  spread / max_spread * cap  // confidence scaling
)
```

If spread is barely profitable → size tiny.

This is how pros survive.

---

## 7. Full realistic execution formula

When a signal fires:

```
1. observe spread
2. compute expected profit
3. subtract fees
4. subtract slippage
5. apply latency penalty
6. size position
7. simulate partial fill
8. update wallet
```

If expected profit ≤ 0 → **don’t trade**

This single rule saves money.

---

## 8. Concrete example (Polymarket-like)

Observed:

```
YES = 0.62
NO  = 0.43
spread = +5%
```

Execution reality:

* latency moves YES → 0.60
* slippage adds 1.2%
* fee = 2%
* partial fill = 70%

Final edge:

```
5% - 1.2% - 2% - latency_loss ≈ 0.5%
```

Still profitable — but barely.

That’s **real life**.

---

## 9. How this fits your variable-wallet idea

You still do:

```
wallet.usdc = 1000.00
```

But **updates happen only through the execution simulator**, never directly.

Think:

```rust
ExecutionEngine::execute(order, &mut wallet)
```

Your wallet never lies — the execution engine does the pain.

---

## 10. This is how you know the strategy is real

After you add:

* fees
* slippage
* latency
* partial fills
* sizing

If PnL is still positive after **1,000+ trades**:

✅ you have real edge
❌ if not, it was fake alpha

Most strategies die here. That’s a good thing.


Perfect — **this changes everything** 👍
If you have **access to the Polymarket API**, then you can stop guessing and **derive real execution parameters from live data**.

What you want is **calibration**, not assumptions.

Below is the **exact, professional workflow** real trading teams use when they *do* have API access.

---

# Big picture (important)

You are **NOT** trying to:

* perfectly predict fees, slippage, fills

You **ARE** trying to:

* **estimate upper bounds** on costs
* **reject trades that don’t survive reality**

So your simulator becomes:

```
Polymarket API  →  Parameter Estimator  →  Execution Simulator  →  Wallet
```

---

# 1. What data you need from Polymarket API

From the API you should pull **continuously**:

### Market data

* Best bid / ask
* Last traded price
* Volume
* Liquidity / depth (if exposed)
* Outcome prices

### Trade data (VERY important)

* Executed trades
* Size
* Execution price
* Timestamp

If you have **order book snapshots**, even better.

---

# 2. Fees: how to measure them exactly

This is the **easiest** one.

### From API

You’ll usually see:

* `fee` field
* or infer from `size × price` vs actual balance change

### How to compute

For each trade:

```
fee_rate = (expected_cost - actual_cost) / expected_cost
```

Collect 100+ samples.

Then set:

```
fee_rate = P95(fee_rate_samples)
```

📌 Use **95th percentile**, not average.

This ensures you don’t overfit optimism.

---

# 3. Slippage: how to estimate it properly

Slippage = difference between **observed price** and **execution price**.

### For every trade:

```
slippage = (exec_price - mid_price_at_signal) / mid_price_at_signal
```

Store:

* trade size
* market liquidity proxy
* slippage

You’ll see a curve.

### Fit a function

Empirically you’ll find:

```
slippage ≈ k × (size / volume)^α
```

You don’t need fancy ML.

Just fit:

* `k`
* `α`

Then **inflate them** by 20–30% in simulation.

---

# 4. Partial fills: how to measure fill probability

If the API gives:

* requested size
* filled size

You can directly compute:

```
fill_ratio = filled / requested
```

Group by:

* size buckets
* liquidity buckets

Then build:

```
fill_ratio(size, liquidity)
```

In simulation:

* sample from historical distribution
* or use worst-case percentile

📌 Again: **use P25, not mean**.

---

# 5. Latency & adverse selection (this is the edge killer)

This is the most important one.

### What you measure

For every opportunity:

1. Timestamp when price changed
2. Timestamp when trade executed
3. Price movement in between

Compute:

```
latency = t_exec - t_signal
adverse_move = price_exec - price_signal
```

You’ll notice:

* fast markets → worse adverse moves
* thin liquidity → brutal slippage

### In simulation

When a signal fires:

* delay execution by sampled latency
* apply observed adverse move distribution

This makes your simulator **hostile** — which is good.

---

# 6. Position sizing: derive it from data (not vibes)

From historical data:

* which sizes had positive expectancy?
* which sizes consistently lost due to slippage?

Plot:

```
Expected PnL vs trade size
```

You’ll find:

* a clear “safe zone”
* then a cliff

Your sizing rule becomes:

```
max_size = argmax(ExpectedPnL(size))
```

And then:

```
actual_size = min(
  max_size,
  wallet_equity × 1–2%
)
```

This is **data-driven sizing**.

---

# 7. Putting it all together (REAL decision rule)

When a signal fires:

```
raw_edge = observed_spread

expected_costs =
    fee_estimate +
    slippage_estimate(size) +
    adverse_selection_estimate

expected_profit = raw_edge × size - expected_costs
```

### Decision gate (CRITICAL)

```
if expected_profit <= 0:
    skip trade
```

This one rule removes 80% of bad trades.

---

# 8. How this fits your float-wallet model

Your wallet is still:

```
wallet.usdc = 1000.00
```

But updates only happen through:

```
ExecutionSimulator {
  fee_model,
  slippage_model,
  latency_model,
  fill_model,
}
```

So even though it’s a variable, it behaves like **real money**.

---

# 9. Why this is actually better than live trading early

Ironically:

* Paper trading with **real execution parameters**
* is often **more honest** than early live trading

Why?

* no emotions
* no execution panic
* no overconfidence

If it works here, it *usually* works live.

---

# 10. Final truth (important)

Because you have the Polymarket API:

* ❌ you do NOT need to guess
* ❌ you do NOT need perfect fills
* ❌ you do NOT need to trade yet

You can:

* extract reality
* build a hostile simulator
* validate the edge properly

This is **exactly how professional arb desks operate**.

---


Yes — if you have access to the **Polymarket API** (and you do), then you can **measure real market behavior** instead of guessing fees, slippage, fills, and other execution dynamics. That’s exactly what a *serious arb bot* should do. ([Polymarket Documentation][1])

Here’s how to extract *real parameters* from Polymarket and feed them into your simulator so your bot behaves truthfully.

---

# 📡 What the Polymarket API gives you

Polymarket’s APIs include **multiple layers** of useful data:

### 🧾 Market data & discovery

* `GET /markets` → list active markets, outcomes, prices ([Polymarket Expert][2])

### 🧠 Order book & trading data (CLOB)

* `GET /book`, `GET /books` → order book snapshots ([Polymarket][3])
* `/price`, `/midprice` → pricing info ([Polymarket][3])
* `/data/trades` → historical executed trades ([Docs.rs][4])

### 🔥 Place/cancel orders

* `POST /order`, `POST /orders`, etc. for actual trading ([Polymarket][3])

### 🚀 Real-time feeds

* WebSocket streams for trades, orderbook updates, and user fills ([PolyTrack][5])

And note: the APIs are **rate-limited** — you must throttle your requests and handle 429 codes appropriately. ([Polymarket Documentation][1])

---

# 🧮 How to **measure real execution parameters** from the API

You want to avoid guessing fees, slippage, latency, and fills. Here’s how to **derive them objectively**:

---

## ✅ 1. FEES (from trade execution)

Polymarket’s order book (CLOB) has implicit maker/taker fees. You can recover real costs by comparing:

```
observed_exec_cost vs expected_cost_without_fee
```

Where:

* `expected` = price × size from market quote
* `observed` = actual cost returned by API

Compute:

```text
fee_rate = (expected_cost - actual_received) / expected_cost
```

Gather 50–200 samples per market to get a robust estimate. Use a high percentile (e.g., 95th) to model worse cases in your simulator. ([Polymarket Documentation][6])

---

## ✅ 2. SLIPPAGE (from order book snapshots)

Slippage = when execution price differs from the *current market price*.

**Method:**

1. Take a snapshot of `GET /book` data.
2. Record the best bid/ask.
3. Look at the realized price after execution (from trade history or your simulated API order).
4. `slippage = exec_price / mid_price - 1`

Slippage increases superlinearly with trade size. Model it as:

```rust
slippage_pct = k * (size / liquidity)^α
```

Fit `k` and `α` using least squares or bucket averages from real data. ([Polymarket][3])

---

## ✅ 3. FILL RATES (partial fills)

The API returns:

* size requested
* size filled

So you can compute:

```text
fill_ratio = filled_size / requested_size
```

Group these by:

* market liquidity
* size buckets

and then build a **fill rate table** that your simulator uses to decide how much of an order is realistically filled.

Example transfer into simulation:

```rust
filled_size = requested_size * fill_ratio
```

Partial fills are critical: a strategy might *look profitable on paper* but actually never fill enough size to realize the gain.

---

## ✅ 4. LATENCY (market movement during reaction time)

Real arbitrage bots miss opportunities because prices move between:

* when you see a signal
* when your order hits the order book

Use timestamps from:

* WebSocket events
* REST snapshot times

Measure:

```
latency = t_sent - t_received
adverse_move = price_at_exec - price_at_signal
```

Record a distribution of latency and adverse moves and feed that into your execution simulator. This lets you emulate **execution delay** realistically.

---

## ✅ 5. POSITION SIZING (from real market impact)

Instead of hard-coding sizes, derive sizing rules from data:

**Empirical rule:**

```
size_max = argmax(expected_profit(size))
```

Use observed slippage, fees, and fill rates. A good balance rule usually is:

```
size = min(
    wallet_equity * risk_pct,
    liquidity * liquidity_pct,
    edge * confidence_factor
)
```

This prevents over-sizing and ensures trades are feasible in real markets.

---

# 🧠 Using API streams for real-time bot decisions

Rather than polling, use WebSocket channels for:

* **price updates**
* **order book changes**
* **trade streams**
* **user fills**

This will give near-real-time data with minimal latency and allows the bot to react faster than polling. ([PolyTrack][5])

---

# 📊 Practical example: calibrating slippage

1. Poll `/book` for orderbook depth every 50–100 ms.
2. When your simulator predicts a trade at price `p`, look at the book:

   ```
   best_asks vs best_bids
   ```
3. Compute *expected impact* from actual book structure:

   * sum of sizes at each price level
   * how many levels your size would touch
4. Infer slippage cost

This is real-world market slippage, not a guess.

---

# 🧾 Example execution-parameter extraction flow

```
while sampling:
    book_snapshot = GET /book?market=XYZ
    recent_trades = GET /data/trades?market=XYZ
    fees = derive_fee_from(trade_records)
    slippage_parameters = fit_slippage_model(book + trade data)
    fill_rates = derive_fill_ratio(trade_records, requested_sizes)
    latency_distribution = measure_via_websocket_events()
```

Then feed these into:

```
ExecutionSimulator::new(fee_model, slippage_model, fill_model, latency_model)
```

Your wallet doesn’t change — but the **rules that update it do** based on real market behavior.

---

# 📈 Key Tip: Use Streaming + Caching

Due to rate limits, you *cannot poll excessively*. The API enforces limits like:

* ~1500 requests/10s for price endpoints
* ~900 requests/10s for trade/order data
* throttling instead of rejection ([Polymarket Documentation][1])

Best practices:

* Use WebSockets for live updates
* Cache responses with TTL
* Back off politely on 429s

---

## Final Summary

You now have a **true scientific workflow**:

1. Gather real market data via the API
2. Estimate real execution parameters (fees, slippage, fills, latency)
3. Calibrate your paper simulation to match observed behavior
4. Run your arbitrage strategy in a *hostile, realistic environment*
5. Only go live when simulated edge survives real dynamics

This moves your bot from *fantasy profit* to *credible, data-driven trading logic*.

---

If you want, I can provide:
🔹 Rust client code for Polymarket REST + WebSocket
🔹 Functions that estimate slippage from real orderbook data
🔹 A simulator engine that ingests real market behavior

Just tell me which part you want to build first.

[1]: https://docs.polymarket.com/quickstart/introduction/rate-limits?utm_source=chatgpt.com "API Rate Limits - Polymarket Documentation"
[2]: https://polymarket.expert/api/?utm_source=chatgpt.com "Polymarket API: документация и интеграция"
[3]: https://polymarket-292d1b1b.mintlify.app/quickstart/introduction/rate-limits?utm_source=chatgpt.com "API Rate Limits - Polymarket Documentation"
[4]: https://docs.rs/crate/polysqueeze/latest/source/API.md?utm_source=chatgpt.com "polysqueeze 0.1.7 - Docs.rs"
[5]: https://www.polytrackhq.app/blog/polymarket-api-guide?utm_source=chatgpt.com "Polymarket API Guide 2025 | Build Trading Bots & Tools | PolyTrack"
[6]: https://docs.polymarket.com/developers/CLOB/introduction?utm_source=chatgpt.com "CLOB Introduction - Polymarket Documentation"
