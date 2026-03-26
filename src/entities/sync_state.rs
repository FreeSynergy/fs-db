/// SeaORM entity for the `sync_states` table.
///
/// Stores per-resource CRDT vector-clock state for offline-first synchronisation.
use sea_orm::entity::prelude::*;

/// Sync-state database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sync_states")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// The resource this sync state belongs to (`resources.id`).
    pub resource_id: i64,
    /// Node identifier in the sync network (e.g. hostname or UUID).
    pub node_id: String,
    /// Serialised vector clock as JSON (`{"node_a": 3, "node_b": 7, …}`).
    pub vector_clock: String,
    /// Serialised pending CRDT operations waiting to be merged (JSON array).
    pub pending_ops: Option<String>,
    /// Timestamp of the last successful sync (Unix seconds, UTC).
    pub last_synced: i64,
}

/// Sync-state relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
