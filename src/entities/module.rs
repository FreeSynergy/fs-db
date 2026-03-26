/// SeaORM entity for the `modules` table.
///
/// A module represents a deployed service instance (e.g. a proxy, mail server, or wiki)
/// running on a specific host within a project.
use sea_orm::entity::prelude::*;

/// Module database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "modules")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Human-readable name for this module instance (e.g. `"main-proxy"`).
    pub name: String,
    /// Module type identifier (e.g. `"proxy"`, `"iam"`, `"mail"`, `"wiki"`).
    pub module_type: String,
    /// Foreign key into the `hosts` table — the host this module runs on.
    pub host_id: i64,
    /// Optional foreign key into the `projects` table for project scoping.
    pub project_id: Option<i64>,
    /// Operational status: `"running"`, `"stopped"`, `"error"`, or `"deploying"`.
    pub status: String,
    /// Deployed module version (e.g. `"1.2.0"`).
    pub version: Option<String>,
    /// Module configuration serialised as JSON.
    pub config: Option<String>,
    /// Creation timestamp (Unix seconds, UTC).
    pub created_at: i64,
    /// Last-updated timestamp (Unix seconds, UTC).
    pub updated_at: i64,
}

/// Module relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
