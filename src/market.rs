use crate::types::{Market, OrderBook, PriceLevel};
use std::error::Error;

#[allow(dead_code)]
pub struct MarketDataProvider {
    client: reqwest::Client,
    envio_url: String,
}

impl MarketDataProvider {
    pub fn new(envio_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            envio_url: envio_url.to_string(),
        }
    }

    pub async fn fetch_markets(&self) -> Result<Vec<Market>, Box<dyn Error>> {
        // In a real implementation, this would query the Envio GraphQL endpoint
        // let query = "{ markets { id, question, outcomes, ... } }";
        // let resp = self.client.post(&self.envio_url).json(&query).send().await?;
        
        // Return mock data for demonstration
        println!("🌐 Fetching market data from Envio Indexer at {}...", self.envio_url);
        
        Ok(vec![
            Market {
                id: "0x123".to_string(),
                question: "Will ETH be above $4000 on Jan 1?".to_string(),
                slug: "eth-jan-1".to_string(),
                outcomes: vec!["Yes".to_string(), "No".to_string()],
                outcome_prices: vec![0.45, 0.55],
                clob_token_ids: vec!["t1".to_string(), "t2".to_string()],
                best_bid: Some(0.44),
                best_ask: Some(0.46),
                maker_base_fee: 0,
                taker_base_fee: 200, // 2%
                liquidity: 100000.0,
                volume_24hr: 50000.0,
                active: true,
                accepting_orders: true,
            },
            Market {
                id: "0x456".to_string(),
                question: "Will BTC be above $100k in 2024?".to_string(),
                slug: "btc-2024".to_string(),
                outcomes: vec!["Yes".to_string(), "No".to_string()],
                outcome_prices: vec![0.40, 0.40], // Sum 0.80 -> 20% arb!
                clob_token_ids: vec!["t3".to_string(), "t4".to_string()],
                best_bid: Some(0.39),
                best_ask: Some(0.41),
                maker_base_fee: 0,
                taker_base_fee: 200,
                liquidity: 500000.0,
                volume_24hr: 120000.0,
                active: true,
                accepting_orders: true,
            }
        ])
    }

    /// Fetch order book for a market
    pub async fn fetch_order_book(&self, token_id: &str) -> Result<OrderBook, Box<dyn Error>> {
        // Mock order book
        Ok(OrderBook {
            token_id: token_id.to_string(),
            bids: vec![
                PriceLevel { price: 0.44, size: 100.0 },
                PriceLevel { price: 0.43, size: 500.0 },
            ],
            asks: vec![
                PriceLevel { price: 0.41, size: 200.0 },
                PriceLevel { price: 0.42, size: 300.0 },
            ],
            timestamp: 1234567890,
        })
    }
}
