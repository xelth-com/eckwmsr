use base64::{engine::general_purpose::STANDARD as BASE64_STD, Engine as _};
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use crate::models::sync_packet::EncryptedSyncPacket;

#[derive(Error, Debug)]
pub enum RelayError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Relay returned error status: {0}")]
    StatusError(StatusCode),
    #[error("Serialization failed: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Base64 decode failed: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// Matches eck's `PushRequest` — payload_cipher and nonce are base64-encoded in JSON
#[derive(Debug, Serialize)]
struct RelayPushRequest {
    pub target_instance_id: String,
    pub sender_instance_id: String,
    pub payload_cipher: String, // base64
    pub nonce: String,          // base64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
}

/// Matches eck's `PushResponse`
#[derive(Debug, Deserialize)]
struct RelayPushResponse {
    pub ok: bool,
    pub packet_id: Uuid,
}

/// Matches eck's `EncryptedPacket` in pull response
#[derive(Debug, Deserialize)]
struct RelayPullPacket {
    pub id: Uuid,
    pub target_instance_id: String,
    pub sender_instance_id: String,
    pub payload_cipher: String, // base64
    pub nonce: String,          // base64
    pub created_at: DateTime<Utc>,
    pub ttl: DateTime<Utc>,
}

/// Matches eck's `PullResponse`
#[derive(Debug, Deserialize)]
struct RelayPullResponse {
    pub packets: Vec<RelayPullPacket>,
}

#[derive(Clone)]
pub struct RelayClient {
    client: Client,
    relay_url: String,
    instance_id: String,
}

impl RelayClient {
    pub fn new(relay_url: &str, instance_id: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            client,
            relay_url: relay_url.trim_end_matches('/').to_string(),
            instance_id: instance_id.to_string(),
        }
    }

    /// Pushes an EncryptedSyncPacket to the blind relay for a specific target.
    /// The packet is JSON-serialized and placed into eck's payload_cipher field (base64).
    pub async fn push_packet(
        &self,
        target_instance: &str,
        packet: &EncryptedSyncPacket,
        ttl_seconds: Option<u64>,
    ) -> Result<Uuid, RelayError> {
        let url = format!("{}/E/push", self.relay_url);

        // Serialize our EncryptedSyncPacket into bytes, then base64 for the relay envelope
        let packet_bytes = serde_json::to_vec(packet)?;

        let req_body = RelayPushRequest {
            target_instance_id: target_instance.to_string(),
            sender_instance_id: self.instance_id.clone(),
            payload_cipher: BASE64_STD.encode(&packet_bytes),
            nonce: BASE64_STD.encode(&packet.nonce),
            ttl_seconds,
        };

        let response = self.client.post(&url).json(&req_body).send().await?;

        if !response.status().is_success() {
            return Err(RelayError::StatusError(response.status()));
        }

        let push_resp: RelayPushResponse = response.json().await?;
        Ok(push_resp.packet_id)
    }

    /// Pulls pending packets from the blind relay destined for this instance.
    /// Decodes the relay envelope and deserializes inner EncryptedSyncPackets.
    pub async fn pull_packets(&self) -> Result<Vec<EncryptedSyncPacket>, RelayError> {
        let url = format!("{}/E/pull/{}", self.relay_url, self.instance_id);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(RelayError::StatusError(response.status()));
        }

        let pull_resp: RelayPullResponse = response.json().await?;

        let mut packets = Vec::new();
        for rp in pull_resp.packets {
            let decoded_bytes = BASE64_STD.decode(&rp.payload_cipher)?;
            let packet: EncryptedSyncPacket = serde_json::from_slice(&decoded_bytes)?;
            packets.push(packet);
        }

        Ok(packets)
    }

    /// Pulls packets from the relay for an arbitrary target ID.
    /// Used by pairing to pull from a well-known channel derived from the magic code.
    pub async fn pull_packets_for(
        &self,
        target_id: &str,
    ) -> Result<Vec<EncryptedSyncPacket>, RelayError> {
        let url = format!("{}/E/pull/{}", self.relay_url, target_id);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(RelayError::StatusError(response.status()));
        }

        let pull_resp: RelayPullResponse = response.json().await?;

        let mut packets = Vec::new();
        for rp in pull_resp.packets {
            let decoded_bytes = BASE64_STD.decode(&rp.payload_cipher)?;
            let packet: EncryptedSyncPacket = serde_json::from_slice(&decoded_bytes)?;
            packets.push(packet);
        }

        Ok(packets)
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn relay_url(&self) -> &str {
        &self.relay_url
    }
}
