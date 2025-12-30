#![allow(dead_code)]
use serde::{Deserialize, Serialize};


// represents a polymarket prediction market
#[derive(Debug, Clone , Serialize , Deserialize)]
pub struct Market {
    pub id : String , // unique market ID 
    pub question : String , // Human redabale question
    pub slug : String , // url friendly name
    pub outcomes : Vec<String> , // ["yes" , "no"]
    pub outcome_prices : Vec<f64> , // [0.5 , 0.5]
    pub clob_token_ids : Vec<String> ,  // Token Ids for trading 
    pub best_bid : Option<f64> , // highest by price across outcomes 
    pub best_ask : Option<f64> ,  // lowest sell price across outcomes
    pub maker_base_fee : u32 ,   // In basis points (eg : 0) -> fees if you add liquidity 
    pub taker_base_fee : u32 , // In basis points (eg : 200 = 2%) -> fees if you remove liquidity 
    pub liquidity : f64 ,  // Depth of the market 
    pub volume_24hr : f64 , // trading activity 
    pub active : bool ,  /// is market live ? 
    pub accepting_orders : bool  // can you trade right now ? 
}

// Single price level in order book 
// means -> someone wants to buy/sell Size tokens at price 
// For outcome tokens:
// Buy YES → betting YES happens
// Sell YES → betting YES does NOT happen
// Buy NO → betting NO happens

// Economically:
// Buying YES ≈ selling NO
// Buying NO ≈ selling YES
// But order books are separate, so prices differ.

#[derive(Debug, Clone , Serialize , Deserialize)]
pub struct PriceLevel { 
    pub price : f64 , 
    pub size : f64   // Changed to f64 to match execution_price logic
}

// Order book for a single token 
// You trade YES and NO independently, each has its own order book.
// BIDS:
// 0.49 -> 500 tokens
// 0.48 -> 800 tokens

// ASKS:
// 0.51 -> 400 tokens
// 0.52 -> 700 tokens
#[derive(Debug, Clone , Serialize , Deserialize)]
pub struct OrderBook { 
    pub token_id : String , 
    pub bids : Vec<PriceLevel> , 
    pub asks : Vec<PriceLevel> ,  // Added missing comma
    pub timestamp : u64 
}

// Executed trade records 
// this is a historical fill -> bot 200 yes at 0.49
// Trades are used to estimate latency , detect arbitrage selection , compute VWAP , measure slippage 
#[derive(Debug, Clone , Serialize , Deserialize)]
pub struct Trade {
    pub id : String , 
    pub token_id : String , 
    pub price : f64 ,   // Added missing comma
    pub size : f64 , 
    pub side : Side , 
    pub timestamp : u64 
}

// Order side 
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Side { 
    Buy , 
    Sell 
}

// Arbitrage signal
// core invariant -> YES_price + NO_price ≈ 1
// example arbitrage _> yes = 0.48 , no = 0.47 -> Sum = 0.95 -> one of them settles at $1
// guarenteed profit = 0.05 - fees 
#[derive(Debug, Clone)]
pub struct ArbitrageSignal {
    pub market_id : String , 
    pub spread : f64 ,  // how much the price deviates from 1 
    pub edge : f64 , // Expected profit per unit 
    pub recommended_side : Side , 
    pub yes_price : f64 , 
    pub no_price : f64 
}

// Execution resutl 
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub filed_size : f64  , 
    pub execution_price : f64 , 
    pub fee_paid : f64 , 
    pub slippage : f64 , 
    pub total_cost : f64 , 
    pub success : bool
}


// Implementaion for Market 

impl Market {

    // check if the price sum to exactly 1.0 (no arbitrage)
    pub fn is_balanced(&self) -> bool {
        let sum: f64 = self.outcome_prices.iter().sum();
        (sum - 1.0).abs() < 0.001
    }

    // get the spread (deviation from balanced)
    pub fn get_spread(&self) -> f64 {
        let sum: f64 = self.outcome_prices.iter().sum();
        (sum - 1.0).abs()
    }

    // get YES token price (assumes binary market)
    pub fn yes_price(&self) -> f64 {
        self.outcome_prices.get(0).copied().unwrap_or(0.0)
    }


    // get No token price (assumes binary market)
    pub fn no_price(&self) -> f64 {
        self.outcome_prices.get(1).copied().unwrap_or(0.0)
    }


    // get taker fee as decimal (eg : 0.02 for 2%)
    pub fn taker_fee_rate(&self) -> f64 {
        self.taker_base_fee as f64 / 10000.0
    }
}

// Implemtation of OrderBook 
impl OrderBook {
    // get best bid price 
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|l| l.price)
    }

    // get best ask price 
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|l| l.price)
    }

    // get midpoint price
    pub fn midpoint(&self) -> Option<f64> {
        let best_bid = self.best_bid();
        let best_ask = self.best_ask();
        if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
            Some((bid + ask) / 2.0)
        } else {
            None
        }
    } 

    // get bid ask price 
    pub fn spred(&self)-> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    // get total liquidity on the bid side 
    pub fn total_bid_liquidity(&self) -> f64 {
        self.bids.iter().map(|l| l.size).sum()
    }

    // get total liquidity on the ask side 
    pub fn total_ask_liquidity(&self) -> f64 {
        self.asks.iter().map(|l| l.size).sum()
    }

    // calculates given price for a give size (walks the book)
    pub fn execution_price(&self, size: f64, side: Side) -> Option<f64> {
        let levels = match side {
            Side::Buy => &self.asks,
            Side::Sell => &self.bids,
        };

        let mut remaining = size;
        let mut total_cost = 0.0;

        for level in levels {
            let fill = remaining.min(level.size);
            total_cost += fill * level.price;
            remaining -= fill;
            if remaining <= 0.0 {
                break;
            }
        }

        if remaining > 0.0 {
            None // Not enough liquidity
        } else {
            Some(total_cost / size) // Volume-weighted average price
        }
    }



}