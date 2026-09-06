use crate::domain::receipt::{PaperSize, ReceiptData};

// ==========================================================
// ESC/POS Command Constants
// ==========================================================

const ESC: u8 = 0x1B;
const GS: u8 = 0x1D;
const LF: u8 = 0x0A;

/// Code page สำหรับอักษรไทย TIS-620 (ค่าเริ่มต้น 20 บนเครื่องพิมพ์ส่วนใหญ่ในไทย)
const CODEPAGE_TIS620: u8 = 20;

// ==========================================================
// Text Encoding — UTF-8 → TIS-620 (อักษรไทยบนกระดาษความร้อน)
// ==========================================================

pub mod tis620 {
    /// แปลงข้อความ UTF-8 เป็น bytes TIS-620 (ASCII ผ่านตรง, ไทย U+0E01–U+0E5B, ตัวอื่นเป็น '?')
    pub fn encode(text: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(text.len());
        for ch in text.chars() {
            let code = ch as u32;
            if code < 0x80 {
                out.push(code as u8);
            } else if (0x0E01..=0x0E5B).contains(&code) {
                out.push((code - 0x0E01 + 0xA1) as u8);
            } else {
                out.push(b'?');
            }
        }
        out
    }
}

// ==========================================================
// EscPosBuilder — ประกอบ Raw Bytes ใบเสร็จ (Pure Rust ไม่มี I/O)
// ==========================================================

pub struct EscPosBuilder {
    bytes: Vec<u8>,
    width: usize,
    paper: PaperSize,
}

impl EscPosBuilder {
    pub fn new(paper: PaperSize) -> Self {
        let mut builder = Self {
            bytes: Vec::new(),
            width: paper.chars_per_line(),
            paper,
        };
        builder.push_init();
        builder
    }

    fn raw(&mut self, slice: &[u8]) {
        self.bytes.extend_from_slice(slice);
    }

    fn push_text(&mut self, text: &str) {
        self.bytes.extend_from_slice(&tis620::encode(text));
    }

    /// ESC @ (Initialize) + ESC t (Codepage TIS-620) + ESC a 0 (Align Left)
    pub fn push_init(&mut self) {
        self.raw(&[ESC, b'@']);
        self.raw(&[ESC, b't', CODEPAGE_TIS620]);
        self.raw(&[ESC, b'a', 0x00]);
    }

    /// ESC a n — 0 = Left, 1 = Center, 2 = Right
    pub fn push_align(&mut self, align: u8) {
        self.raw(&[ESC, b'a', align]);
    }

    /// ESC ! n — เปิด/ปิดขนาดตัวอักษร (0x30 = Double Width + Double Height)
    pub fn push_emphasis(&mut self, mode: u8) {
        self.raw(&[ESC, b'!', mode]);
    }

    pub fn push_line(&mut self, text: &str) {
        self.push_text(text);
        self.bytes.push(LF);
    }

    /// ข้อความกึ่งกลางบรรทัดเดียว
    pub fn push_centered(&mut self, text: &str) {
        self.push_align(1);
        self.push_line(text);
        self.push_align(0);
    }

    /// หัวบิล — ชื่อร้านขนาดใหญ่กึ่งกลาง
    pub fn push_store_header(&mut self, name: &str, address: &str, phone: &str) {
        self.push_emphasis(0x30); // Double size
        self.push_centered(name);
        self.push_emphasis(0x00);
        if !address.trim().is_empty() {
            self.push_centered(address);
        }
        if !phone.trim().is_empty() {
            self.push_centered(format!("โทร. {}", phone).as_str());
        }
    }

    /// เส้นประแบ่งส่วนเต็มความกว้าง
    pub fn push_separator(&mut self) {
        let line: String = "-".repeat(self.width);
        self.push_line(&line);
    }

    fn truncate_to_width(&self, text: &str, max_chars: usize) -> String {
        text.chars().take(max_chars).collect()
    }

    /// บรรทัดซ้าย-ขวา (label ซ้าย, value ขวาชิดขอบ)
    pub fn push_kv(&mut self, label: &str, value: &str) {
        let value_chars: Vec<char> = value.chars().collect();
        let max_label = self.width.saturating_sub(value_chars.len() + 1).max(1);
        let label = self.truncate_to_width(label, max_label);
        let line = format!(
            "{:<width$}{}",
            label,
            value,
            width = self.width - value_chars.len()
        );
        self.push_line(&line);
    }

