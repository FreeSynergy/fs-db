/// CRUD repositories for FreeSynergy database entities.
///
/// Each repository wraps a reference to a [`sea_orm::DatabaseConnection`] and
/// provides typed async methods for the corresponding table.
use std::fmt;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, Order,
    QueryFilter, QueryOrder, QuerySelect,
};

use crate::entities::{
    audit_log, host, installed_package, module, permission, plugin, project, resource,
    service_registry,
};
use fs_error::FsError;

// ── helpers ───────────────────────────────────────────────────────────────────

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

// ── CrudRepo trait ────────────────────────────────────────────────────────────

/// Common CRUD operations shared by every database repository.
///
/// All concrete repositories in this module implement this trait.
/// Use it for generic code that operates across repository types.
///
/// # Example
///
/// ```rust,ignore
/// async fn remove_any<R: CrudRepo>(repo: &R, id: i64) -> Result<(), FsError> {
///     repo.delete_by_id(id).await
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait CrudRepo {
    /// The SeaORM model type this repository operates on.
    type Model: Send;

    /// Find a record by its primary key. Returns `None` if not found.
    async fn find_by_id(&self, id: i64) -> Result<Option<Self::Model>, FsError>;

    /// Delete a record by its primary key.
    async fn delete_by_id(&self, id: i64) -> Result<(), FsError>;
}

// ── Status types ──────────────────────────────────────────────────────────────

/// Operational status of a managed host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HostStatus {
    /// Status not yet known — assigned on registration.
    #[default]
    Unknown,
    /// Host is reachable and healthy.
    Online,
    /// Host is unreachable.
    Offline,
    /// Host is reachable but reports degraded health.
    Degraded,
}

impl HostStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Degraded => "degraded",
        }
    }
}

impl fmt::Display for HostStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Lifecycle status of a deployed module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleStatus {
    /// Module process is not running — initial state after registration.
    #[default]
    Stopped,
    /// Module is in the process of starting up.
    Starting,
    /// Module is fully running and healthy.
    Running,
    /// Module is shutting down gracefully.
    Stopping,
    /// Module exited with an error.
    Failed,
}

impl ModuleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Failed => "failed",
        }
    }
}

impl fmt::Display for ModuleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Lifecycle status of a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectStatus {
    /// Project is being set up — initial state.
    #[default]
    Draft,
    /// Project is live and actively managed.
    Active,
    /// Project has been archived and is read-only.
    Archived,
}

impl ProjectStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── ResourceRepo ──────────────────────────────────────────────────────────────

/// Repository for the `resources` table.
pub struct ResourceRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> ResourceRepo<'a> {
    /// Create a new repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new resource. Returns the model with its generated `id`.
    pub async fn insert(
        &self,
        kind: impl Into<String>,
        name: impl Into<String>,
        project_id: Option<i64>,
        parent_id: Option<i64>,
        meta: Option<String>,
    ) -> Result<resource::Model, FsError> {
        let now = unix_now();
        let active = resource::ActiveModel {
            kind: Set(kind.into()),
            name: Set(name.into()),
            project_id: Set(project_id),
            parent_id: Set(parent_id),
            meta: Set(meta),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ResourceRepo::insert: {e}")))
    }

    /// Find a resource by its primary key.
    pub async fn find_by_id(&self, id: i64) -> Result<Option<resource::Model>, FsError> {
        resource::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ResourceRepo::find_by_id: {e}")))
    }

    /// Delete a resource by its primary key.
    pub async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        resource::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ResourceRepo::delete_by_id: {e}")))?;
        Ok(())
    }

    /// List all resources of a given kind.
    pub async fn find_by_kind(&self, kind: &str) -> Result<Vec<resource::Model>, FsError> {
        resource::Entity::find()
            .filter(resource::Column::Kind.eq(kind))
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ResourceRepo::find_by_kind: {e}")))
    }

    /// List all resources.
    pub async fn find_all(&self) -> Result<Vec<resource::Model>, FsError> {
        resource::Entity::find()
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ResourceRepo::find_all: {e}")))
    }

    /// Update the `name` and `meta` of an existing resource.
    pub async fn update(
        &self,
        id: i64,
        name: impl Into<String>,
        meta: Option<String>,
    ) -> Result<resource::Model, FsError> {
        let active = resource::ActiveModel {
            id: Set(id),
            name: Set(name.into()),
            meta: Set(meta),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ResourceRepo::update: {e}")))
    }
}

