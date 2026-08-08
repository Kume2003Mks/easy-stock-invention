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
];

/// Insert default settings if the App_Settings table is empty (first launch).
/// Uses a single transaction to ensure all rows are inserted atomically.
pub fn ensure_default_settings(conn: &Connection) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM App_Settings",
        [],
        |row| row.get(0),
    )?;

    if count == 0 {
        let tx = conn.unchecked_transaction()?;
        for (key, value, description) in DEFAULT_SETTINGS {
            tx.execute(
                "INSERT OR IGNORE INTO App_Settings (setting_key, setting_value, description, updated_at)
                 VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
                rusqlite::params![key, value, description],
            )?;
        }
        tx.commit()?;
    }

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