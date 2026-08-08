use std::sync::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::adapters::persistence::settings_repo;
use crate::domain::error::AppError;

/// Payload for reading/writing all app settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingsPayload {
    pub store_name: String,
    pub store_address: String,
    pub store_phone: String,
    pub store_email: String,
    pub currency: String,
    pub low_stock_threshold: String,
    pub low_stock_alert: String,
    pub daily_report: String,
}

impl Default for SettingsPayload {
    fn default() -> Self {
        Self {
            store_name: "Easy Stock".to_string(),
            store_address: String::new(),
            store_phone: String::new(),
            store_email: String::new(),
            currency: "THB".to_string(),
            low_stock_threshold: "10".to_string(),
            low_stock_alert: "true".to_string(),
            daily_report: "false".to_string(),
        }
    }
}

/// Read all settings from the database.
/// Ensures default settings exist on first launch.
#[tauri::command]
pub fn get_settings(state: State<'_, Mutex<Connection>>) -> Result<SettingsPayload, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    // Create default settings if this is the first launch
    settings_repo::ensure_default_settings(&conn)?;

    let mut payload = SettingsPayload::default();

    // Read each known key from the DB
    if let Some(v) = settings_repo::get_setting(&conn, "store_name")? {
        payload.store_name = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "store_address")? {
        payload.store_address = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "store_phone")? {
        payload.store_phone = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "store_email")? {
        payload.store_email = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "currency")? {
        payload.currency = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "low_stock_threshold")? {
        payload.low_stock_threshold = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "low_stock_alert")? {
        payload.low_stock_alert = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "daily_report")? {
        payload.daily_report = v;
    }

    Ok(payload)
}

/// Save all settings to the database.
#[tauri::command]
pub fn save_settings(
    state: State<'_, Mutex<Connection>>,
    payload: SettingsPayload,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    settings_repo::upsert_setting(&conn, "store_name", &payload.store_name, Some("ชื่อร้านค้า"))?;
    settings_repo::upsert_setting(&conn, "store_address", &payload.store_address, Some("ที่อยู่ร้านค้า"))?;
    settings_repo::upsert_setting(&conn, "store_phone", &payload.store_phone, Some("เบอร์โทรศัพท์ร้านค้า"))?;
    settings_repo::upsert_setting(&conn, "store_email", &payload.store_email, Some("อีเมลติดต่อร้านค้า"))?;
    settings_repo::upsert_setting(&conn, "currency", &payload.currency, Some("สกุลเงินที่ใช้"))?;
    settings_repo::upsert_setting(&conn, "low_stock_threshold", &payload.low_stock_threshold, Some("ระดับสต็อกขั้นต่ำสำหรับแจ้งเตือน"))?;
    settings_repo::upsert_setting(&conn, "low_stock_alert", &payload.low_stock_alert, Some("เปิด/ปิดการแจ้งเตือนสต็อกต่ำ"))?;
    settings_repo::upsert_setting(&conn, "daily_report", &payload.daily_report, Some("เปิด/ปิดรายงานสรุปประจำวัน"))?;

    Ok(())
}
