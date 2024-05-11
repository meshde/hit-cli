use edit::edit;
use reqwest;
use serde::Deserialize;
use strum::Display;

#[derive(Display, Deserialize, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

pub async fn handle_request(
    url: String,
    http_method: &HttpMethod,
) -> Result<String, reqwest::Error> {
    match http_method {
        HttpMethod::GET => handle_get(url).await,
        HttpMethod::POST => handle_post(url).await,
        HttpMethod::PUT => handle_put(url).await,
        HttpMethod::DELETE => handle_delete(url).await,
    }
}

pub async fn handle_get(url: String) -> Result<String, reqwest::Error> {
    return reqwest::get(url).await?.text().await;
}

pub async fn handle_post(url: String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let input = edit("").expect("Unable to open system editor");
    return client.post(url).json(&input).send().await?.text().await;
}

pub async fn handle_put(url: String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let input = edit("").expect("Unable to open system editor");
    return client.put(url).json(&input).send().await?.text().await;
}

pub async fn handle_delete(url: String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    return client.delete(url).send().await?.text().await;
}
