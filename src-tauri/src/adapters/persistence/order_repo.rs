use std::collections::HashMap;

use chrono::Local;
use rusqlite::types::ToSql;
use rusqlite::{params, Connection, Row};

use crate::domain::entities::{Order, OrderItem};
use crate::domain::error::AppError;

// ==========================================================
// Input Data Contracts
// ==========================================================

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutInput {
    pub subtotal: f64,
    pub discount_amount: f64,
    pub payment_method: String,
    pub paid_amount: f64,
    #[serde(default)]
    pub note: Option<String>,
    pub items: Vec<CheckoutInputItem>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutInputItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldInput {
    pub hold_name: String,
    pub subtotal: f64,
    pub discount_amount: f64,
    #[serde(default)]
    pub note: Option<String>,
    pub items: Vec<CheckoutInputItem>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReturnInput {
    pub original_order_id: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    pub items: Vec<ReturnInputItem>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReturnInputItem {
    pub product_id: Option<String>,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Default)]
pub struct OrderFilterParams {
    pub page: u32,
    pub page_size: u32,
    pub search: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PaginatedOrders {
    pub orders: Vec<Order>,
    pub total_items: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

// ==========================================================
// Sales Summary Contracts
// ==========================================================

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SalesSummaryFilter {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub group_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesSummaryKpi {
    pub total_sales: f64,
    pub total_orders: i64,
    pub average_order_value: f64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesSummaryBreakdownItem {
    pub period: String,
    pub total_sales: f64,
    pub order_count: i64,
    pub average_order_value: f64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesSummaryResult {
    pub kpi: SalesSummaryKpi,
    pub breakdown: Vec<SalesSummaryBreakdownItem>,
    pub hourly_breakdown: Option<Vec<SalesSummaryBreakdownItem>>,
    pub period_type: String,
}

// ==========================================================
// Helpers
// ==========================================================

const SALE_PREFIX: &str = "ORD";
const RETURN_PREFIX: &str = "RET";

/// สร้างเลขที่บิลรันต่อวัน เช่น ORD-20260906-0001, RET-20260906-0003
fn generate_order_no(conn: &Connection, prefix: &str) -> Result<String, AppError> {
    let today = Local::now().format("%Y%m%d").to_string();
    let pattern = format!("{}-{}-", prefix, today);

    let max_no: Option<String> = conn
        .query_row(
            "SELECT MAX(order_no) FROM Orders WHERE order_no LIKE ?1",
            params![format!("{}%", pattern)],
            |row| row.get(0),
        )
        .map_err(AppError::from)?;

    let next_seq = match max_no {
        Some(no) => {
            let seq_part = no.rsplit('-').next().unwrap_or("0");
            seq_part.parse::<u32>().unwrap_or(0) + 1
        }
        None => 1,
    };

    Ok(format!("{}-{}-{:04}", prefix, today, next_seq))
}

fn map_order_row(row: &Row) -> rusqlite::Result<Order> {
    Ok(Order {
        order_id: row.get(0)?,
        order_no: row.get(1)?,
        order_type: row.get(2)?,
        status: row.get(3)?,
        subtotal: row.get(4)?,
        discount_amount: row.get(5)?,
        total_amount: row.get(6)?,
        payment_method: row.get(7)?,
        paid_amount: row.get(8)?,
        change_amount: row.get(9)?,
        hold_name: row.get(10)?,
        note: row.get(11)?,
        original_order_id: row.get(12)?,
        order_date: row.get(13)?,
        items: Vec::new(),
    })
}

const ORDER_SELECT: &str = "SELECT
        order_id, order_no, order_type, status, subtotal, discount_amount, total_amount,
        payment_method, paid_amount, change_amount, hold_name, note, original_order_id,
        datetime(order_date, 'localtime')
    FROM Orders";

fn load_order_items(conn: &Connection, order_id: &str) -> Result<Vec<OrderItem>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT item_id, order_id, product_id, product_name, quantity, unit_price, line_total
             FROM Order_Items WHERE order_id = ?1 ORDER BY rowid",
        )
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map(params![order_id], |row| {
            Ok(OrderItem {
                item_id: row.get(0)?,
                order_id: row.get(1)?,
                product_id: row.get(2)?,
                product_name: row.get(3)?,
                quantity: row.get(4)?,
                unit_price: row.get(5)?,
                line_total: row.get(6)?,
                returned_quantity: 0,
            })
        })
        .map_err(AppError::from)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// จำนวนที่ถูกคืนสะสมต่อ product_id ของบิลเดิมหนึ่งใบ (เฉพาะบิลคืนที่ COMPLETED)
fn get_returned_quantities(
    conn: &Connection,
    original_order_id: &str,
) -> Result<HashMap<String, i64>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT ri.product_id, SUM(ri.quantity)
             FROM Return_Items ri
             JOIN Orders ro ON ri.return_order_id = ro.order_id
             WHERE ri.original_order_id = ?1
               AND ri.product_id IS NOT NULL
               AND ro.status = 'COMPLETED'
             GROUP BY ri.product_id",
        )
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map(params![original_order_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(AppError::from)?;

    let mut map = HashMap::new();
    for row in rows {
        let (product_id, qty) = row.map_err(AppError::from)?;
        map.insert(product_id, qty);
    }
    Ok(map)
}

/// เติม returned_quantity ให้รายการสินค้าของบิลขาย (สำหรับหน้า RB)
fn fill_returned_quantities(conn: &Connection, order: &mut Order) -> Result<(), AppError> {
    if order.order_type != "SALE" {
        return Ok(());
    }
    let returned = get_returned_quantities(conn, &order.order_id)?;
    for item in order.items.iter_mut() {
        if let Some(ref pid) = item.product_id {
            item.returned_quantity = *returned.get(pid).unwrap_or(&0) as i32;
        }
    }
    Ok(())
}

fn get_order_by_id_internal(conn: &Connection, order_id: &str) -> Result<Order, AppError> {
    let sql = format!("{} WHERE order_id = ?1", ORDER_SELECT);
    let mut order = conn
        .query_row(&sql, params![order_id], map_order_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("ไม่พบบิลที่ระบุ (order_id: {})", order_id))
            }
            other => AppError::Database(other),
        })?;

    order.items = load_order_items(conn, &order.order_id)?;
    fill_returned_quantities(conn, &mut order)?;
    Ok(order)
}

// ==========================================================
// Checkout (ชำระเงิน) — สร้างบิลขาย + หักสต็อก ใน Transaction เดียว
// ==========================================================

pub fn create_order(conn: &Connection, input: &CheckoutInput) -> Result<Order, AppError> {
    if input.items.is_empty() {
        return Err(AppError::Validation("ไม่มีรายการสินค้าในบิล".to_string()));
    }
    let payment = input.payment_method.trim().to_uppercase();
    if !["CASH", "PROMPTPAY", "TRANSFER"].contains(&payment.as_str()) {
        return Err(AppError::Validation("วิธีชำระเงินไม่ถูกต้อง".to_string()));
    }

    let tx = conn.unchecked_transaction().map_err(AppError::from)?;

    // ตรวจสอบการตั้งค่าอนุญาตให้ขายสินค้าได้เมื่อสินค้าหมดสต๊อก
    let allow_out_of_stock: bool = tx
        .query_row(
            "SELECT setting_value FROM App_Settings WHERE setting_key = 'allow_out_of_stock_sale'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map(|v| v.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let order_no = generate_order_no(&tx, SALE_PREFIX)?;
    let order_id = uuid::Uuid::now_v7().to_string();

    let total = (input.subtotal - input.discount_amount).max(0.0);
    let change = (input.paid_amount - total).max(0.0);

    tx.execute(
        "INSERT INTO Orders (order_id, order_no, order_type, status, subtotal, discount_amount,
            total_amount, payment_method, paid_amount, change_amount, note, order_date)
         VALUES (?1, ?2, 'SALE', 'COMPLETED', ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP)",
        params![
            order_id,
            order_no,
            input.subtotal,
            input.discount_amount,
            total,
            payment,
            input.paid_amount,
            change,
            input.note
        ],
    )
    .map_err(AppError::from)?;

    let mut stmt_item = tx
        .prepare(
            "INSERT INTO Order_Items (item_id, order_id, product_id, product_name, quantity, unit_price, line_total)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(AppError::from)?;

    for item in &input.items {
        if item.quantity <= 0 {
            return Err(AppError::Validation("จำนวนสินค้าต้องมากกว่า 0".to_string()));
        }

        // ตรวจสต็อกปัจจุบันก่อนหัก
        let current_stock: Option<i32> = tx
            .query_row(
                "SELECT current_stock FROM Products WHERE product_id = ?1",
                params![item.product_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AppError::Validation(format!(
                    "ไม่พบสินค้า: {}",
                    item.product_name
                )),
                other => AppError::Database(other),
            })?;

        let stock = current_stock.unwrap_or(0);
        if !allow_out_of_stock && item.quantity > stock {
            return Err(AppError::Validation(format!(
                "สินค้า {} สต็อกไม่เพียงพอ (เหลือ {} ชิ้น)",
                item.product_name, stock
            )));
        }

        let line_total = item.unit_price * item.quantity as f64;
        let item_id = uuid::Uuid::now_v7().to_string();
        stmt_item
            .execute(params![
                item_id,
                order_id,
                item.product_id,
                item.product_name,
                item.quantity,
                item.unit_price,
                line_total
            ])
            .map_err(AppError::from)?;

        // หักสต็อก
        tx.execute(
            "UPDATE Products SET current_stock = current_stock - ?1 WHERE product_id = ?2",
            params![item.quantity, item.product_id],
        )
        .map_err(AppError::from)?;

        // บันทึก Audit Trail การเคลื่อนไหวสต็อก
        tx.execute(
            "INSERT INTO Stock_Transactions (transaction_id, product_id, transaction_type, quantity, reference_no, transaction_date)
             VALUES (?1, ?2, 'OUT', ?3, ?4, CURRENT_TIMESTAMP)",
            params![
                uuid::Uuid::now_v7().to_string(),
                item.product_id,
                item.quantity,
                order_no
            ],
        )
        .map_err(AppError::from)?;
    }
    drop(stmt_item);
    tx.commit().map_err(AppError::from)?;

    get_order_by_id_internal(conn, &order_id)
}

// ==========================================================
// Hold Orders (พักบิล) — ไม่หักสต็อก จนกว่าจะชำระเงินจริง
// ==========================================================

pub fn hold_order(conn: &Connection, input: &HoldInput) -> Result<Order, AppError> {
    if input.items.is_empty() {
        return Err(AppError::Validation("ไม่มีรายการสินค้าในบิล".to_string()));
    }

    let tx = conn.unchecked_transaction().map_err(AppError::from)?;

    let order_no = generate_order_no(&tx, SALE_PREFIX)?;
    let order_id = uuid::Uuid::now_v7().to_string();
    let total = (input.subtotal - input.discount_amount).max(0.0);

    tx.execute(
        "INSERT INTO Orders (order_id, order_no, order_type, status, subtotal, discount_amount,
            total_amount, payment_method, hold_name, note, order_date)
         VALUES (?1, ?2, 'SALE', 'HELD', ?3, ?4, ?5, 'CASH', ?6, ?7, CURRENT_TIMESTAMP)",
        params![
            order_id,
            order_no,
            input.subtotal,
            input.discount_amount,
            total,
            input.hold_name,
            input.note
        ],
    )
    .map_err(AppError::from)?;

    insert_items_for_order(&tx, &order_id, &input.items)?;
    tx.commit().map_err(AppError::from)?;

    get_order_by_id_internal(conn, &order_id)
}

fn insert_items_for_order(
    tx: &rusqlite::Transaction,
    order_id: &str,
    items: &[CheckoutInputItem],
) -> Result<(), AppError> {
    let mut stmt_item = tx
        .prepare(
            "INSERT INTO Order_Items (item_id, order_id, product_id, product_name, quantity, unit_price, line_total)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(AppError::from)?;

    for item in items {
        if item.quantity <= 0 {
            return Err(AppError::Validation("จำนวนสินค้าต้องมากกว่า 0".to_string()));
        }
        let line_total = item.unit_price * item.quantity as f64;
        stmt_item
            .execute(params![
                uuid::Uuid::now_v7().to_string(),
                order_id,
                item.product_id,
                item.product_name,
                item.quantity,
                item.unit_price,
                line_total
            ])
            .map_err(AppError::from)?;
    }
    Ok(())
}

pub fn get_held_orders(conn: &Connection) -> Result<Vec<Order>, AppError> {
    let sql = format!(
        "{} WHERE status = 'HELD' ORDER BY order_date ASC",
        ORDER_SELECT
    );
    let mut stmt = conn.prepare(&sql).map_err(AppError::from)?;
    let rows = stmt.query_map([], map_order_row).map_err(AppError::from)?;

    let mut orders = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    for order in orders.iter_mut() {
        order.items = load_order_items(conn, &order.order_id)?;
    }
    Ok(orders)
}

/// ลบบิลที่พักอยู่เท่านั้น (บิลที่ชำระแล้วไม่ให้ลบ)
pub fn delete_held_order(conn: &Connection, order_id: &str) -> Result<(), AppError> {
    let status: String = conn
        .query_row(
            "SELECT status FROM Orders WHERE order_id = ?1",
            params![order_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound("ไม่พบบิลที่ต้องการลบ".to_string())
            }
            other => AppError::Database(other),
        })?;

    if status != "HELD" {
        return Err(AppError::Validation(
            "ลบได้เฉพาะบิลที่พักอยู่เท่านั้น".to_string(),
        ));
    }

    conn.execute(
        "DELETE FROM Orders WHERE order_id = ?1 AND status = 'HELD'",
        params![order_id],
    )
    .map_err(AppError::from)?;
    Ok(())
}

// ==========================================================
// Sales History (ประวัติการขาย)
// ==========================================================

pub fn get_orders_paginated(
    conn: &Connection,
    filter: &OrderFilterParams,
) -> Result<PaginatedOrders, AppError> {
    let mut where_clauses: Vec<String> = vec![
        "o.order_type = 'SALE'".to_string(),
        "o.status = 'COMPLETED'".to_string(),
    ];
    let mut sql_params: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(ref search) = filter.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            where_clauses.push("o.order_no LIKE ?".to_string());
            sql_params.push(Box::new(format!("%{}%", trimmed)));
        }
    }

    let where_sql = format!("WHERE {}", where_clauses.join(" AND "));

    let count_sql = format!("SELECT COUNT(*) FROM Orders o {}", where_sql);
    let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
    let total_items: i64 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(param_refs.iter().copied()),
            |row| row.get(0),
        )
        .map_err(AppError::from)?;
    let total_items = total_items.max(0) as u64;

    let page = filter.page.max(1);
    let page_size = filter.page_size.clamp(1, 500);
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + page_size as u64 - 1) / page_size as u64) as u32
    };
    let offset = ((page - 1) * page_size) as i64;

    let select_from = ORDER_SELECT.replace("FROM Orders", "FROM Orders o");
    let data_sql = format!(
        "{} {} ORDER BY o.order_date DESC LIMIT {} OFFSET {}",
        select_from, where_sql, page_size, offset
    );

    let mut stmt = conn.prepare(&data_sql).map_err(AppError::from)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(param_refs.iter().copied()), map_order_row)
        .map_err(AppError::from)?;
    let orders = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;

    Ok(PaginatedOrders {
        orders,
        total_items,
        page,
        page_size,
        total_pages,
    })
}

