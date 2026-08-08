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

pub fn update_product(conn: &Connection, product: &Product) -> Result<(), AppError> {
    conn.execute(
        "UPDATE Products 
         SET barcode = ?1, name = ?2, category_id = ?3, supplier_id = ?4,
             cost_price = ?5, selling_price = ?6, wholesale_price = ?7,
             reorder_level = ?8
         WHERE product_id = ?9",
        rusqlite::params![
            product.barcode,
            product.name,
            product.category_id,
            product.supplier_id,
            product.cost_price,
            product.selling_price,
            product.wholesale_price,
            product.reorder_level,
            product.product_id,
        ],
    )?;
    Ok(())
}

pub fn adjust_stock(
    conn: &Connection,
    product_id: &str,
    transaction_type: &str,
    quantity: i32,
    reference_no: Option<&str>,
) -> Result<Product, AppError> {
    let mut stmt = conn.prepare(
        "SELECT product_id, barcode, name, category_id, supplier_id, cost_price, selling_price, wholesale_price, current_stock, reorder_level 
         FROM Products WHERE product_id = ?1",
    )?;

    let mut product = stmt.query_row(rusqlite::params![product_id], |row| {
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

    let old_stock = product.current_stock;
    let (new_stock, diff_qty) = match transaction_type {
        "IN" => {
            if quantity <= 0 {
                return Err(AppError::Validation("จำนวนรับเข้าต้องมากกว่า 0".into()));
            }
            (old_stock + quantity, quantity)
        }
        "OUT" => {
            if quantity <= 0 {
                return Err(AppError::Validation("จำนวนจ่ายออกต้องมากกว่า 0".into()));
            }
            let n = old_stock - quantity;
            if n < 0 {
                return Err(AppError::Validation("จำนวนสต็อกไม่เพียงพอต่อการจ่ายออก".into()));
            }
            (n, -quantity)
        }
        "ADJUST" => {
            if quantity < 0 {
                return Err(AppError::Validation("จำนวนสต็อกต้องไม่ติดลบ".into()));
            }
            let diff = quantity - old_stock;
            (quantity, diff)
        }
        _ => return Err(AppError::Validation("ประเภทการปรับปรุงไม่ถูกต้อง".into())),
    };

    // Update product stock
    conn.execute(
        "UPDATE Products SET current_stock = ?1 WHERE product_id = ?2",
        rusqlite::params![new_stock, product_id],
    )?;

    // Insert stock transaction
    let tx_id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO Stock_Transactions (transaction_id, product_id, transaction_type, quantity, reference_no)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![tx_id, product_id, transaction_type, diff_qty, reference_no],
    )?;

    product.current_stock = new_stock;
    Ok(product)
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
