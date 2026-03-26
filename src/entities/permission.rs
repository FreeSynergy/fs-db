/// SeaORM entity for the `permissions` table.
///
/// RBAC permission grants: which subject may perform which action on which resource.
use sea_orm::entity::prelude::*;

/// Permission database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "permissions")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Subject that holds this permission (user ID, service ID, or role name).
    pub subject: String,
    /// Action that is permitted (e.g. `"node:deploy"`, `"node:read"`).
    pub action: String,
    /// Optional resource scope (`resource.id`); `NULL` = applies to all resources.
    pub resource_id: Option<i64>,
    /// Timestamp when the permission was granted (Unix seconds, UTC).
    pub granted_at: i64,
    /// Optional expiry timestamp (Unix seconds, UTC). `NULL` = never expires.
    pub expires_at: Option<i64>,
}

/// Permission relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
