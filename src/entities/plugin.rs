/// SeaORM entity for the `plugins` table.
///
/// Registry of loaded FreeSynergy plugins (WASM or native).
use sea_orm::entity::prelude::*;

/// Plugin database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "plugins")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Plugin name (e.g. `"zentinel"`, `"kanidm-bridge"`).
    pub name: String,
    /// Semantic version string (e.g. `"1.2.0"`).
    pub version: String,
    /// Plugin kind: `"wasm"` or `"native"`.
    pub kind: String,
    /// SHA-256 hash of the WASM binary (hex). `NULL` for native plugins.
    pub wasm_hash: Option<String>,
    /// Absolute path to the WASM file or native shared library.
    pub path: Option<String>,
    /// Whether this plugin is currently enabled.
    pub enabled: bool,
    /// Serialised capabilities/metadata as JSON.
    pub meta: Option<String>,
    /// Installation timestamp (Unix seconds, UTC).
    pub installed_at: i64,
}

/// Plugin relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
