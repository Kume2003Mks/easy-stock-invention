use rusqlite::Connection;

use crate::domain::entities::{Category, Product, Supplier};
use crate::domain::error::AppError;

pub fn get_all_products(conn: &Connection) -> Result<Vec<Product>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT product_id, barcode, name, category_id, supplier_id, cost_price, selling_price, wholesale_price, current_stock, reorder_level 
         FROM Products ORDER BY product_id ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Product {
            product_id: row.get(0)?,
            barcode: row.get(1)?,
            name: row.get(2)?,
            category_id: row.get(3)?,
            supplier_id: row.get(4)?,
            cost_price: row.get(5)?,
            selling_price: row.get(6)?,
            wholesale_price: row.get(7)?,
            current_stock: row.get(8)?,
            reorder_level: row.get(9)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_all_categories(conn: &Connection) -> Result<Vec<Category>, AppError> {
    let mut stmt = conn.prepare("SELECT category_id, name FROM Categories ORDER BY name ASC")?;

    let rows = stmt.query_map([], |row| {
        Ok(Category {
            category_id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_all_suppliers(conn: &Connection) -> Result<Vec<Supplier>, AppError> {
    let mut stmt = conn.prepare("SELECT supplier_id, name, contact_info FROM Suppliers ORDER BY name ASC")?;

    let rows = stmt.query_map([], |row| {
        Ok(Supplier {
            supplier_id: row.get(0)?,
            name: row.get(1)?,
            contact_info: row.get(2)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn insert_product(conn: &Connection, product: &Product) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO Products (product_id, barcode, name, category_id, supplier_id, cost_price, selling_price, wholesale_price, current_stock, reorder_level)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            product.product_id,
            product.barcode,
            product.name,
            product.category_id,
            product.supplier_id,
            product.cost_price,
            product.selling_price,
            product.wholesale_price,
            product.current_stock,
            product.reorder_level,
        ],
    )?;
    Ok(())
}

pub fn delete_product(conn: &Connection, product_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM Products WHERE product_id = ?1",
        rusqlite::params![product_id],
    )?;
    Ok(())
}

pub fn count_products(conn: &Connection) -> Result<i64, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM Products",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}
