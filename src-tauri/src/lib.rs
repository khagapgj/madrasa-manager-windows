mod commands;
mod db;
mod domain;
mod error;
mod repositories;

use db::Database;
use std::{path::PathBuf,sync::Arc};

pub struct AppState{pub db:Arc<Database>,pub photo_root:PathBuf}

#[cfg(target_os="windows")]
use tauri::Manager;

#[cfg(target_os="windows")]
pub fn run(){tauri::Builder::default().setup(|app|{let data_dir=app.path().app_data_dir()?;let database=Database::open(data_dir.join("database").join("madrasa-manager.sqlite")).map_err(Box::<dyn std::error::Error>::from)?;let photo_root=data_dir.join("photos");std::fs::create_dir_all(&photo_root)?;app.manage(AppState{db:Arc::new(database),photo_root});Ok(())}).invoke_handler(tauri::generate_handler![
 commands::app_initialize,commands::database_health,commands::settings_get,commands::settings_set,
 commands::class_list,commands::class_create,commands::class_rename,commands::class_reorder,commands::class_soft_delete,
 commands::student_next_codes,commands::student_create,commands::student_update,commands::student_get,commands::student_list,commands::student_soft_delete,commands::student_restore,commands::student_change_status,commands::student_bulk_create,commands::student_bulk_soft_delete,commands::student_bulk_move,commands::photo_import
]).run(tauri::generate_context!()).expect("Madrasa Manager failed to start");}

#[cfg(not(target_os="windows"))]
pub fn run(){eprintln!("Madrasa Manager desktop bundle target is Windows.");}