impl CrudRepo for ResourceRepo<'_> {
    type Model = resource::Model;

    async fn find_by_id(&self, id: i64) -> Result<Option<resource::Model>, FsError> {
        resource::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ResourceRepo::find_by_id: {e}")))
    }

    async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        resource::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("ResourceRepo::delete_by_id: {e}")))
    }
}

// ── PermissionRepo ────────────────────────────────────────────────────────────

/// Repository for the `permissions` table.
pub struct PermissionRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> PermissionRepo<'a> {
    /// Create a new permission repository.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Grant a permission to a subject for an action, optionally scoped to a resource.
    pub async fn grant(
        &self,
        subject: impl Into<String>,
        action: impl Into<String>,
        resource_id: Option<i64>,
        expires_at: Option<i64>,
    ) -> Result<permission::Model, FsError> {
        let active = permission::ActiveModel {
            subject: Set(subject.into()),
            action: Set(action.into()),
            resource_id: Set(resource_id),
            granted_at: Set(unix_now()),
            expires_at: Set(expires_at),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("PermissionRepo::grant: {e}")))
    }

    /// List all non-expired permissions for a subject.
    pub async fn find_for_subject(&self, subject: &str) -> Result<Vec<permission::Model>, FsError> {
        let now = unix_now();
        permission::Entity::find()
            .filter(permission::Column::Subject.eq(subject))
            .filter(
                sea_orm::Condition::any()
                    .add(permission::Column::ExpiresAt.is_null())
                    .add(permission::Column::ExpiresAt.gt(now)),
            )
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("PermissionRepo::find_for_subject: {e}")))
    }

    /// Revoke a specific permission by its primary key.
    pub async fn revoke(&self, id: i64) -> Result<(), FsError> {
        permission::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("PermissionRepo::revoke: {e}")))
    }
}

impl CrudRepo for PermissionRepo<'_> {
    type Model = permission::Model;

    async fn find_by_id(&self, id: i64) -> Result<Option<permission::Model>, FsError> {
        permission::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("PermissionRepo::find_by_id: {e}")))
    }

    async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        self.revoke(id).await
    }
}

// ── AuditRepo ─────────────────────────────────────────────────────────────────

/// Append-only repository for the `audit_logs` table.
pub struct AuditRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> AuditRepo<'a> {
    /// Create a new audit repository.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Append an audit log entry. Returns the inserted row.
    #[allow(clippy::too_many_arguments)]
    pub async fn log(
        &self,
        actor: impl Into<String>,
        action: impl Into<String>,
        resource_id: Option<i64>,
        resource_kind: Option<String>,
        payload: Option<String>,
        source: Option<String>,
        outcome: impl Into<String>,
    ) -> Result<audit_log::Model, FsError> {
        let active = audit_log::ActiveModel {
            actor: Set(actor.into()),
            action: Set(action.into()),
            resource_id: Set(resource_id),
            resource_kind: Set(resource_kind),
            payload: Set(payload),
            source: Set(source),
            outcome: Set(outcome.into()),
            created_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("AuditRepo::log: {e}")))
    }

    /// Return the most recent `limit` audit log entries (newest first).
    pub async fn recent(&self, limit: u64) -> Result<Vec<audit_log::Model>, FsError> {
        audit_log::Entity::find()
            .order_by(audit_log::Column::CreatedAt, Order::Desc)
            .limit(Some(limit))
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("AuditRepo::recent: {e}")))
    }

    /// Return all audit entries for a specific resource.
    pub async fn find_for_resource(
        &self,
        resource_id: i64,
    ) -> Result<Vec<audit_log::Model>, FsError> {
        audit_log::Entity::find()
            .filter(audit_log::Column::ResourceId.eq(resource_id))
            .order_by(audit_log::Column::CreatedAt, Order::Desc)
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("AuditRepo::find_for_resource: {e}")))
    }
}

