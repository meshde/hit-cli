use hyper::{HeaderMap, StatusCode};
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use strum::Display;

#[derive(Display, Deserialize, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

pub struct Response {
    pub url: String,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: String,
}

pub async fn handle_request(
    url: String,
    http_method: &HttpMethod,
    headers: &HashMap<String, String>,
    body: Option<String>,
) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let method: reqwest::Method = match http_method {
        HttpMethod::GET => reqwest::Method::GET,
        HttpMethod::POST => reqwest::Method::POST,
        HttpMethod::PUT => reqwest::Method::PUT,
        HttpMethod::DELETE => reqwest::Method::DELETE,
    };
    let request = reqwest::Request::new(method, reqwest::Url::parse(&url).expect("Invalid url"));

    let mut headers_map = reqwest::header::HeaderMap::new();

    headers_map.extend(headers.into_iter().map(|(k, v)| {
        (
            reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(&v).unwrap(),
        )
    }));
    let request_builder = reqwest::RequestBuilder::from_parts(client, request).headers(headers_map);

    let request_builder = match body {
        Some(body) => {
            if let Ok(json_body) = serde_json::from_str::<serde_json::Value>(&body) {
                request_builder.json(&json_body)
            } else {
                request_builder.body(body)
            }
        }
        None => request_builder,
    };

    let response = request_builder.send().await?;

    Ok(Response {
        url: response.url().clone().to_string(),
        status: response.status(),
        headers: response.headers().clone(),
        body: response.text().await?,
    })
}
