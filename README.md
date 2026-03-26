# fs-db

SeaORM-based database abstraction for FreeSynergy.

Part of the [FreeSynergy](https://github.com/FreeSynergy) platform.

## Purpose

Provides connection management, base entity traits, concrete SeaORM entities,
embedded SQL migrations, and a write buffer for high-throughput batched writes.

All FreeSynergy programs that need persistent storage use this crate.

## Features

- `sqlite` (default) — SQLite via `sqlx-sqlite`
- `postgres` — PostgreSQL via `sqlx-postgres`

## Quick Start

```rust
use fs_db::DbManager;

let db = DbManager::open_default().await?;
db.resources().insert("host", "my-server", None, None, None).await?;
db.close().await?;
```

## Architecture

```
DbManager          — top-level handle, owns the connection pool
DbConnection       — thin wrapper around SeaORM DatabaseConnection
Migrator           — sea_orm_migration::MigratorTrait impl (embedded migrations)
FsEntity / Auditable — base traits for all entities
WriteBuffer        — batched-write helper for high-throughput workloads
Repository types   — typed CRUD wrappers per entity
```

## Build

```bash
cargo build
cargo test
```

## Dependencies

- **fs-libs** (`../fs-libs/`) — `fs-error`
- **sea-orm** `=2.0.0-rc.37` with sqlx backend