    /// บรรทัดสรุปยอดตัวใหญ่ (Double Height) เช่น "ยอดรวม  1,234.00"
    pub fn push_big_kv(&mut self, label: &str, value: &str) {
        let value_chars: Vec<char> = value.chars().collect();
        let max_label = self.width.saturating_sub(value_chars.len() + 1).max(1);
        let label = self.truncate_to_width(label, max_label);
        let line = format!(
            "{:<width$}{}",
            label,
            value,
            width = self.width - value_chars.len()
        );
        self.push_emphasis(0x20); // Double height
        self.push_line(&line);
        self.push_emphasis(0x00);
    }

    /// รายการสินค้า 1 รายการ — 80mm: ตารางชื่อ + (จำนวน x ราคา = ยอด)
    /// 58/57mm: ซ้อน 2 บรรทัดตามสเปก
    pub fn push_item_row(&mut self, name: &str, quantity: i32, unit_price: f64, line_total: f64) {
        let price = format!("{:.2}", unit_price);
        let total = format!("{:.2}", line_total);

        if self.paper == PaperSize::Mm80 {
            // บรรทัดบน: ชื่อสินค้า
            self.push_line(&self.truncate_to_width(name, self.width));
            // บรรทัดล่าง: left "  qty x price" / right "line_total"
            let left = format!("  {} x {}", quantity, price);
            let left = self.truncate_to_width(&left, self.width - total.chars().count() - 1);
            let line = format!(
                "{:<width$}{}",
                left,
                total,
                width = self.width - total.chars().count()
            );
            self.push_line(&line);
        } else {
            // 58/57mm — ซ้อน 2 บรรทัด
            self.push_line(&self.truncate_to_width(name, self.width));
            let left = format!("{} x {}", quantity, price);
            let left = self.truncate_to_width(&left, self.width - total.chars().count() - 1);
            let line = format!(
                "{:<width$}{}",
                left,
                total,
                width = self.width - total.chars().count()
            );
            self.push_line(&line);
        }
    }

    /// เด้งลิ้นชักเก็บเงิน (ESC p)
    pub fn push_cash_drawer(&mut self) {
        self.raw(&[ESC, b'p', 0x00, 0x19, 0xFA]);
    }

    /// Feed กระดาษ + สั่งตัดกระดาษอัตโนมัติ (ESC d + GS V)
    pub fn push_cut(&mut self) {
        self.raw(&[ESC, b'd', 0x03]); // Feed 3 บรรทัด
        self.raw(&[GS, b'V', 0x00]); // Full cut
    }

    pub fn build(self) -> Vec<u8> {
        self.bytes
    }
}

// ==========================================================
// QR Code Raster (GS v 0) — Universal Compatibility
// ==========================================================

/// เรนเดอร์ QR Code เป็น ESC/POS Raster Bit Image แล้วแนบต่อท้าย bytes
/// (แยกเป็นฟังก์ชันอิสระเพื่อให้ unit test ตรวจสอบ raw bytes ได้ง่าย)
pub fn qr_raster_bytes(data: &str, module_size: usize) -> Option<Vec<u8>> {
    use qrcode::{Color, EcLevel, QrCode};

    let code = QrCode::with_error_correction_level(data.as_bytes(), EcLevel::M).ok()?;
    let colors = code.to_colors();
    let matrix_width = code.width();
    let scaled = matrix_width * module_size;
    let row_bytes = (scaled + 7) / 8;

    let mut bytes = vec![GS, b'v', b'0'];
    bytes.extend_from_slice(&[
        (row_bytes % 256) as u8,
        (row_bytes / 256) as u8,
        (scaled % 256) as u8,
        (scaled / 256) as u8,
    ]);

    for y in 0..scaled {
        let src_y = y / module_size;
        for byte_index in 0..row_bytes {
            let mut byte: u8 = 0;
            for bit in 0..8 {
                let x = byte_index * 8 + bit;
                if x < scaled {
                    let src_x = x / module_size;
                    if colors[src_y * matrix_width + src_x] == Color::Dark {
                        byte |= 0x80 >> bit;
                    }
                }
            }
            bytes.push(byte);
        }
    }
    Some(bytes)
}

