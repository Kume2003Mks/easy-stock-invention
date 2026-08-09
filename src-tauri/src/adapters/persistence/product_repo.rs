use rusqlite::types::ToSql;
use rusqlite::Connection;

use crate::domain::entities::{Category, Product, Supplier};
use crate::domain::error::AppError;

#[derive(Debug, Clone, Default)]
pub struct ProductFilterParams {
    pub page: u32,
    pub page_size: u32,
    pub search: Option<String>,
    pub category_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PaginatedProducts {
    pub products: Vec<Product>,
    pub total_items: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

pub fn get_products_paginated(
    conn: &Connection,
    params: &ProductFilterParams,
) -> Result<PaginatedProducts, AppError> {
    let mut where_clauses: Vec<String> = Vec::new();
    let mut sql_params: Vec<Box<dyn ToSql>> = Vec::new();

    // Search filter: product name, barcode, category name, or supplier name
    if let Some(ref search) = params.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            let pattern = format!("%{}%", trimmed);
            where_clauses.push(
                "(p.name LIKE ? OR p.barcode LIKE ? OR c.name LIKE ? OR s.name LIKE ?)".to_string(),
            );
            sql_params.push(Box::new(pattern.clone()));
            sql_params.push(Box::new(pattern.clone()));
            sql_params.push(Box::new(pattern.clone()));
            sql_params.push(Box::new(pattern));
        }
    }