pub fn get_order_detail(conn: &Connection, order_id: &str) -> Result<Order, AppError> {
    get_order_by_id_internal(conn, order_id)
}

pub fn get_order_by_no(conn: &Connection, order_no: &str) -> Result<Order, AppError> {
    let sql = format!("{} WHERE order_no = ?1", ORDER_SELECT);
    let mut order = conn
        .query_row(&sql, params![order_no.trim()], map_order_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("ไม่พบบิลเลขที่ {}", order_no))
            }
            other => AppError::Database(other),
        })?;

    order.items = load_order_items(conn, &order.order_id)?;
    fill_returned_quantities(conn, &mut order)?;
    Ok(order)
}

// ==========================================================
// Sales Summary (สรุปยอดขาย รายวัน - เดือน)
// ==========================================================

pub fn get_sales_summary(
    conn: &Connection,
    filter: &SalesSummaryFilter,
) -> Result<SalesSummaryResult, AppError> {
    if let (Some(s), Some(e)) = (&filter.start_date, &filter.end_date) {
        let s_trimmed = s.trim();
        let e_trimmed = e.trim();
        if !s_trimmed.is_empty() && !e_trimmed.is_empty() && e_trimmed < s_trimmed {
            return Err(AppError::Validation(
                "วันที่สิ้นสุดต้องไม่น้อยกว่าวันที่เริ่มต้น".to_string(),
            ));
        }
    }

    let group_by_mode = match filter.group_by.as_deref() {
        Some("monthly") => "monthly",
        Some("daily") => "daily",
        Some("hourly") => "hourly",
        _ => {
            if let (Some(s), Some(e)) = (&filter.start_date, &filter.end_date) {
                if let (Ok(d1), Ok(d2)) = (
                    chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d"),
                    chrono::NaiveDate::parse_from_str(e.trim(), "%Y-%m-%d"),
                ) {
                    let diff_days = (d2 - d1).num_days();
                    if diff_days > 90 {
                        "monthly"
                    } else {
                        "daily"
                    }
                } else {
                    "daily"
                }
            } else {
                "daily"
            }
        }
    };

    let strftime_fmt = match group_by_mode {
        "monthly" => "%Y-%m",
        "hourly" => "%H:00",
        _ => "%Y-%m-%d",
    };

    let mut where_clauses: Vec<String> = vec!["status = 'COMPLETED'".to_string()];
    let mut sql_params: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(ref start) = filter.start_date {
        let trimmed = start.trim();
        if !trimmed.is_empty() {
            where_clauses.push("date(order_date, 'localtime') >= ?".to_string());
            sql_params.push(Box::new(trimmed.to_string()));
        }
    }

    if let Some(ref end) = filter.end_date {
        let trimmed = end.trim();
        if !trimmed.is_empty() {
            where_clauses.push("date(order_date, 'localtime') <= ?".to_string());
            sql_params.push(Box::new(trimmed.to_string()));
        }
    }

    let where_sql = format!("WHERE {}", where_clauses.join(" AND "));
    let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();

    // 1. คำนวณ KPI ภาพรวม
    let kpi_sql = format!(
        "SELECT 
            COALESCE(SUM(CASE WHEN order_type = 'SALE' THEN total_amount WHEN order_type = 'RETURN' THEN -total_amount ELSE 0.0 END), 0.0) as net_sales,
            COALESCE(SUM(CASE WHEN order_type = 'SALE' THEN 1 ELSE 0 END), 0) as total_orders
         FROM Orders {}",
        where_sql
    );
    let (net_sales, total_orders): (f64, i64) = conn
        .query_row(
            &kpi_sql,
            rusqlite::params_from_iter(param_refs.iter().copied()),
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(AppError::from)?;

    let total_sales = net_sales.max(0.0);
    let average_order_value = if total_orders > 0 {
        total_sales / total_orders as f64
    } else {
        0.0
    };

    // 2. แจกแจงตามช่วงเวลา (Breakdown)
    let breakdown_sql = format!(
        "SELECT 
            strftime('{}', order_date, 'localtime') as period,
            COALESCE(SUM(CASE WHEN order_type = 'SALE' THEN total_amount WHEN order_type = 'RETURN' THEN -total_amount ELSE 0.0 END), 0.0) as period_sales,
            COALESCE(SUM(CASE WHEN order_type = 'SALE' THEN 1 ELSE 0 END), 0) as order_count
         FROM Orders {}
         GROUP BY period
         ORDER BY period ASC",
        strftime_fmt, where_sql
    );
    let mut stmt = conn.prepare(&breakdown_sql).map_err(AppError::from)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(param_refs.iter().copied()), |row| {
            let period: Option<String> = row.get(0)?;
            let period_sales: f64 = row.get(1)?;
            let order_count: i64 = row.get(2)?;
            Ok((period.unwrap_or_default(), period_sales, order_count))
        })
        .map_err(AppError::from)?;

    let mut breakdown = Vec::new();
    for r in rows {
        let (period, period_sales, order_count) = r.map_err(AppError::from)?;
        if period.is_empty() {
            continue;
        }
        let sales = period_sales.max(0.0);
        let avg = if order_count > 0 {
            sales / order_count as f64
        } else {
            0.0
        };
        breakdown.push(SalesSummaryBreakdownItem {
            period,
            total_sales: sales,
            order_count,
            average_order_value: avg,
        });
    }

    // 3. แจกแจงรายชั่วโมงเฉพาะกรณีดูวันเดียว (Single Day)
    let is_single_day = match (&filter.start_date, &filter.end_date) {
        (Some(s), Some(e)) if !s.trim().is_empty() && s.trim() == e.trim() => true,
        _ => false,
    };

    let hourly_breakdown = if is_single_day {
        let hourly_sql = format!(
            "SELECT 
                strftime('%H:00', order_date, 'localtime') as hr,
                COALESCE(SUM(CASE WHEN order_type = 'SALE' THEN total_amount WHEN order_type = 'RETURN' THEN -total_amount ELSE 0.0 END), 0.0) as hr_sales,
                COALESCE(SUM(CASE WHEN order_type = 'SALE' THEN 1 ELSE 0 END), 0) as hr_count
             FROM Orders {}
             GROUP BY hr
             ORDER BY hr ASC",
            where_sql
        );
        let mut h_stmt = conn.prepare(&hourly_sql).map_err(AppError::from)?;
        let h_rows = h_stmt
            .query_map(rusqlite::params_from_iter(param_refs.iter().copied()), |row| {
                let hr: Option<String> = row.get(0)?;
                let hr_sales: f64 = row.get(1)?;
                let hr_count: i64 = row.get(2)?;
                Ok((hr.unwrap_or_default(), hr_sales, hr_count))
            })
            .map_err(AppError::from)?;

        let mut h_list = Vec::new();
        for r in h_rows {
            let (hr, hr_sales, hr_count) = r.map_err(AppError::from)?;
            if hr.is_empty() {
                continue;
            }
            let sales = hr_sales.max(0.0);
            let avg = if hr_count > 0 {
                sales / hr_count as f64
            } else {
                0.0
            };
            h_list.push(SalesSummaryBreakdownItem {
                period: hr,
                total_sales: sales,
                order_count: hr_count,
                average_order_value: avg,
            });
        }
        Some(h_list)
    } else {
        None
    };

    Ok(SalesSummaryResult {
        kpi: SalesSummaryKpi {
            total_sales,
            total_orders,
            average_order_value,
        },
        breakdown,
        hourly_breakdown,
        period_type: group_by_mode.to_string(),
    })
}

