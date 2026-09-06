use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

use crate::domain::error::AppError;

/// คอลัมน์ที่ต้องมีครบตาม schema ปัจจุบัน (ใช้ตรวจ legacy table จาก prototype เก่า)
const ORDERS_REQUIRED_COLUMNS: &[&str] = &[
    "order_id",
    "order_no",
    "order_type",
    "status",
    "subtotal",
    "discount_amount",
    "total_amount",
    "payment_method",
    "paid_amount",
    "change_amount",
    "hold_name",
    "note",
    "original_order_id",
    "order_date",
];

const ORDER_ITEMS_REQUIRED_COLUMNS: &[&str] = &[
    "item_id",
    "order_id",
    "product_id",
    "product_name",
    "quantity",
    "unit_price",
    "line_total",
];

const RETURN_ITEMS_REQUIRED_COLUMNS: &[&str] = &[
    "return_item_id",
    "return_order_id",
    "original_order_id",
    "product_id",
    "product_name",
    "quantity",
    "reason",
    "return_date",
];

/// ดึงรายชื่อคอลัมน์ของตาราง (ถ้าไม่มีตารางนี้จะคืน Vec ว่าง)
fn get_table_columns(conn: &Connection, table: &str) -> Result<Vec<String>, AppError> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        rusqlite::params![table],
        |row| row.get(0),
    )?;

    if exists == 0 {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(AppError::from)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Migration: ตรวจว่าตาราง POS (Orders / Order_Items / Return_Items) เดิมที่ค้างจาก
/// prototype เก่ามีโครงสร้างตรงกับ schema ปัจจุบันหรือไม่ หากไม่ตรงจะ DROP ทิ้ง
/// เพื่อให้ schema.sql สร้างใหม่อย่างถูกต้อง (ตารางเก่าจาก prototype ไม่มีข้อมูลจริงให้ย้าย)
fn migrate_legacy_order_tables(conn: &Connection) -> Result<(), AppError> {
    let checks: [(&str, &[&str]); 3] = [
        ("Orders", ORDERS_REQUIRED_COLUMNS),
        ("Order_Items", ORDER_ITEMS_REQUIRED_COLUMNS),
        ("Return_Items", RETURN_ITEMS_REQUIRED_COLUMNS),
    ];

    for (table, required) in checks {
        let columns = get_table_columns(conn, table)?;
        if columns.is_empty() {
            continue; // ยังไม่มีตาราง — schema.sql จะสร้างให้
        }

        let has_missing = required
            .iter()
            .any(|col| !columns.iter().any(|c| c == col));

        if has_missing {
            // Legacy schema — ลบทิ้งทั้งชุดเพื่อสร้างใหม่ให้สอดคล้องกัน
            conn.execute_batch(
                "DROP TABLE IF EXISTS Return_Items;
                 DROP TABLE IF EXISTS Order_Items;
                 DROP TABLE IF EXISTS Orders;",
            )
            .map_err(AppError::from)?;
            break;
        }
    }

    Ok(())
}

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

    // ตรวจและลบ legacy ตาราง POS เก่า (ถ้ามี) ก่อนรัน schema migration
    migrate_legacy_order_tables(&conn)?;

    // Run schema migration
    let schema = include_str!("schema.sql");
    conn.execute_batch(schema)?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_legacy_orders_table() {
        let conn = Connection::open_in_memory().unwrap();
        // จำลองตาราง Orders เก่าจาก prototype ที่มีคอลัมน์ไม่ตรงกับ schema ปัจจุบัน
        conn.execute_batch("CREATE TABLE Orders (order_id TEXT PRIMARY KEY, order_no TEXT);")
            .unwrap();

        // Migration ต้องลบ legacy table ทิ้ง
        migrate_legacy_order_tables(&conn).unwrap();

        // หลัง migration ต้องรัน schema.sql ใหม่ได้โดยไม่พัง (สร้างตาราง + index ครบ)
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema).unwrap();

        let cols = get_table_columns(&conn, "Orders").unwrap();
        assert!(cols.iter().any(|c| c == "order_date"));
        assert!(cols.iter().any(|c| c == "order_type"));
    }

    #[test]
    fn test_current_schema_not_dropped() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("schema.sql")).unwrap();

        // Migration ต้องไม่ลบตารางที่โครงสร้างถูกต้องแล้ว
        migrate_legacy_order_tables(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'Orders'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_fresh_database_has_all_pos_tables() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("schema.sql")).unwrap();

        for table in ["Orders", "Order_Items", "Return_Items"] {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    rusqlite::params![table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "ตาราง {} ต้องถูกสร้าง", table);
        }
    }
}
