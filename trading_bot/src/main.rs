use reqwest::header::{HeaderMap, HeaderValue};
use std::collections::HashMap;
use serde::Deserialize;
use std::fs;
use sha2::Sha256;
use hmac::{Hmac, Mac};

#[derive(Deserialize)]
struct OrderResponse {
    symbol: String,
    order_id: u64,
    client_order_id: String,
    transact_time: u64,
    price: String,
    orig_qty: String,
    executed_qty: String,
    status: String,
    time_in_force: String,
    r#type: String,
    side: String,
}

#[derive(Deserialize)]
struct TickerResponse {
    symbol: String,
    price: String,
}

async fn place_sell_order(api_key: &str, secret_key: &str, symbol: &str, quantity: &str) -> Result<OrderResponse, reqwest::Error> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let base_url = "https://testnet.binance.vision/api/v3";
    let endpoint = "/order";
    let url = format!("{}{}", base_url, endpoint);

    // Create the query parameters for the sell order
    let mut query_params: HashMap<&str, String> = HashMap::new();
    query_params.insert("symbol", symbol.to_string());
    query_params.insert("side", "SELL".to_string());
    query_params.insert("type", "MARKET".to_string());
    query_params.insert("quantity", quantity.to_string());
    query_params.insert("timestamp", timestamp.to_string());

    // Generate the query string
    let query_string = query_params.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&");

    // Create the HMAC SHA256 signature
    // Create alias for HMAC-SHA256
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(query_string.as_bytes());

    // `result` has type `CtOutput` which is a thin wrapper around array of
    // bytes for providing constant time equality check
    let signature = mac.finalize();
    
    // Create the request headers
    let mut headers = HeaderMap::new();
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(api_key).unwrap());

    // Build the request
    let client = reqwest::Client::new();
    let response = client.post(&url)
        .headers(headers)
        .query(&[("symbol", symbol)])
        .query(&[("side", "SELL")])
        .query(&[("type", "MARKET")])
        .query(&[("quantity", quantity)])
        .query(&[("timestamp", &timestamp.to_string())])
        .query(&[("signature", &hex::encode(signature.into_bytes()))])
        .send()
        .await?
        .json::<OrderResponse>()
        .await?;
    
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let url_account_info = "https://testnet.binance.vision/api/v3/account";
    
    let url = "https://testnet.binance.vision/api/v3/ticker/price?symbol=ETHUSDT";

    loop
    {
        let response: TickerResponse = client.get(url).send().await?.json::<TickerResponse>().await?;
        let eth_price = response.price.parse::<f32>().unwrap();
        println!("eth price is : {}", eth_price);
    
        if eth_price > 1830.0
        {
            println!("ETH to USD price exceeds 1830, sell");
    
            let pub_key_file = "../test-pub-key.pem";
            let prv_key_file = "../test-prv-key.pem";
            let pub_key_content = "";
            let prv_key_content = "";
            let symbol = "ETHUSD";
            let quantity = "1.5";
    
            match fs::read_to_string(pub_key_file) {
                Ok(pub_key_content) => {
                    println!("Read success\n");
                }
                Err(error) => {
                    eprintln!("Error reading file: {}", error);
                }
            }
    
            match fs::read_to_string(prv_key_file) {
                Ok(prv_key_content) => {
                    println!("Read success\n");
                }
                Err(error) => {
                    eprintln!("Error reading file: {}", error);
                }
            }
        
            let response = place_sell_order(pub_key_content, prv_key_content, symbol, quantity).await;
            match response {
                Ok(order) => println!("Sell order placed successfully. Order ID: {}", order.order_id),
                Err(e) => eprintln!("Error placing sell order: {}", e),
            }
        }
    
        println!("Current ETH price: {}", eth_price);
    }
}
