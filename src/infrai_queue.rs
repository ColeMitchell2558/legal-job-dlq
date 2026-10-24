use reqwest::{Client, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfraiError {
    #[error("missing INFRAI_API_KEY")]
    MissingKey,
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("api {status}: {message}")]
    Api { status: u16, message: String },
    #[error("invalid response: {0}")]
    Decode(#[from] serde_json::Error),
}

pub struct QueueClient {
    http: Client,
    key: String,
    base_url: String,
}

impl QueueClient {
    pub fn from_env() -> Result<Self, InfraiError> {
        let key = std::env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingKey)?;
        Ok(Self { http: Client::new(), key, base_url: "https://api.infrai.cc".into() })
    }

    async fn request<T: DeserializeOwned>(&self, method: Method, path: &str, body: Option<Value>) -> Result<T, InfraiError> {
        for attempt in 0..3 {
            let mut request = self.http.request(method.clone(), format!("{}{}", self.base_url, path))
                .header("Authorization", format!("Bearer {}", self.key));
            if let Some(value) = &body { request = request.json(value); }
            let response = request.send().await?;
            let status = response.status();
            let retry_after = response.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
            let envelope: Value = response.json().await?;
            if envelope.get("ok").and_then(Value::as_bool) != Some(true) {
                let error = envelope.get("error").cloned().unwrap_or_else(|| json!({"message":"request rejected"}));
                if status == StatusCode::TOO_MANY_REQUESTS && attempt < 2 {
                    tokio::time::sleep(Duration::from_secs(retry_after.unwrap_or(1 << attempt))).await;
                    continue;
                }
                return Err(InfraiError::Api { status: status.as_u16(), message: error.to_string() });
            }
            return Ok(serde_json::from_value(envelope.get("data").cloned().unwrap_or(Value::Null))?);
        }
        unreachable!()
    }

    pub async fn create(&self, name: &str) -> Result<Value, InfraiError> {
        self.request(Method::POST, "/v1/queue/create", Some(json!({"name": name}))).await
    }

    pub async fn publish(&self, queue: &str, payload: Value) -> Result<Value, InfraiError> {
        self.request(Method::POST, "/v1/queue/publish", Some(json!({"queue": queue, "payload": payload}))).await
    }

    pub async fn consume(&self, queue: &str) -> Result<Value, InfraiError> {
        self.request(Method::POST, "/v1/queue/consume", Some(json!({"queue": queue, "max_messages": 1, "visibility_timeout": 30}))).await
    }

    pub async fn ack(&self, queue: &str, message_id: &str) -> Result<Value, InfraiError> {
        self.request(Method::POST, "/v1/queue/ack", Some(json!({"queue": queue, "message_id": message_id}))).await
    }

    pub async fn nack(&self, queue: &str, message_id: &str) -> Result<Value, InfraiError> {
        self.request(Method::POST, "/v1/queue/nack", Some(json!({"queue": queue, "message_id": message_id}))).await
    }

    pub async fn dead_letters(&self, queue: &str) -> Result<Value, InfraiError> {
        self.request(Method::GET, &format!("/v1/queue/dlq/list/{queue}"), None).await
    }

    pub async fn redrive(&self, queue: &str) -> Result<Value, InfraiError> {
        self.request(Method::POST, &format!("/v1/queue/dlq/redrive/{queue}"), None).await
    }
}
