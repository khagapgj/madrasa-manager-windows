use crate::{db::Database, error::AppResult};
use rusqlite::params;
use serde_json::Value;

pub trait SettingsRepository {
    fn get(&self, key: &str) -> AppResult<Option<Value>>;
    fn set(&self, key: &str, value: &Value) -> AppResult<()>;
}

pub struct SqliteSettingsRepository<'a> { pub db: &'a Database }

impl SettingsRepository for SqliteSettingsRepository<'_> {
    fn get(&self, key: &str) -> AppResult<Option<Value>> {
        self.db.read(|conn| {
            let mut stmt = conn.prepare("SELECT value_json FROM app_settings WHERE setting_key=?1")?;
            let mut rows = stmt.query([key])?;
            match rows.next()? {
                Some(row) => {
                    let raw: String = row.get(0)?;
                    Ok(serde_json::from_str(&raw).ok())
                }
                None => Ok(None),
            }
        })
    }

    fn set(&self, key: &str, value: &Value) -> AppResult<()> {
        let raw = serde_json::to_string(value).map_err(|e| crate::error::AppError::Validation(e.to_string()))?;
        self.db.transaction(|tx| {
            tx.execute(
                "INSERT INTO app_settings(setting_key,value_json,updated_at)
                 VALUES (?1,?2,strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT(setting_key) DO UPDATE SET value_json=excluded.value_json,updated_at=excluded.updated_at",
                params![key, raw]
            )?;
            Ok(())
        })
    }
}
