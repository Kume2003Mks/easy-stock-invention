use rusqlite::Connection;

use crate::domain::entities::AppSetting;
use crate::domain::error::AppError;

/// Default settings created on first app launch
pub const DEFAULT_SETTINGS: &[(&str, &str, &str)] = &[
    ("store_name", "Easy Stock", "ชื่อร้านค้า"),
    ("store_address", "", "ที่อยู่ร้านค้า"),
    ("store_phone", "", "เบอร์โทรศัพท์ร้านค้า"),
    ("store_email", "", "อีเมลติดต่อร้านค้า"),
    ("currency", "THB", "สกุลเงินที่ใช้"),
    ("low_stock_threshold", "10", "ระดับสต็อกขั้นต่ำสำหรับแจ้งเตือน"),
    ("low_stock_alert", "true", "เปิด/ปิดการแจ้งเตือนสต็อกต่ำ"),
    ("daily_report", "false", "เปิด/ปิดรายงานสรุปประจำวัน"),
    ("allow_out_of_stock_sale", "false", "อนุญาตให้ขายสินค้าได้เมื่อสินค้าหมดสต๊อก"),
    ("auto_print_enabled", "true", "พิมพ์ใบเสร็จอัตโนมัติหลังชำระเงิน"),
    ("receipt_preview_enabled", "true", "แสดงตัวอย่างใบเสร็จก่อนพิมพ์"),
    ("paper_size", "80", "ขนาดกระดาษใบเสร็จ (80/58/57 มม.)"),
    ("printer_connection", "none", "ประเภทการเชื่อมต่อเครื่องพิมพ์ (none/usb/network)"),
    ("printer_target", "", "ที่อยู่เครื่องพิมพ์ เช่น 192.168.1.200:9100 หรือชื่อเครื่องพิมพ์"),
    ("promptpay_id", "", "เลข PromptPay สำหรับ QR บนใบเสร็จ"),
];

/// Insert default settings if they do not exist yet.
/// Uses a single transaction with INSERT OR IGNORE to ensure all defaults exist
/// even when new settings are added to existing databases.
pub fn ensure_default_settings(conn: &Connection) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    for (key, value, description) in DEFAULT_SETTINGS {
        tx.execute(
            "INSERT OR IGNORE INTO App_Settings (setting_key, setting_value, description, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
            rusqlite::params![key, value, description],
        )?;
    }
    tx.commit()?;

    Ok(())
}

/// Read all settings as key-value pairs (String -> String)
pub fn get_all_settings_map(conn: &Connection) -> Result<Vec<AppSetting>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT setting_key, setting_value, description, updated_at FROM App_Settings",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(AppSetting {
            setting_key: row.get(0)?,
            setting_value: row.get(1)?,
            description: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Get a single setting value by key, returns None if not found
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let result = conn.query_row(
        "SELECT setting_value FROM App_Settings WHERE setting_key = ?1",
        rusqlite::params![key],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

/// Insert or update a single setting.
/// Uses UPSERT (ON CONFLICT) so existing rows are updated in-place
/// without deleting/recreating them — preserving `description` when
/// only `value` changes.
pub fn upsert_setting(
    conn: &Connection,
    key: &str,
    value: &str,
    description: Option<&str>,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO App_Settings (setting_key, setting_value, description, updated_at)
         VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
         ON CONFLICT(setting_key) DO UPDATE SET
            setting_value = excluded.setting_value,
            description = COALESCE(excluded.description, App_Settings.description),
            updated_at = CURRENT_TIMESTAMP",
        rusqlite::params![key, value, description],
    )?;
    Ok(())
}