use std::sync::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::adapters::persistence::product_repo;
use crate::domain::entities::{Category, Product, Supplier};
use crate::domain::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProductQueryParams {
    #[serde(alias = "page")]
    pub page: Option<u32>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u32>,
    #[serde(alias = "search")]
    pub search: Option<String>,
    #[serde(alias = "category_id")]
    pub category_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductsPageData {
    pub products: Vec<Product>,
    #[serde(alias = "total_items")]
    pub total_items: u64,
    pub page: u32,
    #[serde(alias = "page_size")]
    pub page_size: u32,
    #[serde(alias = "total_pages")]
    pub total_pages: u32,
    pub categories: Vec<Category>,
    pub suppliers: Vec<Supplier>,
}

#[tauri::command]
pub fn get_products_data(
    state: State<'_, Mutex<Connection>>,
    params: Option<ProductQueryParams>,
) -> Result<ProductsPageData, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let qp = params.unwrap_or_default();
    let filter = product_repo::ProductFilterParams {
        page: qp.page.unwrap_or(1),
        page_size: qp.page_size.unwrap_or(10),
        search: qp.search,
        category_id: qp.category_id,
    };

    let paginated = product_repo::get_products_paginated(&conn, &filter)?;
    let categories = product_repo::get_all_categories(&conn)?;
    let suppliers = product_repo::get_all_suppliers(&conn)?;

    Ok(ProductsPageData {
        products: paginated.products,
        total_items: paginated.total_items,
        page: paginated.page,
        page_size: paginated.page_size,
        total_pages: paginated.total_pages,
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