impl CrudRepo for AuditRepo<'_> {
    type Model = audit_log::Model;

    async fn find_by_id(&self, id: i64) -> Result<Option<audit_log::Model>, FsError> {
        audit_log::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("AuditRepo::find_by_id: {e}")))
    }

    /// Audit logs are immutable — this always returns `Ok(())` without deleting.
    ///
    /// Audit entries must never be removed to preserve the audit trail.
    /// Use archival or retention policies at the database level instead.
    async fn delete_by_id(&self, _id: i64) -> Result<(), FsError> {
        Ok(())
    }
}

// ── PluginRepo ────────────────────────────────────────────────────────────────

/// Repository for the `plugins` table.
pub struct PluginRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> PluginRepo<'a> {
    /// Create a new plugin repository.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Register a new plugin or update its version/path if it already exists.
    pub async fn upsert(
        &self,
        name: impl Into<String>,
        version: impl Into<String>,
        kind: impl Into<String>,
        path: Option<String>,
        wasm_hash: Option<String>,
        meta: Option<String>,
    ) -> Result<plugin::Model, FsError> {
        let name = name.into();
        let existing = plugin::Entity::find()
            .filter(plugin::Column::Name.eq(&name))
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("PluginRepo::upsert find: {e}")))?;

        if let Some(existing) = existing {
            let active = plugin::ActiveModel {
                id: Set(existing.id),
                version: Set(version.into()),
                kind: Set(kind.into()),
                path: Set(path),
                wasm_hash: Set(wasm_hash),
                meta: Set(meta),
                ..Default::default()
            };
            active
                .update(self.conn)
                .await
                .map_err(|e| FsError::internal(format!("PluginRepo::upsert update: {e}")))
        } else {
            let active = plugin::ActiveModel {
                name: Set(name),
                version: Set(version.into()),
                kind: Set(kind.into()),
                path: Set(path),
                wasm_hash: Set(wasm_hash),
                enabled: Set(true),
                meta: Set(meta),
                installed_at: Set(unix_now()),
                ..Default::default()
            };
            active
                .insert(self.conn)
                .await
                .map_err(|e| FsError::internal(format!("PluginRepo::upsert insert: {e}")))
        }
    }

    /// List all enabled plugins.
    pub async fn find_enabled(&self) -> Result<Vec<plugin::Model>, FsError> {
        plugin::Entity::find()
            .filter(plugin::Column::Enabled.eq(true))
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("PluginRepo::find_enabled: {e}")))
    }

    /// Find a plugin by name.
    pub async fn find_by_name(&self, name: &str) -> Result<Option<plugin::Model>, FsError> {
        plugin::Entity::find()
            .filter(plugin::Column::Name.eq(name))
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("PluginRepo::find_by_name: {e}")))
    }

    /// Enable or disable a plugin by its primary key.
    pub async fn set_enabled(&self, id: i64, enabled: bool) -> Result<(), FsError> {
        let active = plugin::ActiveModel {
            id: Set(id),
            enabled: Set(enabled),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("PluginRepo::set_enabled: {e}")))
    }
}

impl CrudRepo for PluginRepo<'_> {
    type Model = plugin::Model;

    async fn find_by_id(&self, id: i64) -> Result<Option<plugin::Model>, FsError> {
        plugin::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("PluginRepo::find_by_id: {e}")))
    }

    async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        plugin::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("PluginRepo::delete_by_id: {e}")))
    }
}

// ── HostRepo ──────────────────────────────────────────────────────────────────

