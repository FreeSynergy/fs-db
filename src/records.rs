// records.rs — DbRecord implementations for all fs-db SeaORM entities.
//
// Each implementation maps the SeaORM Model to the engine-agnostic DbRecord
// trait, enabling use with Repository<T> and EngineRepository<E>.
//
// Column order in `column_names()` MUST match `to_values()`.
// Primary key (`id`) is always excluded from column lists.

use serde_json::Value;

use crate::engine::DbRow;
use crate::entities::{
    audit_log, host, installed_package, module, permission, plugin, project, resource,
    service_registry,
};
use crate::record::{DbRecord, DbRowExt};
use fs_error::FsError;

// ── resource ──────────────────────────────────────────────────────────────────

impl DbRecord for resource::Model {
    fn table_name() -> &'static str {
        "resources"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "kind",
            "name",
            "project_id",
            "parent_id",
            "meta",
            "created_at",
            "updated_at",
        ]
    }

    fn primary_key(&self) -> Option<i64> {
        Some(self.id)
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = id;
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.kind.clone()),
            Value::String(self.name.clone()),
            self.project_id
                .map_or(Value::Null, |v| Value::Number(v.into())),
            self.parent_id
                .map_or(Value::Null, |v| Value::Number(v.into())),
            self.meta.clone().map_or(Value::Null, Value::String),
            Value::Number(self.created_at.into()),
            Value::Number(self.updated_at.into()),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: row.get_i64("id")?,
            kind: row.get_string("kind")?,
            name: row.get_string("name")?,
            project_id: row.get_opt_i64("project_id")?,
            parent_id: row.get_opt_i64("parent_id")?,
            meta: row.get_opt_string("meta")?,
            created_at: row.get_i64("created_at")?,
            updated_at: row.get_i64("updated_at")?,
        })
    }
}

// ── host ──────────────────────────────────────────────────────────────────────

impl DbRecord for host::Model {
    fn table_name() -> &'static str {
        "hosts"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "name",
            "fqdn",
            "ip_address",
            "ssh_port",
            "status",
            "os",
            "architecture",
            "agent_version",
            "project_id",
            "joined_at",
            "updated_at",
        ]
    }

    fn primary_key(&self) -> Option<i64> {
        Some(self.id)
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = id;
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.name.clone()),
            Value::String(self.fqdn.clone()),
            Value::String(self.ip_address.clone()),
            Value::Number(self.ssh_port.into()),
            Value::String(self.status.clone()),
            self.os.clone().map_or(Value::Null, Value::String),
            self.architecture.clone().map_or(Value::Null, Value::String),
            self.agent_version
                .clone()
                .map_or(Value::Null, Value::String),
            self.project_id
                .map_or(Value::Null, |v| Value::Number(v.into())),
            Value::Number(self.joined_at.into()),
            Value::Number(self.updated_at.into()),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: row.get_i64("id")?,
            name: row.get_string("name")?,
            fqdn: row.get_string("fqdn")?,
            ip_address: row.get_string("ip_address")?,
            ssh_port: i32::try_from(row.get_i64("ssh_port")?)
                .map_err(|_| FsError::internal("ssh_port out of i32 range"))?,
            status: row.get_string("status")?,
            os: row.get_opt_string("os")?,
            architecture: row.get_opt_string("architecture")?,
            agent_version: row.get_opt_string("agent_version")?,
            project_id: row.get_opt_i64("project_id")?,
            joined_at: row.get_i64("joined_at")?,
            updated_at: row.get_i64("updated_at")?,
        })
    }
}

// ── audit_log ─────────────────────────────────────────────────────────────────

impl DbRecord for audit_log::Model {
    fn table_name() -> &'static str {
        "audit_log"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "actor",
            "action",
            "resource_id",
            "resource_kind",
            "payload",
            "source",
            "outcome",
            "created_at",
        ]
    }

    fn primary_key(&self) -> Option<i64> {
        Some(self.id)
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = id;
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.actor.clone()),
            Value::String(self.action.clone()),
            self.resource_id
                .map_or(Value::Null, |v| Value::Number(v.into())),
            self.resource_kind
                .clone()
                .map_or(Value::Null, Value::String),
            self.payload.clone().map_or(Value::Null, Value::String),
            self.source.clone().map_or(Value::Null, Value::String),
            Value::String(self.outcome.clone()),
            Value::Number(self.created_at.into()),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: row.get_i64("id")?,
            actor: row.get_string("actor")?,
            action: row.get_string("action")?,
            resource_id: row.get_opt_i64("resource_id")?,
            resource_kind: row.get_opt_string("resource_kind")?,
            payload: row.get_opt_string("payload")?,
            source: row.get_opt_string("source")?,
            outcome: row.get_string("outcome")?,
            created_at: row.get_i64("created_at")?,
        })
    }
}

// ── installed_package ─────────────────────────────────────────────────────────

