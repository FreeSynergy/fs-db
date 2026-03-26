/// SeaORM entity for the `service_registry` table.
///
/// The service registry tracks what capabilities each deployed module exposes
/// and whether it is currently healthy.
use sea_orm::entity::prelude::*;

/// Service registry database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "service_registry")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Foreign key into the `modules` table.
    pub module_id: i64,
    /// Cached module name for display without joins.
    pub module_name: String,
    /// JSON array of capability strings (e.g. `["oidc-provider","scim-server"]`).
    pub capabilities: String,
    /// Optional base URL that other modules can use to reach this service.
    pub endpoint_url: Option<String>,
    /// Whether the last health check returned a healthy status.
    pub healthy: bool,
    /// Timestamp of the last health check (Unix seconds, UTC).
    pub last_check: Option<i64>,
}

/// Service registry relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
