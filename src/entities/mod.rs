pub mod audit_log;
pub mod host;
pub mod installed_package;
pub mod module;
pub mod permission;
pub mod plugin;
pub mod project;
/// SeaORM entity definitions for the FreeSynergy core schema.
///
/// # Entities
/// | module               | table name           | description                                  |
/// |----------------------|----------------------|----------------------------------------------|
/// | [`resource`]         | `resources`          | Generic managed resource (host, service, …)  |
/// | [`permission`]       | `permissions`        | RBAC permission grants                       |
/// | [`sync_state`]       | `sync_states`        | CRDT vector-clock per resource               |
/// | [`plugin`]           | `plugins`            | Plugin registry entries                      |
/// | [`audit_log`]        | `audit_logs`         | Immutable audit trail                        |
/// | [`host`]             | `hosts`              | Physical/virtual host machines               |
/// | [`project`]          | `projects`           | Logical deployment projects                  |
/// | [`module`]           | `modules`            | Deployed service instances                   |
/// | [`service_registry`] | `service_registry`   | Module capabilities and health state         |
/// | [`installed_package`] | `installed_packages` | Installed package version tracking          |
pub mod resource;
pub mod service_registry;
pub mod sync_state;
