use serde::{Deserialize, Serialize};

// ==========================================================
// Receipt (ใบเสร็จ) — Domain Entities (Pure, ไม่มี I/O)
// ตามสถาปัตยกรรม Thermal Printer (.agents/rules/thermal_printer.md)
// ==========================================================

/// ขนาดกระดาษความร้อนที่รองรับ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperSize {
    Mm80,
    Mm58,
    Mm57,
}

impl PaperSize {
    /// แปลงจากค่า setting ("80" | "58" | "57")
    pub fn from_setting(value: &str) -> PaperSize {
        match value.trim() {
            "58" => PaperSize::Mm58,
            "57" => PaperSize::Mm57,
            _ => PaperSize::Mm80,
        }
    }

    /// ความกว้างสูงสุดเป็นจำนวนตัวอักษรต่อบรรทัด
    /// (80mm: Font A 42 chars, 58/57mm: Font B 30 chars)
    pub fn chars_per_line(&self) -> usize {
        match self {
            PaperSize::Mm80 => 42,
            PaperSize::Mm58 | PaperSize::Mm57 => 30,
        }
    }

    /// ขนาดจุด (module size) สำหรับ QR Code — ปรับตามหน้ากว้างกระดาษ
    pub fn qr_module_size(&self) -> usize {
        match self {
            PaperSize::Mm80 => 6,
            PaperSize::Mm58 | PaperSize::Mm57 => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptItem {
    pub name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub line_total: f64,
}

/// ข้อมูลทั้งหมดที่ใช้เรนเดอร์ใบเสร็จหนึ่งใบ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptData {
    pub store_name: String,
    pub store_address: String,
    pub store_phone: String,
    pub order_no: String,
    pub order_date: String,
    /// SALE หรือ RETURN
    pub order_type: String,
    pub items: Vec<ReceiptItem>,
    pub subtotal: f64,
    pub discount_amount: f64,
    pub total_amount: f64,
    pub payment_method: String,
    pub paid_amount: f64,
    pub change_amount: f64,
    pub note: Option<String>,
    /// เลข PromptPay สำหรับสร้าง QR บนสลิป (ถ้ามี)
    pub promptpay_id: Option<String>,
    /// กำหนดให้ระบุยอดเงินใน PromptPay QR ตามยอดบิลหรือไม่ (true = Dynamic, false = Static)
    #[serde(default = "default_true")]
    pub promptpay_amount_enabled: bool,
    pub paper_size: PaperSize,
    /// Code page สำหรับภาษาไทย (ค่าเริ่มต้น 26 = TIS18 บน Epson)
    #[serde(default = "default_codepage")]
    pub codepage: u8,
    /// รูปแบบฟอนต์ใบเสร็จ ("sarabun" หรือ "device")
    #[serde(default = "default_receipt_font")]
    pub receipt_font: String,
}

fn default_true() -> bool {
    true
}

fn default_codepage() -> u8 {
    26
}

fn default_receipt_font() -> String {
    "sarabun".to_string()
}

/// Trait พอร์ตเครื่องพิมพ์ — Implementation อยู่ที่ Infrastructure Layer
pub trait PrinterPort: Send {
    fn send_raw(&self, bytes: &[u8]) -> Result<(), String>;
}