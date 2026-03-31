#![deny(clippy::all, clippy::pedantic, warnings)]
//! `SeaORM`-based database abstraction for `FreeSynergy`.
//!
//! Provides connection management, base entity traits, concrete `SeaORM` entities,
//! embedded SQL migrations, and a write buffer for high-throughput batched writes.
//!
//! # Features
//! - `sqlite` (default) — `SQLite` via `sqlx-sqlite`
//! - `postgres` — `PostgreSQL` via `sqlx-postgres`
//!
//! # Quick start
//! ```rust,ignore
//! use fs_db::DbManager;
//!
//! let db = DbManager::open_default().await?;
//! db.resources().insert("host", "my-server", None, None, None).await?;
//! db.close().await?;
//! ```

pub mod connection;
pub mod engine;
pub mod entities;
pub mod entity;
pub mod filter;
pub mod manager;
pub mod migration;
pub mod record;
pub mod repo;
pub mod repository;
pub mod write_buffer;

pub use connection::{DbBackend, DbConnection};
pub use engine::{DbConfig, DbEngine, DbHealth, DbRow, DbRows};
pub use entity::{Auditable, FsEntity};
pub use filter::{Filter, Order};
pub use manager::DbManager;
pub use migration::Migrator;
pub use record::{DbRecord, DbRowExt};
pub use repo::{EngineRepository, Repository};
pub use repository::{
    AuditRepo, HostRepo, InstalledPackageRepo, ModuleRepo, PermissionRepo, PluginRepo, ProjectRepo,
    ResourceRepo, ServiceRegistryRepo,
};
pub use write_buffer::{BufferedWrite, FlushResult, WriteBuffer};

// Re-export sea_orm so consumers don't need a separate dependency.
pub use sea_orm;