impl DbRecord for installed_package::Model {
    fn table_name() -> &'static str {
        "installed_packages"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "package_id",
            "version",
            "channel",
            "package_type",
            "active",
            "signature",
            "trust_unsigned",
            "installed_at",
            "updated_at",
        ]
    }

    fn primary_key(&self) -> Option<i64> {
        Some(self.id)
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = id;
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.package_id.clone()),
            Value::String(self.version.clone()),
            Value::String(self.channel.clone()),
            Value::String(self.package_type.clone()),
            Value::Bool(self.active),
            self.signature.clone().map_or(Value::Null, Value::String),
            Value::Bool(self.trust_unsigned),
            Value::Number(self.installed_at.into()),
            Value::Number(self.updated_at.into()),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: row.get_i64("id")?,
            package_id: row.get_string("package_id")?,
            version: row.get_string("version")?,
            channel: row.get_string("channel")?,
            package_type: row.get_string("package_type")?,
            active: row.get_bool("active")?,
            signature: row.get_opt_string("signature")?,
            trust_unsigned: row.get_bool("trust_unsigned")?,
            installed_at: row.get_i64("installed_at")?,
            updated_at: row.get_i64("updated_at")?,
        })
    }
}

// ── permission ────────────────────────────────────────────────────────────────

impl DbRecord for permission::Model {
    fn table_name() -> &'static str {
        "permissions"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "subject",
            "action",
            "resource_id",
            "granted_at",
            "expires_at",
        ]
    }

    fn primary_key(&self) -> Option<i64> {
        Some(self.id)
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = id;
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.subject.clone()),
            Value::String(self.action.clone()),
            self.resource_id
                .map_or(Value::Null, |v| Value::Number(v.into())),
            Value::Number(self.granted_at.into()),
            self.expires_at
                .map_or(Value::Null, |v| Value::Number(v.into())),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: row.get_i64("id")?,
            subject: row.get_string("subject")?,
            action: row.get_string("action")?,
            resource_id: row.get_opt_i64("resource_id")?,
            granted_at: row.get_i64("granted_at")?,
            expires_at: row.get_opt_i64("expires_at")?,
        })
    }
}

// ── plugin ────────────────────────────────────────────────────────────────────

impl DbRecord for plugin::Model {
    fn table_name() -> &'static str {
        "plugins"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "name",
            "version",
            "kind",
            "wasm_hash",
            "path",
            "enabled",
            "meta",
            "installed_at",
        ]
    }

    fn primary_key(&self) -> Option<i64> {
        Some(self.id)
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = id;
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.name.clone()),
            Value::String(self.version.clone()),
            Value::String(self.kind.clone()),
            self.wasm_hash.clone().map_or(Value::Null, Value::String),
            self.path.clone().map_or(Value::Null, Value::String),
            Value::Bool(self.enabled),
            self.meta.clone().map_or(Value::Null, Value::String),
            Value::Number(self.installed_at.into()),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: row.get_i64("id")?,
            name: row.get_string("name")?,
            version: row.get_string("version")?,
            kind: row.get_string("kind")?,
            wasm_hash: row.get_opt_string("wasm_hash")?,
            path: row.get_opt_string("path")?,
            enabled: row.get_bool("enabled")?,
            meta: row.get_opt_string("meta")?,
            installed_at: row.get_i64("installed_at")?,
        })
    }
}

// ── project ───────────────────────────────────────────────────────────────────

impl DbRecord for project::Model {
    fn table_name() -> &'static str {
        "projects"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "name",
            "domain",
            "description",
            "status",
            "created_at",
            "updated_at",
        ]
    }

    fn primary_key(&self) -> Option<i64> {
        Some(self.id)
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = id;
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.name.clone()),
            self.domain.clone().map_or(Value::Null, Value::String),
            self.description.clone().map_or(Value::Null, Value::String),
            Value::String(self.status.clone()),
            Value::Number(self.created_at.into()),
            Value::Number(self.updated_at.into()),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: row.get_i64("id")?,
            name: row.get_string("name")?,
            domain: row.get_opt_string("domain")?,
            description: row.get_opt_string("description")?,
            status: row.get_string("status")?,
            created_at: row.get_i64("created_at")?,
            updated_at: row.get_i64("updated_at")?,
        })
    }
}

// ── module ────────────────────────────────────────────────────────────────────

impl DbRecord for module::Model {
    fn table_name() -> &'static str {
        "modules"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "name",
            "module_type",
            "host_id",
            "project_id",
            "status",
            "version",
            "config",
            "created_at",
            "updated_at",
        ]
    }

    fn primary_key(&self) -> Option<i64> {
        Some(self.id)
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = id;
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.name.clone()),
            Value::String(self.module_type.clone()),
            Value::Number(self.host_id.into()),
            self.project_id
                .map_or(Value::Null, |v| Value::Number(v.into())),
            Value::String(self.status.clone()),
            self.version.clone().map_or(Value::Null, Value::String),
            self.config.clone().map_or(Value::Null, Value::String),
            Value::Number(self.created_at.into()),
            Value::Number(self.updated_at.into()),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: row.get_i64("id")?,
            name: row.get_string("name")?,
            module_type: row.get_string("module_type")?,
            host_id: row.get_i64("host_id")?,
            project_id: row.get_opt_i64("project_id")?,
            status: row.get_string("status")?,
            version: row.get_opt_string("version")?,
            config: row.get_opt_string("config")?,
            created_at: row.get_i64("created_at")?,
            updated_at: row.get_i64("updated_at")?,
        })
    }
}

