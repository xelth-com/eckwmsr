use anyhow::Result;

use crate::handlers::mesh_sync::{PullRequest, PullResponse, PushPayload};
use crate::models::{location, product, stock_picking_delivery};
use crate::sync::merkle_tree::{MerkleNode, MerkleRequest};

/// HTTP client for communicating with peer mesh nodes
pub struct MeshClient {
    client: reqwest::Client,
    base_url: String,
}

impl MeshClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Get Merkle root (level 0) for an entity type
    pub async fn get_merkle_root(&self, entity_type: &str) -> Result<MerkleNode> {
        let url = format!("{}/E/mesh/merkle", self.base_url);
        let req = MerkleRequest {
            entity_type: entity_type.to_string(),
            level: 0,
            bucket: None,
        };

        let resp = self.client.post(&url).json(&req).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Remote merkle root failed: {}",
                resp.status()
            ));
        }
        Ok(resp.json::<MerkleNode>().await?)
    }

    /// Get Merkle bucket (level 1) for an entity type + bucket key
    pub async fn get_merkle_bucket(&self, entity_type: &str, bucket: &str) -> Result<MerkleNode> {
        let url = format!("{}/E/mesh/merkle", self.base_url);
        let req = MerkleRequest {
            entity_type: entity_type.to_string(),
            level: 1,
            bucket: Some(bucket.to_string()),
        };

        let resp = self.client.post(&url).json(&req).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Remote merkle bucket failed: {}",
                resp.status()
            ));
        }
        Ok(resp.json::<MerkleNode>().await?)
    }

    /// Pull specific entities from a peer by ID
    pub async fn pull_entities(&self, entity_type: &str, ids: Vec<String>) -> Result<PullResponse> {
        let url = format!("{}/E/mesh/pull", self.base_url);
        let req = PullRequest {
            entity_type: entity_type.to_string(),
            ids,
        };

        let resp = self.client.post(&url).json(&req).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("Remote pull failed: {}", resp.status()));
        }
        Ok(resp.json::<PullResponse>().await?)
    }

    /// Push entities to a peer
    pub async fn push_entities(
        &self,
        products: Vec<product::Model>,
        locations: Vec<location::Model>,
        shipments: Vec<stock_picking_delivery::Model>,
    ) -> Result<()> {
        let url = format!("{}/E/mesh/push", self.base_url);
        let payload = PushPayload {
            products,
            locations,
            shipments,
        };

        let resp = self.client.post(&url).json(&payload).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("Remote push failed: {}", resp.status()));
        }
        Ok(())
    }
}
