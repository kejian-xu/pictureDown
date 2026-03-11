// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

pub mod commands;
pub mod models;

pub use commands::run;
pub use models::{Post, Posts};
