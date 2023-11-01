use reqwest::header::{HeaderMap, HeaderValue};
use std::{collections::HashMap, str::FromStr};
use serde::Deserialize;
use std::fs;
use rsa::{RsaPrivateKey, pkcs8::DecodePrivateKey, Pkcs1v15Sign};
use sha2::{Sha256, Digest};
use base64; 

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
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

static mut APIKEY: String = String::new();
static mut PRVKEY: String = String::new();
static BASEURL: &'static str = "https://testnet.binance.vision/";

fn initialize()
{
    let api_key_file = "./APIKey.pem";
    let prv_key_file = "../test-prv-key.pem";

    match fs::read_to_string(api_key_file) {
        Ok(content) => {
            unsafe{
                APIKEY = content;
            }
            println!("Read success\n");
        }
        Err(error) => {
            eprintln!("Error reading file: {}", error);
        }
    }

    match fs::read_to_string(prv_key_file) {
        Ok(content) => {
            unsafe{
                PRVKEY = content;
            }
            println!("Read success\n");
        }
        Err(error) => {
            eprintln!("Error reading file: {}", error);
        }
    }  
}


async fn place_order(query_params: &mut HashMap<&str, &str>) -> Result<OrderResponse, Box<dyn std::error::Error>> {
    // Generate the query string
    let mut query_string: String = query_params.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&");

    let timestamp = chrono::Utc::now().timestamp_millis().to_string();
    query_string.push_str("&timestamp=");
    query_string.push_str(timestamp.as_str());

    let query_string_clone = query_string.clone();
    // First step: create a SHA-256 hash of the message.
    let mut hasher = Sha256::new();
    hasher.update(query_string);
    let hashed_message = hasher.finalize();

    // Sign the hashed message.
    let private_key = unsafe{ RsaPrivateKey::from_pkcs8_pem(&PRVKEY).expect("invalid pem")};
    let padding = Pkcs1v15Sign::new::<rsa::sha2::Sha256>();
    let signature = private_key.sign(padding, &hashed_message).expect("failed to encrypt");

    // Create the request headers
    let mut headers = HeaderMap::new();
    unsafe{
        headers.insert("X-MBX-APIKEY", HeaderValue::from_str(APIKEY.as_str()).unwrap());
    }

    // Build the request
    let client = reqwest::Client::new();

    // Build url
    let mut url: String = String::from(BASEURL);
    // String::from_str("/api/v3/order/") + query_string + String::from_str("/") + base64::encode(&signature)
    url.push_str("/api/v3/order/");
    url.push_str(&query_string_clone);
    url.push_str("&signature=");
    url.push_str(&base64::encode(&signature));

    let response = client.post(&url)
        .headers(headers)
        .send()
        .await?;

    // Read the response text
    let text = response.text().await?;

    // Print the response text
    println!("{}", text);

    // Parse the text as JSON
    let order_response: OrderResponse = serde_json::from_str(&text)?;

    Ok(order_response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize();

    let client = reqwest::Client::new();

    let url_account_info = "https://testnet.binance.vision/api/v3/account";
    
    let url = "https://testnet.binance.vision/api/v3/ticker/price?symbol=ETHUSDT";


    let response: TickerResponse = client.get(url).send().await?.json::<TickerResponse>().await?;
    let eth_price = response.price.parse::<f32>().unwrap();

    if eth_price > 1000.0
    {
        println!("ETH to USD price exceeds 1900, sell");

        // Create the query parameters for the sell order
        let mut query_params: HashMap<&str, &str> = HashMap::new();
        query_params.insert("symbol", "ETHUSDT");
        query_params.insert("side", "sell");
        query_params.insert("type", "LIMIT");
        query_params.insert("quantity", "0.5");
    
        let response = place_order(&mut query_params).await;
        match response {
            Ok(order) => println!("Sell order placed successfully. Order ID: {}", order.order_id),
            Err(e) => eprintln!("Error placing sell order: {}", e),
        }
    }

    println!("Current ETH price: {}", eth_price);

    return Ok(());
}