// ==========================================================
// Return Order (RB — รับคืนสินค้า/คืนเงิน)
// ==========================================================

pub fn create_return_order(conn: &Connection, input: &ReturnInput) -> Result<Order, AppError> {
    if input.items.is_empty() {
        return Err(AppError::Validation(
            "กรุณาเลือกรายการสินค้าที่ต้องการคืนอย่างน้อย 1 รายการ".to_string(),
        ));
    }

    // โหลดบิลเดิมเพื่อตรวจสอบ
    let original = get_order_by_id_internal(conn, &input.original_order_id)?;
    if original.order_type != "SALE" || original.status != "COMPLETED" {
        return Err(AppError::Validation(
            "คืนสินค้าได้เฉพาะบิลขายที่ชำระเงินแล้วเท่านั้น".to_string(),
        ));
    }

    // จำนวนขายสะสมต่อ product_id
    let mut sold_qty: HashMap<String, i64> = HashMap::new();
    for item in &original.items {
        if let Some(ref pid) = item.product_id {
            *sold_qty.entry(pid.clone()).or_insert(0) += item.quantity as i64;
        }
    }
    // จำนวนที่คืนไปแล้วสะสม
    let returned = get_returned_quantities(conn, &original.order_id)?;

    // ตรวจสอบว่าคืนเกินบิลเดิมหรือไม่
    for item in &input.items {
        if item.quantity <= 0 {
            continue;
        }
        let pid = match &item.product_id {
            Some(pid) => pid.clone(),
            None => {
                return Err(AppError::Validation(format!(
                    "สินค้า {} ไม่มีรหัสสินค้าอ้างอิง จึงไม่สามารถคืนได้",
                    item.product_name
                )))
            }
        };
        let sold = *sold_qty.get(&pid).unwrap_or(&0);
        let already = *returned.get(&pid).unwrap_or(&0);
        if (already + item.quantity as i64) > sold {
            return Err(AppError::Validation(format!(
                "สินค้า {} คืนเกินจำนวนที่ขาย (ขาย {} ชิ้น, คืนไปแล้ว {} ชิ้น)",
                item.product_name, sold, already
            )));
        }
    }

    let tx = conn.unchecked_transaction().map_err(AppError::from)?;

    let order_no = generate_order_no(&tx, RETURN_PREFIX)?;
    let order_id = uuid::Uuid::now_v7().to_string();
    let total: f64 = input
        .items
        .iter()
        .map(|i| i.unit_price * i.quantity as f64)
        .sum();

    tx.execute(
        "INSERT INTO Orders (order_id, order_no, order_type, status, subtotal, discount_amount,
            total_amount, payment_method, paid_amount, change_amount, note, original_order_id, order_date)
         VALUES (?1, ?2, 'RETURN', 'COMPLETED', ?3, 0.0, ?3, 'CASH', ?3, 0.0, ?4, ?5, CURRENT_TIMESTAMP)",
        params![order_id, order_no, total, input.note, original.order_id],
    )
    .map_err(AppError::from)?;

    let mut stmt_item = tx
        .prepare(
            "INSERT INTO Order_Items (item_id, order_id, product_id, product_name, quantity, unit_price, line_total)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(AppError::from)?;
    let mut stmt_return = tx
        .prepare(
            "INSERT INTO Return_Items (return_item_id, return_order_id, original_order_id, product_id, product_name, quantity, reason, return_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
        )
        .map_err(AppError::from)?;

    for item in &input.items {
        if item.quantity <= 0 {
            continue;
        }
        let line_total = item.unit_price * item.quantity as f64;
        stmt_item
            .execute(params![
                uuid::Uuid::now_v7().to_string(),
                order_id,
                item.product_id,
                item.product_name,
                item.quantity,
                item.unit_price,
                line_total
            ])
            .map_err(AppError::from)?;

        stmt_return
            .execute(params![
                uuid::Uuid::now_v7().to_string(),
                order_id,
                original.order_id,
                item.product_id,
                item.product_name,
                item.quantity,
                input.reason
            ])
            .map_err(AppError::from)?;

        // คืนสต็อกเข้าคลัง
        if let Some(ref pid) = item.product_id {
            tx.execute(
                "UPDATE Products SET current_stock = current_stock + ?1 WHERE product_id = ?2",
                params![item.quantity, pid],
            )
            .map_err(AppError::from)?;

            tx.execute(
                "INSERT INTO Stock_Transactions (transaction_id, product_id, transaction_type, quantity, reference_no, transaction_date)
                 VALUES (?1, ?2, 'IN', ?3, ?4, CURRENT_TIMESTAMP)",
                params![uuid::Uuid::now_v7().to_string(), pid, item.quantity, order_no],
            )
            .map_err(AppError::from)?;
        }
    }
    drop(stmt_item);
    drop(stmt_return);
    tx.commit().map_err(AppError::from)?;

    get_order_by_id_internal(conn, &order_id)
}

