use anyhow::{anyhow, Result};
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

use crate::models::sync_packet::EntityMetadata;
use crate::sync::relay_client::RelayClient;
use crate::sync::security::{SecurityLayer, SyncNodeRole};
use crate::sync::vector_clock::VectorClock;

const PAIRING_TTL_SECONDS: u64 = 300; // 5 minutes

/// What the Host publishes so the Client can find it
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PairingOffer {
    pub instance_id: String,
    pub instance_name: String,
    pub relay_url: String,
    pub generated_at: i64,
}

/// What the Client sends back after finding the offer
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PairingResponse {
    pub instance_id: String,
    pub instance_name: String,
    pub relay_url: String,
}

/// The approval payload containing the network key, sent from Host to Client
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PairingApproval {
    pub host_instance_id: String,
    pub network_key: String,
}

pub struct PairingService {
    instance_id: String,
    instance_name: String,
    relay_url: String,
    relay: RelayClient,
}

impl PairingService {
    pub fn new(
        instance_id: String,
        instance_name: String,
        relay_url: String,
        relay: RelayClient,
    ) -> Self {
        Self {
            instance_id,
            instance_name,
            relay_url,
            relay,
        }
    }

    /// Generates a random 6-digit code formatted as "XXX-XXX"
    pub fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        let n: u32 = rng.gen_range(100_000..999_999);
        format!("{}-{}", &n.to_string()[..3], &n.to_string()[3..])
    }

    /// Host: Publishes a pairing offer to the Relay.
    /// The offer is encrypted with a key derived from the code,
    /// and pushed to a relay channel derived from the code hash.
    pub async fn publish_offer(&self, code: &str) -> Result<()> {
        let clean_code = code.replace('-', "");
        let (routing_id, enc_key) = Self::derive_keys(&clean_code, "offer");

        let offer = PairingOffer {
            instance_id: self.instance_id.clone(),
            instance_name: self.instance_name.clone(),
            relay_url: self.relay_url.clone(),
            generated_at: Utc::now().timestamp(),
        };

        let temp_security = SecurityLayer::new(SyncNodeRole::Peer, &hex::encode(&enc_key));

        let mut vc = VectorClock::new();
        vc.increment(&self.instance_id);

        let metadata = EntityMetadata {
            entity_id: routing_id.clone(),
            entity_type: "pairing_offer".to_string(),
            version: 1,
            updated_at: Utc::now(),
            source: "pairing".to_string(),
            source_priority: 0,
            instance_id: self.instance_id.clone(),
            device_id: None,
            vector_clock: vc,
        };

        let packet = temp_security
            .encrypt_packet(&metadata, &offer)
            .map_err(|e| anyhow!("Failed to encrypt offer: {}", e))?;

        // Push to relay: target_instance_id = routing_id (the "channel")
        self.relay
            .push_packet(&routing_id, &packet, Some(PAIRING_TTL_SECONDS))
            .await
            .map_err(|e| anyhow!("Failed to push offer to relay: {}", e))?;

        info!(
            "Pairing: Published offer on channel {} (code: {})",
            &routing_id[..12],
            code
        );
        Ok(())
    }

    /// Client: Tries to find an offer with the given code on the relay.
    pub async fn find_offer(&self, code: &str) -> Result<PairingOffer> {
        let clean_code = code.replace('-', "");
        let (routing_id, enc_key) = Self::derive_keys(&clean_code, "offer");
        let temp_security = SecurityLayer::new(SyncNodeRole::Peer, &hex::encode(&enc_key));

        // Pull packets from the relay channel
        let packets = self
            .relay
            .pull_packets_for(&routing_id)
            .await
            .map_err(|e| anyhow!("Failed to pull from relay: {}", e))?;

        if packets.is_empty() {
            return Err(anyhow!(
                "No pairing offer found for this code. It may have expired."
            ));
        }

        // Decrypt the first packet
        let offer: PairingOffer = temp_security
            .decrypt_packet(&packets[0])
            .map_err(|e| anyhow!("Failed to decrypt offer (wrong code?): {}", e))?;

        // Check expiry (5 min)
        let age = Utc::now().timestamp() - offer.generated_at;
        if age > PAIRING_TTL_SECONDS as i64 {
            return Err(anyhow!("Pairing offer has expired."));
        }

        info!(
            "Pairing: Found offer from instance '{}' (age: {}s)",
            offer.instance_id, age
        );
        Ok(offer)
    }

    /// Client: After finding the offer, sends a response back through the relay
    /// on a response channel so the Host can discover the Client.
    pub async fn send_response(&self, code: &str) -> Result<()> {
        let clean_code = code.replace('-', "");
        let (resp_routing_id, enc_key) = Self::derive_keys(&clean_code, "response");

        let response = PairingResponse {
            instance_id: self.instance_id.clone(),
            instance_name: self.instance_name.clone(),
            relay_url: self.relay_url.clone(),
        };

        let temp_security = SecurityLayer::new(SyncNodeRole::Peer, &hex::encode(&enc_key));

        let mut vc = VectorClock::new();
        vc.increment(&self.instance_id);

        let metadata = EntityMetadata {
            entity_id: resp_routing_id.clone(),
            entity_type: "pairing_response".to_string(),
            version: 1,
            updated_at: Utc::now(),
            source: "pairing".to_string(),
            source_priority: 0,
            instance_id: self.instance_id.clone(),
            device_id: None,
            vector_clock: vc,
        };

        let packet = temp_security
            .encrypt_packet(&metadata, &response)
            .map_err(|e| anyhow!("Failed to encrypt response: {}", e))?;

        self.relay
            .push_packet(&resp_routing_id, &packet, Some(PAIRING_TTL_SECONDS))
            .await
            .map_err(|e| anyhow!("Failed to push response to relay: {}", e))?;

        info!(
            "Pairing: Sent response on channel {}",
            &resp_routing_id[..12]
        );
        Ok(())
    }

    /// Host: Polls the response channel to see if a Client has connected.
    pub async fn check_response(&self, code: &str) -> Result<Option<PairingResponse>> {
        let clean_code = code.replace('-', "");
        let (resp_routing_id, enc_key) = Self::derive_keys(&clean_code, "response");
        let temp_security = SecurityLayer::new(SyncNodeRole::Peer, &hex::encode(&enc_key));

        let packets = self
            .relay
            .pull_packets_for(&resp_routing_id)
            .await
            .map_err(|e| anyhow!("Failed to check response channel: {}", e))?;

        if packets.is_empty() {
            return Ok(None);
        }

        let response: PairingResponse = temp_security
            .decrypt_packet(&packets[0])
            .map_err(|e| anyhow!("Failed to decrypt response: {}", e))?;

        info!(
            "Pairing: Got response from instance '{}'",
            response.instance_id
        );
        Ok(Some(response))
    }

    /// Host: Sends the approved network key to the Client via the relay
    pub async fn send_approval(&self, code: &str, network_key: &str) -> Result<()> {
        let clean_code = code.replace('-', "");
        let (routing_id, enc_key) = Self::derive_keys(&clean_code, "approval");

        let approval = PairingApproval {
            host_instance_id: self.instance_id.clone(),
            network_key: network_key.to_string(),
        };

        let temp_security = SecurityLayer::new(SyncNodeRole::Peer, &hex::encode(&enc_key));

        let mut vc = VectorClock::new();
        vc.increment(&self.instance_id);

        let metadata = EntityMetadata {
            entity_id: routing_id.clone(),
            entity_type: "pairing_approval".to_string(),
            version: 1,
            updated_at: Utc::now(),
            source: "pairing".to_string(),
            source_priority: 0,
            instance_id: self.instance_id.clone(),
            device_id: None,
            vector_clock: vc,
        };

        let packet = temp_security
            .encrypt_packet(&metadata, &approval)
            .map_err(|e| anyhow!("Failed to encrypt approval: {}", e))?;

        self.relay
            .push_packet(&routing_id, &packet, Some(PAIRING_TTL_SECONDS))
            .await
            .map_err(|e| anyhow!("Failed to push approval to relay: {}", e))?;

        info!("Pairing: Sent approval on channel {}", &routing_id[..12]);
        Ok(())
    }

    /// Client: Receives the approval containing the network key
    pub async fn receive_approval(&self, code: &str) -> Result<PairingApproval> {
        let clean_code = code.replace('-', "");
        let (routing_id, enc_key) = Self::derive_keys(&clean_code, "approval");
        let temp_security = SecurityLayer::new(SyncNodeRole::Peer, &hex::encode(&enc_key));

        let packets = self
            .relay
            .pull_packets_for(&routing_id)
            .await
            .map_err(|e| anyhow!("Failed to pull approval from relay: {}", e))?;

        if packets.is_empty() {
            return Err(anyhow!("No approval packet yet"));
        }

        let approval: PairingApproval = temp_security
            .decrypt_packet(&packets[0])
            .map_err(|e| anyhow!("Failed to decrypt approval: {}", e))?;

        info!(
            "Pairing: Received approval from host '{}'",
            approval.host_instance_id
        );
        Ok(approval)
    }

    /// Derives a routing ID (used as target_instance_id on relay) and
    /// an AES-256 encryption key from the pairing code + context.
    fn derive_keys(code: &str, context: &str) -> (String, Vec<u8>) {
        let routing_id =
            hex::encode(Sha256::digest(format!("eck:pairing:id:{}:{}", context, code).as_bytes()));
        let enc_key =
            Sha256::digest(format!("eck:pairing:key:{}:{}", context, code).as_bytes()).to_vec();
        (routing_id, enc_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code_format() {
        let code = PairingService::generate_code();
        assert_eq!(code.len(), 7); // "XXX-XXX"
        assert_eq!(&code[3..4], "-");
    }

    #[test]
    fn test_derive_keys_deterministic() {
        let (id1, key1) = PairingService::derive_keys("123456", "offer");
        let (id2, key2) = PairingService::derive_keys("123456", "offer");
        assert_eq!(id1, id2);
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32); // AES-256 key
    }

    #[test]
    fn test_derive_keys_different_contexts() {
        let (id_offer, key_offer) = PairingService::derive_keys("123456", "offer");
        let (id_resp, key_resp) = PairingService::derive_keys("123456", "response");
        assert_ne!(id_offer, id_resp);
        assert_ne!(key_offer, key_resp);
    }

    #[test]
    fn test_derive_keys_different_codes() {
        let (id1, _) = PairingService::derive_keys("123456", "offer");
        let (id2, _) = PairingService::derive_keys("654321", "offer");
        assert_ne!(id1, id2);
    }
}
