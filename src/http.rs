use edit::edit;
use reqwest;

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