/// Repository for the `hosts` table.
pub struct HostRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> HostRepo<'a> {
    /// Create a new host repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new host record. Returns the inserted model with its generated `id`.
    pub async fn insert(
        &self,
        name: impl Into<String>,
        fqdn: impl Into<String>,
        ip_address: impl Into<String>,
        ssh_port: i32,
        project_id: Option<i64>,
    ) -> Result<host::Model, FsError> {
        let now = unix_now();
        let active = host::ActiveModel {
            name: Set(name.into()),
            fqdn: Set(fqdn.into()),
            ip_address: Set(ip_address.into()),
            ssh_port: Set(ssh_port),
            status: Set(HostStatus::default().to_string()),
            os: Set(None),
            architecture: Set(None),
            agent_version: Set(None),
            project_id: Set(project_id),
            joined_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("HostRepo::insert: {e}")))
    }

    /// Find a host by its primary key.
    pub async fn find_by_id(&self, id: i64) -> Result<Option<host::Model>, FsError> {
        host::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("HostRepo::find_by_id: {e}")))
    }

    /// Delete a host by its primary key.
    pub async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        host::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("HostRepo::delete_by_id: {e}")))?;
        Ok(())
    }

    /// List all hosts.
    pub async fn find_all(&self) -> Result<Vec<host::Model>, FsError> {
        host::Entity::find()
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("HostRepo::find_all: {e}")))
    }

    /// Update the operational status of a host.
    pub async fn update_status(&self, id: i64, status: HostStatus) -> Result<(), FsError> {
        let active = host::ActiveModel {
            id: Set(id),
            status: Set(status.to_string()),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("HostRepo::update_status: {e}")))
    }
}

impl CrudRepo for HostRepo<'_> {
    type Model = host::Model;

    async fn find_by_id(&self, id: i64) -> Result<Option<host::Model>, FsError> {
        host::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("HostRepo::find_by_id: {e}")))
    }

    async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        host::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("HostRepo::delete_by_id: {e}")))
    }
}

// ── ProjectRepo ───────────────────────────────────────────────────────────────

/// Repository for the `projects` table.
pub struct ProjectRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> ProjectRepo<'a> {
    /// Create a new project repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new project. Returns the inserted model with its generated `id`.
    pub async fn insert(
        &self,
        name: impl Into<String>,
        domain: Option<String>,
        description: Option<String>,
    ) -> Result<project::Model, FsError> {
        let now = unix_now();
        let active = project::ActiveModel {
            name: Set(name.into()),
            domain: Set(domain),
            description: Set(description),
            status: Set(ProjectStatus::default().to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ProjectRepo::insert: {e}")))
    }

    /// List all projects.
    pub async fn find_all(&self) -> Result<Vec<project::Model>, FsError> {
        project::Entity::find()
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ProjectRepo::find_all: {e}")))
    }

    /// Update a project's name, domain, description, and status.
    pub async fn update(
        &self,
        id: i64,
        name: impl Into<String>,
        domain: Option<String>,
        description: Option<String>,
        status: ProjectStatus,
    ) -> Result<project::Model, FsError> {
        let active = project::ActiveModel {
            id: Set(id),
            name: Set(name.into()),
            domain: Set(domain),
            description: Set(description),
            status: Set(status.to_string()),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ProjectRepo::update: {e}")))
    }
}

impl CrudRepo for ProjectRepo<'_> {
    type Model = project::Model;

    async fn find_by_id(&self, id: i64) -> Result<Option<project::Model>, FsError> {
        project::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ProjectRepo::find_by_id: {e}")))
    }

    async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        project::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("ProjectRepo::delete_by_id: {e}")))
    }
}

// ── ModuleRepo ────────────────────────────────────────────────────────────────

