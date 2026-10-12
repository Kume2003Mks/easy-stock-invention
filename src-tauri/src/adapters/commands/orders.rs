use std::sync::Mutex;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::adapters::persistence::{order_repo, settings_repo};
use crate::adapters::printer::{self, PrinterConnection};
use crate::domain::entities::Order;
use crate::domain::error::AppError;
use crate::domain::receipt::{PaperSize, ReceiptData, ReceiptItem};
use crate::use_cases::print_receipt::PrintReceiptUseCase;

// ==========================================================
// Printer Settings Helper
// ==========================================================

#[derive(Debug, Clone)]
struct ReceiptSettings {
    store_name: String,
    store_address: String,
    store_phone: String,
    auto_print_enabled: bool,
    receipt_preview_enabled: bool,
    paper_size: PaperSize,
    printer_connection: String,
    printer_target: String,
    promptpay_id: Option<String>,
    promptpay_amount_enabled: bool,
    printer_codepage: u8,
    receipt_font: String,
    receipt_footer: String,
}

impl Default for ReceiptSettings {
    fn default() -> Self {
        Self {
            store_name: "Easy Stock".to_string(),
            store_address: String::new(),
            store_phone: String::new(),
            auto_print_enabled: true,
            receipt_preview_enabled: true,
            paper_size: PaperSize::Mm80,
            printer_connection: "network".to_string(),
            printer_target: String::new(),
            promptpay_id: None,
            promptpay_amount_enabled: true,
            printer_codepage: 26,
            receipt_font: "sarabun".to_string(),
            receipt_footer: "ขอบคุณที่ใช้บริการ".to_string(),
        }
    }
}

fn load_receipt_settings(conn: &Connection) -> ReceiptSettings {
    let mut s = ReceiptSettings::default();
    let get = |key: &str| settings_repo::get_setting(conn, key).ok().flatten();

    if let Some(v) = get("store_name") {
        s.store_name = v;
    }
    if let Some(v) = get("store_address") {
        s.store_address = v;
    }
    if let Some(v) = get("store_phone") {
        s.store_phone = v;
    }
    let print_behavior = get("print_behavior");
    s.auto_print_enabled = match print_behavior.as_deref() {
        Some("direct") => true,
        Some("preview") | Some("none") => false,
        _ => get("auto_print_enabled").map(|v| v == "true").unwrap_or(true),
    };
    s.receipt_preview_enabled = match print_behavior.as_deref() {
        Some("preview") => true,
        Some("direct") | Some("none") => false,
        _ => get("receipt_preview_enabled")
            .map(|v| v == "true")
            .unwrap_or(false),
    };
    if let Some(v) = get("paper_size") {
        s.paper_size = PaperSize::from_setting(&v);
    }
    if let Some(v) = get("printer_connection") {
        s.printer_connection = v;
    }
    if let Some(v) = get("printer_target") {
        s.printer_target = v;
    }
    let qr_enabled = get("promptpay_qr_enabled").map(|v| v == "true").unwrap_or(false);
    s.promptpay_id = if qr_enabled {
        get("promptpay_id").filter(|v| !v.trim().is_empty())
    } else {
        None
    };
    let amount_enabled = get("promptpay_amount_enabled").map(|v| v != "false").unwrap_or(true);
    s.promptpay_amount_enabled = amount_enabled;
    if let Some(v) = get("printer_codepage") {
        s.printer_codepage = v.parse::<u8>().unwrap_or(26);
    }
    if let Some(v) = get("receipt_font") {
        s.receipt_font = v;
    }
    if let Some(v) = get("receipt_footer") {
        s.receipt_footer = v;
    }
    s
}

fn build_receipt_data(order: &Order, s: &ReceiptSettings) -> ReceiptData {
    ReceiptData {
        store_name: s.store_name.clone(),
        store_address: s.store_address.clone(),
        store_phone: s.store_phone.clone(),
        order_no: order.order_no.clone(),
        order_date: order.order_date.clone(),
        order_type: order.order_type.clone(),
        items: order
            .items
            .iter()
            .map(|i| ReceiptItem {
                name: i.product_name.clone(),
                quantity: i.quantity,
                unit_price: i.unit_price,
                line_total: i.line_total,
            })
            .collect(),
        subtotal: order.subtotal,
        discount_amount: order.discount_amount,
        total_amount: order.total_amount,
        payment_method: order.payment_method.clone(),
        paid_amount: order.paid_amount,
        change_amount: order.change_amount,
        note: order.note.clone(),
        promptpay_id: s.promptpay_id.clone(),
        promptpay_amount_enabled: s.promptpay_amount_enabled,
        paper_size: s.paper_size,
        codepage: s.printer_codepage,
        receipt_font: s.receipt_font.clone(),
        receipt_footer: s.receipt_footer.clone(),
    }
}

