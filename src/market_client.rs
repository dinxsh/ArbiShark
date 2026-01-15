pub struct PolymarketClient {
    pub gamma_url: String,
    pub clob_url: String,
    pub client: reqwest::Client,
}

#[async_trait]
impl MarketClient for PolymarketClient {
    async fn get_markets(&self) -> Result<Vec<Market>, Box<dyn Error + Send + Sync>> {
        let resp = self.client.get(&self.gamma_url).send().await?.text().await?;
        let json: serde_json::Value = serde_json::from_str(&resp)?;
        let mut markets = Vec::new();
        if let Some(events) = json.as_array() {
            for event in events {
                if let Some(event_markets) = event["markets"].as_array() {
                    for m in event_markets {
                        let id = m["id"].as_str().unwrap_or("").to_string();
                        let question = m["question"].as_str().unwrap_or("").to_string();
                        let slug = event["slug"].as_str().unwrap_or("").to_string();
                        let outcomes: Vec<String> = m["outcomes"].as_array()
                            .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect())
                            .unwrap_or_default();
                        let clob_token_ids: Vec<String> = if let Some(s) = m["clobTokenIds"].as_str() {
                            serde_json::from_str(s).unwrap_or_default()
                        } else {
                            m["clobTokenIds"].as_array()
                                .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect())
                                .unwrap_or_default()
                        };
                        if clob_token_ids.len() < 2 { continue; }
                        markets.push(Market {
                            id,
                            question,
                            slug,
                            outcomes,
                            outcome_prices: vec![0.5, 0.5],
                            clob_token_ids,
                            best_bid: None,
                            best_ask: None,
                            maker_base_fee: 0,
                            taker_base_fee: 200,
                            liquidity: 0.0,
                            volume_24hr: 0.0,
                            active: true,
                            accepting_orders: true,
                        });
                    }
                }
            }
        }
        Ok(markets)
    }
    async fn get_order_book(&self, token_id: &str) -> Result<OrderBook, Box<dyn Error + Send + Sync>> {
        let url = format!("{}?tokenId={}", self.clob_url, token_id);
        let resp = self.client.get(&url).send().await?.text().await?;
        let json: serde_json::Value = serde_json::from_str(&resp)?;
        let bids = json["bids"].as_array().map(|a| a.iter().map(|v| crate::types::PriceLevel {
            price: v["price"].as_f64().unwrap_or(0.0),
            size: v["size"].as_f64().unwrap_or(0.0),
        }).collect()).unwrap_or_default();
        let asks = json["asks"].as_array().map(|a| a.iter().map(|v| crate::types::PriceLevel {
            price: v["price"].as_f64().unwrap_or(0.0),
            size: v["size"].as_f64().unwrap_or(0.0),
        }).collect()).unwrap_or_default();
        Ok(OrderBook {
            token_id: token_id.to_string(),
            bids,
            asks,
            timestamp: json["timestamp"].as_u64().unwrap_or(0),
        })
    }
    async fn stream_quotes(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}

use async_trait::async_trait;
use crate::types::{Market, OrderBook};
use std::error::Error;

#[async_trait]
pub trait MarketClient {
    async fn get_markets(&self) -> Result<Vec<Market>, Box<dyn Error + Send + Sync>>;
    async fn get_order_book(&self, token_id: &str) -> Result<OrderBook, Box<dyn Error + Send + Sync>>;
    async fn stream_quotes(&self) -> Result<(), Box<dyn Error + Send + Sync>>; // Placeholder for streaming
}


pub struct ArbitrumMarketClient {
    pub endpoint: String,
    pub client: reqwest::Client,
    pub last_query_time: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
}

impl ArbitrumMarketClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
            last_query_time: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Check Envio health and data freshness
    pub async fn health_check(&self) -> Result<EnvioHealth, Box<dyn Error + Send + Sync>> {
        let start = std::time::Instant::now();
        
        // Simple health query to Envio
        let query = r#"{
            _meta {
                block {
                    number
                    timestamp
                }
            }
        }"#;
        
        let response = self.client.post(&self.endpoint)
            .json(&serde_json::json!({"query": query}))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        if !response.status().is_success() {
            return Err(format!("Envio health check failed: {}", response.status()).into());
        }
        
        let json: serde_json::Value = response.json().await?;
        let block_number = json["data"]["_meta"]["block"]["number"]
            .as_u64()
            .unwrap_or(0);
        let block_timestamp = json["data"]["_meta"]["block"]["timestamp"]
            .as_u64()
            .unwrap_or(0);
        
        // Calculate data delay
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let data_delay_ms = ((now - block_timestamp) * 1000) as u64;
        
        Ok(EnvioHealth {
            latency_ms,
            block_number,
            block_timestamp,
            data_delay_ms,
            is_healthy: data_delay_ms < 5000, // Healthy if < 5s delay
        })
    }
}

