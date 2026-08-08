use rusqlite::Connection;

use crate::domain::entities::Supplier;
use crate::domain::error::AppError;

pub fn get_all_suppliers(conn: &Connection) -> Result<Vec<Supplier>, AppError> {
    let mut stmt = conn.prepare("SELECT supplier_id, name, contact_info FROM Suppliers ORDER BY supplier_id ASC")?;

    let rows = stmt.query_map([], |row| {
        Ok(Supplier {
            supplier_id: row.get(0)?,
            name: row.get(1)?,
            contact_info: row.get(2)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn insert_supplier(conn: &Connection, supplier: &Supplier) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO Suppliers (supplier_id, name, contact_info) VALUES (?1, ?2, ?3)",
        rusqlite::params![supplier.supplier_id, supplier.name, supplier.contact_info],
    )?;
    Ok(())
}

pub fn delete_supplier(conn: &Connection, supplier_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM Suppliers WHERE supplier_id = ?1",
        rusqlite::params![supplier_id],
    )?;
    Ok(())
}

pub fn update_supplier(conn: &Connection, supplier: &Supplier) -> Result<(), AppError> {
    let rows_affected = conn.execute(
        "UPDATE Suppliers SET name = ?1, contact_info = ?2 WHERE supplier_id = ?3",
        rusqlite::params![supplier.name, supplier.contact_info, supplier.supplier_id],
    )?;

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!(
            "ไม่พบผู้จัดจำหน่ายรหัส {}",
            supplier.supplier_id
        )));
    }

    Ok(())
}

pub fn count_suppliers(conn: &Connection) -> Result<i64, AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM Suppliers", [], |row| row.get(0))?;
    Ok(count)
}
