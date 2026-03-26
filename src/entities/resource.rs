/// SeaORM entity for the `resources` table.
///
/// A resource represents any managed object within FreeSynergy:
/// a host, a service instance, a project, etc.
use sea_orm::entity::prelude::*;

/// Resource database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "resources")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Resource kind tag (e.g. `"host"`, `"service"`, `"project"`).
    pub kind: String,
    /// Human-readable name of the resource.
    pub name: String,
    /// Optional project scope (foreign key into `resources` where kind=project).
    pub project_id: Option<i64>,
    /// Optional parent resource (for hierarchical resources).
    pub parent_id: Option<i64>,
    /// Serialised metadata as JSON (arbitrary key-value pairs).
    pub meta: Option<String>,
    /// Creation timestamp (Unix seconds, UTC).
    pub created_at: i64,
    /// Last-updated timestamp (Unix seconds, UTC).
    pub updated_at: i64,
}

/// Resource relations (self-referential parent).
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// A resource may have many child resources.
    #[sea_orm(has_many = "Entity")]
    Children,
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Children.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
