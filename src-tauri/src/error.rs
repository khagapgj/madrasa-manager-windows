use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub field_errors: BTreeMap<String, String>,
    pub trace_id: String,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        let (code, message, retryable) = match self {
            Self::Database(_) => ("DATABASE_ERROR", "ডাটাবেজের কাজ সম্পন্ন করা সম্ভব হয়নি। আবার চেষ্টা করুন।", true),
            Self::Io(_) => ("FILE_ERROR", "ফাইলের কাজ সম্পন্ন করা সম্ভব হয়নি।", true),
            Self::Migration(_) => ("MIGRATION_ERROR", "ডাটাবেজ আপডেট করা সম্ভব হয়নি।", false),
            Self::Validation(message) => ("VALIDATION_ERROR", message.as_str(), false),
            Self::Internal(_) => ("INTERNAL_ERROR", "অপ্রত্যাশিত সমস্যা হয়েছে।", false),
        };
        ErrorEnvelope {
            code: code.into(), message: message.into(), retryable,
            field_errors: BTreeMap::new(), trace_id: uuid::Uuid::new_v4().to_string(),
        }.serialize(serializer)
    }
}

pub type AppResult<T> = Result<T, AppError>;
