use crate::{domain::{class::{ClassDto,ClassInput},student::{PhotoInput,StudentDto,StudentInput,StudentListRequest,StudentPage}},error::AppResult,repositories::{class::ClassRepository,photo::PhotoRepository,settings::{SettingsRepository,SqliteSettingsRepository},student::StudentRepository},AppState};
use serde::Serialize;
use serde_json::Value;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct InitializationResult{database_ready:bool,schema_version:i64,storage_engine:&'static str}
#[tauri::command] pub fn app_initialize(state:State<'_,AppState>)->AppResult<InitializationResult>{state.db.health_check()?;Ok(InitializationResult{database_ready:true,schema_version:state.db.schema_version(),storage_engine:"sqlite"})}
#[tauri::command] pub fn database_health(state:State<'_,AppState>)->AppResult<Value>{Ok(serde_json::json!({"status":state.db.health_check()?,"schemaVersion":state.db.schema_version()}))}
#[tauri::command] pub fn settings_get(state:State<'_,AppState>,key:String)->AppResult<Option<Value>>{SqliteSettingsRepository{db:&state.db}.get(&key)}
#[tauri::command] pub fn settings_set(state:State<'_,AppState>,key:String,value:Value)->AppResult<()>{if key.trim().is_empty(){return Err(crate::error::AppError::Validation("Setting key প্রয়োজন।".into()))}SqliteSettingsRepository{db:&state.db}.set(&key,&value)}

#[tauri::command] pub fn class_list(state:State<'_,AppState>,include_deleted:bool)->AppResult<Vec<ClassDto>>{ClassRepository{db:&state.db}.list(include_deleted)}
#[tauri::command] pub fn class_create(state:State<'_,AppState>,input:ClassInput)->AppResult<ClassDto>{ClassRepository{db:&state.db}.create(input)}
#[tauri::command] pub fn class_rename(state:State<'_,AppState>,id:String,name:String)->AppResult<ClassDto>{ClassRepository{db:&state.db}.rename(&id,&name)}
#[tauri::command] pub fn class_reorder(state:State<'_,AppState>,ordered_ids:Vec<String>)->AppResult<()>{ClassRepository{db:&state.db}.reorder(&ordered_ids)}
#[tauri::command] pub fn class_soft_delete(state:State<'_,AppState>,id:String)->AppResult<()>{ClassRepository{db:&state.db}.soft_delete(&id)}

fn absolute_photos(state:&AppState,mut student:StudentDto)->StudentDto{if let Some(relative)=student.photo_path.take(){let rel=relative.strip_prefix("photos/").unwrap_or(&relative);student.photo_path=Some(state.photo_root.join(rel).to_string_lossy().into_owned());}student}
#[tauri::command] pub fn student_next_codes(state:State<'_,AppState>,class_id:Option<String>,class_name:Option<String>,digits:i64)->AppResult<Value>{let(roll,student_code)=StudentRepository{db:&state.db}.next_codes(class_id.as_deref(),class_name.as_deref(),digits)?;Ok(serde_json::json!({"roll":roll,"studentCode":student_code}))}
#[tauri::command] pub fn student_create(state:State<'_,AppState>,input:StudentInput)->AppResult<StudentDto>{Ok(absolute_photos(&state,StudentRepository{db:&state.db}.create(input)?))}
#[tauri::command] pub fn student_update(state:State<'_,AppState>,id:String,input:StudentInput,expected_updated_at:Option<String>)->AppResult<StudentDto>{Ok(absolute_photos(&state,StudentRepository{db:&state.db}.update(&id,input,expected_updated_at.as_deref())?))}
#[tauri::command] pub fn student_get(state:State<'_,AppState>,id:String)->AppResult<Option<StudentDto>>{Ok(StudentRepository{db:&state.db}.get(&id)?.map(|s|absolute_photos(&state,s)))}
#[tauri::command] pub fn student_list(state:State<'_,AppState>,request:StudentListRequest)->AppResult<StudentPage>{let mut page=StudentRepository{db:&state.db}.list(&request)?;page.items=page.items.into_iter().map(|s|absolute_photos(&state,s)).collect();Ok(page)}
#[tauri::command] pub fn student_soft_delete(state:State<'_,AppState>,id:String)->AppResult<()>{StudentRepository{db:&state.db}.soft_delete(&id)}
#[tauri::command] pub fn student_restore(state:State<'_,AppState>,id:String)->AppResult<()>{StudentRepository{db:&state.db}.restore(&id)}
#[tauri::command] pub fn student_change_status(state:State<'_,AppState>,id:String,status:String)->AppResult<StudentDto>{Ok(absolute_photos(&state,StudentRepository{db:&state.db}.change_status(&id,&status)?))}
#[tauri::command] pub fn student_bulk_create(state:State<'_,AppState>,inputs:Vec<StudentInput>)->AppResult<u64>{StudentRepository{db:&state.db}.bulk_create(inputs)}
#[tauri::command] pub fn student_bulk_soft_delete(state:State<'_,AppState>,student_ids:Vec<String>)->AppResult<u64>{StudentRepository{db:&state.db}.bulk_soft_delete(&student_ids)}
#[tauri::command] pub fn student_bulk_move(state:State<'_,AppState>,student_ids:Vec<String>,class_id:String)->AppResult<u64>{StudentRepository{db:&state.db}.bulk_move(&student_ids,&class_id)}
#[tauri::command] pub fn photo_import(state:State<'_,AppState>,input:PhotoInput)->AppResult<String>{PhotoRepository{db:&state.db,root:&state.photo_root}.import(&input.student_id,&input.file_name,&input.mime_type,&input.bytes)}
