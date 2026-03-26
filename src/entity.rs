// Base entity traits for FreeSynergy database models.

// ── FsEntity ─────────────────────────────────────────────────────────────────

/// Marker trait for all FreeSynergy database entities.
///
/// Combine with SeaORM's `EntityTrait` when defining concrete entity types.
/// The `entity_name()` method is used in logs and error messages.
pub trait FsEntity {
    /// Human-readable name for this entity type (e.g. `"host"`, `"module"`).
    fn entity_name() -> &'static str;
}

// ── Auditable ─────────────────────────────────────────────────────────────────

/// Trait for models that carry standard audit timestamps.
///
/// Implementing types must store `created_at` and `updated_at` as Unix
/// seconds (i64). These fields are set by the application, not the DB trigger.
pub trait Auditable {
    /// Creation timestamp as Unix seconds.
    fn created_at(&self) -> i64;

    /// Last-update timestamp as Unix seconds.
    fn updated_at(&self) -> i64;

    /// `true` if the record was created within the last `seconds`.
    fn is_recent(&self, seconds: i64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        now - self.created_at() < seconds
    }
}