// ── service_registry ──────────────────────────────────────────────────────────

impl DbRecord for service_registry::Model {
    fn table_name() -> &'static str {
        "service_registry"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "module_id",
            "module_name",
            "capabilities",
            "endpoint_url",
            "healthy",
            "last_check",
        ]
    }

    fn primary_key(&self) -> Option<i64> {
        Some(self.id)
    }

    fn set_primary_key(&mut self, id: i64) {
        self.id = id;
    }

    fn to_values(&self) -> Vec<Value> {
        vec![
            Value::Number(self.module_id.into()),
            Value::String(self.module_name.clone()),
            Value::String(self.capabilities.clone()),
            self.endpoint_url.clone().map_or(Value::Null, Value::String),
            Value::Bool(self.healthy),
            self.last_check
                .map_or(Value::Null, |v| Value::Number(v.into())),
        ]
    }

    fn from_row(row: &DbRow) -> Result<Self, FsError> {
        Ok(Self {
            id: row.get_i64("id")?,
            module_id: row.get_i64("module_id")?,
            module_name: row.get_string("module_name")?,
            capabilities: row.get_string("capabilities")?,
            endpoint_url: row.get_opt_string("endpoint_url")?,
            healthy: row.get_bool("healthy")?,
            last_check: row.get_opt_i64("last_check")?,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_column_count_matches_values() {
        let m = resource::Model {
            id: 1,
            kind: "host".into(),
            name: "srv".into(),
            project_id: None,
            parent_id: None,
            meta: None,
            created_at: 0,
            updated_at: 0,
        };
        assert_eq!(resource::Model::column_names().len(), m.to_values().len());
    }

    #[test]
    fn host_column_count_matches_values() {
        let m = host::Model {
            id: 1,
            name: "h".into(),
            fqdn: "h.example.com".into(),
            ip_address: "10.0.0.1".into(),
            ssh_port: 22,
            status: "online".into(),
            os: None,
            architecture: None,
            agent_version: None,
            project_id: None,
            joined_at: 0,
            updated_at: 0,
        };
        assert_eq!(host::Model::column_names().len(), m.to_values().len());
    }

    #[test]
    fn installed_package_column_count_matches_values() {
        let m = installed_package::Model {
            id: 1,
            package_id: "kanidm".into(),
            version: "1.0".into(),
            channel: "stable".into(),
            package_type: "container".into(),
            active: true,
            signature: None,
            trust_unsigned: false,
            installed_at: 0,
            updated_at: 0,
        };
        assert_eq!(
            installed_package::Model::column_names().len(),
            m.to_values().len()
        );
    }

    #[test]
    fn permission_column_count_matches_values() {
        let m = permission::Model {
            id: 1,
            subject: "user:1".into(),
            action: "read".into(),
            resource_id: None,
            granted_at: 0,
            expires_at: None,
        };
        assert_eq!(permission::Model::column_names().len(), m.to_values().len());
    }

    #[test]
    fn plugin_column_count_matches_values() {
        let m = plugin::Model {
            id: 1,
            name: "plugin".into(),
            version: "0.1".into(),
            kind: "wasm".into(),
            wasm_hash: None,
            path: None,
            enabled: true,
            meta: None,
            installed_at: 0,
        };
        assert_eq!(plugin::Model::column_names().len(), m.to_values().len());
    }

    #[test]
    fn project_column_count_matches_values() {
        let m = project::Model {
            id: 1,
            name: "proj".into(),
            domain: None,
            description: None,
            status: "active".into(),
            created_at: 0,
            updated_at: 0,
        };
        assert_eq!(project::Model::column_names().len(), m.to_values().len());
    }

    #[test]
    fn module_column_count_matches_values() {
        let m = module::Model {
            id: 1,
            name: "mod".into(),
            module_type: "service".into(),
            host_id: 2,
            project_id: None,
            status: "running".into(),
            version: None,
            config: None,
            created_at: 0,
            updated_at: 0,
        };
        assert_eq!(module::Model::column_names().len(), m.to_values().len());
    }

    #[test]
    fn service_registry_column_count_matches_values() {
        let m = service_registry::Model {
            id: 1,
            module_id: 2,
            module_name: "kanidm".into(),
            capabilities: "iam.oidc".into(),
            endpoint_url: None,
            healthy: true,
            last_check: None,
        };
        assert_eq!(
            service_registry::Model::column_names().len(),
            m.to_values().len()
        );
    }

    #[test]
    fn audit_log_column_count_matches_values() {
        let m = audit_log::Model {
            id: 1,
            actor: "user:1".into(),
            action: "login".into(),
            resource_id: None,
            resource_kind: None,
            payload: None,
            source: None,
            outcome: "ok".into(),
            created_at: 0,
        };
        assert_eq!(audit_log::Model::column_names().len(), m.to_values().len());
    }
}
