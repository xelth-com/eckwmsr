use crate::db::AppState;
use crate::models::{device_intake, product_alias};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use std::sync::Arc;
use tracing::{error, info};

pub struct RepairService;

impl RepairService {
    /// ProcessIntake handles the 'intake_save' event from Android.
    /// Mirrors Go's `repair.Service.ProcessIntake` from `internal/services/repair/service.go`.
    pub async fn process_intake(
        state: Arc<AppState>,
        target_device_id: String,
        payload_json: &str,
    ) -> Result<(), anyhow::Error> {
        info!("RepairService: Processing intake for {}", target_device_id);

        let data: serde_json::Value = serde_json::from_str(payload_json)?;

        let hwb_number = data
            .get("hwb_number")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let serial_number = data
            .get("serial_number")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let packaging = data
            .get("packaging")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let has_psu = data
            .get("has_psu")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let cables_included = data
            .get("cables_included")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let intake = device_intake::ActiveModel {
            device_id: Set(target_device_id),
            user_id: Set(String::new()),
            tracking_number: Set(hwb_number.clone()),
            serial_number: Set(serial_number.clone()),
            has_power_supply: Set(has_psu),
            packaging: Set(packaging),
            cables_included: Set(cables_included),
            raw_data: Set(data),
            odoo_repair_id: Set(0),
            sync_status: Set("pending".to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let inserted = intake.insert(&state.db).await?;
        info!("DeviceIntake saved. ID: {}", inserted.id);

        // Link HWB <-> Serial via ProductAlias
        if !hwb_number.is_empty() && !serial_number.is_empty() {
            if let Err(e) = Self::create_alias(&state, &hwb_number, &serial_number).await {
                error!("Failed to create alias: {}", e);
            }
        }

        // Odoo sync: create repair order if client is configured
        if let Some(ref odoo_mutex) = state.odoo_client {
            let mut odoo = odoo_mutex.lock().await;
            // Authenticate on first use
            if odoo.authenticate().await.is_ok() {
                let description = format!(
                    "Intake #{} — SN: {}, HWB: {}",
                    inserted.id, serial_number, hwb_number
                );
                // product_id 0 = unknown; real mapping comes from barcode lookup
                match odoo.create_repair_order(0, &serial_number, &description).await {
                    Ok(repair_id) => {
                        info!("Odoo: Created repair order #{} for intake #{}", repair_id, inserted.id);
                    }
                    Err(e) => {
                        error!("Odoo: Failed to create repair order: {}", e);
                    }
                }
            } else {
                error!("Odoo: Authentication failed, skipping repair order creation");
            }
        } else {
            info!("Odoo: Not configured, skipping repair order sync for intake #{}", inserted.id);
        }

        Ok(())
    }

    async fn create_alias(
        state: &Arc<AppState>,
        external: &str,
        internal: &str,
    ) -> Result<(), anyhow::Error> {
        let exists = product_alias::Entity::find()
            .filter(product_alias::Column::ExternalCode.eq(external))
            .filter(product_alias::Column::InternalId.eq(internal))
            .count(&state.db)
            .await?;

        if exists > 0 {
            return Ok(());
        }

        let alias = product_alias::ActiveModel {
            external_code: Set(external.to_string()),
            internal_id: Set(internal.to_string()),
            r#type: Set("tracking_to_serial".to_string()),
            is_verified: Set(true),
            confidence_score: Set(100),
            created_context: Set(Some("device_intake".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        alias.insert(&state.db).await?;
        info!("Alias created: {} -> {}", external, internal);
        Ok(())
    }
}
