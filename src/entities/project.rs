/// SeaORM entity for the `projects` table.
///
/// A project groups hosts, modules, and services into a logical deployment unit.
use sea_orm::entity::prelude::*;

/// Project database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Unique project name (e.g. `"acme-corp"`).
    pub name: String,
    /// Optional primary domain for this project (e.g. `"acme.example.com"`).
    pub domain: Option<String>,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Lifecycle status: `"active"`, `"archived"`, or `"draft"`.
    pub status: String,
    /// Creation timestamp (Unix seconds, UTC).
    pub created_at: i64,
    /// Last-updated timestamp (Unix seconds, UTC).
    pub updated_at: i64,
}

/// Project relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
