# Phase 2B — Student/Class SQLite Backend Completion Report

**Status:** Student/Class scope implemented. Phase 2C শুরু করা হয়নি।  
**Reference UI design changes:** none.

## 1. Implemented architecture

```text
Existing Student/Class UI handlers
        ↓ (Phase 2B compatibility controller)
StudentService / ClassService / PhotoService
        ↓ Tauri invoke
Rust commands
        ↓
Repositories + BEGIN IMMEDIATE transactions
        ↓
SQLite + permanent photo directory
```

- UI-তে existing button, form, modal, menu, table/card markup ও workflow রাখা হয়েছে।
- `students` array এখন database নয়; এটি সর্বোচ্চ loaded paginated rows-এর UI cache।
- `classes` string array এখন SQLite class rows থেকে তৈরি compatibility lookup; canonical relationship UUID/FK।

## 2. Changed/created files

### Created

- `src/controllers/phase2b-adapter.js`
- `src-tauri/src/domain/mod.rs`
- `src-tauri/src/domain/student.rs`
- `src-tauri/src/domain/class.rs`
- `src-tauri/src/repositories/student.rs`
- `src-tauri/src/repositories/class.rs`
- `src-tauri/src/repositories/photo.rs`
- `tests/test_student_class_sql.py`
- `backend-core-check/Cargo.toml`
- `backend-core-check/Cargo.lock`
- `backend-core-check/lib.rs`
- `docs/PHASE_2B_COMPLETION_REPORT_BN.md`

### Modified

- `src/index.html`
- `src/services/facades.js`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/repositories/mod.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `tests/contracts/service-contracts.test.js`
- `scripts/verify-reference.mjs`

Immutable `reference/madrashamaneger V2.6.reference.html` পরিবর্তন করা হয়নি।

## 3. Removed/rebound old Student/Class storage calls

Reference HTML-এ Student/Class-এর explicit legacy persistence call পাওয়া গিয়েছিল **17টি**। Phase 2B working UI-তে একই audit pattern-এর ফল **0টি**।

Removed/rebound categories:

- initial `localStorage.getItem(STORAGE_KEYS.students)`
- initial `localStorage.getItem(STORAGE_KEYS.classes)`
- startup ShadowStorage restore-এর Student/Class rehydration
- `saveData()` → SQLite page refresh/service operation
- `saveClasses()` → `ClassService.list()` refresh
- `ShadowStorage.save(STORAGE_KEYS.students, ...)`
- `ShadowStorage.save(STORAGE_KEYS.classes, ...)`
- direct `localStorage.setItem('cc_students', ...)`
- direct `localStorage.setItem(STORAGE_KEYS.students, ...)`
- startup text-based `syncClasses()` canonicalization

Additional safety:

- `ShadowStorage.save()` Student/Class keys reject/ignore করে।
- Shadow restore/migrate startup loops থেকে Student/Class keys বাদ।
- নতুন `services`, `controllers`, Rust backend-এ `localStorage`, `indexedDB`, `ShadowStorage` usage নেই।

Legacy storage অন্য module-এর জন্য এখনো Phase 2C+ scope অনুযায়ী থাকতে পারে; Student/Class-এর primary/runtime persistence নয়।

## 4. SQLite tables used

### Direct Student/Class runtime

- `classes`
- `students`
- `student_types`
- `student_status_history`
- `custom_field_definitions`
- `student_custom_field_values`
- `student_photos`
- `students_fts`
- `schema_migrations`

### Schema-enforced relationships

- `students.class_id → classes.id`
- `students.student_type_id → student_types.id`
- status/custom/photo rows → `students.id`
- IDs are Rust-generated UUIDs.
- class hard deletion is blocked by FK/reference policy; UI delete uses soft delete and backend student-count validation।

`sections` table schema-তে প্রস্তুত, কিন্তু বর্তমান Reference UI-তে separate section CRUD workflow না থাকায় Phase 2B handler-এ ব্যবহার করা হয়নি।

## 5. Backend commands

### Class

- `class_list`
- `class_create`
- `class_rename`
- `class_reorder`
- `class_soft_delete`

### Student

- `student_next_codes`
- `student_create`
- `student_update`
- `student_get`
- `student_list`
- `student_soft_delete`
- `student_restore`
- `student_change_status`
- `student_bulk_create`
- `student_bulk_soft_delete`
- `student_bulk_move`

### Photo

- `photo_import`

## 6. Existing functions rebound to backend