// ==========================================================
// PromptPay QR Payload (EMVCo) + CRC16-CCITT
// ==========================================================

/// CRC16-CCITT (FALSE): init 0xFFFF, poly 0x1021 — ใช้ตรวจสอบ Payload ของ PromptPay
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

fn emv_tlv(tag: &str, value: &str) -> String {
    format!("{}{:02}{}", tag, value.chars().count(), value)
}

fn normalize_promptpay_id(id: &str) -> Result<(String, String), String> {
    let digits: String = id.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return Err("เลข PromptPay ไม่ถูกต้อง".to_string());
    }
    match digits.len() {
        // เบอร์โทรศัพท์ 10 หลัก (ขึ้นต้น 0) → 0066 + เบอร์ตัด 0 หน้า
        10 if digits.starts_with('0') => Ok(("01".to_string(), format!("0066{}", &digits[1..]))),
        // เบอร์ที่มีรหัสประเทศแล้ว (0066 + 9 หลัก)
        13 if digits.starts_with("0066") => Ok(("01".to_string(), digits)),
        // เลขบัตรประชาชน 13 หลัก / เลขผู้เสียภาษี 15 หลัก
        13 | 15 => Ok(("02".to_string(), digits)),
        _ => Err(format!(
            "เลข PromptPay '{}' ไม่ถูกฟอร์แมต (รองรับเบอร์โทร 10 หลัก, เลขบัตร 13 หลัก, เลขผู้เสียภาษี 15 หลัก)",
            id
        )),
    }
}

/// สร้าง PromptPay QR Payload (EMVCo) พร้อม CRC — ใช้สร้าง QR บนใบเสร็จ
pub fn promptpay_payload(id: &str, amount: f64) -> Result<String, String> {
    let (target_tag, target_value) = normalize_promptpay_id(id)?;

    let mut payload = String::from("000201");
    if amount > 0.0 {
        payload.push_str(&emv_tlv("01", "12")); // Dynamic (มียอดเงิน)
    } else {
        payload.push_str(&emv_tlv("01", "11")); // Static
    }

    let merchant_account = emv_tlv("00", "A000000677010111") + &emv_tlv(&target_tag, &target_value);
    payload.push_str(&emv_tlv("29", &merchant_account));
    payload.push_str(&emv_tlv("53", "764")); // สกุลเงิน THB
    if amount > 0.0 {
        payload.push_str(&emv_tlv("54", &format!("{:.2}", amount)));
    }
    payload.push_str(&emv_tlv("58", "TH")); // ประเทศ

    // CRC16 คำนวณจาก payload รวม tag "6304"
    payload.push_str("6304");
    let crc = crc16_ccitt(payload.as_bytes());
    payload.push_str(&format!("{:04X}", crc));
    Ok(payload)
}

// ==========================================================
// PrintReceiptUseCase — ประกอบใบเสร็จทั้งใบจาก ReceiptData
// ==========================================================

pub struct PrintReceiptUseCase;

fn payment_label(method: &str) -> &str {
    match method {
        "PROMPTPAY" => "พร้อมเพย์",
        "TRANSFER" => "โอนเงิน",
        _ => "เงินสด",
    }
}

