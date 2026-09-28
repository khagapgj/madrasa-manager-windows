-- Madrasa Manager / Tauri 2 / SQLite
-- Schema version: 1 (architecture baseline; not yet applied to the Reference UI)
-- IDs are UUID strings generated in Rust. Timestamps are ISO-8601 UTC strings.
-- Monetary values use integer minor units (poisha) to avoid floating-point mismatch.




CREATE TABLE classes (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL COLLATE NOCASE,
    display_order INTEGER NOT NULL DEFAULT 0,
    status        TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_classes_active_name ON classes(name) WHERE deleted_at IS NULL;
CREATE INDEX idx_classes_order ON classes(deleted_at, status, display_order, name);

CREATE TABLE sections (
    id            TEXT PRIMARY KEY,
    class_id      TEXT NOT NULL REFERENCES classes(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    name          TEXT NOT NULL COLLATE NOCASE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_sections_active_name ON sections(class_id, name) WHERE deleted_at IS NULL;
CREATE INDEX idx_sections_class ON sections(class_id, deleted_at, display_order);

CREATE TABLE student_types (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL COLLATE NOCASE,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_system     INTEGER NOT NULL DEFAULT 0 CHECK (is_system IN (0,1)),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_student_types_active_name ON student_types(name) WHERE deleted_at IS NULL;

CREATE TABLE students (
    id                    TEXT PRIMARY KEY,
    student_code          TEXT,
    admission_id          TEXT,
    roll_no               TEXT,
    name                  TEXT NOT NULL,
    father_name           TEXT,
    mother_name           TEXT,
    guardian_name         TEXT,
    guardian_mobile       TEXT,
    mobile                TEXT,
    date_of_birth         TEXT,
    birth_registration_no TEXT,
    gender                TEXT,
    blood_group           TEXT,
    class_id              TEXT NOT NULL REFERENCES classes(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    section_id            TEXT REFERENCES sections(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    student_type_id       TEXT REFERENCES student_types(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    address               TEXT,
    notes                 TEXT,
    status                TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','disabled','inactive')),
    admitted_at           TEXT,
    pro_rate_threshold    INTEGER NOT NULL DEFAULT 10 CHECK (pro_rate_threshold BETWEEN 1 AND 28),
    legacy_uid            TEXT,
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL,
    deleted_at            TEXT
) STRICT;
CREATE UNIQUE INDEX uq_students_code ON students(student_code) WHERE student_code IS NOT NULL AND deleted_at IS NULL;
CREATE UNIQUE INDEX uq_students_admission ON students(admission_id) WHERE admission_id IS NOT NULL AND deleted_at IS NULL;
CREATE UNIQUE INDEX uq_students_legacy_uid ON students(legacy_uid) WHERE legacy_uid IS NOT NULL;
CREATE INDEX idx_students_class_status ON students(class_id, status, deleted_at, roll_no);
CREATE INDEX idx_students_section ON students(section_id, deleted_at);
CREATE INDEX idx_students_type ON students(student_type_id, deleted_at);
CREATE INDEX idx_students_name ON students(name COLLATE NOCASE);
CREATE INDEX idx_students_mobile ON students(mobile);
CREATE INDEX idx_students_updated ON students(updated_at);

CREATE VIRTUAL TABLE students_fts USING fts5(
    student_id UNINDEXED,
    student_code,
    admission_id,
    roll_no,
    name,
    father_name,
    mother_name,
    mobile,
    address,
    tokenize='unicode61 remove_diacritics 0'
);

CREATE TABLE student_status_history (
    id          TEXT PRIMARY KEY,
    student_id  TEXT NOT NULL REFERENCES students(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    status      TEXT NOT NULL CHECK (status IN ('active','disabled','inactive')),
    effective_at TEXT NOT NULL,
    reason      TEXT,
    created_at  TEXT NOT NULL
) STRICT;
CREATE INDEX idx_student_status_history ON student_status_history(student_id, effective_at);

CREATE TABLE custom_field_definitions (
    id            TEXT PRIMARY KEY,
    field_key     TEXT NOT NULL COLLATE NOCASE,
    label         TEXT NOT NULL,
    field_type    TEXT NOT NULL DEFAULT 'text' CHECK (field_type IN ('text','number','date','select','textarea','file')),
    options_json  TEXT CHECK (options_json IS NULL OR json_valid(options_json)),
    display_order INTEGER NOT NULL DEFAULT 0,
    is_visible    INTEGER NOT NULL DEFAULT 1 CHECK (is_visible IN (0,1)),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_custom_field_key ON custom_field_definitions(field_key) WHERE deleted_at IS NULL;

CREATE TABLE student_custom_field_values (
    id          TEXT PRIMARY KEY,
    student_id  TEXT NOT NULL REFERENCES students(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    field_id    TEXT NOT NULL REFERENCES custom_field_definitions(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    value_text  TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE(student_id, field_id)
) STRICT;
CREATE INDEX idx_custom_values_field ON student_custom_field_values(field_id, value_text);

CREATE TABLE student_photos (
    id            TEXT PRIMARY KEY,
    student_id    TEXT NOT NULL REFERENCES students(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    file_name     TEXT NOT NULL,
    relative_path TEXT NOT NULL UNIQUE,
    mime_type     TEXT NOT NULL,
    byte_size     INTEGER NOT NULL CHECK (byte_size >= 0),
    sha256        TEXT NOT NULL,
    width_px      INTEGER,
    height_px     INTEGER,
    is_primary    INTEGER NOT NULL DEFAULT 1 CHECK (is_primary IN (0,1)),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_student_primary_photo ON student_photos(student_id) WHERE is_primary = 1 AND deleted_at IS NULL;
CREATE INDEX idx_student_photos_student ON student_photos(student_id, deleted_at);

CREATE TABLE student_attachments (
    id            TEXT PRIMARY KEY,
    student_id    TEXT NOT NULL REFERENCES students(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    attachment_type TEXT NOT NULL,
    file_name     TEXT NOT NULL,
    relative_path TEXT NOT NULL UNIQUE,
    mime_type     TEXT NOT NULL,
    byte_size     INTEGER NOT NULL CHECK (byte_size >= 0),
    sha256        TEXT NOT NULL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE INDEX idx_student_attachments ON student_attachments(student_id, attachment_type, deleted_at);

CREATE TABLE collectors (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    mobile      TEXT,
    role        TEXT NOT NULL DEFAULT 'collector' CHECK (role IN ('admin','collector')),
    status      TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
) STRICT;

CREATE TABLE fee_types (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL COLLATE NOCASE,
    schedule_type TEXT NOT NULL CHECK (schedule_type IN ('one_time','monthly','weekly','yearly','semester','optional')),
    optional_cycle TEXT CHECK (optional_cycle IS NULL OR optional_cycle IN ('one_time','monthly','weekly','yearly','semester')),
    start_date    TEXT NOT NULL,
    end_date      TEXT,
    due_day       INTEGER CHECK (due_day IS NULL OR due_day BETWEEN 1 AND 31),
    days_before   INTEGER NOT NULL DEFAULT 0 CHECK (days_before >= 0),
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_fee_types_active_name ON fee_types(name) WHERE deleted_at IS NULL;
CREATE INDEX idx_fee_types_schedule ON fee_types(deleted_at, schedule_type, start_date, end_date);

CREATE TABLE fee_rates (
    id              TEXT PRIMARY KEY,
    fee_type_id     TEXT NOT NULL REFERENCES fee_types(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    class_id        TEXT NOT NULL REFERENCES classes(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    student_type_id TEXT REFERENCES student_types(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    amount_minor    INTEGER NOT NULL CHECK (amount_minor >= 0),
    effective_from  TEXT NOT NULL,
    effective_to    TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
) STRICT;
CREATE UNIQUE INDEX uq_fee_rate_default ON fee_rates(fee_type_id, class_id, effective_from)
    WHERE student_type_id IS NULL AND deleted_at IS NULL;
CREATE UNIQUE INDEX uq_fee_rate_by_type ON fee_rates(fee_type_id, class_id, student_type_id, effective_from)
    WHERE student_type_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_fee_rates_lookup ON fee_rates(fee_type_id, class_id, student_type_id, effective_from, effective_to, deleted_at);

CREATE TABLE student_fee_assignments (
    id          TEXT PRIMARY KEY,
    student_id  TEXT NOT NULL REFERENCES students(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    fee_type_id TEXT NOT NULL REFERENCES fee_types(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    assigned_at TEXT NOT NULL,
    detached_at TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
) STRICT;
CREATE UNIQUE INDEX uq_active_fee_assignment ON student_fee_assignments(student_id, fee_type_id)
    WHERE detached_at IS NULL AND deleted_at IS NULL;
CREATE INDEX idx_fee_assignments_fee ON student_fee_assignments(fee_type_id, assigned_at, detached_at);

-- Materialized obligations. The Rust FeeService generates these idempotently.
-- This ledger makes due/search/dashboard queries SQL-based instead of loading all students.
CREATE TABLE fee_charges (
    id              TEXT PRIMARY KEY,
    student_id      TEXT NOT NULL REFERENCES students(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    fee_type_id     TEXT REFERENCES fee_types(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    period_key      TEXT NOT NULL,
    label           TEXT NOT NULL,
    charge_date     TEXT NOT NULL,
    due_date        TEXT,
    amount_minor    INTEGER NOT NULL CHECK (amount_minor >= 0),
    source          TEXT NOT NULL DEFAULT 'schedule' CHECK (source IN ('schedule','instant','migration','adjustment')),
    source_ref      TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
) STRICT;
CREATE UNIQUE INDEX uq_fee_charge_period ON fee_charges(student_id, fee_type_id, period_key)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_fee_charges_student_date ON fee_charges(student_id, charge_date, deleted_at);
CREATE INDEX idx_fee_charges_fee_date ON fee_charges(fee_type_id, charge_date, deleted_at);
CREATE INDEX idx_fee_charges_due_date ON fee_charges(due_date, deleted_at);

CREATE TABLE student_fee_overrides (
    id            TEXT PRIMARY KEY,
    student_id    TEXT NOT NULL REFERENCES students(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    fee_type_id   TEXT NOT NULL REFERENCES fee_types(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    period_key    TEXT NOT NULL,
    amount_minor  INTEGER NOT NULL CHECK (amount_minor >= 0),
    reason        TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_fee_override ON student_fee_overrides(student_id, fee_type_id, period_key) WHERE deleted_at IS NULL;

CREATE TABLE fee_waivers (
    id            TEXT PRIMARY KEY,
    student_id    TEXT NOT NULL REFERENCES students(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    fee_charge_id TEXT NOT NULL REFERENCES fee_charges(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    amount_minor  INTEGER NOT NULL CHECK (amount_minor >= 0),
    reason        TEXT,
    created_by    TEXT REFERENCES collectors(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE INDEX idx_fee_waivers_charge ON fee_waivers(fee_charge_id, deleted_at);
CREATE INDEX idx_fee_waivers_student ON fee_waivers(student_id, deleted_at);

CREATE TABLE fee_payments (
    id                    TEXT PRIMARY KEY,
    receipt_no            TEXT NOT NULL,
    student_id            TEXT NOT NULL REFERENCES students(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    collector_id          TEXT REFERENCES collectors(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    amount_minor          INTEGER NOT NULL CHECK (amount_minor > 0),
    unapplied_amount_minor INTEGER NOT NULL DEFAULT 0 CHECK (unapplied_amount_minor >= 0),
    paid_at               TEXT NOT NULL,
    note                  TEXT,
    legacy_id             TEXT,
    sync_status           TEXT NOT NULL DEFAULT 'local' CHECK (sync_status IN ('local','exported','verified')),
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL,
    deleted_at            TEXT
) STRICT;
CREATE UNIQUE INDEX uq_fee_payment_receipt ON fee_payments(receipt_no) WHERE deleted_at IS NULL;
CREATE INDEX idx_fee_payments_student_date ON fee_payments(student_id, paid_at, deleted_at);
CREATE INDEX idx_fee_payments_collector_date ON fee_payments(collector_id, paid_at, deleted_at);
CREATE INDEX idx_fee_payments_date ON fee_payments(paid_at, deleted_at);

CREATE TABLE payment_allocations (
    id            TEXT PRIMARY KEY,
    payment_id    TEXT NOT NULL REFERENCES fee_payments(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    fee_charge_id TEXT NOT NULL REFERENCES fee_charges(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    amount_minor  INTEGER NOT NULL CHECK (amount_minor > 0),
    created_at    TEXT NOT NULL,
    UNIQUE(payment_id, fee_charge_id)
) STRICT;
CREATE INDEX idx_payment_alloc_charge ON payment_allocations(fee_charge_id);

CREATE TABLE expense_types (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL COLLATE NOCASE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_expense_type_name ON expense_types(name) WHERE deleted_at IS NULL;

CREATE TABLE expenses (
    id            TEXT PRIMARY KEY,
    voucher_no    TEXT NOT NULL,
    expense_type_id TEXT REFERENCES expense_types(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    title         TEXT NOT NULL,
    expense_date  TEXT NOT NULL,
    total_minor   INTEGER NOT NULL CHECK (total_minor >= 0),
    collector_id  TEXT REFERENCES collectors(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    note          TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_expense_voucher ON expenses(voucher_no) WHERE deleted_at IS NULL;
CREATE INDEX idx_expenses_date_type ON expenses(expense_date, expense_type_id, deleted_at);

CREATE TABLE expense_items (
    id          TEXT PRIMARY KEY,
    expense_id  TEXT NOT NULL REFERENCES expenses(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    description TEXT NOT NULL,
    amount_minor INTEGER NOT NULL CHECK (amount_minor > 0),
    line_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL
) STRICT;
CREATE INDEX idx_expense_items_parent ON expense_items(expense_id, line_order);

CREATE TABLE income_types (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL COLLATE NOCASE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_income_type_name ON income_types(name) WHERE deleted_at IS NULL;

CREATE TABLE incomes (
    id            TEXT PRIMARY KEY,
    voucher_no    TEXT NOT NULL,
    income_type_id TEXT REFERENCES income_types(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    title         TEXT NOT NULL,
    income_date   TEXT NOT NULL,
    total_minor   INTEGER NOT NULL CHECK (total_minor >= 0),
    collector_id  TEXT REFERENCES collectors(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    note          TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    deleted_at    TEXT
) STRICT;
CREATE UNIQUE INDEX uq_income_voucher ON incomes(voucher_no) WHERE deleted_at IS NULL;
CREATE INDEX idx_incomes_date_type ON incomes(income_date, income_type_id, deleted_at);

CREATE TABLE income_items (
    id          TEXT PRIMARY KEY,
    income_id   TEXT NOT NULL REFERENCES incomes(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    description TEXT NOT NULL,
    amount_minor INTEGER NOT NULL CHECK (amount_minor > 0),
    line_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL
) STRICT;
CREATE INDEX idx_income_items_parent ON income_items(income_id, line_order);

CREATE TABLE app_settings (
    setting_key TEXT PRIMARY KEY,
    value_json  TEXT NOT NULL CHECK (json_valid(value_json)),
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE form_drafts (
    draft_key   TEXT PRIMARY KEY,
    value_json  TEXT NOT NULL CHECK (json_valid(value_json)),
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE legacy_id_map (
    entity_type TEXT NOT NULL,
    legacy_id   TEXT NOT NULL,
    new_id      TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    PRIMARY KEY(entity_type, legacy_id),
    UNIQUE(entity_type, new_id)
) STRICT;

CREATE TABLE data_import_runs (
    id             TEXT PRIMARY KEY,
    source_type    TEXT NOT NULL CHECK (source_type IN ('legacy_json','legacy_webview','csv','backup_restore')),
    source_name    TEXT,
    source_sha256  TEXT,
    mode           TEXT NOT NULL CHECK (mode IN ('validate','replace','merge')),
    status         TEXT NOT NULL CHECK (status IN ('started','validated','committed','failed','rolled_back')),
    summary_json   TEXT CHECK (summary_json IS NULL OR json_valid(summary_json)),
    error_json     TEXT CHECK (error_json IS NULL OR json_valid(error_json)),
    started_at     TEXT NOT NULL,
    finished_at    TEXT
) STRICT;

CREATE TABLE audit_log (
    id          TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id   TEXT NOT NULL,
    action      TEXT NOT NULL,
    before_json TEXT CHECK (before_json IS NULL OR json_valid(before_json)),
    after_json  TEXT CHECK (after_json IS NULL OR json_valid(after_json)),
    actor_id    TEXT,
    created_at  TEXT NOT NULL
) STRICT;
CREATE INDEX idx_audit_entity ON audit_log(entity_type, entity_id, created_at);

CREATE VIEW v_student_balances AS
SELECT
    s.id AS student_id,
    COALESCE(ch.total_charge_minor, 0) AS total_charge_minor,
    COALESCE(w.total_waiver_minor, 0) AS total_waiver_minor,
    COALESCE(a.total_allocated_minor, 0) AS total_allocated_minor,
    MAX(0, COALESCE(ch.total_charge_minor, 0) - COALESCE(w.total_waiver_minor, 0) - COALESCE(a.total_allocated_minor, 0)) AS due_minor,
    COALESCE(p.total_unapplied_minor, 0) AS advance_minor
FROM students s
LEFT JOIN (
    SELECT student_id, SUM(amount_minor) total_charge_minor
    FROM fee_charges WHERE deleted_at IS NULL GROUP BY student_id
) ch ON ch.student_id = s.id
LEFT JOIN (
    SELECT student_id, SUM(amount_minor) total_waiver_minor
    FROM fee_waivers WHERE deleted_at IS NULL GROUP BY student_id
) w ON w.student_id = s.id
LEFT JOIN (
    SELECT fc.student_id, SUM(pa.amount_minor) total_allocated_minor
    FROM payment_allocations pa
    JOIN fee_payments fp ON fp.id = pa.payment_id AND fp.deleted_at IS NULL
    JOIN fee_charges fc ON fc.id = pa.fee_charge_id AND fc.deleted_at IS NULL
    GROUP BY fc.student_id
) a ON a.student_id = s.id
LEFT JOIN (
    SELECT student_id, SUM(unapplied_amount_minor) total_unapplied_minor
    FROM fee_payments WHERE deleted_at IS NULL GROUP BY student_id
) p ON p.student_id = s.id
WHERE s.deleted_at IS NULL;