| Existing UI function/workflow | New operation |
|---|---|
| startup Student list | `StudentService.list` |
| `renderStudents` data source | SQL paginated query |
| `getFilteredStudents` data source | already-filtered SQLite page cache |
| search inputs | debounced SQL/FTS query |
| `setFilter` | class UUID SQL filter |
| `calculateAutoValues` | `student_next_codes` SQL aggregate |
| `handleFormSubmit` | `student_create` / `student_update` |
| `deleteStudent` | `student_soft_delete` |
| `deleteSelectedStudents` | `student_bulk_soft_delete` transaction |
| `toggleStudentStatus` | `student_change_status` transaction |
| `moveSelectedStudents` | `student_bulk_move` transaction |
| spreadsheet preview `finalizeImport` | `student_bulk_create` single transaction |
| photo input in Student form | permanent `photo_import` |
| `handlePhotoSync` | SQLite student search + permanent file import |
| class initial/filter data | `class_list` |
| `confirmAddClass` | `class_create` |
| `editClass` / `renameClass` | `class_rename` |
| `deleteClass` / `deleteCurrentClass` | `class_soft_delete` |
| `saveClassOrder` | `class_reorder` transaction |

Pure UI functions—modal open/close, card HTML, button style, selection visuals, print—অপরিবর্তিত রাখা হয়েছে।

## 7. Search, filter and pagination

- default page size: **50**
- maximum API page size: **100**
- pagination query: SQL `LIMIT/OFFSET`
- existing UI-তে নতুন visible pagination control যোগ করা হয়নি। Design অপরিবর্তিত রাখতে invisible 1px IntersectionObserver sentinel দিয়ে lazy next-page load হয়।
- class filter UUID দিয়ে।
- status filter SQL predicate দিয়ে।
- core text search `students_fts MATCH` দিয়ে।
- custom-field search SQLite `EXISTS` query দিয়ে।
- sorting backend whitelist: student code, roll, name, created date।
- UI-তে পুরো database load করা হয় না।

## 8. Transaction and integrity behaviour

All multi-row operations `BEGIN IMMEDIATE` transaction wrapper ব্যবহার করে:

- student create + initial status + custom fields + FTS row
- student update + status transition + custom field replacement + FTS sync
- bulk import all-or-nothing
- bulk delete all-or-nothing
- bulk class move all-or-nothing
- class reorder all-or-nothing
- photo metadata replacement transaction

Photo handling:

- accepted: JPEG/PNG/WEBP
- MIME + magic signature validation
- limit: 10 MB
- SHA-256 stored
- file name replaced with UUID-based safe name
- path: `AppData/photos/students/<student_uuid>/<photo_uuid>.<ext>`
- database stores relative path/metadata only
- DB failure হলে staged/final file cleanup হয়; orphan file রাখা হয় না
- existing primary photo metadata retained with `is_primary=0`; new photo primary হয়

## 9. Tests

### JavaScript service contracts

- **8 passed, 0 failed**
- pagination contract
- UUID validation
- error normalization
- Class command mapping
- Student status/bulk command mapping
- Settings isolation
- facade inventory

### Rust backend-core

- `cargo check`: passed for database/domain/repository core
- `cargo test`: **2 passed, 0 failed**
  - migration repeatability/checksum path
  - Student/Class create, read, update, FTS search, pagination, status, soft-delete/restore, custom field and permanent photo

### SQLite/Python

- `SQL_MIGRATION_OK`
- `STUDENT_CLASS_SQL_OK`
- FK class-delete protection: passed
- partial Student/status transaction rollback: passed
- FTS lookup: passed
- paginated query: passed

### Full Tauri Linux check note

Full desktop `cargo check` in this Linux sandbox reaches Tauri native dependency resolution but cannot finish because system package `gdk-3.0` is absent. এটি Rust Student/Class core error নয়। Backend core is compiled/tested independently. Final Windows Tauri build must still run on the Windows CI/build environment.

## 10. UI reference comparison

Result:

```text
REFERENCE_OK: immutable hash, static DOM/CSS and class inventory verified
```

Verified:

- immutable reference hash unchanged
- static DOM unchanged
- all existing CSS unchanged
- existing class-attribute inventory unchanged
- local dependency replacement remains offline
- no new visible pagination/button/modal/layout element

Reference SHA-256:

```text
901592016ae4011839a25e8e1a924a71d9b7bb79b227eff7bcdd2b7e3a8164f8
```

## 11. Phase boundary

Phase 2C শুরু করা হয়নি। Fee/Payment legacy calculation/persistence এই phase-এ SQLite ledger-এ migrate করা হয়নি। Student list এখন paginated হওয়ায় legacy finance functionsকে full Student array-এর ওপর নির্ভর করা যাবে না; Phase 2C-তে সেগুলো SQL-backed Fee/Payment services-এ বদলাতে হবে।
