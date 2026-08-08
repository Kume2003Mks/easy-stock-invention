use std::sync::Mutex;
use rusqlite::Connection;
use tauri::State;

use crate::adapters::persistence::supplier_repo;
use crate::domain::entities::Supplier;
use crate::domain::error::AppError;

#[tauri::command]
pub fn get_suppliers(
    state: State<'_, Mutex<Connection>>,
) -> Result<Vec<Supplier>, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    supplier_repo::get_all_suppliers(&conn)
}

#[tauri::command]
pub fn create_supplier(
    state: State<'_, Mutex<Connection>>,
    mut supplier: Supplier,
) -> Result<Supplier, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    if supplier.supplier_id.trim().is_empty() {
        supplier.supplier_id = uuid::Uuid::now_v7().to_string();
    }

    supplier_repo::insert_supplier(&conn, &supplier)?;
    Ok(supplier)
}

#[tauri::command]
pub fn update_supplier(
    state: State<'_, Mutex<Connection>>,
    supplier: Supplier,
) -> Result<Supplier, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    supplier_repo::update_supplier(&conn, &supplier)?;
    Ok(supplier)
}

#[tauri::command]
pub fn delete_supplier(
    state: State<'_, Mutex<Connection>>,
    supplier_id: String,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    supplier_repo::delete_supplier(&conn, &supplier_id)?;
    Ok(())
}
