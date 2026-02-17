use reqwest::Client;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use tracing::{info, warn};

use super::delivery::{DeliveryProvider, DeliveryRequest, DeliveryResponse};

pub struct OpalProvider {
    username: String,
    password: String,
    url: String,
    client: Client,
}

impl OpalProvider {
    pub fn new(username: String, password: String, url: String) -> Self {
        let url = if url.is_empty() {
            "https://opal-kurier.de".to_string()
        } else {
            url
        };
        Self {
            username,
            password,
            url,
            client: Client::new(),
        }
    }
}

impl DeliveryProvider for OpalProvider {
    fn code(&self) -> &'static str {
        "opal"
    }

    fn name(&self) -> &'static str {
        "OPAL Kurier"
    }

    fn create_shipment<'a>(
        &'a self,
        req: &'a DeliveryRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<DeliveryResponse>> + Send + 'a>> {
        Box::pin(async move {
            info!("OPAL: Delegating shipment {} to Playwright microservice", req.order_number);

            let payload = json!({
                "username": self.username,
                "password": self.password,
                "url": self.url,
                "order_number": req.order_number,
                "weight": req.weight,
                "ref_number": req.ref_number,
                "sender_address": {
                    "name1": req.sender_address.name1,
                    "street": req.sender_address.street,
                    "house_number": req.sender_address.house_number,
                    "zip": req.sender_address.zip,
                    "city": req.sender_address.city,
                    "country": req.sender_address.country,
                },
                "receiver_address": {
                    "name1": req.receiver_address.name1,
                    "street": req.receiver_address.street,
                    "house_number": req.receiver_address.house_number,
                    "zip": req.receiver_address.zip,
                    "city": req.receiver_address.city,
                    "country": req.receiver_address.country,
                }
            });

            let res = self.client
                .post("http://127.0.0.1:3211/api/opal/create")
                .json(&payload)
                .send()
                .await?;

            if !res.status().is_success() {
                let err_text = res.text().await.unwrap_or_default();
                warn!("OPAL Scraper failed: {}", err_text);
                return Err(anyhow::anyhow!("OPAL Scraper Service Error: {}", err_text));
            }

            let data: serde_json::Value = res.json().await?;

            let tracking_number = data["tracking_number"]
                .as_str()
                .unwrap_or(&format!("OPAL-UNKNOWN-{}", req.order_number))
                .to_string();

            Ok(DeliveryResponse {
                tracking_number,
                raw_response: data["raw_response"].clone(),
            })
        })
    }

    fn fetch_recent_shipments(
        &self,
        _days: i32,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<serde_json::Value>>> + Send + '_>> {
        Box::pin(async move { Ok(vec![]) })
    }
}
