use anyhow::{anyhow, Result};
use tracing::{error, info};

#[derive(Clone)]
pub struct TwentyClient {
    url: String,
    api_key: String,
    client: reqwest::Client,
}

impl TwentyClient {
    pub fn new(url: String, api_key: String) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// GET /rest/companies/{uuid}
    pub async fn get_company(&self, uuid: &str) -> Result<serde_json::Value> {
        self.get_entity("companies", uuid).await
    }

    /// GET /rest/people/{uuid}
    pub async fn get_person(&self, uuid: &str) -> Result<serde_json::Value> {
        self.get_entity("people", uuid).await
    }

    /// GET /rest/opportunities/{uuid}
    pub async fn get_opportunity(&self, uuid: &str) -> Result<serde_json::Value> {
        self.get_entity("opportunities", uuid).await
    }

    async fn get_entity(&self, endpoint: &str, uuid: &str) -> Result<serde_json::Value> {
        let url = format!("{}/rest/{}/{}", self.url, endpoint, uuid);
        info!("[Twenty] GET {}", url);

        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| anyhow!("Twenty request failed: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            error!("[Twenty] {} returned {}: {}", endpoint, status, body);
            return Err(anyhow!("{} {} returned {}", endpoint, uuid, status));
        }

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| anyhow!("Twenty JSON parse error: {}", e))
    }
}
