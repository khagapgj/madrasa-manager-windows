# Phase 2A — Foundation Completion Report

**Status:** Foundation scope completed; Phase 2B শুরু করা হয়নি।  
**Date:** 2026-09-28

## 1. Reference UI protection

- Immutable copy: `reference/madrashamaneger V2.6.reference.html`
- File mode: read-only (`0444`)
- SHA-256: `901592016ae4011839a25e8e1a924a71d9b7bb79b227eff7bcdd2b7e3a8164f8`
- `scripts/verify-reference.mjs` working UI-কে reference থেকে allowlisted transformation দিয়ে byte-level compare করে।
- Allowed changes only:
  1. remote dependency URL → equivalent pinned local file path
  2. remote placeholder → embedded SVG data URI
  3. one module script (`controllers/bootstrap.js`) before final `</body>`
- HTML body markup, existing CSS rules/classes, Bengali labels, layout, menu, form, modal, table, dashboard ও print design ইচ্ছাকৃতভাবে পরিবর্তন করা হয়নি।

## 2. Tauri 2 scaffold

Created:

- `src-tauri/Cargo.toml`
- `src-tauri/build.rs`
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`
- `src-tauri/src/main.rs`
- `src-tauri/src/lib.rs`

Configuration:

- Tauri 2
- Windows NSIS/MSI target configuration
- app identifier `bd.madrasamanager.desktop`
- minimum capability `core:default`
- CSP restricted to self/local data/blob and Tauri IPC
- global Tauri API enabled for the no-bundler service bridge

## 3. SQLite bootstrap/migrations

Created:

- `src-tauri/migrations/001_initial.sql`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/mod.rs`

Features:

- application data DB path: `database/madrasa-manager.sqlite`
- bundled SQLite through `rusqlite`
- `foreign_keys=ON`
- WAL journal
- `synchronous=FULL`
- busy timeout
- numbered embedded migration list
- SHA-256 migration checksum
- checksum mismatch rejection
- `BEGIN IMMEDIATE` transaction per migration
- foreign-key check before commit
- repeatable migration test included

Migration SQL was executed against SQLite in automated validation: **passed**, foreign-key violations: **0**.

## 4. Repository/transaction/error foundation

Created:

- central `Database` connection owner
- serialized connection access
- `read()` boundary
- `transaction()` with explicit commit/rollback
- `SettingsRepository` trait
- `SqliteSettingsRepository`
- stable serializable Bengali error envelope with code/retry/trace ID

Foundation commands implemented:

- `app_initialize`
- `database_health`
- `settings_get`
- `settings_set`

CRUD commands declared in JS facade but not falsely registered until their Rust domain/repository implementation is added in the relevant implementation phase.

## 5. Offline dependencies

Bundled locally:

- Tailwind CSS browser bundle 3.4.17
- Lucide 0.468.0
- html2canvas 1.4.1
- SortableJS 1.15.0
- SolaimanLipi regular/bold/thin
- Inter 300/400/500/600/700

Checksums: `src/assets/THIRD_PARTY_SHA256SUMS.txt`  
Notices: `docs/THIRD_PARTY_NOTICES.md`

Automated check confirms mandatory UI dependency remote URLs are absent. WhatsApp remains only an optional user-triggered external link; it is not required for offline core features.

## 6. JavaScript Service Facades

Created:

- `src/services/core.js`
- `src/services/facades.js`
- `src/controllers/bootstrap.js`

Facades:

- AppService
- StudentService
- ClassService
- FeeService
- PaymentService
- ExpenseService
- IncomeService
- SettingsService
- PhotoService
- DashboardService
- BackupService
- MigrationService

Contract features:

- only typed command/payload calls
- UUID validation
- page size enforcement (25/50/100; default 50)
- paginated response validation
- normalized `ServiceError`
- mock transport
- global read-only `AppServices`
- `madrasa:services-ready` / `madrasa:services-error` events

## 7. Handler connection boundary

`bootstrap.js` is connected to the existing document without changing its markup/design. It initializes SQLite through `AppService` and exposes the service facades for handler adapters.

**Important phase boundary:** Existing legacy CRUD handler bodies have not been bulk rewritten in Phase 2A. তাদের Student/Class persistence replacement is Phase 2B work. Therefore `src/index.html` still contains quarantined legacy `localStorage`/`ShadowStorage` code copied from the reference; no new service/controller/Rust code uses it. This foundation must not be treated as the final production data runtime until handlers are migrated module-by-module. Removing/rebinding all of that in Phase 2A would effectively start Phase 2B and violate the approval gate.

Production acceptance criterion remains:

- service/controller/Rust runtime: zero localStorage/IndexedDB persistence now
- final migrated UI: legacy persistence calls will be removed/rebound before release
- legacy storage will only be reachable through the later one-time migration tool

## 8. Tests and results

Command: `npm run verify:phase2a`

- Service contract tests: **6 passed, 0 failed**
- immutable reference hash: **passed**
- allowlisted UI transformation diff: **passed**
- offline dependency check: **passed**

Command: `python3 tests/test_migration_sql.py`

- SQL bootstrap/migration: **passed**
- foreign-key check: **passed**

Rust source compilation was not executed in this sandbox because `cargo`/`rustc` are not installed. Rust unit tests and build definitions are present, but a Rust + Tauri system dependency environment must run `cargo test`/`cargo check` before installer acceptance.

## 9. Files changed from Reference UI

Only working copy `src/index.html` differs, with:

- local asset paths replacing CDN paths
- embedded local placeholder
- service bootstrap module inclusion
- newline normalization caused by file processing

Immutable reference itself is unchanged. No design change list exists because no design change was made.

## 10. Phase 2B gate

Not started:

- Student/Class Rust commands/repositories
- handler-by-handler Student/Class binding
- paginated Student SQL queries
- photo persistence integration
- removal of Student/Class legacy array persistence

Explicit approval is required before Phase 2B.
