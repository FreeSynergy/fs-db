/// SeaORM entity for the `installed_packages` table.
///
/// Tracks every installed package with its version, channel, and install state.
use sea_orm::entity::prelude::*;

/// Installed package database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "installed_packages")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Package identifier (e.g. `"proxy/zentinel"`).
    pub package_id: String,
    /// Installed version string (semver).
    pub version: String,
    /// Release channel: `"stable"`, `"testing"`, or `"nightly"`.
    pub channel: String,
    /// Package type: `"app"`, `"container"`, `"bundle"`, `"theme"`, etc.
    pub package_type: String,
    /// Whether this is the currently active version (`true`) or a kept older version.
    pub active: bool,
    /// Ed25519 signature of the package, base64-encoded. `None` for unsigned packages.
    pub signature: Option<String>,
    /// Whether the package was explicitly trusted without a valid signature.
    pub trust_unsigned: bool,
    /// Install timestamp (Unix seconds, UTC).
    pub installed_at: i64,
    /// Last-updated timestamp (Unix seconds, UTC).
    pub updated_at: i64,
}

/// Installed package relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
