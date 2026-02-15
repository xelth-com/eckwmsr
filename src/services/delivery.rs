use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

#[derive(Debug)]
pub struct Address {
    pub name1: String,
    pub street: String,
    pub house_number: String,
    pub zip: String,
    pub city: String,
    pub country: String,
}

#[derive(Debug)]
pub struct DeliveryRequest {
    pub order_number: String,
    pub sender_address: Address,
    pub receiver_address: Address,
    pub weight: f64,
    pub ref_number: String,
}

#[derive(Debug)]
pub struct DeliveryResponse {
    pub tracking_number: String,
    pub raw_response: serde_json::Value,
}

pub trait DeliveryProvider: Send + Sync {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn create_shipment<'a>(&'a self, req: &'a DeliveryRequest) -> Pin<Box<dyn Future<Output = anyhow::Result<DeliveryResponse>> + Send + 'a>>;
    fn fetch_recent_shipments(&self, days: i32) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<serde_json::Value>>> + Send + '_>>;
}

pub struct DeliveryService {
    pub db: DatabaseConnection,
    pub config: Config,
    providers: Arc<RwLock<HashMap<String, Box<dyn DeliveryProvider>>>>,
}

impl DeliveryService {
    pub fn new(db: DatabaseConnection, config: Config) -> Self {
        Self {
            db,
            config,
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_provider(&self, provider: Box<dyn DeliveryProvider>) {
        let mut w = self.providers.write().await;
        w.insert(provider.code().to_string(), provider);
    }
}