/// พิมพ์เงียบเบื้องหลัง (Non-blocking) — ใช้หลังบิล Commit สำเร็จแล้วเท่านั้น
/// หากพิมพ์ล้มเหลวจะไม่ทำให้ Transaction การขายล้ม (บิลถูกบันทึกไปแล้ว)
fn spawn_background_print(order: Order, settings: ReceiptSettings) {
    std::thread::spawn(move || {
        let conn_type = PrinterConnection::from_setting(&settings.printer_connection);
        if conn_type == PrinterConnection::None {
            return;
        }
        let port = match printer::create_port(
            &conn_type,
            &settings.printer_target,
        ) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Background print skipped: {}", e);
                return;
            }
        };
        let data = build_receipt_data(&order, &settings);
        if let Err(e) = PrintReceiptUseCase::execute(port.as_ref(), &data) {
            eprintln!("Background print failed: {}", e);
        }
    });
}

// ==========================================================
// Checkout (ชำระเงิน)
// ==========================================================

#[tauri::command]
pub fn create_order(
    state: State<'_, Mutex<Connection>>,
    payload: order_repo::CheckoutInput,
) -> Result<Order, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let order = order_repo::create_order(&conn, &payload)?;

    let mut settings = load_receipt_settings(&conn);
    if let Some(pp_amount) = payload.promptpay_amount_enabled {
        settings.promptpay_amount_enabled = pp_amount;
    }
    if settings.auto_print_enabled && !settings.receipt_preview_enabled {
        spawn_background_print(order.clone(), settings);
    }

    Ok(order)
}

// ==========================================================
// Hold Orders (พักบิล)
// ==========================================================

#[tauri::command]
pub fn hold_order(
    state: State<'_, Mutex<Connection>>,
    payload: order_repo::HoldInput,
) -> Result<Order, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    order_repo::hold_order(&conn, &payload)
}

#[tauri::command]
pub fn get_held_orders(state: State<'_, Mutex<Connection>>) -> Result<Vec<Order>, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    order_repo::get_held_orders(&conn)
}

#[tauri::command]
pub fn delete_held_order(
    state: State<'_, Mutex<Connection>>,
    order_id: String,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    order_repo::delete_held_order(&conn, &order_id)
}

// ==========================================================
// Sales History (ประวัติการขาย)
// ==========================================================

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OrderQueryParams {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub page_size: Option<u32>,
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdersPageData {
    pub orders: Vec<Order>,
    pub total_items: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[tauri::command]
pub fn get_orders(
    state: State<'_, Mutex<Connection>>,
    params: Option<OrderQueryParams>,
) -> Result<OrdersPageData, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let qp = params.unwrap_or_default();
    let paginated = order_repo::get_orders_paginated(
        &conn,
        &order_repo::OrderFilterParams {
            page: qp.page.unwrap_or(1),
            page_size: qp.page_size.unwrap_or(10),
            search: qp.search,
        },
    )?;

    Ok(OrdersPageData {
        orders: paginated.orders,
        total_items: paginated.total_items,
        page: paginated.page,
        page_size: paginated.page_size,
        total_pages: paginated.total_pages,
    })
}

#[tauri::command]
pub fn get_order_detail(
    state: State<'_, Mutex<Connection>>,
    order_id: String,
) -> Result<Order, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    order_repo::get_order_detail(&conn, &order_id)
}

/// ค้นหาบิลด้วยเลขที่บิล (ใช้ในหน้า RB)
#[tauri::command]
pub fn get_order_by_no(
    state: State<'_, Mutex<Connection>>,
    order_no: String,
) -> Result<Order, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    order_repo::get_order_by_no(&conn, &order_no)
}

/// สรุปยอดขาย รายวัน - เดือน
#[tauri::command]
pub fn get_sales_summary(
    state: State<'_, Mutex<Connection>>,
    filter: Option<order_repo::SalesSummaryFilter>,
) -> Result<order_repo::SalesSummaryResult, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let f = filter.unwrap_or_default();
    order_repo::get_sales_summary(&conn, &f)
}

