# CLAUDE.md – fs-db

## What is this?

FreeSynergy DB abstraction — SeaORM over SQLite/PostgreSQL, used by all FS programs.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

## Quality Gates (before every commit)

```
1. Design Pattern (Traits, Object hierarchy)
2. Structs + Traits — no impl code yet
3. cargo check
4. Impl (OOP)
5. cargo clippy --all-targets -- -D warnings
6. cargo fmt --check
7. Unit tests (min. 1 per public module)
8. cargo test
9. commit + push
```

Every lib.rs / main.rs must have:
```rust
#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings)]
```

## Architecture

- `DbManager` — top-level handle, owns the connection pool
- `DbConnection` — thin SeaORM wrapper
- `Migrator` — embedded SQL migrations
- `FsEntity` / `Auditable` — base entity traits
- `WriteBuffer` — batched writes
- Repository types — typed CRUD per entity

## Dependencies

- `fs-error` from `../fs-libs/`
- `sea-orm =2.0.0-rc.37`
