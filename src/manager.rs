/// High-level database manager: opens connection, runs migrations, exposes repositories.
///
/// `DbManager` is the primary entry point for FreeSynergy applications that need
/// persistent storage. It handles connection setup, schema migration, and provides
/// typed repository accessors for every core table.
///
/// # Example
/// ```rust,ignore
/// let db = DbManager::open_default().await?;
/// db.resources().insert("host", "my-server", None, None, None).await?;
/// db.audit().log("system", "init", None, None, None, None, "ok").await?;
/// db.close().await?;
/// ```
use std::path::Path;

use crate::{
    connection::{DbBackend, DbConnection},
    migration::Migrator,
    repository::{
        AuditRepo, HostRepo, InstalledPackageRepo, ModuleRepo, PermissionRepo, PluginRepo,
        ProjectRepo, ResourceRepo, ServiceRegistryRepo,
    },
};
use fs_error::FsError;

/// Combined database handle with connection lifecycle and repository accessors.
pub struct DbManager {
    conn: DbConnection,
}

impl DbManager {
    /// Open (or create) a SQLite database at `path`, running all pending migrations.
    pub async fn open_sqlite(path: impl AsRef<Path>) -> Result<Self, FsError> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| FsError::internal("non-UTF-8 db path"))?
            .to_string();
        let conn = DbConnection::connect(DbBackend::Sqlite { path: path_str }).await?;
        Migrator::run(conn.inner()).await?;
        Ok(Self { conn })
    }

    /// Open an in-memory SQLite database. Primarily for tests.
    pub async fn open_memory() -> Result<Self, FsError> {
        let conn = DbConnection::connect(DbBackend::SqliteMemory).await?;
        Migrator::run(conn.inner()).await?;
        Ok(Self { conn })
    }

    /// Default FSN SQLite path: `~/.local/share/fsn/fsn.db`.
    pub fn default_path() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{home}/.local/share/fsn/fsn.db")
    }

    /// Open the default FSN SQLite database, creating parent directories as needed.
    pub async fn open_default() -> Result<Self, FsError> {
        let path = Self::default_path();
        let dir = Path::new(&path).parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)
            .map_err(|e| FsError::internal(format!("create db dir: {e}")))?;
        Self::open_sqlite(&path).await
    }

    /// Repository for the `resources` table.
    pub fn resources(&self) -> ResourceRepo<'_> {
        ResourceRepo::new(self.conn.inner())
    }

    /// Repository for the `permissions` table.
    pub fn permissions(&self) -> PermissionRepo<'_> {
        PermissionRepo::new(self.conn.inner())
    }

    /// Append-only repository for the `audit_logs` table.
    pub fn audit(&self) -> AuditRepo<'_> {
        AuditRepo::new(self.conn.inner())
    }

    /// Repository for the `plugins` table.
    pub fn plugins(&self) -> PluginRepo<'_> {
        PluginRepo::new(self.conn.inner())
    }

    /// Repository for the `hosts` table.
    pub fn hosts(&self) -> HostRepo<'_> {
        HostRepo::new(self.conn.inner())
    }

    /// Repository for the `projects` table.
    pub fn projects(&self) -> ProjectRepo<'_> {
        ProjectRepo::new(self.conn.inner())
    }

    /// Repository for the `modules` table.
    pub fn modules(&self) -> ModuleRepo<'_> {
        ModuleRepo::new(self.conn.inner())
    }

    /// Repository for the `service_registry` table.
    pub fn service_registry(&self) -> ServiceRegistryRepo<'_> {
        ServiceRegistryRepo::new(self.conn.inner())
    }

    /// Repository for the `installed_packages` table.
    pub fn installed_packages(&self) -> InstalledPackageRepo<'_> {
        InstalledPackageRepo::new(self.conn.inner())
    }

    /// Access the underlying [`DbConnection`] for custom SeaORM queries.
    pub fn conn(&self) -> &DbConnection {
        &self.conn
    }

    /// Close the database connection pool.
    pub async fn close(self) -> Result<(), FsError> {
        self.conn.close().await
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{CrudRepo, HostStatus, ModuleStatus, ProjectStatus};

    #[tokio::test]
    async fn open_memory_and_migrate() {
        let db = DbManager::open_memory().await.expect("open in-memory db");
        db.close().await.expect("close");
    }

    #[tokio::test]
    async fn resource_crud() {
        let db = DbManager::open_memory().await.unwrap();

        let model = db
            .resources()
            .insert("host", "test-server", None, None, None)
            .await
            .unwrap();
        assert_eq!(model.kind, "host");
        assert_eq!(model.name, "test-server");

        let found = db.resources().find_by_id(model.id).await.unwrap();
        assert!(found.is_some());

        let updated = db
            .resources()
            .update(model.id, "test-server-renamed", None)
            .await
            .unwrap();
        assert_eq!(updated.name, "test-server-renamed");

        db.resources().delete_by_id(model.id).await.unwrap();
        let gone = db.resources().find_by_id(model.id).await.unwrap();
        assert!(gone.is_none());

        db.close().await.unwrap();
    }

    #[tokio::test]
    async fn audit_log_append() {
        let db = DbManager::open_memory().await.unwrap();

        db.audit()
            .log(
                "system",
                "deploy",
                None,
                Some("host".into()),
                None,
                None,
                "ok",
            )
            .await
            .unwrap();

        let entries = db.audit().recent(10).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].actor, "system");
        assert_eq!(entries[0].outcome, "ok");

        db.close().await.unwrap();
    }

    #[tokio::test]
    async fn plugin_upsert() {
        let db = DbManager::open_memory().await.unwrap();

        let p = db
            .plugins()
            .upsert("zentinel", "0.1.0", "wasm", None, None, None)
            .await
            .unwrap();
        assert_eq!(p.name, "zentinel");
        assert!(p.enabled);

        // Second upsert updates version.
        let p2 = db
            .plugins()
            .upsert("zentinel", "0.2.0", "wasm", None, None, None)
            .await
            .unwrap();
        assert_eq!(p2.version, "0.2.0");

        db.close().await.unwrap();
    }

    #[tokio::test]
    async fn host_crud() {
        let db = DbManager::open_memory().await.unwrap();

        let h = db
            .hosts()
            .insert("web-01", "web-01.example.com", "10.0.0.1", 22, None)
            .await
            .unwrap();
        assert_eq!(h.name, "web-01");
        assert_eq!(h.status, "unknown");

        db.hosts()
            .update_status(h.id, HostStatus::Online)
            .await
            .unwrap();
        let updated = db.hosts().find_by_id(h.id).await.unwrap().unwrap();
        assert_eq!(updated.status, "online");

        let all = db.hosts().find_all().await.unwrap();
        assert_eq!(all.len(), 1);

        db.hosts().delete_by_id(h.id).await.unwrap();
        assert!(db.hosts().find_by_id(h.id).await.unwrap().is_none());

        db.close().await.unwrap();
    }

    #[tokio::test]
    async fn project_crud() {
        let db = DbManager::open_memory().await.unwrap();

        let p = db
            .projects()
            .insert("acme", Some("acme.example.com".into()), None)
            .await
            .unwrap();
        assert_eq!(p.name, "acme");
        assert_eq!(p.status, "draft");

        let updated = db
            .projects()
            .update(
                p.id,
                "acme",
                Some("acme.example.com".into()),
                None,
                ProjectStatus::Active,
            )
            .await
            .unwrap();
        assert_eq!(updated.status, "active");

        let all = db.projects().find_all().await.unwrap();
        assert_eq!(all.len(), 1);

        db.projects().delete_by_id(p.id).await.unwrap();
        assert!(db.projects().find_by_id(p.id).await.unwrap().is_none());

        db.close().await.unwrap();
    }

    #[tokio::test]
    async fn module_crud() {
        let db = DbManager::open_memory().await.unwrap();

        let h = db
            .hosts()
            .insert("srv-01", "srv-01.local", "192.168.1.1", 22, None)
            .await
            .unwrap();

        let m = db
            .modules()
            .insert(
                "main-proxy",
                "proxy",
                h.id,
                None,
                Some("1.0.0".into()),
                None,
            )
            .await
            .unwrap();
        assert_eq!(m.module_type, "proxy");
        assert_eq!(m.status, "stopped");

        db.modules()
            .update_status(m.id, ModuleStatus::Running)
            .await
            .unwrap();
        let found = db.modules().find_by_id(m.id).await.unwrap().unwrap();
        assert_eq!(found.status, "running");

        let by_host = db.modules().find_by_host(h.id).await.unwrap();
        assert_eq!(by_host.len(), 1);

        db.modules().delete_by_id(m.id).await.unwrap();
        assert!(db.modules().find_by_id(m.id).await.unwrap().is_none());

        db.close().await.unwrap();
    }

    #[tokio::test]
    async fn service_registry_upsert_and_health() {
        let db = DbManager::open_memory().await.unwrap();

        let h = db
            .hosts()
            .insert("srv-02", "srv-02.local", "192.168.1.2", 22, None)
            .await
            .unwrap();
        let m = db
            .modules()
            .insert("kanidm", "iam", h.id, None, None, None)
            .await
            .unwrap();

        let entry = db
            .service_registry()
            .upsert(
                m.id,
                "kanidm",
                r#"["oidc-provider","scim-server"]"#,
                Some("https://iam.example.com".into()),
            )
            .await
            .unwrap();
        assert!(!entry.healthy);

        db.service_registry()
            .set_healthy(entry.id, true)
            .await
            .unwrap();
        let updated = db
            .service_registry()
            .find_by_module(m.id)
            .await
            .unwrap()
            .unwrap();
        assert!(updated.healthy);
        assert!(updated.last_check.is_some());

        let by_cap = db
            .service_registry()
            .find_by_capability("oidc-provider")
            .await
            .unwrap();
        assert_eq!(by_cap.len(), 1);

        db.close().await.unwrap();
    }
}