// ==========================================================
// Tests — ตรวจสอบ Order Flow กับ In-Memory SQLite
// ==========================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema).unwrap();

        conn.execute(
            "INSERT INTO Products (product_id, barcode, name, selling_price, current_stock, reorder_level)
             VALUES ('p1', '885000000001', 'น้ำดื่ม 500ml', 7.0, 10, 2)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Products (product_id, barcode, name, selling_price, current_stock, reorder_level)
             VALUES ('p2', '885000000002', 'ขนม', 20.0, 5, 2)",
            [],
        )
        .unwrap();
        conn
    }

    fn checkout_input(pid: &str, name: &str, qty: i32, price: f64) -> CheckoutInput {
        CheckoutInput {
            subtotal: price * qty as f64,
            discount_amount: 0.0,
            payment_method: "CASH".to_string(),
            paid_amount: price * qty as f64,
            note: None,
            items: vec![CheckoutInputItem {
                product_id: pid.to_string(),
                product_name: name.to_string(),
                quantity: qty,
                unit_price: price,
            }],
        }
    }

    #[test]
    fn test_create_order_deducts_stock() {
        let conn = setup();
        let order = create_order(&conn, &checkout_input("p1", "น้ำดื่ม 500ml", 3, 7.0)).unwrap();

        assert!(order.order_no.starts_with("ORD-"));
        assert_eq!(order.status, "COMPLETED");
        assert_eq!(order.total_amount, 21.0);
        assert_eq!(order.items.len(), 1);

        let stock: i32 = conn
            .query_row(
                "SELECT current_stock FROM Products WHERE product_id = 'p1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 7);

        // มีบันทึก Stock_Transactions OUT
        let out_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Stock_Transactions WHERE product_id = 'p1' AND transaction_type = 'OUT'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(out_count, 1);
    }

    #[test]
    fn test_create_order_insufficient_stock() {
        let conn = setup();
        let result = create_order(&conn, &checkout_input("p1", "น้ำดื่ม 500ml", 99, 7.0));
        assert!(result.is_err());

        // สต็อกต้องไม่ถูกหัก และไม่มีบิลค้างในระบบ
        let stock: i32 = conn
            .query_row(
                "SELECT current_stock FROM Products WHERE product_id = 'p1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 10);

        let order_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM Orders", [], |r| r.get(0))
            .unwrap();
        assert_eq!(order_count, 0);
    }

    #[test]
    fn test_create_order_allow_out_of_stock_sale() {
        let conn = setup();
        // เปิดใช้งาน allow_out_of_stock_sale ใน App_Settings
        conn.execute(
            "INSERT OR REPLACE INTO App_Settings (setting_key, setting_value, description, updated_at)
             VALUES ('allow_out_of_stock_sale', 'true', 'อนุญาตให้ขายสินค้าได้เมื่อสินค้าหมดสต๊อก', CURRENT_TIMESTAMP)",
            [],
        )
        .unwrap();

        // สินค้า p1 มีสต็อกเดิม = 10, ทำการขาย 15 ชิ้น
        let order = create_order(&conn, &checkout_input("p1", "น้ำดื่ม 500ml", 15, 7.0)).unwrap();

        assert_eq!(order.status, "COMPLETED");
        assert_eq!(order.total_amount, 105.0);

        // สต็อกสินค้าต้องถูกหักเหลือ -5
        let stock: i32 = conn
            .query_row(
                "SELECT current_stock FROM Products WHERE product_id = 'p1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, -5);

        // มีบันทึก Stock_Transactions OUT จำนวน 15 ชิ้น
        let out_qty: i32 = conn
            .query_row(
                "SELECT quantity FROM Stock_Transactions WHERE product_id = 'p1' AND transaction_type = 'OUT'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(out_qty, 15);
    }

    #[test]
    fn test_hold_and_recall_flow() {
        let conn = setup();
        let input = HoldInput {
            hold_name: "พัก 10:30".to_string(),
            subtotal: 21.0,
            discount_amount: 0.0,
            note: None,
            items: vec![CheckoutInputItem {
                product_id: "p1".to_string(),
                product_name: "น้ำดื่ม 500ml".to_string(),
                quantity: 3,
                unit_price: 7.0,
            }],
        };

        let held = hold_order(&conn, &input).unwrap();
        assert_eq!(held.status, "HELD");
        assert_eq!(held.hold_name.as_deref(), Some("พัก 10:30"));

        // พักบิลต้องไม่หักสต็อก
        let stock: i32 = conn
            .query_row(
                "SELECT current_stock FROM Products WHERE product_id = 'p1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 10);

        let held_list = get_held_orders(&conn).unwrap();
        assert_eq!(held_list.len(), 1);
        assert_eq!(held_list[0].items.len(), 1);

        delete_held_order(&conn, &held.order_id).unwrap();
        assert!(get_held_orders(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_delete_non_held_order_rejected() {
        let conn = setup();
        let order = create_order(&conn, &checkout_input("p1", "น้ำดื่ม 500ml", 1, 7.0)).unwrap();
        // บิลที่ COMPLETED ลบไม่ได้
        assert!(delete_held_order(&conn, &order.order_id).is_err());
    }

    #[test]
    fn test_return_flow_with_over_return_guard() {
        let conn = setup();
        let sale = create_order(&conn, &checkout_input("p1", "น้ำดื่ม 500ml", 5, 7.0)).unwrap();

        // คืน 2 ชิ้น
        let return_input = ReturnInput {
            original_order_id: sale.order_id.clone(),
            note: None,
            reason: Some("ลูกค้าเปลี่ยนใจ".to_string()),
            items: vec![ReturnInputItem {
                product_id: Some("p1".to_string()),
                product_name: "น้ำดื่ม 500ml".to_string(),
                quantity: 2,
                unit_price: 7.0,
            }],
        };
        let rb = create_return_order(&conn, &return_input).unwrap();

        assert!(rb.order_no.starts_with("RET-"));
        assert_eq!(rb.order_type, "RETURN");
        assert_eq!(rb.total_amount, 14.0);

        // บิลเดิมต้องแสดง returned_quantity = 2
        let detail = get_order_detail(&conn, &sale.order_id).unwrap();
        assert_eq!(detail.items[0].returned_quantity, 2);

        // สต็อก = 10 - 5 + 2 = 7
        let stock: i32 = conn
            .query_row(
                "SELECT current_stock FROM Products WHERE product_id = 'p1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 7);

        // คืนซ้ำ 4 ชิ้น (เหลือสิทธิ์คืน 3) ต้อง error
        let over_return = ReturnInput {
            original_order_id: sale.order_id.clone(),
            note: None,
            reason: None,
            items: vec![ReturnInputItem {
                product_id: Some("p1".to_string()),
                product_name: "น้ำดื่ม 500ml".to_string(),
                quantity: 4,
                unit_price: 7.0,
            }],
        };
        assert!(create_return_order(&conn, &over_return).is_err());

        // คืนต่ออีก 3 ชิ้น (คืนครบ) สำเร็จ
        let final_return = ReturnInput {
            original_order_id: sale.order_id.clone(),
            note: None,
            reason: None,
            items: vec![ReturnInputItem {
                product_id: Some("p1".to_string()),
                product_name: "น้ำดื่ม 500ml".to_string(),
                quantity: 3,
                unit_price: 7.0,
            }],
        };
        assert!(create_return_order(&conn, &final_return).is_ok());

        let detail2 = get_order_detail(&conn, &sale.order_id).unwrap();
        assert_eq!(detail2.items[0].returned_quantity, 5);

        // สต็อกกลับมาเท่าเดิม = 10
        let stock2: i32 = conn
            .query_row(
                "SELECT current_stock FROM Products WHERE product_id = 'p1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock2, 10);
    }

    #[test]
    fn test_order_no_sequence_per_day() {
        let conn = setup();
        let o1 = create_order(&conn, &checkout_input("p1", "น้ำดื่ม 500ml", 1, 7.0)).unwrap();
        let o2 = create_order(&conn, &checkout_input("p2", "ขนม", 1, 20.0)).unwrap();

        let seq1: u32 = o1.order_no.rsplit('-').next().unwrap().parse().unwrap();
        let seq2: u32 = o2.order_no.rsplit('-').next().unwrap().parse().unwrap();
        assert_eq!(seq2, seq1 + 1);
        assert_ne!(o1.order_no, o2.order_no);
    }

    #[test]
    fn test_get_order_by_no_and_history() {
        let conn = setup();
        let order = create_order(&conn, &checkout_input("p1", "น้ำดื่ม 500ml", 2, 7.0)).unwrap();

        let found = get_order_by_no(&conn, &order.order_no).unwrap();
        assert_eq!(found.order_id, order.order_id);
        assert!(get_order_by_no(&conn, "ORD-20990101-9999").is_err());

        let page = get_orders_paginated(
            &conn,
            &OrderFilterParams {
                page: 1,
                page_size: 10,
                search: None,
            },
        )
        .unwrap();
        assert_eq!(page.total_items, 1);
        assert_eq!(page.orders[0].order_id, order.order_id);
    }

    #[test]
    fn test_sales_summary() {
        let conn = setup();
        // บิลที่ 1: น้ำดื่ม 2 ขวด @ 7 = 14
        let _o1 = create_order(&conn, &checkout_input("p1", "น้ำดื่ม 500ml", 2, 7.0)).unwrap();
        // บิลที่ 2: ขนม 1 ชิ้น @ 20 = 20
        let _o2 = create_order(&conn, &checkout_input("p2", "ขนม", 1, 20.0)).unwrap();

        let today = Local::now().format("%Y-%m-%d").to_string();
        let summary = get_sales_summary(
            &conn,
            &SalesSummaryFilter {
                start_date: Some(today.clone()),
                end_date: Some(today.clone()),
                group_by: None,
            },
        )
        .unwrap();

        assert_eq!(summary.kpi.total_orders, 2);
        assert!((summary.kpi.total_sales - 34.0).abs() < 1e-6);
        assert!((summary.kpi.average_order_value - 17.0).abs() < 1e-6);
        assert_eq!(summary.period_type, "daily");
        assert_eq!(summary.breakdown.len(), 1);
        assert_eq!(summary.breakdown[0].period, today);
        assert!((summary.breakdown[0].total_sales - 34.0).abs() < 1e-6);
        assert!(summary.hourly_breakdown.is_some());
    }

    #[test]
    fn test_sales_summary_invalid_date_range() {
        let conn = setup();
        let err = get_sales_summary(
            &conn,
            &SalesSummaryFilter {
                start_date: Some("2026-09-18".to_string()),
                end_date: Some("2026-09-06".to_string()),
                group_by: None,
            },
        )
        .unwrap_err();

        match err {
            AppError::Validation(msg) => {
                assert_eq!(msg, "วันที่สิ้นสุดต้องไม่น้อยกว่าวันที่เริ่มต้น");
            }
            _ => panic!("Expected AppError::Validation, got {:?}", err),
        }
    }
}