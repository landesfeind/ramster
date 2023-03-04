// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


mod error;
pub use error::*;
pub mod models;
use models::*;
mod db;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


#[tokio::main]
async fn main() {
		let db = db::connect().await.expect("Cannot connect to database");

    tauri::Builder::default()
				.manage(db)
        .invoke_handler(tauri::generate_handler![
				    greet
				  ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
