use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassInput { pub name: String }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassDto {
    pub id: String,
    pub name: String,
    pub display_order: i64,
    pub status: String,
}