#[derive(Debug, Clone)]
pub struct EnvioHealth {
    pub latency_ms: u64,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub data_delay_ms: u64,
    pub is_healthy: bool,
}

#[async_trait]
impl MarketClient for ArbitrumMarketClient {
    async fn get_markets(&self) -> Result<Vec<Market>, Box<dyn Error + Send + Sync>> {
        // Update last query time
        if let Ok(mut last_time) = self.last_query_time.lock() {
            *last_time = Some(std::time::Instant::now());
        }

        let query = r#"{
            markets {
                id
                question
                slug
                outcomes
                outcomePrices
                clobTokenIds
                bestBid
                bestAsk
                makerBaseFee
                takerBaseFee
                liquidity
                volume24hr
                active
                acceptingOrders
            }
        }"#;
        
        let response = self.client.post(&self.endpoint)
            .json(&serde_json::json!({"query": query}))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Envio request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Envio returned error: {}", response.status()).into());
        }
        
        let json: serde_json::Value = response.json().await?;
        
        // Check for GraphQL errors
        if let Some(errors) = json.get("errors") {
            return Err(format!("GraphQL errors: {:?}", errors).into());
        }
        
        let markets_json = &json["data"]["markets"];
        let mut markets = Vec::new();
        
        if let Some(arr) = markets_json.as_array() {
            for m in arr {
                let market = Market {
                    id: m["id"].as_str().unwrap_or("").to_string(),
                    question: m["question"].as_str().unwrap_or("").to_string(),
                    slug: m["slug"].as_str().unwrap_or("").to_string(),
                    outcomes: m["outcomes"].as_array().map(|a| a.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default(),
                    outcome_prices: m["outcomePrices"].as_array().map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect()).unwrap_or_default(),
                    clob_token_ids: m["clobTokenIds"].as_array().map(|a| a.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default(),
                    best_bid: m["bestBid"].as_f64(),
                    best_ask: m["bestAsk"].as_f64(),
                    maker_base_fee: m["makerBaseFee"].as_u64().unwrap_or(0) as u32,
                    taker_base_fee: m["takerBaseFee"].as_u64().unwrap_or(0) as u32,
                    liquidity: m["liquidity"].as_f64().unwrap_or(0.0),
                    volume_24hr: m["volume24hr"].as_f64().unwrap_or(0.0),
                    active: m["active"].as_bool().unwrap_or(false),
                    accepting_orders: m["acceptingOrders"].as_bool().unwrap_or(false),
                };
                markets.push(market);
            }
        }
        
        Ok(markets)
    }
    
    async fn get_order_book(&self, token_id: &str) -> Result<OrderBook, Box<dyn Error + Send + Sync>> {
        let query = format!(r#"{{
            orderBook(tokenId: "{}") {{
                tokenId
                bids {{ price size }}
                asks {{ price size }}
                timestamp
            }}
        }}"#, token_id);
        
        let response = self.client.post(&self.endpoint)
            .json(&serde_json::json!({"query": query}))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Envio request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Envio returned error: {}", response.status()).into());
        }
        
        let json: serde_json::Value = response.json().await?;
        
        // Check for GraphQL errors
        if let Some(errors) = json.get("errors") {
            return Err(format!("GraphQL errors: {:?}", errors).into());
        }
        
        let ob = &json["data"]["orderBook"];
        let bids = ob["bids"].as_array().map(|a| a.iter().map(|v| crate::types::PriceLevel {
            price: v["price"].as_f64().unwrap_or(0.0),
            size: v["size"].as_f64().unwrap_or(0.0),
        }).collect()).unwrap_or_default();
        let asks = ob["asks"].as_array().map(|a| a.iter().map(|v| crate::types::PriceLevel {
            price: v["price"].as_f64().unwrap_or(0.0),
            size: v["size"].as_f64().unwrap_or(0.0),
        }).collect()).unwrap_or_default();
        
        Ok(OrderBook {
            token_id: ob["tokenId"].as_str().unwrap_or("").to_string(),
            bids,
            asks,
            timestamp: ob["timestamp"].as_u64().unwrap_or(0),
        })
    }
    
    async fn stream_quotes(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Not implemented: would require websocket or polling
        Ok(())
    }
}
