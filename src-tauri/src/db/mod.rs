pub mod migrations;

use crate::error::AppResult;
use parking_lot::Mutex;
use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::path::{Path, PathBuf};

pub struct Database {
    connection: Mutex<Connection>,
    path: PathBuf,
    schema_version: i64,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> AppResult<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        let mut conn = Connection::open(&path)?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;
             PRAGMA journal_mode=WAL;
             PRAGMA synchronous=FULL;
             PRAGMA busy_timeout=5000;
             PRAGMA trusted_schema=OFF;"
        )?;
        let schema_version = migrations::run(&mut conn)?;
        Ok(Self { connection: Mutex::new(conn), path, schema_version })
    }

    pub fn schema_version(&self) -> i64 { self.schema_version }
    pub fn path(&self) -> &Path { &self.path }

    pub fn read<T>(&self, operation: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.connection.lock();
        operation(&conn)
    }

    pub fn transaction<T>(&self, operation: impl FnOnce(&Transaction<'_>) -> AppResult<T>) -> AppResult<T> {
        let mut conn = self.connection.lock();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        match operation(&tx) {
            Ok(value) => { tx.commit()?; Ok(value) }
            Err(error) => { let _ = tx.rollback(); Err(error) }
        }
    }

    pub fn health_check(&self) -> AppResult<String> {
        self.read(|conn| conn.query_row("PRAGMA quick_check", [], |row| row.get(0)).map_err(Into::into))
    }
}
