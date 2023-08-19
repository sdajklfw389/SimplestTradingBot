use reqwest::header::{HeaderMap, HeaderValue};
use std::collections::HashMap;
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

struct Name
{
    Name(base_url: &str);

    join_query_parameters();
    place_order();
    
    str api_key;
    str pri_key;
    str base_url = "https://testnet.binance.vision/api/v3";
}

/*
* ret: query_string: joined parameters + '&' + timestamp
*/
(str, str) join_query_parameters(url_path: &str, parameters: &HashMap<&str, String>)
{
    let timestamp = chrono::Utc::now().timestamp_millis();

    // Create the query parameters for the sell order
    let mut query_params: HashMap<&str, String> = HashMap::new();
    query_params.insert("timestamp", timestamp.to_string());

    // Generate the query string
    let query_string = parameters.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&");
}

async fn place_order(method: &str, url_path: &str, query_string: &str, parameters: &HashMap<&str, String>) -> Result<OrderResponse, Box<dyn std::error::Error>> {
    
    let (url, query_string) = join_query_parameters(url_path, parameters);
    // First step: create a SHA-256 hash of the message.
    let mut hasher = Sha256::new();
    hasher.update(query_string);
    let hashed_message = hasher.finalize();

    // Sign the hashed message.
    let private_key = RsaPrivateKey::from_pkcs8_pem(&secret_key).expect("invalid pem");
    let padding = Pkcs1v15Sign::new::<rsa::sha2::Sha256>();
    let signature = private_key.sign(padding, &hashed_message).expect("failed to encrypt");

    // Create the request headers
    let mut headers = HeaderMap::new();
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(api_key).unwrap());

    // Build the request
    let client = reqwest::Client::new();
    // This is just retrieve testnet account info
    // let url = format!("{}?timestamp={}&signature={}",
    //     "https://testnet.binance.vision/api/v3/account",
    //     timestamp,
    //     base64::encode(&signature)
    // );

    let url = base_url + url_path + query_string + signature;

    let response = client.get(&url)
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
    let client = reqwest::Client::new();

    let url_account_info = "https://testnet.binance.vision/api/v3/account";
    
    let url = "https://testnet.binance.vision/api/v3/ticker/price?symbol=ETHUSDT";

    loop
    {
        let response: TickerResponse = client.get(url).send().await?.json::<TickerResponse>().await?;
        let eth_price = response.price.parse::<f32>().unwrap();
        println!("eth price is : {}", eth_price);
    
        if eth_price > 1650.0
        {
            println!("ETH to USD price exceeds 1830, sell");
    
            let api_key_file = "./APIKey.pem";
            let prv_key_file = "../test-prv-key.pem";
            let mut api_key_content = String::new();
            let mut prv_key_content = String::new();
            let symbol = "ETHUSD";
            let quantity = "1.5";
    
            match fs::read_to_string(api_key_file) {
                Ok(content) => {
                    api_key_content = content;
                    println!("Read success\n");
                }
                Err(error) => {
                    eprintln!("Error reading file: {}", error);
                }
            }
    
            match fs::read_to_string(prv_key_file) {
                Ok(content) => {
                    prv_key_content = content;
                    println!("Read success\n");
                }
                Err(error) => {
                    eprintln!("Error reading file: {}", error);
                }
            }
        
            let response = place_order(api_key_content.as_str(), prv_key_content.as_str(), url, query_string).await;
            match response {
                Ok(order) => println!("Sell order placed successfully. Order ID: {}", order.order_id),
                Err(e) => eprintln!("Error placing sell order: {}", e),
            }
        }
    
        println!("Current ETH price: {}", eth_price);
    }
}
