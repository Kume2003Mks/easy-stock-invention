use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub category_id: String, // Or Uuid depending on format, but SQL says TEXT
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Supplier {
    pub supplier_id: String,
    pub name: String,
    pub contact_info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub product_id: String,
    pub barcode: Option<String>,
    pub name: String,
    pub category_id: Option<String>,
    pub supplier_id: Option<String>,
    pub cost_price: f64,
    pub selling_price: f64,
    pub wholesale_price: f64,
    pub current_stock: i32,
    pub reorder_level: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockTransaction {
    pub transaction_id: String,
    pub product_id: Option<String>,
    pub transaction_type: String,
    pub quantity: i32,
    pub reference_no: Option<String>,
    pub transaction_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSetting {
    pub setting_key: String,
    pub setting_value: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}