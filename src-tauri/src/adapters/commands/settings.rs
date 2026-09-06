use std::sync::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::adapters::persistence::settings_repo;
use crate::domain::error::AppError;

fn default_false() -> String {
    "false".to_string()
}

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
    #[serde(default = "default_false")]
    pub allow_out_of_stock_sale: String,
    pub auto_print_enabled: String,
    pub receipt_preview_enabled: String,
    pub paper_size: String,
    pub printer_connection: String,
    pub printer_target: String,
    pub promptpay_id: String,
    #[serde(default = "default_false")]
    pub promptpay_qr_enabled: String,
    #[serde(default = "default_codepage_str")]
    pub printer_codepage: String,
}

fn default_codepage_str() -> String {
    "26".to_string()
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
            allow_out_of_stock_sale: "false".to_string(),
            auto_print_enabled: "true".to_string(),
            receipt_preview_enabled: "true".to_string(),
            paper_size: "80".to_string(),
            printer_connection: "none".to_string(),
            printer_target: String::new(),
            promptpay_id: String::new(),
            promptpay_qr_enabled: "false".to_string(),
            printer_codepage: "26".to_string(),
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
    if let Some(v) = settings_repo::get_setting(&conn, "allow_out_of_stock_sale")? {
        payload.allow_out_of_stock_sale = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "auto_print_enabled")? {
        payload.auto_print_enabled = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "receipt_preview_enabled")? {
        payload.receipt_preview_enabled = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "paper_size")? {
        payload.paper_size = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "printer_connection")? {
        payload.printer_connection = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "printer_target")? {
        payload.printer_target = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "promptpay_id")? {
        payload.promptpay_id = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "promptpay_qr_enabled")? {
        payload.promptpay_qr_enabled = v;
    }
    if let Some(v) = settings_repo::get_setting(&conn, "printer_codepage")? {
        payload.printer_codepage = v;
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
    settings_repo::upsert_setting(&conn, "allow_out_of_stock_sale", &payload.allow_out_of_stock_sale, Some("อนุญาตให้ขายสินค้าได้เมื่อสินค้าหมดสต๊อก"))?;
    settings_repo::upsert_setting(&conn, "auto_print_enabled", &payload.auto_print_enabled, Some("พิมพ์ใบเสร็จอัตโนมัติหลังชำระเงิน"))?;
    settings_repo::upsert_setting(&conn, "receipt_preview_enabled", &payload.receipt_preview_enabled, Some("แสดงตัวอย่างใบเสร็จก่อนพิมพ์"))?;
    settings_repo::upsert_setting(&conn, "paper_size", &payload.paper_size, Some("ขนาดกระดาษใบเสร็จ (80/58/57 มม.)"))?;
    settings_repo::upsert_setting(&conn, "printer_connection", &payload.printer_connection, Some("ประเภทการเชื่อมต่อเครื่องพิมพ์ (network/usb)"))?;
    settings_repo::upsert_setting(&conn, "printer_target", &payload.printer_target, Some("ที่อยู่เครื่องพิมพ์ เช่น 192.168.1.200:9100 หรือชื่อเครื่องพิมพ์"))?;
    settings_repo::upsert_setting(&conn, "promptpay_id", &payload.promptpay_id, Some("เลข PromptPay สำหรับ QR บนใบเสร็จ"))?;
    settings_repo::upsert_setting(&conn, "promptpay_qr_enabled", &payload.promptpay_qr_enabled, Some("เปิด/ปิดการพิมพ์ QR พร้อมเพย์บนใบเสร็จ"))?;
    settings_repo::upsert_setting(&conn, "printer_codepage", &payload.printer_codepage, Some("ชุดรหัสภาษาไทยสำหรับเครื่องพิมพ์ ESC/POS (26=TIS18, 21=TIS11, 255=CP874, 20=KU42)"))?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemPrintersResponse {
    pub printers: Vec<String>,
    pub default_printer: Option<String>,
}

/// ดึงรายชื่อเครื่องพิมพ์ทั้งหมดที่ติดตั้งในระบบ (Windows/Linux)
#[tauri::command]
pub fn get_system_printers() -> Result<SystemPrintersResponse, AppError> {
    let printers = crate::adapters::printer::get_installed_printers();
    let default_printer = crate::adapters::printer::get_default_printer();
    Ok(SystemPrintersResponse {
        printers,
        default_printer,
    })
}