    // Category filter: "none" for null category, or specific category_id
    if let Some(ref cat) = params.category_id {
        let cat_trimmed = cat.trim();
        if cat_trimmed == "none" {
            where_clauses.push("p.category_id IS NULL".to_string());
        } else if !cat_trimmed.is_empty() && cat_trimmed != "all" {
            where_clauses.push("p.category_id = ?".to_string());
            sql_params.push(Box::new(cat_trimmed.to_string()));
        }
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // 1. Count total matching items
    let count_sql = format!(
        "SELECT COUNT(*) 
         FROM Products p 
         LEFT JOIN Categories c ON p.category_id = c.category_id 
         LEFT JOIN Suppliers s ON p.supplier_id = s.supplier_id 
         {}",
        where_sql
    );

    let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
    let total_items: i64 = conn.query_row(
        &count_sql,
        rusqlite::params_from_iter(param_refs.iter().copied()),
        |row| row.get(0),
    )?;
    let total_items = total_items.max(0) as u64;

    // Calculate pagination values
    let page = params.page.max(1);
    let page_size = params.page_size.clamp(1, 500);
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + page_size as u64 - 1) / page_size as u64) as u32
    };
    let offset = ((page - 1) * page_size) as i64;

    // 2. Query paginated product rows
    let data_sql = format!(
        "SELECT 
            p.product_id, 
            p.barcode, 
            p.name, 
            p.category_id, 
            p.supplier_id, 
            p.cost_price, 
            p.selling_price, 
            p.wholesale_price, 
            p.current_stock, 
            p.reorder_level 
         FROM Products p 
         LEFT JOIN Categories c ON p.category_id = c.category_id 
         LEFT JOIN Suppliers s ON p.supplier_id = s.supplier_id 
         {} 
         ORDER BY p.name ASC, p.product_id ASC 
         LIMIT ? OFFSET ?",
        where_sql
    );

    let mut data_params = sql_params;
    data_params.push(Box::new(page_size as i64));
    data_params.push(Box::new(offset));
    let data_param_refs: Vec<&dyn ToSql> = data_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&data_sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(data_param_refs.iter().copied()),
        |row| {
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
        },
    )?;

    let products = rows.collect::<Result<Vec<_>, _>>()?;

    Ok(PaginatedProducts {
        products,
        total_items,
        page,
        page_size,
        total_pages,
    })
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE Categories (
                category_id TEXT PRIMARY KEY, 
                name TEXT NOT NULL UNIQUE
            );
            CREATE TABLE Suppliers (
                supplier_id TEXT PRIMARY KEY, 
                name TEXT NOT NULL,
                contact_info TEXT
            );
            CREATE TABLE Products (
                product_id TEXT PRIMARY KEY,
                barcode TEXT UNIQUE, 
                name TEXT NOT NULL,
                category_id TEXT, 
                supplier_id TEXT,              
                cost_price REAL NOT NULL DEFAULT 0.0,          
                selling_price REAL NOT NULL DEFAULT 0.0,        
                wholesale_price REAL NOT NULL DEFAULT 0.0,     
                current_stock INTEGER NOT NULL DEFAULT 0,
                reorder_level INTEGER NOT NULL DEFAULT 10,
                FOREIGN KEY (category_id) REFERENCES Categories(category_id) ON UPDATE CASCADE ON DELETE SET NULL,
                FOREIGN KEY (supplier_id) REFERENCES Suppliers(supplier_id) ON UPDATE CASCADE ON DELETE SET NULL
            );
            CREATE TABLE Stock_Transactions (
                transaction_id TEXT PRIMARY KEY,
                product_id TEXT, 
                transaction_type TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                reference_no TEXT, 
                transaction_date DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_pagination_and_filtering() {
        let conn = setup_test_db();

        // Insert categories
        conn.execute(
            "INSERT INTO Categories (category_id, name) VALUES ('cat-1', 'เครื่องดื่ม'), ('cat-2', 'ขนม')",
            [],
        )
        .unwrap();

        // Insert supplier
        conn.execute(
            "INSERT INTO Suppliers (supplier_id, name) VALUES ('sup-1', 'CP All')",
            [],
        )
        .unwrap();

        // Insert 25 products
        for i in 1..=25 {
            let cat = if i <= 10 {
                Some("cat-1")
            } else if i <= 20 {
                Some("cat-2")
            } else {
                None
            };
            let sup = if i % 2 == 0 { Some("sup-1") } else { None };
            let p = Product {
                product_id: format!("p-{}", i),
                barcode: Some(format!("8850000000{:02}", i)),
                name: format!("สินค้าทดสอบ {:02}", i),
                category_id: cat.map(String::from),
                supplier_id: sup.map(String::from),
                cost_price: 10.0,
                selling_price: 20.0,
                wholesale_price: 15.0,
                current_stock: 50,
                reorder_level: 10,
            };
            insert_product(&conn, &p).unwrap();
        }

        // Test 1: Default page 1, size 10 -> total 25 items, 3 pages
        let res = get_products_paginated(
            &conn,
            &ProductFilterParams {
                page: 1,
                page_size: 10,
                search: None,
                category_id: None,
            },
        )
        .unwrap();
        assert_eq!(res.total_items, 25);
        assert_eq!(res.total_pages, 3);
        assert_eq!(res.products.len(), 10);

        // Test 2: Category filter 'cat-1' -> 10 items
        let res = get_products_paginated(
            &conn,
            &ProductFilterParams {
                page: 1,
                page_size: 10,
                search: None,
                category_id: Some("cat-1".into()),
            },
        )
        .unwrap();
        assert_eq!(res.total_items, 10);
        assert_eq!(res.total_pages, 1);
        assert_eq!(res.products.len(), 10);

        // Test 3: Category filter 'none' (uncategorized) -> 5 items
        let res = get_products_paginated(
            &conn,
            &ProductFilterParams {
                page: 1,
                page_size: 10,
                search: None,
                category_id: Some("none".into()),
            },
        )
        .unwrap();
        assert_eq!(res.total_items, 5);
        assert_eq!(res.products.len(), 5);

        // Test 4: Search by barcode
        let res = get_products_paginated(
            &conn,
            &ProductFilterParams {
                page: 1,
                page_size: 10,
                search: Some("885000000005".into()),
                category_id: None,
            },
        )
        .unwrap();
        assert_eq!(res.total_items, 1);
        assert_eq!(res.products[0].product_id, "p-5");

        // Test 5: Search by category name
        let res = get_products_paginated(
            &conn,
            &ProductFilterParams {
                page: 1,
                page_size: 10,
                search: Some("ขนม".into()),
                category_id: None,
            },
        )
        .unwrap();
        assert_eq!(res.total_items, 10);
    }
}
