/// SeaORM entity for the `hosts` table.
///
/// A host represents a physical or virtual machine managed by FreeSynergy.
use sea_orm::entity::prelude::*;

/// Host database model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "hosts")]
pub struct Model {
    /// Auto-incremented primary key.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Human-readable name of the host (e.g. `"web-01"`).
    pub name: String,
    /// Fully qualified domain name or hostname.
    pub fqdn: String,
    /// Primary IP address of the host.
    pub ip_address: String,
    /// SSH port (default `22`).
    pub ssh_port: i32,
    /// Operational status: `"online"`, `"offline"`, or `"unknown"`.
    pub status: String,
    /// Operating system label (e.g. `"Fedora 40"`).
    pub os: Option<String>,
    /// CPU architecture (e.g. `"x86_64"`, `"aarch64"`).
    pub architecture: Option<String>,
    /// Version of the FSN agent running on this host.
    pub agent_version: Option<String>,
    /// Optional foreign key into the `resources` table for project scoping.
    pub project_id: Option<i64>,
    /// Timestamp when this host joined the cluster (Unix seconds, UTC).
    pub joined_at: i64,
    /// Last-updated timestamp (Unix seconds, UTC).
    pub updated_at: i64,
}

/// Host relations.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
