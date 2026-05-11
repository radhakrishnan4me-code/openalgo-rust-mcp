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

    /// Generic request with method, endpoint and body.
    pub async fn call_method(
        &self,
        method: &str,
        endpoint: &str,
        body: Value,
    ) -> Result<Value, String> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut payload = body;
        if let Some(obj) = payload.as_object_mut() {
            if !obj.contains_key("apikey") {
                obj.insert("apikey".to_string(), Value::String(self.api_key.clone()));
            }
        }

        let builder = match method.to_uppercase().as_str() {
            "GET" => self.client.get(&url).query(&payload),
            "POST" => self.client.post(&url).json(&payload),
            "DELETE" => self.client.delete(&url).query(&payload),
            "PUT" => self.client.put(&url).json(&payload),
            _ => return Err(format!("Unsupported method: {}", method)),
        };

        let resp = builder
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        resp.json::<Value>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// POST to an OpenAlgo API endpoint and return the JSON response.
    pub async fn post(&self, endpoint: &str, body: Value) -> Result<Value, String> {
        self.call_method("POST", endpoint, body).await
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
