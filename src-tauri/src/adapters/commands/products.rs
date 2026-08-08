use std::sync::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::adapters::persistence::product_repo;
use crate::domain::entities::{Category, Product, Supplier};
use crate::domain::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductsPageData {
    pub products: Vec<Product>,
    pub categories: Vec<Category>,
    pub suppliers: Vec<Supplier>,
}

#[tauri::command]
pub fn get_products_data(
    state: State<'_, Mutex<Connection>>,
) -> Result<ProductsPageData, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let products = product_repo::get_all_products(&conn)?;
    let categories = product_repo::get_all_categories(&conn)?;
    let suppliers = product_repo::get_all_suppliers(&conn)?;

    Ok(ProductsPageData {
        products,
        categories,
        suppliers,
    })
}

#[tauri::command]
pub fn create_product(
    state: State<'_, Mutex<Connection>>,
    mut product: Product,
) -> Result<Product, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    if product.product_id.trim().is_empty() {
        product.product_id = uuid::Uuid::now_v7().to_string();
    }

    product_repo::insert_product(&conn, &product)?;

    Ok(product)
}

#[tauri::command]
pub fn update_product(
    state: State<'_, Mutex<Connection>>,
    product: Product,
) -> Result<Product, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    product_repo::update_product(&conn, &product)?;

    Ok(product)
}

#[tauri::command]
pub fn adjust_stock(
    state: State<'_, Mutex<Connection>>,
    product_id: String,
    transaction_type: String,
    quantity: i32,
    reference_no: Option<String>,
) -> Result<Product, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let updated = product_repo::adjust_stock(
        &conn,
        &product_id,
        &transaction_type,
        quantity,
        reference_no.as_deref(),
    )?;

    Ok(updated)
}

#[tauri::command]
pub fn delete_product(
    state: State<'_, Mutex<Connection>>,
    product_id: String,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    product_repo::delete_product(&conn, &product_id)?;

    Ok(())
}
