use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use strum::Display;

#[derive(Display, Deserialize, Serialize, Clone, Debug)]
#[expect(
    clippy::upper_case_acronyms,
    reason = "HTTP method names are conventionally uppercase and are serialized as-is in config files"
)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Response {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub async fn handle_request(
    url: String,
    http_method: &HttpMethod,
    headers: &HashMap<String, String>,
    body: Option<String>,
    file_fields: Option<HashMap<String, PathBuf>>,
) -> Result<Response, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let method: reqwest::Method = match http_method {
        HttpMethod::GET => reqwest::Method::GET,
        HttpMethod::POST => reqwest::Method::POST,
        HttpMethod::PUT => reqwest::Method::PUT,
        HttpMethod::DELETE => reqwest::Method::DELETE,
        HttpMethod::PATCH => reqwest::Method::PATCH,
    };
    let request = reqwest::Request::new(method, reqwest::Url::parse(&url).expect("Invalid url"));

    let mut owned_headers = headers.clone();
    owned_headers.insert("User-Agent".to_string(), "hit-cli".to_string());

    let has_file_fields = file_fields
        .as_ref()
        .is_some_and(|fields| !fields.is_empty());

    if has_file_fields {
        owned_headers.remove("Content-Type");
        owned_headers.remove("content-type");
    }

    let mut headers_map = reqwest::header::HeaderMap::new();

    headers_map.extend(owned_headers.into_iter().map(|(k, v)| {
        (
            reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(&v).unwrap(),
        )
    }));

    let request_builder = reqwest::RequestBuilder::from_parts(client, request).headers(headers_map);

    let request_builder = if has_file_fields {
        let file_fields = file_fields.unwrap();
        let mut form = reqwest::multipart::Form::new();

        for (field_name, file_path) in &file_fields {
            let file_bytes = std::fs::read(file_path)?;
            let file_name = file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let part = reqwest::multipart::Part::bytes(file_bytes).file_name(file_name);
            form = form.part(field_name.clone(), part);
        }

        if let Some(ref body_str) = body {
            if let Ok(json_body) = serde_json::from_str::<serde_json::Value>(body_str) {
                if let Some(obj) = json_body.as_object() {
                    for (key, value) in obj {
                        let text_value = match value {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        form = form.text(key.clone(), text_value);
                    }
                }
            }
        }

        request_builder.multipart(form)
    } else {
        match body {
            Some(body) => {
                if let Ok(json_body) = serde_json::from_str::<serde_json::Value>(&body) {
                    request_builder.json(&json_body)
                } else {
                    request_builder.body(body)
                }
            }
            None => request_builder,
        }
    };

    let response = request_builder.send().await?;
    let mut response_headers = HashMap::new();

    for (key, value) in response.headers().iter() {
        response_headers.insert(key.to_string(), value.to_str().unwrap().to_string());
    }

    Ok(Response {
        url: response.url().clone().to_string(),
        status: response.status().as_u16(),
        headers: response_headers,
        body: response.text().await?,
    })
}