/// บันทึกไฟล์ CSV โดยเปิดหน้าต่างให้ผู้ใช้เลือกปลายทางบันทึก (Native Save File Dialog)
#[tauri::command]
pub async fn export_csv_file(
    default_filename: String,
    content: String,
) -> Result<Option<String>, AppError> {
    let dialog = rfd::AsyncFileDialog::new()
        .set_file_name(&default_filename)
        .add_filter("CSV Files (*.csv)", &["csv"]);

    if let Some(file_handle) = dialog.save_file().await {
        let path = file_handle.path().to_path_buf();
        std::fs::write(&path, content.as_bytes())
            .map_err(|e| AppError::Internal(format!("บันทึกไฟล์ไม่สำเร็จ: {}", e)))?;
        Ok(Some(path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

// ==========================================================
// Return Order (RB — รับคืนสินค้า/คืนเงิน)
// ==========================================================

#[tauri::command]
pub fn create_return_order(
    state: State<'_, Mutex<Connection>>,
    payload: order_repo::ReturnInput,
) -> Result<Order, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let order = order_repo::create_return_order(&conn, &payload)?;

    // Auto-Print สลิปคืนเงิน (ตามเงื่อนไขเดียวกับบิลขาย)
    let settings = load_receipt_settings(&conn);
    if settings.auto_print_enabled && !settings.receipt_preview_enabled {
        spawn_background_print(order.clone(), settings);
    }

    Ok(order)
}

// ==========================================================
// Printing (พิมพ์ใบเสร็จ)
// ==========================================================

/// พิมพ์ใบเสร็จตาม order_id — ใช้จากปุ่ม "พิมพ์ใบเสร็จ" ในหน้า Preview และ Reprint
#[tauri::command]
pub fn print_receipt(
    state: State<'_, Mutex<Connection>>,
    order_id: String,
    promptpay_amount_enabled: Option<bool>,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let order = order_repo::get_order_detail(&conn, &order_id)?;
    let mut settings = load_receipt_settings(&conn);
    if let Some(pp_amount) = promptpay_amount_enabled {
        settings.promptpay_amount_enabled = pp_amount;
    }

    let conn_type = PrinterConnection::from_setting(&settings.printer_connection);
    if conn_type == PrinterConnection::None {
        return Err(AppError::Validation("ไม่ได้ระบุเครื่องพิมพ์ (กรุณาตั้งค่าเครื่องพิมพ์ในหน้าตั้งค่าก่อนสั่งพิมพ์)".to_string()));
    }

    let port = printer::create_port(
        &conn_type,
        &settings.printer_target,
    )
    .map_err(AppError::Printer)?;

    let data = build_receipt_data(&order, &settings);
    PrintReceiptUseCase::execute(port.as_ref(), &data).map_err(AppError::Printer)
}

#[derive(Debug, Deserialize)]
pub struct TestPrintPayload {
    pub printer_connection: Option<String>,
    pub printer_target: Option<String>,
    pub paper_size: Option<String>,
    pub promptpay_id: Option<String>,
    pub promptpay_qr_enabled: Option<String>,
    pub promptpay_amount_enabled: Option<String>,
    pub store_name: Option<String>,
    pub store_address: Option<String>,
    pub store_phone: Option<String>,
    pub receipt_footer: Option<String>,
    pub printer_codepage: Option<String>,
    pub receipt_font: Option<String>,
}

/// ทดสอบการเชื่อมต่อเครื่องพิมพ์ — พิมพ์สลิปทดสอบสั้น ๆ จากหน้า Settings
#[tauri::command]
pub fn print_test_receipt(
    state: State<'_, Mutex<Connection>>,
    payload: Option<TestPrintPayload>,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let mut settings = load_receipt_settings(&conn);

    if let Some(p) = payload {
        if let Some(c) = p.printer_connection {
            settings.printer_connection = c;
        }
        if let Some(t) = p.printer_target {
            settings.printer_target = t;
        }
        if let Some(s) = p.paper_size {
            settings.paper_size = PaperSize::from_setting(&s);
        }
        if let Some(pp) = p.promptpay_id {
            settings.promptpay_id = if pp.trim().is_empty() { None } else { Some(pp) };
        }
        if let Some(qr_en) = p.promptpay_qr_enabled {
            if qr_en == "false" {
                settings.promptpay_id = None;
            }
        }
        if let Some(pp_am) = p.promptpay_amount_enabled {
            settings.promptpay_amount_enabled = pp_am != "false";
        }
        if let Some(n) = p.store_name {
            if !n.trim().is_empty() {
                settings.store_name = n;
            }
        }
        if let Some(a) = p.store_address {
            settings.store_address = a;
        }
        if let Some(ph) = p.store_phone {
            settings.store_phone = ph;
        }
        if let Some(rf) = p.receipt_footer {
            settings.receipt_footer = rf;
        }
        if let Some(cp) = p.printer_codepage {
            settings.printer_codepage = cp.parse::<u8>().unwrap_or(26);
        }
        if let Some(rf) = p.receipt_font {
            settings.receipt_font = rf;
        }
    }

    let conn_type = PrinterConnection::from_setting(&settings.printer_connection);
    if conn_type == PrinterConnection::None {
        return Err(AppError::Validation("ไม่ได้ระบุเครื่องพิมพ์ (กรุณาเลือกประเภทการเชื่อมต่อเครื่องพิมพ์ก่อนทดสอบ)".to_string()));
    }

    let port = printer::create_port(
        &conn_type,
        &settings.printer_target,
    )
    .map_err(AppError::Printer)?;

    let data = ReceiptData {
        store_name: settings.store_name.clone(),
        store_address: settings.store_address.clone(),
        store_phone: settings.store_phone.clone(),
        order_no: "TEST-PRINT".to_string(),
        order_date: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        order_type: "SALE".to_string(),
        items: vec![ReceiptItem {
            name: "ทดสอบการพิมพ์ (Test Item)".to_string(),
            quantity: 1,
            unit_price: 0.0,
            line_total: 0.0,
        }],
        subtotal: 0.0,
        discount_amount: 0.0,
        total_amount: 0.0,
        payment_method: "CASH".to_string(),
        paid_amount: 0.0,
        change_amount: 0.0,
        note: Some("สลิปทดสอบเครื่องพิมพ์ใบเสร็จ".to_string()),
        promptpay_id: settings.promptpay_id.clone(),
        promptpay_amount_enabled: settings.promptpay_amount_enabled,
        paper_size: settings.paper_size,
        codepage: settings.printer_codepage,
        receipt_font: settings.receipt_font.clone(),
        receipt_footer: settings.receipt_footer.clone(),
    };

    PrintReceiptUseCase::execute(port.as_ref(), &data).map_err(AppError::Printer)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPayQrResponse {
    pub promptpay_id: String,
    pub promptpay_qr_enabled: bool,
    pub promptpay_amount_enabled: bool,
    pub qr_svg: Option<String>,
}

/// สร้าง PromptPay QR สำหรับแสดงผลบนหน้าจอชำระเงิน POS หรือหน้าพรีวิว
#[tauri::command]
pub fn get_promptpay_qr(
    state: State<'_, Mutex<Connection>>,
    amount: Option<f64>,
    force_amount: Option<bool>,
) -> Result<PromptPayQrResponse, AppError> {
    let conn = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let get = |k: &str| settings_repo::get_setting(&conn, k).ok().flatten();

    let id = get("promptpay_id").unwrap_or_default();
    let qr_enabled = get("promptpay_qr_enabled").map(|v| v == "true").unwrap_or(false);
    let amount_enabled_setting = get("promptpay_amount_enabled").map(|v| v != "false").unwrap_or(true);
    let use_amount = force_amount.unwrap_or(amount_enabled_setting);

    let mut qr_svg = None;
    if !id.trim().is_empty() {
        let final_amount = if use_amount { amount.unwrap_or(0.0) } else { 0.0 };
        if let Ok(payload) = crate::use_cases::print_receipt::promptpay_payload(&id, final_amount) {
            if let Ok(code) = qrcode::QrCode::with_error_correction_level(payload.as_bytes(), qrcode::EcLevel::M) {
                qr_svg = Some(crate::use_cases::print_receipt::qr_to_svg(&code));
            }
        }
    }

    Ok(PromptPayQrResponse {
        promptpay_id: id,
        promptpay_qr_enabled: qr_enabled,
        promptpay_amount_enabled: use_amount,
        qr_svg,
    })
}