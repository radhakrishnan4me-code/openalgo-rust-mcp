use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

/// HTTP client for communicating with the OpenAlgo REST API.
#[derive(Clone)]
pub struct OpenAlgoClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl OpenAlgoClient {
    pub fn new(api_key: String, host: String) -> Self {
        let base_url = format!("{}/api/v1", host.trim_end_matches('/'));
        Self {
            client: Client::new(),
            base_url,
            api_key,
        }
    }

    /// POST to an OpenAlgo API endpoint and return the JSON response.
    pub async fn post(&self, endpoint: &str, body: Value) -> Result<Value, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        resp.json::<Value>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Convenience: POST with a simple key-value map.
    pub async fn post_map(
        &self,
        endpoint: &str,
        params: HashMap<&str, Value>,
    ) -> Result<Value, String> {
        let body = Value::Object(
            params
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        );
        self.post(endpoint, body).await
    }
}
