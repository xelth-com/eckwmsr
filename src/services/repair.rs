use crate::db::AppState;
use crate::models::{device_intake, item, order, order_item_event, product_alias};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DatabaseBackend, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, Set, Statement};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

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

    /// Find an existing item by barcode or create a new one
    async fn find_or_create_item(
        state: &Arc<AppState>,
        barcode: &str,
    ) -> Result<item::Model, anyhow::Error> {
        // Try primary_barcode first (fast, indexed)
        if let Some(found) = item::Entity::find()
            .filter(item::Column::PrimaryBarcode.eq(barcode))
            .filter(item::Column::DeletedAt.is_null())
            .one(&state.db)
            .await?
        {
            return Ok(found);
        }

        // Fallback: search JSONB barcodes array via @> containment
        let json_needle = serde_json::json!([barcode]).to_string();
        if let Some(found) = item::Model::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT * FROM items WHERE barcodes @> $1::jsonb AND deleted_at IS NULL LIMIT 1",
            [json_needle.into()],
        ))
        .one(&state.db)
        .await?
        {
            return Ok(found);
        }

        // Create new item
        let new_item = item::ActiveModel {
            id: Set(Uuid::new_v4()),
            primary_barcode: Set(barcode.to_string()),
            barcodes: Set(serde_json::json!([barcode])),
            name: Set(None),
            main_photo_id: Set(None),
            metadata: Set(serde_json::json!({})),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            deleted_at: Set(None),
        };

        let inserted = new_item.insert(&state.db).await?;
        info!("Created new item {} for barcode {}", inserted.id, barcode);
        Ok(inserted)
    }

    /// Process device_bound handles the auto-creation of a Repair Order when a PDA binds a slot
    pub async fn process_device_bind(
        state: Arc<AppState>,
        serial_number: String,
    ) -> Result<(), anyhow::Error> {
        info!("RepairService: Processing device bind for {}", serial_number);

        // Find or create item for this barcode
        let item_record = Self::find_or_create_item(&state, &serial_number).await?;

        // Check if an active order already exists for this serial number
        let existing_order = order::Entity::find()
            .filter(order::Column::SerialNumber.eq(&serial_number))
            .filter(
                Condition::all()
                    .add(order::Column::Status.ne("completed"))
                    .add(order::Column::Status.ne("cancelled"))
            )
            .one(&state.db)
            .await?;

        if let Some(existing) = existing_order {
            info!("Active repair order already exists for {} (Order #{})", serial_number, existing.order_number);
            return Ok(());
        }

        // Generate new order number
        let order_number = format!(
            "REP-{}-{:04}",
            Utc::now().format("%Y%m%d"),
            rand::random::<u16>()
        );

        // Create new repair order (all NOT NULL fields must be set)
        let new_order = order::ActiveModel {
            id: Set(Uuid::new_v4()),
            order_number: Set(order_number.clone()),
            order_type: Set("repair".to_string()),
            serial_number: Set(serial_number.clone()),
            customer_name: Set("PDA Intake".to_string()),
            customer_email: Set(String::new()),
            customer_phone: Set(String::new()),
            product_sku: Set("UNKNOWN".to_string()),
            product_name: Set(String::new()),
            issue_description: Set("Auto-created from PDA repair slot bind".to_string()),
            diagnosis_notes: Set(String::new()),
            status: Set("pending".to_string()),
            priority: Set("normal".to_string()),
            repair_notes: Set(String::new()),
            parts_used: Set(serde_json::json!([])),
            labor_hours: Set(0.0),
            total_cost: Set(0.0),
            resolution: Set(String::new()),
            notes: Set(String::new()),
            metadata: Set(serde_json::json!({})),
            rma_reason: Set(String::new()),
            is_refund_requested: Set(false),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let inserted_order = new_order.insert(&state.db).await?;
        info!("Created auto-repair order #{} (ID: {}) for serial {}", order_number, inserted_order.id, serial_number);

        // Create order_item_event: item added to this order
        let event = order_item_event::ActiveModel {
            id: Set(Uuid::new_v4()),
            order_id: Set(inserted_order.id),
            item_id: Set(item_record.id),
            event_type: Set("added".to_string()),
            user_id: Set(None),
            notes: Set(Some("Auto-created from PDA repair slot bind".to_string())),
            created_at: Set(Utc::now()),
            deleted_at: Set(None),
        };
        let inserted_event = event.insert(&state.db).await?;
        info!("Created order_item_event: item {} added to order {}", item_record.id, inserted_order.id);

        // Push order + item + event to mesh peers
        let payload = crate::handlers::mesh_sync::PushPayload {
            products: vec![], locations: vec![], shipments: vec![], users: vec![],
            orders: vec![crate::handlers::mesh_sync::SyncableOrder::from(inserted_order.clone())],
            documents: vec![], file_resources: vec![], attachments: vec![],
            items: vec![item_record],
            order_item_events: vec![inserted_event],
        };
        crate::handlers::mesh_sync::push_to_all_peers(state.clone(), "order", &inserted_order.id.to_string(), payload);

        Ok(())
    }
}
