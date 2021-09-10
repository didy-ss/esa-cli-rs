use async_std::task;

use serde_json::{json, Value};

pub struct Client {
    api_key: String,
}

const ESA_ENDPOINT: &str = "https://api.esa.io";

impl Client {
    pub fn new(api_key: impl Into<String>) -> Client {
        Client {
            api_key: api_key.into(),
        }
    }

    pub fn post(&self, path: impl Into<String>, params: Value) -> surf::Result<Value> {
        return task::block_on(async {
            let url = format!("{}{}", ESA_ENDPOINT, path.into());
            let builder = surf::post(url.clone());
            return self.get_json(builder, params).await;
        });
    }

    pub fn patch(&self, path: impl Into<String>, params: Value) -> surf::Result<Value> {
        return task::block_on(async {
            let url = format!("{}{}", ESA_ENDPOINT, path.into());
            let builder = surf::patch(url.clone());
            return self.get_json(builder, params).await;
        });
    }

    pub fn get(&self, path: impl Into<String>) -> surf::Result<Value> {
        return task::block_on(async {
            let url = format!("{}{}", ESA_ENDPOINT, path.into());
            let builder = surf::get(url.clone());
            return self.get_json(builder, json!({})).await;
        });
    }

    async fn get_json(&self, builder: surf::RequestBuilder, params: Value) -> surf::Result<Value> {
        return builder
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .body(params)
            .recv_json()
            .await;
    }
}