/// Repository for the `modules` table.
pub struct ModuleRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> ModuleRepo<'a> {
    /// Create a new module repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new module instance. Returns the inserted model with its generated `id`.
    pub async fn insert(
        &self,
        name: impl Into<String>,
        module_type: impl Into<String>,
        host_id: i64,
        project_id: Option<i64>,
        version: Option<String>,
        config: Option<String>,
    ) -> Result<module::Model, FsError> {
        let now = unix_now();
        let active = module::ActiveModel {
            name: Set(name.into()),
            module_type: Set(module_type.into()),
            host_id: Set(host_id),
            project_id: Set(project_id),
            status: Set(ModuleStatus::default().to_string()),
            version: Set(version),
            config: Set(config),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ModuleRepo::insert: {e}")))
    }

    /// List all modules running on a specific host.
    pub async fn find_by_host(&self, host_id: i64) -> Result<Vec<module::Model>, FsError> {
        module::Entity::find()
            .filter(module::Column::HostId.eq(host_id))
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ModuleRepo::find_by_host: {e}")))
    }

    /// List all modules belonging to a specific project.
    pub async fn find_by_project(&self, project_id: i64) -> Result<Vec<module::Model>, FsError> {
        module::Entity::find()
            .filter(module::Column::ProjectId.eq(project_id))
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ModuleRepo::find_by_project: {e}")))
    }

    /// Update the operational status of a module.
    pub async fn update_status(&self, id: i64, status: ModuleStatus) -> Result<(), FsError> {
        let active = module::ActiveModel {
            id: Set(id),
            status: Set(status.to_string()),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("ModuleRepo::update_status: {e}")))
    }
}

impl CrudRepo for ModuleRepo<'_> {
    type Model = module::Model;

    async fn find_by_id(&self, id: i64) -> Result<Option<module::Model>, FsError> {
        module::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ModuleRepo::find_by_id: {e}")))
    }

    async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        module::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("ModuleRepo::delete_by_id: {e}")))
    }
}

// ── InstalledPackageRepo ──────────────────────────────────────────────────────

/// Repository for the `installed_packages` table.
///
/// Tracks all installed package versions, supporting multi-version coexistence
/// and rollback by toggling the `active` flag.
pub struct InstalledPackageRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> InstalledPackageRepo<'a> {
    /// Create a new repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert a new installed package record. Returns the auto-generated `id`.
    ///
    /// The new record is always inserted as `active = true`. Call
    /// [`set_active`](Self::set_active) on any previous version to deactivate it.
    pub async fn insert(
        &self,
        package_id: impl Into<String>,
        version: impl Into<String>,
        channel: impl Into<String>,
        package_type: impl Into<String>,
        signature: Option<String>,
        trust_unsigned: bool,
    ) -> Result<i64, FsError> {
        let now = unix_now();
        let active = installed_package::ActiveModel {
            package_id: Set(package_id.into()),
            version: Set(version.into()),
            channel: Set(channel.into()),
            package_type: Set(package_type.into()),
            active: Set(true),
            signature: Set(signature),
            trust_unsigned: Set(trust_unsigned),
            installed_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active
            .insert(self.conn)
            .await
            .map(|m| m.id)
            .map_err(|e| FsError::internal(format!("InstalledPackageRepo::insert: {e}")))
    }

    /// Find the currently active version for `package_id`.
    pub async fn find_active(
        &self,
        package_id: &str,
    ) -> Result<Option<installed_package::Model>, FsError> {
        installed_package::Entity::find()
            .filter(installed_package::Column::PackageId.eq(package_id))
            .filter(installed_package::Column::Active.eq(true))
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("InstalledPackageRepo::find_active: {e}")))
    }

    /// List every installed package record across all versions.
    pub async fn list_all(&self) -> Result<Vec<installed_package::Model>, FsError> {
        installed_package::Entity::find()
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("InstalledPackageRepo::list_all: {e}")))
    }

    /// Set the `active` flag for a specific record by `id`.
    pub async fn set_active(&self, id: i64, active: bool) -> Result<(), FsError> {
        let model = installed_package::ActiveModel {
            id: Set(id),
            active: Set(active),
            updated_at: Set(unix_now()),
            ..Default::default()
        };
        model
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("InstalledPackageRepo::set_active: {e}")))
    }
}

