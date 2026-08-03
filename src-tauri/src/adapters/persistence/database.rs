use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

use crate::domain::error::AppError;

pub fn init_db(app_data_dir: PathBuf) -> Result<Connection, AppError> {
    // Ensure the app data directory exists
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).map_err(|e| AppError::Internal(e.to_string()))?;
    }

    let db_path = app_data_dir.join("database.sqlite");
    
    // Connect to SQLite
    let conn = Connection::open(db_path)?;

    // Optimize SQLite for speed and SSD preservation
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA temp_store = MEMORY;
        PRAGMA foreign_keys = ON;
        ",
    )?;

    // Run schema migration
    let schema = include_str!("schema.sql");
    conn.execute_batch(schema)?;

    Ok(conn)
}
