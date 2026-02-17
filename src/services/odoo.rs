use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info};

#[derive(Clone)]
pub struct OdooClient {
    url: String,
    db: String,
    username: String,
    password: String,
    client: reqwest::Client,
    uid: Option<i64>,
}

#[derive(Deserialize)]
struct JsonRpcResponse<T> {
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize, Debug)]
struct JsonRpcError {
    message: String,
    #[allow(dead_code)]
    data: Option<serde_json::Value>,
}

impl OdooClient {
    pub fn new(url: String, db: String, username: String, password: String) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            db,
            username,
            password,
            client: reqwest::Client::new(),
            uid: None,
        }
    }

    /// Authenticate with Odoo to get UID
    pub async fn authenticate(&mut self) -> Result<i64> {
        let auth_url = format!("{}/jsonrpc", self.url);
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "call",
            "params": {
                "service": "common",
                "method": "login",
                "args": [self.db, self.username, self.password]
            },
            "id": 1
        });

        let resp = self.client.post(&auth_url).json(&payload).send().await?;
        let rpc_resp: JsonRpcResponse<serde_json::Value> = resp.json().await?;

        if let Some(err) = rpc_resp.error {
            return Err(anyhow!("Odoo auth error: {}", err.message));
        }

        if let Some(result) = rpc_resp.result {
            if let Some(uid) = result.as_i64() {
                self.uid = Some(uid);
                info!("Odoo: Authenticated as UID {}", uid);
                return Ok(uid);
            }
        }

        Err(anyhow!("Odoo authentication failed (invalid credentials?)"))
    }

    /// Call an Odoo model method (execute_kw)
    pub async fn execute_kw(
        &self,
        model: &str,
        method: &str,
        args: Vec<serde_json::Value>,
        kwargs: serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let uid = self.uid.ok_or_else(|| anyhow!("Not authenticated"))?;
        let execute_url = format!("{}/jsonrpc", self.url);

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "call",
            "params": {
                "service": "object",
                "method": "execute_kw",
                "args": [
                    self.db,
                    uid,
                    self.password,
                    model,
                    method,
                    args,
                    kwargs
                ]
            },
            "id": 2
        });

        let resp = self.client.post(&execute_url).json(&payload).send().await?;
        let rpc_resp: JsonRpcResponse<serde_json::Value> = resp.json().await?;

        if let Some(err) = rpc_resp.error {
            error!("Odoo RPC error: {:?}", err);
            return Err(anyhow!("Odoo RPC error: {}", err.message));
        }

        Ok(rpc_resp.result.unwrap_or(serde_json::Value::Null))
    }

    /// Create a repair order in Odoo
    pub async fn create_repair_order(
        &self,
        product_id: i64,
        serial: &str,
        description: &str,
    ) -> Result<i64> {
        let lot_ids = self
            .search_read(
                "stock.lot",
                vec![json!(["name", "=", serial])],
                vec!["id"],
                1,
            )
            .await?;

        let lot_id = lot_ids
            .as_array()
            .and_then(|a| a.first())
            .and_then(|lot| lot["id"].as_i64());

        let mut values = serde_json::Map::new();
        values.insert("product_id".to_string(), json!(product_id));
        values.insert("description".to_string(), json!(description));
        values.insert("state".to_string(), json!("draft"));

        if let Some(lid) = lot_id {
            values.insert("lot_id".to_string(), json!(lid));
        }

        let res = self
            .execute_kw(
                "repair.order",
                "create",
                vec![json!(values)],
                serde_json::Map::new(),
            )
            .await?;

        res.as_i64()
            .ok_or_else(|| anyhow!("Failed to parse create response"))
    }

    /// Helper: Search and Read
    pub async fn search_read(
        &self,
        model: &str,
        domain: Vec<serde_json::Value>,
        fields: Vec<&str>,
        limit: i64,
    ) -> Result<serde_json::Value> {
        let mut kwargs = serde_json::Map::new();
        kwargs.insert("fields".to_string(), json!(fields));
        kwargs.insert("limit".to_string(), json!(limit));

        self.execute_kw(model, "search_read", vec![json!(domain)], kwargs)
            .await
    }
}
