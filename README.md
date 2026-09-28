# Madrasa Manager — Tauri 2 Foundation

Reference UI-preserving offline desktop architecture.

## Phase status

- Phase 1: approved
- Phase 2A Foundation: completed in source; see `docs/PHASE_2A_COMPLETION_REPORT_BN.md`
- Phase 2B: not started

## Verification

```bash
npm run verify:phase2a
python3 tests/test_migration_sql.py
# In a Windows/Tauri development environment:
cd src-tauri
cargo test
cargo check
```

## Reference UI

`reference/madrashamaneger V2.6.reference.html` is read-only and protected by SHA-256 verification. `src/index.html` only contains allowlisted non-visual asset-path/bootstrap integration changes.

## Data architecture rule

New services and Rust backend never use localStorage/IndexedDB. Legacy persistence still present inside the quarantined Reference-derived working UI will be removed/rebound incrementally beginning with the separately approved Student/Class implementation phase; it is not part of the new data layer.
