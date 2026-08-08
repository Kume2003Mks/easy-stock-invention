use rusqlite::Connection;

use crate::domain::entities::Category;
use crate::domain::error::AppError;

pub fn get_all_categories(conn: &Connection) -> Result<Vec<Category>, AppError> {
    let mut stmt = conn.prepare("SELECT category_id, name FROM Categories ORDER BY category_id ASC")?;

    let rows = stmt.query_map([], |row| {
        Ok(Category {
            category_id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn insert_category(conn: &Connection, category: &Category) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO Categories (category_id, name) VALUES (?1, ?2)",
        rusqlite::params![category.category_id, category.name],
    )?;
    Ok(())
}

pub fn delete_category(conn: &Connection, category_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM Categories WHERE category_id = ?1",
        rusqlite::params![category_id],
    )?;
    Ok(())
}

pub fn update_category(conn: &Connection, category: &Category) -> Result<(), AppError> {
    let rows_affected = conn.execute(
        "UPDATE Categories SET name = ?1 WHERE category_id = ?2",
        rusqlite::params![category.name, category.category_id],
    )?;

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!(
            "ไม่พบหมวดหมู่รหัส {}",
            category.category_id
        )));
    }

    Ok(())
}

pub fn count_categories(conn: &Connection) -> Result<i64, AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM Categories", [], |row| row.get(0))?;
    Ok(count)
}
