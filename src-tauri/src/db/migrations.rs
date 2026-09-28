use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection, TransactionBehavior};
use sha2::{Digest, Sha256};

struct Migration { version: i64, name: &'static str, sql: &'static str }

const MIGRATIONS: &[Migration] = &[
    Migration { version: 1, name: "001_initial", sql: include_str!("../../migrations/001_initial.sql") },
];

pub fn run(conn: &mut Connection) -> AppResult<i64> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
          version INTEGER PRIMARY KEY,
          name TEXT NOT NULL UNIQUE,
          checksum TEXT NOT NULL,
          applied_at TEXT NOT NULL
        ) STRICT;"
    )?;

    for migration in MIGRATIONS {
        let checksum = format!("{:x}", Sha256::digest(migration.sql.as_bytes()));
        let existing = conn.query_row(
            "SELECT checksum FROM schema_migrations WHERE version=?1",
            [migration.version], |row| row.get::<_, String>(0)
        );
        match existing {
            Ok(value) if value == checksum => continue,
            Ok(_) => return Err(AppError::Migration(format!("checksum mismatch at version {}", migration.version))),
            Err(rusqlite::Error::QueryReturnedNoRows) => {}
            Err(error) => return Err(error.into()),
        }

        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,name,checksum,applied_at)
             VALUES (?1,?2,?3,strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            params![migration.version, migration.name, checksum]
        )?;
        let violations: i64 = tx.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| r.get(0))?;
        if violations != 0 { return Err(AppError::Migration(format!("{} foreign key violations", violations))); }
        tx.commit()?;
    }

    Ok(MIGRATIONS.last().map(|m| m.version).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_is_repeatable_and_valid() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        assert_eq!(run(&mut conn).unwrap(), 1);
        assert_eq!(run(&mut conn).unwrap(), 1);
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
