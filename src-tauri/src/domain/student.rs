use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentInput {
    pub student_code: Option<String>,
    pub admission_id: Option<String>,
    pub roll_no: Option<String>,
    pub name: String,
    pub father_name: Option<String>,
    pub mother_name: Option<String>,
    pub mobile: Option<String>,
    pub class_id: Option<String>,
    pub class_name: Option<String>,
    pub student_type_id: Option<String>,
    pub student_type_name: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub custom_fields: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentListRequest {
    pub query: Option<String>,
    pub class_id: Option<String>,
    pub class_name: Option<String>,
    pub status: Option<String>,
    pub sort_key: Option<String>,
    pub sort_direction: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentDto {
    pub uid: String,
    pub id: Option<String>,
    pub admission_id: Option<String>,
    pub roll: Option<String>,
    pub name: String,
    pub fathers_name: Option<String>,
    pub mothers_name: Option<String>,
    pub mobile: Option<String>,
    pub class_id: String,
    pub class: String,
    pub student_type_id: Option<String>,
    #[serde(rename = "type")]
    pub student_type: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub custom_fields: BTreeMap<String, String>,
    pub photo_path: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentPage {
    pub items: Vec<StudentDto>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub has_next: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoInput {
    pub student_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}
