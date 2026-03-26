/// SeaORM entity for the `audit_logs` table.
///
/// Immutable audit trail for all resource mutations.
/// Rows are append-only — never updated or deleted.
use sea_orm::entity::prelude::*;

/// Audit log database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_logs")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Subject that performed the action (user ID, service ID, or `"system"`).
    pub actor: String,
    /// Action verb (e.g. `"deploy"`, `"delete"`, `"update"`).
    pub action: String,
    /// Affected resource ID (`resources.id`). `NULL` for system-level events.
    pub resource_id: Option<i64>,
    /// Optional resource kind snapshot (denormalised for readability).
    pub resource_kind: Option<String>,
    /// Serialised JSON payload with before/after state or extra context.
    pub payload: Option<String>,
    /// Source IP address or hostname, if available.
    pub source: Option<String>,
    /// Outcome: `"ok"`, `"error"`, or `"denied"`.
    pub outcome: String,
    /// Event timestamp (Unix seconds, UTC).
    pub created_at: i64,
}

/// Audit log relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
