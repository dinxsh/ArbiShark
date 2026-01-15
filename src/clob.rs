use crate::types::{OrderBook, PriceLevel, Side, Trade};
use std::error::Error;

use serde::Deserialize;

#[derive(Deserialize)]
struct BookResponse {
    token_id: String,
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
    timestamp: u64,
}

#[derive(Deserialize)]
struct TradesResponse {
    trades: Vec<Trade>,
}
/// Client for Polymarket CLOB API
pub struct ClobClient {
    base_url: String,
    client: reqwest::Client,
}

impl ClobClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Fetch order book for a specific token
    pub async fn get_book(&self, token_id: &str) -> Result<OrderBook, Box<dyn Error>> {
        let url = format!("{}/book?token_id={}", self.base_url, token_id);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Failed to fetch order book: {}", resp.status()).into());
        }
        let book: BookResponse = resp.json().await?;
        Ok(OrderBook {
            token_id: book.token_id,
            bids: book.bids,
            asks: book.asks,
            timestamp: book.timestamp,
        })
    }

    /// Fetch recent trades
    pub async fn get_trades(&self, market_id: &str) -> Result<Vec<Trade>, Box<dyn Error>> {
        let url = format!("{}/trades?market_id={}", self.base_url, market_id);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Failed to fetch trades: {}", resp.status()).into());
        }
        let trades: TradesResponse = resp.json().await?;
        Ok(trades.trades)
    }
}