impl CrudRepo for InstalledPackageRepo<'_> {
    type Model = installed_package::Model;

    async fn find_by_id(&self, id: i64) -> Result<Option<installed_package::Model>, FsError> {
        installed_package::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("InstalledPackageRepo::find_by_id: {e}")))
    }

    async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        installed_package::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("InstalledPackageRepo::delete_by_id: {e}")))
    }
}

// ── ServiceRegistryRepo ───────────────────────────────────────────────────────

/// Repository for the `service_registry` table.
pub struct ServiceRegistryRepo<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> ServiceRegistryRepo<'a> {
    /// Create a new service registry repository backed by `conn`.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Insert or update a service registry entry for `module_id`.
    ///
    /// If a row for `module_id` already exists it is updated; otherwise a new
    /// row is inserted.
    pub async fn upsert(
        &self,
        module_id: i64,
        module_name: impl Into<String>,
        capabilities: impl Into<String>,
        endpoint_url: Option<String>,
    ) -> Result<service_registry::Model, FsError> {
        let existing = service_registry::Entity::find()
            .filter(service_registry::Column::ModuleId.eq(module_id))
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ServiceRegistryRepo::upsert find: {e}")))?;

        if let Some(row) = existing {
            let active = service_registry::ActiveModel {
                id: Set(row.id),
                module_name: Set(module_name.into()),
                capabilities: Set(capabilities.into()),
                endpoint_url: Set(endpoint_url),
                ..Default::default()
            };
            active
                .update(self.conn)
                .await
                .map_err(|e| FsError::internal(format!("ServiceRegistryRepo::upsert update: {e}")))
        } else {
            let active = service_registry::ActiveModel {
                module_id: Set(module_id),
                module_name: Set(module_name.into()),
                capabilities: Set(capabilities.into()),
                endpoint_url: Set(endpoint_url),
                healthy: Set(false),
                last_check: Set(None),
                ..Default::default()
            };
            active
                .insert(self.conn)
                .await
                .map_err(|e| FsError::internal(format!("ServiceRegistryRepo::upsert insert: {e}")))
        }
    }

    /// Find the registry entry for a specific module.
    pub async fn find_by_module(
        &self,
        module_id: i64,
    ) -> Result<Option<service_registry::Model>, FsError> {
        service_registry::Entity::find()
            .filter(service_registry::Column::ModuleId.eq(module_id))
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ServiceRegistryRepo::find_by_module: {e}")))
    }

    /// List all registry entries whose `capabilities` JSON contains `capability`.
    ///
    /// Uses a SQL `LIKE` search — suitable for simple string matching.
    pub async fn find_by_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<service_registry::Model>, FsError> {
        service_registry::Entity::find()
            .filter(service_registry::Column::Capabilities.contains(capability))
            .all(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ServiceRegistryRepo::find_by_capability: {e}")))
    }

    /// Mark a registry entry as healthy or unhealthy, updating `last_check`.
    pub async fn set_healthy(&self, id: i64, healthy: bool) -> Result<(), FsError> {
        let active = service_registry::ActiveModel {
            id: Set(id),
            healthy: Set(healthy),
            last_check: Set(Some(unix_now())),
            ..Default::default()
        };
        active
            .update(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("ServiceRegistryRepo::set_healthy: {e}")))
    }
}

impl CrudRepo for ServiceRegistryRepo<'_> {
    type Model = service_registry::Model;

    async fn find_by_id(&self, id: i64) -> Result<Option<service_registry::Model>, FsError> {
        service_registry::Entity::find_by_id(id)
            .one(self.conn)
            .await
            .map_err(|e| FsError::internal(format!("ServiceRegistryRepo::find_by_id: {e}")))
    }

    async fn delete_by_id(&self, id: i64) -> Result<(), FsError> {
        service_registry::Entity::delete_by_id(id)
            .exec(self.conn)
            .await
            .map(|_| ())
            .map_err(|e| FsError::internal(format!("ServiceRegistryRepo::delete_by_id: {e}")))
    }
}