impl PrintReceiptUseCase {
    /// ประกอบ Raw Bytes ของใบเสร็จทั้งใบ (สำหรับทดสอบและส่งให้ PrinterPort)
    pub fn build_receipt_bytes(data: &ReceiptData) -> Vec<u8> {
        let mut b = EscPosBuilder::new(data.paper_size);

        // หัวบิล
        b.push_store_header(&data.store_name, &data.store_address, &data.store_phone);
        b.push_separator();

        // ประเภทบิล
        if data.order_type == "RETURN" {
            b.push_centered("** ใบเสร็จรับเงิน (คืนสินค้า) **");
        }

        // เลขที่บิล / วันที่
        b.push_kv("เลขที่", &data.order_no);
        b.push_kv("วันที่", &data.order_date);
        b.push_separator();

        // รายการสินค้า
        for item in &data.items {
            b.push_item_row(&item.name, item.quantity, item.unit_price, item.line_total);
        }
        b.push_separator();

        // สรุปยอด
        b.push_kv("รวมย่อย", &format!("{:.2}", data.subtotal));
        if data.discount_amount > 0.0 {
            b.push_kv("ส่วนลด", &format!("-{:.2}", data.discount_amount));
        }
        b.push_big_kv("ยอดรวม", &format!("{:.2}", data.total_amount));
        b.push_kv("ชำระ", payment_label(&data.payment_method));
        b.push_kv("รับมา", &format!("{:.2}", data.paid_amount));
        if data.change_amount > 0.0 {
            b.push_kv("เงินทอน", &format!("{:.2}", data.change_amount));
        }

        if let Some(ref note) = data.note {
            if !note.trim().is_empty() {
                b.push_separator();
                b.push_kv("หมายเหตุ", note);
            }
        }

        // QR PromptPay เฉพาะบิลขายที่มีเลข PromptPay และมียอดรับชำระ
        if data.order_type == "SALE" {
            if let Some(ref pp_id) = data.promptpay_id {
                if !pp_id.trim().is_empty() && data.total_amount > 0.0 {
                    if let Ok(payload) = promptpay_payload(pp_id, data.total_amount) {
                        b.push_separator();
                        b.push_centered("สแกนชำระเงินผ่าน PromptPay");
                        if let Some(raster) = qr_raster_bytes(&payload, data.paper_size.qr_module_size()) {
                            b.push_align(1);
                            b.raw(&raster);
                            b.raw(&[LF, LF]);
                            b.push_align(0);
                        }
                    }
                }
            }
        }

        // ท้ายบิล + ตัดกระดาษ
        b.push_separator();
        b.push_centered("ขอบคุณที่ใช้บริการ");
        b.push_cash_drawer();
        b.push_cut();

        b.build()
    }

    /// ส่งใบเสร็จเข้าเครื่องพิมพ์ผ่าน PrinterPort (Infrastructure Layer)
    pub fn execute(
        port: &dyn crate::domain::receipt::PrinterPort,
        data: &ReceiptData,
    ) -> Result<(), String> {
        let bytes = Self::build_receipt_bytes(data);
        port.send_raw(&bytes)
    }
}

