use std::sync::Mutex;
use rusqlite::Connection;
use tauri::State;

use crate::adapters::persistence::category_repo;
use crate::domain::entities::Category;
use crate::domain::error::AppError;

#[tauri::command]
pub fn get_categories(
    state: State<'_, Mutex<Connection>>,
) -> Result<Vec<Category>, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    category_repo::get_all_categories(&conn)
}

#[tauri::command]
pub fn create_category(
    state: State<'_, Mutex<Connection>>,
    mut category: Category,
) -> Result<Category, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    if category.category_id.trim().is_empty() {
        category.category_id = uuid::Uuid::now_v7().to_string();
    }

    category_repo::insert_category(&conn, &category)?;
    Ok(category)
}

#[tauri::command]
pub fn update_category(
    state: State<'_, Mutex<Connection>>,
    category: Category,
) -> Result<Category, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    category_repo::update_category(&conn, &category)?;
    Ok(category)
}

#[tauri::command]
pub fn delete_category(
    state: State<'_, Mutex<Connection>>,
    category_id: String,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    category_repo::delete_category(&conn, &category_id)?;
    Ok(())
}
