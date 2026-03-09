use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260309_000001_initial_schema"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. user_auths
        manager
            .create_table(
                Table::create()
                    .table(UserAuths::Table)
                    .if_not_exists()
                    .col(uuid(UserAuths::Id).primary_key())
                    .col(string(UserAuths::Username))
                    .col(string(UserAuths::Password))
                    .col(string(UserAuths::Email))
                    .col(string_null(UserAuths::Name))
                    .col(string(UserAuths::Role))
                    .col(string(UserAuths::UserType))
                    .col(string_null(UserAuths::Company))
                    .col(string_null(UserAuths::GoogleId))
                    .col(string(UserAuths::Pin))
                    .col(boolean(UserAuths::IsActive))
                    .col(timestamp_with_time_zone_null(UserAuths::LastLogin))
                    .col(big_integer(UserAuths::FailedLoginAttempts).default(0))
                    .col(string(UserAuths::PreferredLanguage))
                    .col(timestamp_with_time_zone(UserAuths::CreatedAt))
                    .col(timestamp_with_time_zone(UserAuths::UpdatedAt))
                    .col(timestamp_with_time_zone_null(UserAuths::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 2. product_product
        manager
            .create_table(
                Table::create()
                    .table(ProductProduct::Table)
                    .if_not_exists()
                    .col(uuid(ProductProduct::Id).primary_key())
                    .col(string(ProductProduct::DefaultCode))
                    .col(string(ProductProduct::Barcode))
                    .col(string(ProductProduct::Name))
                    .col(boolean(ProductProduct::Active).default(true))
                    .col(string(ProductProduct::Type))
                    .col(double(ProductProduct::ListPrice))
                    .col(double(ProductProduct::StandardPrice))
                    .col(double(ProductProduct::Weight))
                    .col(double(ProductProduct::Volume))
                    .col(timestamp_with_time_zone(ProductProduct::WriteDate))
                    .col(timestamp_with_time_zone(ProductProduct::LastSyncedAt))
                    .to_owned(),
            )
            .await?;

        // 3. product_aliases
        manager
            .create_table(
                Table::create()
                    .table(ProductAliases::Table)
                    .if_not_exists()
                    .col(uuid(ProductAliases::Id).primary_key())
                    .col(string(ProductAliases::ExternalCode))
                    .col(string(ProductAliases::InternalId))
                    .col(string(ProductAliases::Type))
                    .col(boolean(ProductAliases::IsVerified).default(false))
                    .col(integer(ProductAliases::ConfidenceScore).default(0))
                    .col(string_null(ProductAliases::CreatedContext))
                    .col(timestamp_with_time_zone(ProductAliases::CreatedAt))
                    .col(timestamp_with_time_zone(ProductAliases::UpdatedAt))
                    .col(timestamp_with_time_zone_null(ProductAliases::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 4. stock_location
        manager
            .create_table(
                Table::create()
                    .table(StockLocation::Table)
                    .if_not_exists()
                    .col(uuid(StockLocation::Id).primary_key())
                    .col(string(StockLocation::Name))
                    .col(string(StockLocation::CompleteName))
                    .col(string(StockLocation::Barcode).unique_key())
                    .col(string(StockLocation::Usage))
                    .col(uuid_null(StockLocation::LocationId))
                    .col(boolean(StockLocation::Active).default(true))
                    .col(timestamp_with_time_zone(StockLocation::LastSyncedAt))
                    .col(timestamp_with_time_zone(StockLocation::CreatedAt))
                    .col(timestamp_with_time_zone(StockLocation::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockLocation::Table, StockLocation::LocationId)
                            .to(StockLocation::Table, StockLocation::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // 5. stock_quant
        manager
            .create_table(
                Table::create()
                    .table(StockQuant::Table)
                    .if_not_exists()
                    .col(uuid(StockQuant::Id).primary_key())
                    .col(uuid(StockQuant::ProductId))
                    .col(uuid(StockQuant::LocationId))
                    .col(uuid_null(StockQuant::LotId))
                    .col(uuid_null(StockQuant::PackageId))
                    .col(double(StockQuant::Quantity))
                    .col(double(StockQuant::ReservedQty))
                    .to_owned(),
            )
            .await?;

        // 6. entity_checksums
        manager
            .create_table(
                Table::create()
                    .table(EntityChecksums::Table)
                    .if_not_exists()
                    .col(uuid(EntityChecksums::Id).primary_key())
                    .col(string(EntityChecksums::EntityType))
                    .col(string(EntityChecksums::EntityId))
                    .col(string(EntityChecksums::ContentHash))
                    .col(string_null(EntityChecksums::ChildrenHash))
                    .col(string(EntityChecksums::FullHash))
                    .col(integer(EntityChecksums::ChildCount))
                    .col(timestamp_with_time_zone(EntityChecksums::LastUpdated))
                    .col(string(EntityChecksums::SourceInstance))
                    .col(string_null(EntityChecksums::SourceDevice))
                    .col(timestamp_with_time_zone(EntityChecksums::CreatedAt))
                    .col(timestamp_with_time_zone(EntityChecksums::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 7. res_partner
        manager
            .create_table(
                Table::create()
                    .table(ResPartner::Table)
                    .if_not_exists()
                    .col(uuid(ResPartner::Id).primary_key())
                    .col(string(ResPartner::Name))
                    .col(string(ResPartner::Street))
                    .col(string(ResPartner::Street2))
                    .col(string(ResPartner::Zip))
                    .col(string(ResPartner::City))
                    .col(uuid_null(ResPartner::StateId))
                    .col(uuid_null(ResPartner::CountryId))
                    .col(string(ResPartner::Phone))
                    .col(string(ResPartner::Email))
                    .col(string(ResPartner::Vat))
                    .col(string(ResPartner::CompanyType))
                    .col(boolean(ResPartner::IsCompany))
                    .to_owned(),
            )
            .await?;

        // 8. stock_picking
        manager
            .create_table(
                Table::create()
                    .table(StockPicking::Table)
                    .if_not_exists()
                    .col(uuid(StockPicking::Id).primary_key())
                    .col(string(StockPicking::Name).unique_key())
                    .col(string(StockPicking::State))
                    .col(uuid(StockPicking::LocationId))
                    .col(uuid(StockPicking::LocationDestId))
                    .col(timestamp_with_time_zone(StockPicking::ScheduledDate))
                    .col(string(StockPicking::Origin))
                    .col(string(StockPicking::Priority))
                    .col(uuid_null(StockPicking::PickingTypeId))
                    .col(uuid_null(StockPicking::PartnerId))
                    .col(timestamp_with_time_zone_null(StockPicking::DateDone))
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockPicking::Table, StockPicking::LocationId)
                            .to(StockLocation::Table, StockLocation::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockPicking::Table, StockPicking::LocationDestId)
                            .to(StockLocation::Table, StockLocation::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockPicking::Table, StockPicking::PartnerId)
                            .to(ResPartner::Table, ResPartner::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // 9. stock_move_line
        manager
            .create_table(
                Table::create()
                    .table(StockMoveLine::Table)
                    .if_not_exists()
                    .col(uuid(StockMoveLine::Id).primary_key())
                    .col(uuid(StockMoveLine::PickingId))
                    .col(uuid(StockMoveLine::ProductId))
                    .col(double(StockMoveLine::QtyDone))
                    .col(uuid(StockMoveLine::LocationId))
                    .col(uuid(StockMoveLine::LocationDestId))
                    .col(uuid_null(StockMoveLine::PackageId))
                    .col(uuid_null(StockMoveLine::ResultPackageId))
                    .col(uuid_null(StockMoveLine::LotId))
                    .col(string(StockMoveLine::State))
                    .col(string(StockMoveLine::Reference))
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockMoveLine::Table, StockMoveLine::PickingId)
                            .to(StockPicking::Table, StockPicking::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockMoveLine::Table, StockMoveLine::ProductId)
                            .to(ProductProduct::Table, ProductProduct::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockMoveLine::Table, StockMoveLine::LocationId)
                            .to(StockLocation::Table, StockLocation::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockMoveLine::Table, StockMoveLine::LocationDestId)
                            .to(StockLocation::Table, StockLocation::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // 10. warehouse_racks
        manager
            .create_table(
                Table::create()
                    .table(WarehouseRacks::Table)
                    .if_not_exists()
                    .col(uuid(WarehouseRacks::Id).primary_key())
                    .col(string(WarehouseRacks::Name))
                    .col(string_null(WarehouseRacks::Prefix))
                    .col(integer(WarehouseRacks::Columns).default(1))
                    .col(integer(WarehouseRacks::Rows).default(1))
                    .col(integer(WarehouseRacks::StartIndex))
                    .col(integer(WarehouseRacks::SortOrder).default(0))
                    .col(uuid_null(WarehouseRacks::WarehouseId))
                    .col(uuid_null(WarehouseRacks::MappedLocationId))
                    .col(integer(WarehouseRacks::PosX).default(0))
                    .col(integer(WarehouseRacks::PosY).default(0))
                    .col(integer(WarehouseRacks::Rotation).default(0))
                    .col(integer(WarehouseRacks::VisualWidth).default(0))
                    .col(integer(WarehouseRacks::VisualHeight).default(0))
                    .col(timestamp_with_time_zone(WarehouseRacks::CreatedAt))
                    .col(timestamp_with_time_zone(WarehouseRacks::UpdatedAt))
                    .col(timestamp_with_time_zone_null(WarehouseRacks::DeletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(WarehouseRacks::Table, WarehouseRacks::WarehouseId)
                            .to(StockLocation::Table, StockLocation::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(WarehouseRacks::Table, WarehouseRacks::MappedLocationId)
                            .to(StockLocation::Table, StockLocation::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // 11. file_resources
        manager
            .create_table(
                Table::create()
                    .table(FileResources::Table)
                    .if_not_exists()
                    .col(uuid(FileResources::Id).primary_key())
                    .col(string(FileResources::Hash).unique_key())
                    .col(string(FileResources::FileName))
                    .col(string(FileResources::MimeType))
                    .col(big_integer(FileResources::Size))
                    .col(integer(FileResources::Width))
                    .col(integer(FileResources::Height))
                    .col(ColumnDef::new(FileResources::AvatarData).binary().null())
                    .col(string(FileResources::FilePath))
                    .col(string(FileResources::SourceInstance))
                    .col(string(FileResources::Context))
                    .col(timestamp_with_time_zone(FileResources::CreatedAt))
                    .col(timestamp_with_time_zone(FileResources::UpdatedAt))
                    .col(timestamp_with_time_zone_null(FileResources::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 12. entity_attachments
        manager
            .create_table(
                Table::create()
                    .table(EntityAttachments::Table)
                    .if_not_exists()
                    .col(uuid(EntityAttachments::Id).primary_key())
                    .col(uuid(EntityAttachments::FileResourceId))
                    .col(string(EntityAttachments::ResModel))
                    .col(string(EntityAttachments::ResId))
                    .col(boolean(EntityAttachments::IsMain).default(false))
                    .col(string_null(EntityAttachments::Tags))
                    .col(string_null(EntityAttachments::Comment))
                    .col(timestamp_with_time_zone(EntityAttachments::CreatedAt))
                    .col(timestamp_with_time_zone(EntityAttachments::UpdatedAt))
                    .col(timestamp_with_time_zone_null(EntityAttachments::DeletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(EntityAttachments::Table, EntityAttachments::FileResourceId)
                            .to(FileResources::Table, FileResources::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // 13. delivery_carrier
        manager
            .create_table(
                Table::create()
                    .table(DeliveryCarrier::Table)
                    .if_not_exists()
                    .col(uuid(DeliveryCarrier::Id).primary_key())
                    .col(string(DeliveryCarrier::Name))
                    .col(string(DeliveryCarrier::ProviderCode).unique_key())
                    .col(boolean(DeliveryCarrier::Active).default(true))
                    .col(text(DeliveryCarrier::ConfigJson))
                    .col(timestamp_with_time_zone(DeliveryCarrier::CreatedAt))
                    .col(timestamp_with_time_zone(DeliveryCarrier::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 14. stock_picking_delivery
        manager
            .create_table(
                Table::create()
                    .table(StockPickingDelivery::Table)
                    .if_not_exists()
                    .col(uuid(StockPickingDelivery::Id).primary_key())
                    .col(uuid_null(StockPickingDelivery::PickingId).unique_key())
                    .col(uuid_null(StockPickingDelivery::CarrierId))
                    .col(string(StockPickingDelivery::TrackingNumber))
                    .col(double(StockPickingDelivery::CarrierPrice))
                    .col(string(StockPickingDelivery::Currency))
                    .col(string(StockPickingDelivery::Status).default("draft"))
                    .col(text(StockPickingDelivery::ErrorMessage))
                    .col(string(StockPickingDelivery::LabelUrl))
                    .col(ColumnDef::new(StockPickingDelivery::LabelData).binary().null())
                    .col(string(StockPickingDelivery::RawResponse))
                    .col(timestamp_with_time_zone_null(StockPickingDelivery::ShippedAt))
                    .col(timestamp_with_time_zone_null(StockPickingDelivery::DeliveredAt))
                    .col(timestamp_with_time_zone_null(StockPickingDelivery::LastActivityAt))
                    .col(timestamp_with_time_zone(StockPickingDelivery::CreatedAt))
                    .col(timestamp_with_time_zone(StockPickingDelivery::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockPickingDelivery::Table, StockPickingDelivery::PickingId)
                            .to(StockPicking::Table, StockPicking::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(StockPickingDelivery::Table, StockPickingDelivery::CarrierId)
                            .to(DeliveryCarrier::Table, DeliveryCarrier::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // 15. delivery_tracking
        manager
            .create_table(
                Table::create()
                    .table(DeliveryTracking::Table)
                    .if_not_exists()
                    .col(uuid(DeliveryTracking::Id).primary_key())
                    .col(uuid(DeliveryTracking::PickingDeliveryId))
                    .col(timestamp_with_time_zone(DeliveryTracking::Timestamp))
                    .col(string(DeliveryTracking::Status))
                    .col(string(DeliveryTracking::StatusCode))
                    .col(string(DeliveryTracking::Location))
                    .col(text(DeliveryTracking::Description))
                    .col(timestamp_with_time_zone(DeliveryTracking::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(DeliveryTracking::Table, DeliveryTracking::PickingDeliveryId)
                            .to(StockPickingDelivery::Table, StockPickingDelivery::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on delivery_tracking.picking_delivery_id
        manager
            .create_index(
                Index::create()
                    .name("idx_delivery_tracking_picking_delivery_id")
                    .table(DeliveryTracking::Table)
                    .col(DeliveryTracking::PickingDeliveryId)
                    .to_owned(),
            )
            .await?;

        // 16. sync_history
        manager
            .create_table(
                Table::create()
                    .table(SyncHistory::Table)
                    .if_not_exists()
                    .col(string(SyncHistory::Id).primary_key())
                    .col(string(SyncHistory::InstanceId))
                    .col(string(SyncHistory::Provider))
                    .col(string(SyncHistory::Status))
                    .col(timestamp_with_time_zone(SyncHistory::StartedAt))
                    .col(timestamp_with_time_zone_null(SyncHistory::CompletedAt))
                    .col(big_integer(SyncHistory::Duration).default(0))
                    .col(big_integer(SyncHistory::Created).default(0))
                    .col(big_integer(SyncHistory::Updated).default(0))
                    .col(big_integer(SyncHistory::Skipped).default(0))
                    .col(big_integer(SyncHistory::Errors).default(0))
                    .col(text(SyncHistory::ErrorDetail))
                    .col(ColumnDef::new(SyncHistory::DebugInfo).json_binary().null())
                    .col(timestamp_with_time_zone(SyncHistory::CreatedAt))
                    .col(timestamp_with_time_zone(SyncHistory::UpdatedAt))
                    .col(timestamp_with_time_zone_null(SyncHistory::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // Indexes on sync_history
        manager
            .create_index(
                Index::create()
                    .name("idx_sync_history_provider")
                    .table(SyncHistory::Table)
                    .col(SyncHistory::Provider)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_sync_history_status")
                    .table(SyncHistory::Table)
                    .col(SyncHistory::Status)
                    .to_owned(),
            )
            .await?;

        // 17. orders
        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(uuid(Orders::Id).primary_key())
                    .col(string(Orders::OrderNumber).unique_key())
                    .col(string(Orders::OrderType))
                    .col(string(Orders::CustomerName))
                    .col(string(Orders::CustomerEmail))
                    .col(string(Orders::CustomerPhone))
                    .col(ColumnDef::new(Orders::ItemId).integer().null())
                    .col(string(Orders::ProductSku))
                    .col(string(Orders::ProductName))
                    .col(string(Orders::SerialNumber))
                    .col(timestamp_with_time_zone_null(Orders::PurchaseDate))
                    .col(text(Orders::IssueDescription))
                    .col(text(Orders::DiagnosisNotes))
                    .col(string_null(Orders::AssignedTo))
                    .col(string(Orders::Status))
                    .col(string(Orders::Priority))
                    .col(text(Orders::RepairNotes))
                    .col(ColumnDef::new(Orders::PartsUsed).json_binary().not_null())
                    .col(double(Orders::LaborHours))
                    .col(double(Orders::TotalCost))
                    .col(text(Orders::Resolution))
                    .col(text(Orders::Notes))
                    .col(ColumnDef::new(Orders::Metadata).json_binary().not_null())
                    .col(string(Orders::RmaReason))
                    .col(boolean(Orders::IsRefundRequested))
                    .col(timestamp_with_time_zone_null(Orders::StartedAt))
                    .col(timestamp_with_time_zone_null(Orders::CompletedAt))
                    .col(timestamp_with_time_zone(Orders::CreatedAt))
                    .col(timestamp_with_time_zone(Orders::UpdatedAt))
                    .col(timestamp_with_time_zone_null(Orders::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // Indexes on orders
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_order_type")
                    .table(Orders::Table)
                    .col(Orders::OrderType)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_customer_name")
                    .table(Orders::Table)
                    .col(Orders::CustomerName)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_product_sku")
                    .table(Orders::Table)
                    .col(Orders::ProductSku)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_serial_number")
                    .table(Orders::Table)
                    .col(Orders::SerialNumber)
                    .to_owned(),
            )
            .await?;

        // 18. device_intakes
        manager
            .create_table(
                Table::create()
                    .table(DeviceIntakes::Table)
                    .if_not_exists()
                    .col(uuid(DeviceIntakes::Id).primary_key())
                    .col(string(DeviceIntakes::DeviceId))
                    .col(string(DeviceIntakes::UserId))
                    .col(string(DeviceIntakes::TrackingNumber))
                    .col(string(DeviceIntakes::SerialNumber))
                    .col(boolean(DeviceIntakes::HasPowerSupply))
                    .col(string(DeviceIntakes::Packaging))
                    .col(boolean(DeviceIntakes::CablesIncluded))
                    .col(ColumnDef::new(DeviceIntakes::RawData).json_binary().not_null())
                    .col(big_integer(DeviceIntakes::OdooRepairId))
                    .col(string(DeviceIntakes::SyncStatus))
                    .col(timestamp_with_time_zone(DeviceIntakes::CreatedAt))
                    .col(timestamp_with_time_zone(DeviceIntakes::UpdatedAt))
                    .col(timestamp_with_time_zone_null(DeviceIntakes::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 19. inventory_discrepancy
        manager
            .create_table(
                Table::create()
                    .table(InventoryDiscrepancy::Table)
                    .if_not_exists()
                    .col(uuid(InventoryDiscrepancy::Id).primary_key())
                    .col(string(InventoryDiscrepancy::DocumentId))
                    .col(uuid(InventoryDiscrepancy::ProductId))
                    .col(string(InventoryDiscrepancy::ProductBarcode))
                    .col(string(InventoryDiscrepancy::ProductName))
                    .col(string(InventoryDiscrepancy::ProductCode))
                    .col(uuid(InventoryDiscrepancy::LocationId))
                    .col(string(InventoryDiscrepancy::LocationBarcode))
                    .col(string(InventoryDiscrepancy::LocationName))
                    .col(double(InventoryDiscrepancy::ExpectedQty))
                    .col(double(InventoryDiscrepancy::CountedQty))
                    .col(double(InventoryDiscrepancy::Delta))
                    .col(string(InventoryDiscrepancy::ItemType))
                    .col(string(InventoryDiscrepancy::DeviceId))
                    .col(string(InventoryDiscrepancy::Status))
                    .col(string_null(InventoryDiscrepancy::Notes))
                    .col(string_null(InventoryDiscrepancy::ReviewedBy))
                    .col(timestamp_with_time_zone_null(InventoryDiscrepancy::ReviewedAt))
                    .col(timestamp_with_time_zone(InventoryDiscrepancy::CreatedAt))
                    .col(timestamp_with_time_zone(InventoryDiscrepancy::UpdatedAt))
                    .col(timestamp_with_time_zone_null(InventoryDiscrepancy::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 20. documents
        manager
            .create_table(
                Table::create()
                    .table(Documents::Table)
                    .if_not_exists()
                    .col(uuid(Documents::DocumentId).primary_key())
                    .col(string(Documents::Type))
                    .col(string(Documents::Status))
                    .col(ColumnDef::new(Documents::Payload).json_binary().not_null())
                    .col(string(Documents::DeviceId))
                    .col(string(Documents::UserId))
                    .col(timestamp_with_time_zone(Documents::CreatedAt))
                    .col(timestamp_with_time_zone(Documents::UpdatedAt))
                    .col(timestamp_with_time_zone_null(Documents::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 21. mesh_nodes
        manager
            .create_table(
                Table::create()
                    .table(MeshNodes::Table)
                    .if_not_exists()
                    .col(string(MeshNodes::InstanceId).primary_key())
                    .col(string(MeshNodes::Name))
                    .col(string(MeshNodes::BaseUrl))
                    .col(string(MeshNodes::Role))
                    .col(string(MeshNodes::Status))
                    .col(timestamp_with_time_zone(MeshNodes::LastSeen))
                    .col(timestamp_with_time_zone(MeshNodes::CreatedAt))
                    .col(timestamp_with_time_zone(MeshNodes::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 22. registered_devices
        manager
            .create_table(
                Table::create()
                    .table(RegisteredDevices::Table)
                    .if_not_exists()
                    .col(string(RegisteredDevices::DeviceId).primary_key())
                    .col(string_null(RegisteredDevices::Name))
                    .col(string_null(RegisteredDevices::PublicKey))
                    .col(string_null(RegisteredDevices::Status))
                    .col(string_null(RegisteredDevices::HomeInstanceId))
                    .col(timestamp_with_time_zone_null(RegisteredDevices::LastSeenAt))
                    .col(timestamp_with_time_zone_null(RegisteredDevices::CreatedAt))
                    .col(timestamp_with_time_zone_null(RegisteredDevices::UpdatedAt))
                    .col(timestamp_with_time_zone_null(RegisteredDevices::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 23. items
        manager
            .create_table(
                Table::create()
                    .table(Items::Table)
                    .if_not_exists()
                    .col(uuid(Items::Id).primary_key())
                    .col(string(Items::PrimaryBarcode).unique_key())
                    .col(ColumnDef::new(Items::Barcodes).json_binary().not_null())
                    .col(string_null(Items::Name))
                    .col(uuid_null(Items::MainPhotoId))
                    .col(ColumnDef::new(Items::Metadata).json_binary().not_null())
                    .col(timestamp_with_time_zone(Items::CreatedAt))
                    .col(timestamp_with_time_zone(Items::UpdatedAt))
                    .col(timestamp_with_time_zone_null(Items::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 24. order_item_events
        manager
            .create_table(
                Table::create()
                    .table(OrderItemEvents::Table)
                    .if_not_exists()
                    .col(uuid(OrderItemEvents::Id).primary_key())
                    .col(uuid(OrderItemEvents::OrderId))
                    .col(uuid(OrderItemEvents::ItemId))
                    .col(string(OrderItemEvents::EventType))
                    .col(string_null(OrderItemEvents::UserId))
                    .col(string_null(OrderItemEvents::Notes))
                    .col(timestamp_with_time_zone(OrderItemEvents::CreatedAt))
                    .col(timestamp_with_time_zone_null(OrderItemEvents::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // Indexes on order_item_events
        manager
            .create_index(
                Index::create()
                    .name("idx_order_item_events_order_id")
                    .table(OrderItemEvents::Table)
                    .col(OrderItemEvents::OrderId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_order_item_events_item_id")
                    .table(OrderItemEvents::Table)
                    .col(OrderItemEvents::ItemId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop in reverse order to respect FK dependencies
        let tables = vec![
            OrderItemEvents::Table.into_table_ref(),
            Items::Table.into_table_ref(),
            RegisteredDevices::Table.into_table_ref(),
            MeshNodes::Table.into_table_ref(),
            Documents::Table.into_table_ref(),
            InventoryDiscrepancy::Table.into_table_ref(),
            DeviceIntakes::Table.into_table_ref(),
            Orders::Table.into_table_ref(),
            SyncHistory::Table.into_table_ref(),
            DeliveryTracking::Table.into_table_ref(),
            StockPickingDelivery::Table.into_table_ref(),
            DeliveryCarrier::Table.into_table_ref(),
            EntityAttachments::Table.into_table_ref(),
            FileResources::Table.into_table_ref(),
            WarehouseRacks::Table.into_table_ref(),
            StockMoveLine::Table.into_table_ref(),
            StockPicking::Table.into_table_ref(),
            ResPartner::Table.into_table_ref(),
            EntityChecksums::Table.into_table_ref(),
            StockQuant::Table.into_table_ref(),
            StockLocation::Table.into_table_ref(),
            ProductAliases::Table.into_table_ref(),
            ProductProduct::Table.into_table_ref(),
            UserAuths::Table.into_table_ref(),
        ];

        for table in tables {
            manager
                .drop_table(Table::drop().table(table).if_exists().to_owned())
                .await?;
        }

        Ok(())
    }
}

// ── Iden enums for each table ─────────────────────────────────────────

#[derive(DeriveIden)]
enum UserAuths {
    Table,
    Id,
    Username,
    Password,
    Email,
    Name,
    Role,
    #[sea_orm(iden = "user_type")]
    UserType,
    Company,
    #[sea_orm(iden = "google_id")]
    GoogleId,
    Pin,
    #[sea_orm(iden = "is_active")]
    IsActive,
    #[sea_orm(iden = "last_login")]
    LastLogin,
    #[sea_orm(iden = "failed_login_attempts")]
    FailedLoginAttempts,
    #[sea_orm(iden = "preferred_language")]
    PreferredLanguage,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum ProductProduct {
    Table,
    Id,
    #[sea_orm(iden = "default_code")]
    DefaultCode,
    Barcode,
    Name,
    Active,
    #[sea_orm(iden = "type")]
    Type,
    #[sea_orm(iden = "list_price")]
    ListPrice,
    #[sea_orm(iden = "standard_price")]
    StandardPrice,
    Weight,
    Volume,
    #[sea_orm(iden = "write_date")]
    WriteDate,
    #[sea_orm(iden = "last_synced_at")]
    LastSyncedAt,
}

#[derive(DeriveIden)]
enum ProductAliases {
    Table,
    Id,
    #[sea_orm(iden = "external_code")]
    ExternalCode,
    #[sea_orm(iden = "internal_id")]
    InternalId,
    #[sea_orm(iden = "type")]
    Type,
    #[sea_orm(iden = "is_verified")]
    IsVerified,
    #[sea_orm(iden = "confidence_score")]
    ConfidenceScore,
    #[sea_orm(iden = "created_context")]
    CreatedContext,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum StockLocation {
    Table,
    Id,
    Name,
    #[sea_orm(iden = "complete_name")]
    CompleteName,
    Barcode,
    Usage,
    #[sea_orm(iden = "location_id")]
    LocationId,
    Active,
    #[sea_orm(iden = "last_synced_at")]
    LastSyncedAt,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

#[derive(DeriveIden)]
enum StockQuant {
    Table,
    Id,
    #[sea_orm(iden = "product_id")]
    ProductId,
    #[sea_orm(iden = "location_id")]
    LocationId,
    #[sea_orm(iden = "lot_id")]
    LotId,
    #[sea_orm(iden = "package_id")]
    PackageId,
    Quantity,
    #[sea_orm(iden = "reserved_qty")]
    ReservedQty,
}

#[derive(DeriveIden)]
enum EntityChecksums {
    Table,
    Id,
    #[sea_orm(iden = "entity_type")]
    EntityType,
    #[sea_orm(iden = "entity_id")]
    EntityId,
    #[sea_orm(iden = "content_hash")]
    ContentHash,
    #[sea_orm(iden = "children_hash")]
    ChildrenHash,
    #[sea_orm(iden = "full_hash")]
    FullHash,
    #[sea_orm(iden = "child_count")]
    ChildCount,
    #[sea_orm(iden = "last_updated")]
    LastUpdated,
    #[sea_orm(iden = "source_instance")]
    SourceInstance,
    #[sea_orm(iden = "source_device")]
    SourceDevice,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ResPartner {
    Table,
    Id,
    Name,
    Street,
    Street2,
    Zip,
    City,
    #[sea_orm(iden = "state_id")]
    StateId,
    #[sea_orm(iden = "country_id")]
    CountryId,
    Phone,
    Email,
    Vat,
    #[sea_orm(iden = "company_type")]
    CompanyType,
    #[sea_orm(iden = "is_company")]
    IsCompany,
}

#[derive(DeriveIden)]
enum StockPicking {
    Table,
    Id,
    Name,
    State,
    #[sea_orm(iden = "location_id")]
    LocationId,
    #[sea_orm(iden = "location_dest_id")]
    LocationDestId,
    #[sea_orm(iden = "scheduled_date")]
    ScheduledDate,
    Origin,
    Priority,
    #[sea_orm(iden = "picking_type_id")]
    PickingTypeId,
    #[sea_orm(iden = "partner_id")]
    PartnerId,
    #[sea_orm(iden = "date_done")]
    DateDone,
}

#[derive(DeriveIden)]
enum StockMoveLine {
    Table,
    Id,
    #[sea_orm(iden = "picking_id")]
    PickingId,
    #[sea_orm(iden = "product_id")]
    ProductId,
    #[sea_orm(iden = "qty_done")]
    QtyDone,
    #[sea_orm(iden = "location_id")]
    LocationId,
    #[sea_orm(iden = "location_dest_id")]
    LocationDestId,
    #[sea_orm(iden = "package_id")]
    PackageId,
    #[sea_orm(iden = "result_package_id")]
    ResultPackageId,
    #[sea_orm(iden = "lot_id")]
    LotId,
    State,
    Reference,
}

#[derive(DeriveIden)]
enum WarehouseRacks {
    Table,
    Id,
    Name,
    Prefix,
    Columns,
    Rows,
    #[sea_orm(iden = "start_index")]
    StartIndex,
    #[sea_orm(iden = "sort_order")]
    SortOrder,
    #[sea_orm(iden = "warehouse_id")]
    WarehouseId,
    #[sea_orm(iden = "mapped_location_id")]
    MappedLocationId,
    #[sea_orm(iden = "pos_x")]
    PosX,
    #[sea_orm(iden = "pos_y")]
    PosY,
    Rotation,
    #[sea_orm(iden = "visual_width")]
    VisualWidth,
    #[sea_orm(iden = "visual_height")]
    VisualHeight,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum FileResources {
    Table,
    Id,
    Hash,
    #[sea_orm(iden = "file_name")]
    FileName,
    #[sea_orm(iden = "mime_type")]
    MimeType,
    Size,
    Width,
    Height,
    #[sea_orm(iden = "avatar_data")]
    AvatarData,
    #[sea_orm(iden = "file_path")]
    FilePath,
    #[sea_orm(iden = "source_instance")]
    SourceInstance,
    Context,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum EntityAttachments {
    Table,
    Id,
    #[sea_orm(iden = "file_resource_id")]
    FileResourceId,
    #[sea_orm(iden = "res_model")]
    ResModel,
    #[sea_orm(iden = "res_id")]
    ResId,
    #[sea_orm(iden = "is_main")]
    IsMain,
    Tags,
    Comment,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum DeliveryCarrier {
    Table,
    Id,
    Name,
    #[sea_orm(iden = "provider_code")]
    ProviderCode,
    Active,
    #[sea_orm(iden = "config_json")]
    ConfigJson,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

#[derive(DeriveIden)]
enum StockPickingDelivery {
    Table,
    Id,
    #[sea_orm(iden = "picking_id")]
    PickingId,
    #[sea_orm(iden = "carrier_id")]
    CarrierId,
    #[sea_orm(iden = "tracking_number")]
    TrackingNumber,
    #[sea_orm(iden = "carrier_price")]
    CarrierPrice,
    Currency,
    Status,
    #[sea_orm(iden = "error_message")]
    ErrorMessage,
    #[sea_orm(iden = "label_url")]
    LabelUrl,
    #[sea_orm(iden = "label_data")]
    LabelData,
    #[sea_orm(iden = "raw_response")]
    RawResponse,
    #[sea_orm(iden = "shipped_at")]
    ShippedAt,
    #[sea_orm(iden = "delivered_at")]
    DeliveredAt,
    #[sea_orm(iden = "last_activity_at")]
    LastActivityAt,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

#[derive(DeriveIden)]
enum DeliveryTracking {
    Table,
    Id,
    #[sea_orm(iden = "picking_delivery_id")]
    PickingDeliveryId,
    Timestamp,
    Status,
    #[sea_orm(iden = "status_code")]
    StatusCode,
    Location,
    Description,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
}

#[derive(DeriveIden)]
enum SyncHistory {
    Table,
    Id,
    #[sea_orm(iden = "instance_id")]
    InstanceId,
    Provider,
    Status,
    #[sea_orm(iden = "started_at")]
    StartedAt,
    #[sea_orm(iden = "completed_at")]
    CompletedAt,
    Duration,
    Created,
    Updated,
    Skipped,
    Errors,
    #[sea_orm(iden = "error_detail")]
    ErrorDetail,
    #[sea_orm(iden = "debug_info")]
    DebugInfo,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
    #[sea_orm(iden = "order_number")]
    OrderNumber,
    #[sea_orm(iden = "order_type")]
    OrderType,
    #[sea_orm(iden = "customer_name")]
    CustomerName,
    #[sea_orm(iden = "customer_email")]
    CustomerEmail,
    #[sea_orm(iden = "customer_phone")]
    CustomerPhone,
    #[sea_orm(iden = "item_id")]
    ItemId,
    #[sea_orm(iden = "product_sku")]
    ProductSku,
    #[sea_orm(iden = "product_name")]
    ProductName,
    #[sea_orm(iden = "serial_number")]
    SerialNumber,
    #[sea_orm(iden = "purchase_date")]
    PurchaseDate,
    #[sea_orm(iden = "issue_description")]
    IssueDescription,
    #[sea_orm(iden = "diagnosis_notes")]
    DiagnosisNotes,
    #[sea_orm(iden = "assigned_to")]
    AssignedTo,
    Status,
    Priority,
    #[sea_orm(iden = "repair_notes")]
    RepairNotes,
    #[sea_orm(iden = "parts_used")]
    PartsUsed,
    #[sea_orm(iden = "labor_hours")]
    LaborHours,
    #[sea_orm(iden = "total_cost")]
    TotalCost,
    Resolution,
    Notes,
    Metadata,
    #[sea_orm(iden = "rma_reason")]
    RmaReason,
    #[sea_orm(iden = "is_refund_requested")]
    IsRefundRequested,
    #[sea_orm(iden = "started_at")]
    StartedAt,
    #[sea_orm(iden = "completed_at")]
    CompletedAt,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum DeviceIntakes {
    Table,
    Id,
    #[sea_orm(iden = "device_id")]
    DeviceId,
    #[sea_orm(iden = "user_id")]
    UserId,
    #[sea_orm(iden = "tracking_number")]
    TrackingNumber,
    #[sea_orm(iden = "serial_number")]
    SerialNumber,
    #[sea_orm(iden = "has_power_supply")]
    HasPowerSupply,
    Packaging,
    #[sea_orm(iden = "cables_included")]
    CablesIncluded,
    #[sea_orm(iden = "raw_data")]
    RawData,
    #[sea_orm(iden = "odoo_repair_id")]
    OdooRepairId,
    #[sea_orm(iden = "sync_status")]
    SyncStatus,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum InventoryDiscrepancy {
    Table,
    Id,
    #[sea_orm(iden = "document_id")]
    DocumentId,
    #[sea_orm(iden = "product_id")]
    ProductId,
    #[sea_orm(iden = "product_barcode")]
    ProductBarcode,
    #[sea_orm(iden = "product_name")]
    ProductName,
    #[sea_orm(iden = "product_code")]
    ProductCode,
    #[sea_orm(iden = "location_id")]
    LocationId,
    #[sea_orm(iden = "location_barcode")]
    LocationBarcode,
    #[sea_orm(iden = "location_name")]
    LocationName,
    #[sea_orm(iden = "expected_qty")]
    ExpectedQty,
    #[sea_orm(iden = "counted_qty")]
    CountedQty,
    Delta,
    #[sea_orm(iden = "item_type")]
    ItemType,
    #[sea_orm(iden = "device_id")]
    DeviceId,
    Status,
    Notes,
    #[sea_orm(iden = "reviewed_by")]
    ReviewedBy,
    #[sea_orm(iden = "reviewed_at")]
    ReviewedAt,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum Documents {
    Table,
    #[sea_orm(iden = "document_id")]
    DocumentId,
    #[sea_orm(iden = "type")]
    Type,
    Status,
    Payload,
    #[sea_orm(iden = "device_id")]
    DeviceId,
    #[sea_orm(iden = "user_id")]
    UserId,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum MeshNodes {
    Table,
    #[sea_orm(iden = "instance_id")]
    InstanceId,
    Name,
    #[sea_orm(iden = "base_url")]
    BaseUrl,
    Role,
    Status,
    #[sea_orm(iden = "last_seen")]
    LastSeen,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RegisteredDevices {
    Table,
    #[sea_orm(iden = "device_id")]
    DeviceId,
    Name,
    #[sea_orm(iden = "public_key")]
    PublicKey,
    Status,
    #[sea_orm(iden = "home_instance_id")]
    HomeInstanceId,
    #[sea_orm(iden = "last_seen_at")]
    LastSeenAt,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum Items {
    Table,
    Id,
    #[sea_orm(iden = "primary_barcode")]
    PrimaryBarcode,
    Barcodes,
    Name,
    #[sea_orm(iden = "main_photo_id")]
    MainPhotoId,
    Metadata,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}

#[derive(DeriveIden)]
enum OrderItemEvents {
    Table,
    Id,
    #[sea_orm(iden = "order_id")]
    OrderId,
    #[sea_orm(iden = "item_id")]
    ItemId,
    #[sea_orm(iden = "event_type")]
    EventType,
    #[sea_orm(iden = "user_id")]
    UserId,
    Notes,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
}