// ==========================================================
// Unit Tests — ตรวจสอบ Raw Bytes โดยไม่ต้องต่อเครื่องพิมพ์จริง
// ==========================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::receipt::ReceiptItem;

    fn sample_data(paper: PaperSize) -> ReceiptData {
        ReceiptData {
            store_name: "ร้านทดสอบ".to_string(),
            store_address: "กรุงเทพฯ".to_string(),
            store_phone: "021234567".to_string(),
            order_no: "ORD-20260906-0001".to_string(),
            order_date: "2026-09-06 10:00".to_string(),
            order_type: "SALE".to_string(),
            items: vec![ReceiptItem {
                name: "น้ำดื่ม 500ml".to_string(),
                quantity: 2,
                unit_price: 7.0,
                line_total: 14.0,
            }],
            subtotal: 14.0,
            discount_amount: 0.0,
            total_amount: 14.0,
            payment_method: "CASH".to_string(),
            paid_amount: 20.0,
            change_amount: 6.0,
            note: None,
            promptpay_id: None,
            paper_size: paper,
        }
    }

    #[test]
    fn test_crc16_ccitt_known_vector() {
        // ค่ามาตรฐาน CRC16-CCITT (FALSE) ของ "123456789"
        assert_eq!(crc16_ccitt(b"123456789"), 0x29B1);
    }

    #[test]
    fn test_promptpay_payload_structure() {
        let payload = promptpay_payload("0812345678", 100.0).unwrap();
        // Payload Format Indicator + Dynamic QR
        assert!(payload.starts_with("000201010212"));
        // Merchant Account (AID PromptPay)
        assert!(payload.contains("A000000677010111"));
        // เบอร์ 0812345678 → TLV(01, "0066812345678") = 01130066812345678
        assert!(payload.contains("01130066812345678"));
        // สกุลเงิน THB + ยอดเงิน
        assert!(payload.contains("5303764"));
        assert!(payload.contains("5406100.00"));
        // CRC 4 หลัก hex ปิดท้าย
        let crc = &payload[payload.len() - 4..];
        assert!(crc.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_promptpay_tax_id() {
        let payload = promptpay_payload("1234567890123", 0.0).unwrap();
        assert!(payload.contains("02131234567890123"));
        // ยอด 0 → Static QR (010211) และไม่มี tag 54
        assert!(payload.contains("010211"));
        assert!(!payload.contains("5406"));
    }

    #[test]
    fn test_promptpay_invalid_id() {
        assert!(promptpay_payload("abc", 10.0).is_err());
        assert!(promptpay_payload("123", 10.0).is_err());
    }

    #[test]
    fn test_tis620_encoding() {
        // 'ก' U+0E01 → 0xA1
        assert_eq!(tis620::encode("ก"), vec![0xA1]);
        // ASCII ผ่านตรง
        assert_eq!(tis620::encode("ABC"), vec![65, 66, 67]);
        // ฿ U+0E3F → 0xDF
        assert_eq!(tis620::encode("฿"), vec![0xDF]);
        // ตัวอักษรอื่นแทนด้วย '?'
        assert_eq!(tis620::encode("é"), vec![b'?']);
    }

    #[test]
    fn test_escpos_init_sequence() {
        let bytes = EscPosBuilder::new(PaperSize::Mm80).build();
        // เริ่มด้วย ESC @ + ESC t 20 + ESC a 0
        assert_eq!(&bytes[0..6], &[0x1B, b'@', 0x1B, b't', 20, 0x1B]);
        assert_eq!(&bytes[6..8], &[b'a', 0x00]);
    }

    #[test]
    fn test_receipt_layout_80mm() {
        let data = sample_data(PaperSize::Mm80);
        let bytes = PrintReceiptUseCase::build_receipt_bytes(&data);

        // จบด้วยคำสั่งตัดกระดาษ GS V 0
        assert!(bytes.ends_with(&[GS, b'V', 0x00]));
        // มีคำสั่งเด้งลิ้นชัก ESC p
        assert!(bytes.windows(5).any(|w| w == [ESC, b'p', 0x00, 0x19, 0xFA]));
        // เส้นประยาวเท่าความกว้างกระดาษ (42) และจบบรรทัดด้วย LF
        let mut sep: Vec<u8> = vec![b'-'; 42];
        sep.push(LF);
        assert!(bytes.windows(sep.len()).any(|w| w == sep.as_slice()));
        // ยอดรวมและเงินทอนปรากฏในสลิป
        assert!(bytes.windows(5).any(|w| w == b"14.00"));
        assert!(bytes.windows(4).any(|w| w == b"6.00"));
    }

    #[test]
    fn test_receipt_58mm_narrow_lines() {
        let data = sample_data(PaperSize::Mm58);
        let bytes = PrintReceiptUseCase::build_receipt_bytes(&data);
        // 58mm กว้าง 30 ตัวอักษร: เส้นประ 30 ขีด
        let mut sep: Vec<u8> = vec![b'-'; 30];
        sep.push(LF);
        assert!(bytes.windows(sep.len()).any(|w| w == sep.as_slice()));
        assert!(bytes.ends_with(&[GS, b'V', 0x00]));
    }

    #[test]
    fn test_qr_raster_header() {
        let bytes = qr_raster_bytes("TEST-PAYLOAD", 4).expect("QR raster ต้องสร้างสำเร็จ");
        // ขึ้นต้นด้วย GS v 0
        assert_eq!(&bytes[0..3], &[0x1D, b'v', b'0']);
        // Header: xL xH (bytes/row) yL yH (rows) เริ่มที่ byte ที่ 3
        let row_bytes = u16::from(bytes[3]) | (u16::from(bytes[4]) << 8);
        let rows = u16::from(bytes[5]) | (u16::from(bytes[6]) << 8);
        // จำนวน bytes ตรงกับ header (3 bytes "GS v 0" + 4 bytes header + ข้อมูลภาพ)
        assert_eq!(bytes.len(), 7 + row_bytes as usize * rows as usize);
        // Module size 4 → ความสูงภาพต้องเป็นจำนวนเต็มของ 4
        assert_eq!(rows % 4, 0);
    }

    #[test]
    fn test_return_receipt_has_marker() {
        let mut data = sample_data(PaperSize::Mm80);
        data.order_type = "RETURN".to_string();
        data.order_no = "RET-20260906-0001".to_string();

        let bytes = PrintReceiptUseCase::build_receipt_bytes(&data);
        // สลิปคืนเงินต้องมีข้อความ "(คืนสินค้า)" ใน TIS-620
        let marker = tis620::encode("(คืนสินค้า)");
        assert!(bytes.windows(marker.len()).any(|w| w == marker.as_slice()));
    }
}