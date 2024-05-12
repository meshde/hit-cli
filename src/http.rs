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

pub async fn handle_request(
    url: String,
    http_method: &HttpMethod,
    headers: &HashMap<String, String>,
    body: Option<String>,
) -> Result<String, reqwest::Error> {
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
        Some(body) => request_builder.json(&body),
        None => request_builder,
    };

    request_builder.send().await?.text().await
}
