use crate::domain::receipt::{PaperSize, ReceiptData};

// ==========================================================
// ESC/POS Command Constants
// ==========================================================

const ESC: u8 = 0x1B;
const GS: u8 = 0x1D;
const LF: u8 = 0x0A;

/// Code page สำหรับอักษรไทย TIS-620/TIS-18 บนเครื่องพิมพ์ ESC/POS
/// ค่าเริ่มต้น 26 (TIS18 3-pass บน Epson TM Series)
pub const DEFAULT_CODEPAGE: u8 = 26;

pub const SARABUN_REGULAR_BYTES: &[u8] = include_bytes!("../../../static/fonts/printer/Sarabun-Regular.ttf");
pub const SARABUN_BOLD_BYTES: &[u8] = include_bytes!("../../../static/fonts/printer/Sarabun-Bold.ttf");

#[cfg(target_os = "windows")]
pub mod gdi_raster {
    use super::*;
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::Graphics::Gdi::*;
    use qrcode::{Color, EcLevel, QrCode};

    static FONTS_LOADED: OnceLock<()> = OnceLock::new();

    pub fn ensure_fonts() {
        FONTS_LOADED.get_or_init(|| {
            unsafe {
                let mut c1 = 0u32;
                AddFontMemResourceEx(
                    SARABUN_REGULAR_BYTES.as_ptr() as *const _,
                    SARABUN_REGULAR_BYTES.len() as u32,
                    std::ptr::null_mut(),
                    &mut c1,
                );
                let mut c2 = 0u32;
                AddFontMemResourceEx(
                    SARABUN_BOLD_BYTES.as_ptr() as *const _,
                    SARABUN_BOLD_BYTES.len() as u32,
                    std::ptr::null_mut(),
                    &mut c2,
                );
            }
        });
    }

    unsafe fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe fn create_sarabun_font(height_px: i32, bold: bool) -> HFONT {
        let font_name = to_wide("Sarabun");
        CreateFontW(
            -height_px,
            0,
            0,
            0,
            if bold { 700 } else { 400 },
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32,
            (DEFAULT_PITCH | FF_DONTCARE) as u32,
            font_name.as_ptr(),
        )
    }

    struct ReceiptFonts {
        store_title: HFONT,
        section_title: HFONT,
        body_bold: HFONT,
        body: HFONT,
        small: HFONT,
    }

    impl ReceiptFonts {
        unsafe fn create(paper: PaperSize) -> Self {
            let scale = match paper {
                PaperSize::Mm80 => 1.0f32,
                PaperSize::Mm58 | PaperSize::Mm57 => 0.80f32,
            };
            Self {
                store_title: create_sarabun_font((34.0 * scale) as i32, true),
                section_title: create_sarabun_font((26.0 * scale) as i32, true),
                body_bold: create_sarabun_font((24.0 * scale) as i32, true),
                body: create_sarabun_font((22.0 * scale) as i32, false),
                small: create_sarabun_font((19.0 * scale) as i32, false),
            }
        }

        unsafe fn cleanup(&self) {
            DeleteObject(self.store_title);
            DeleteObject(self.section_title);
            DeleteObject(self.body_bold);
            DeleteObject(self.body);
            DeleteObject(self.small);
        }
    }

    unsafe fn draw_text_line(
        hdc: HDC,
        text: &str,
        font: HFONT,
        left: i32,
        top: i32,
        right: i32,
        align: u32,
        is_draw: bool,
    ) -> i32 {
        let wide = to_wide(text);
        let old_font = SelectObject(hdc, font);
        let mut rc = RECT {
            left,
            top,
            right,
            bottom: top + 1000,
        };
        let flags = align | DT_NOPREFIX;
        let drawn_h = DrawTextW(
            hdc,
            wide.as_ptr(),
            wide.len() as i32 - 1,
            &mut rc,
            if is_draw { flags } else { flags | DT_CALCRECT },
        );
        SelectObject(hdc, old_font);
        drawn_h
    }

    unsafe fn draw_dashed_line(
        hdc: HDC,
        left: i32,
        right: i32,
        y: i32,
        is_draw: bool,
    ) -> i32 {
        if is_draw {
            let hpen = CreatePen(PS_SOLID as i32, 2, 0x00000000);
            let old_pen = SelectObject(hdc, hpen);
            let dash = 8;
            let gap = 5;
            let mut x = left;
            while x < right {
                let end_x = (x + dash).min(right);
                MoveToEx(hdc, x, y, std::ptr::null_mut());
                LineTo(hdc, end_x, y);
                x += dash + gap;
            }
            SelectObject(hdc, old_pen);
            DeleteObject(hpen);
        }
        18 // Vertical space consumed
    }

    unsafe fn layout_receipt(
        hdc: HDC,
        data: &ReceiptData,
        fonts: &ReceiptFonts,
        width: i32,
        margin: i32,
        is_draw: bool,
    ) -> i32 {
        let mut y = 14;
        let left = margin;
        let right = width - margin;

        // 1. หัวบิล — ชื่อร้านค้า (Bold)
        let h = draw_text_line(hdc, &data.store_name, fonts.store_title, left, y, right, DT_CENTER, is_draw);
        y += h + 6;

        // ที่อยู่ร้าน
        if !data.store_address.trim().is_empty() {
            let h = draw_text_line(hdc, &data.store_address, fonts.small, left, y, right, DT_CENTER | DT_WORDBREAK, is_draw);
            y += h + 4;
        }

        // โทรศัพท์
        if !data.store_phone.trim().is_empty() {
            let text = format!("โทร. {}", data.store_phone.trim());
            let h = draw_text_line(hdc, &text, fonts.small, left, y, right, DT_CENTER, is_draw);
            y += h + 6;
        }

        // เส้นคั่น
        y += draw_dashed_line(hdc, left, right, y + 4, is_draw);

        // ประเภทบิลคืนสินค้า
        if data.order_type == "RETURN" {
            let h = draw_text_line(hdc, "** ใบเสร็จรับเงิน (คืนสินค้า) **", fonts.section_title, left, y, right, DT_CENTER, is_draw);
            y += h + 6;
        }

        // เลขที่บิล / วันที่
        let lh_body = draw_text_line(hdc, "เลขที่", fonts.body, left, y, right, DT_LEFT, is_draw);
        draw_text_line(hdc, &data.order_no, fonts.body, left, y, right, DT_RIGHT, is_draw);
        y += lh_body + 6;

        let lh_date = draw_text_line(hdc, "วันที่", fonts.body, left, y, right, DT_LEFT, is_draw);
        draw_text_line(hdc, &data.order_date, fonts.body, left, y, right, DT_RIGHT, is_draw);
        y += lh_date + 8;

        // เส้นคั่น
        y += draw_dashed_line(hdc, left, right, y + 4, is_draw);

        // รายการสินค้า
        for item in &data.items {
            // บรรทัดบน: ชื่อสินค้า (ตัดคำอัตโนมัติหากยาว)
            let item_name_h = draw_text_line(hdc, &item.name, fonts.body, left, y, right, DT_LEFT | DT_WORDBREAK, is_draw);
            y += item_name_h + 3;

            // บรรทัดล่าง: จำนวน x ราคา (ซ้าย) / รวม (ขวา)
            let prefix = if data.paper_size == PaperSize::Mm80 { "    " } else { "  " };
            let qty_price = format!("{}{} x {:.2}", prefix, item.quantity, item.unit_price);
            let total_str = format!("{:.2}", item.line_total);

            let row_h = draw_text_line(hdc, &qty_price, fonts.small, left, y, right, DT_LEFT, is_draw);
            draw_text_line(hdc, &total_str, fonts.body_bold, left, y, right, DT_RIGHT, is_draw);
            y += row_h + 8;
        }

        // เส้นคั่น
        y += draw_dashed_line(hdc, left, right, y + 4, is_draw);

        // สรุปยอด
        let h_sub = draw_text_line(hdc, "รวมย่อย", fonts.body, left, y, right, DT_LEFT, is_draw);
        draw_text_line(hdc, &format!("{:.2}", data.subtotal), fonts.body, left, y, right, DT_RIGHT, is_draw);
        y += h_sub + 6;

        if data.discount_amount > 0.0 {
            let h_disc = draw_text_line(hdc, "ส่วนลด", fonts.body, left, y, right, DT_LEFT, is_draw);
            draw_text_line(hdc, &format!("-{:.2}", data.discount_amount), fonts.body, left, y, right, DT_RIGHT, is_draw);
            y += h_disc + 6;
        }

        // ยอดรวม (Bold เด่นชัด)
        let h_tot = draw_text_line(hdc, "ยอดรวม", fonts.section_title, left, y, right, DT_LEFT, is_draw);
        draw_text_line(hdc, &format!("{:.2}", data.total_amount), fonts.section_title, left, y, right, DT_RIGHT, is_draw);
        y += h_tot + 8;

        // ชำระ
        let h_pay = draw_text_line(hdc, "ชำระ", fonts.body, left, y, right, DT_LEFT, is_draw);
        draw_text_line(hdc, payment_label(&data.payment_method), fonts.body, left, y, right, DT_RIGHT, is_draw);
        y += h_pay + 6;

        let h_paid = draw_text_line(hdc, "รับมา", fonts.body, left, y, right, DT_LEFT, is_draw);
        draw_text_line(hdc, &format!("{:.2}", data.paid_amount), fonts.body, left, y, right, DT_RIGHT, is_draw);
        y += h_paid + 6;

        if data.change_amount > 0.0 {
            let h_chg = draw_text_line(hdc, "เงินทอน", fonts.body, left, y, right, DT_LEFT, is_draw);
            draw_text_line(hdc, &format!("{:.2}", data.change_amount), fonts.body, left, y, right, DT_RIGHT, is_draw);
            y += h_chg + 6;
        }

        // หมายเหตุ (ถ้ามี)
        if let Some(ref note) = data.note {
            if !note.trim().is_empty() {
                y += draw_dashed_line(hdc, left, right, y + 4, is_draw);
                let h_note_lbl = draw_text_line(hdc, "หมายเหตุ:", fonts.small, left, y, right, DT_LEFT, is_draw);
                y += h_note_lbl + 2;
                let h_note = draw_text_line(hdc, note.trim(), fonts.small, left + 10, y, right, DT_LEFT | DT_WORDBREAK, is_draw);
                y += h_note + 6;
            }
        }

        // QR PromptPay
        if data.order_type == "SALE" {
            if let Some(ref pp_id) = data.promptpay_id {
                if !pp_id.trim().is_empty() && (data.total_amount > 0.0 || data.order_no == "TEST-PRINT") {
                    if let Ok(payload) = promptpay_payload(pp_id, data.total_amount) {
                        y += draw_dashed_line(hdc, left, right, y + 4, is_draw);
                        let h_qr_lbl = draw_text_line(hdc, "สแกนชำระเงินผ่าน PromptPay", fonts.body, left, y, right, DT_CENTER, is_draw);
                        y += h_qr_lbl + 8;

                        if let Ok(code) = QrCode::with_error_correction_level(payload.as_bytes(), EcLevel::M) {
                            let matrix_w = code.width();
                            let colors = code.to_colors();
                            let mod_size = data.paper_size.qr_module_size();
                            let quiet = 4;
                            let total_qr_dots = (matrix_w + quiet * 2) * mod_size;
                            let start_x = (width - total_qr_dots as i32) / 2;

                            if is_draw {
                                let hbrush = CreateSolidBrush(0x00000000);
                                for my in 0..matrix_w {
                                    for mx in 0..matrix_w {
                                        if colors[my * matrix_w + mx] == Color::Dark {
                                            let rx = start_x + (mx + quiet) as i32 * mod_size as i32;
                                            let ry = y + (my + quiet) as i32 * mod_size as i32;
                                            let rc = RECT {
                                                left: rx,
                                                top: ry,
                                                right: rx + mod_size as i32,
                                                bottom: ry + mod_size as i32,
                                            };
                                            FillRect(hdc, &rc, hbrush);
                                        }
                                    }
                                }
                                DeleteObject(hbrush);
                            }
                            y += total_qr_dots as i32 + 8;
                        }
                    }
                }
            }
        }

        // ท้ายบิล
        y += draw_dashed_line(hdc, left, right, y + 4, is_draw);
        let h_end = draw_text_line(hdc, "ขอบคุณที่ใช้บริการ", fonts.body, left, y, right, DT_CENTER, is_draw);
        y += h_end + 18;

        y // คืนค่าความสูงทั้งหมดของใบเสร็จ (pixels)
    }

    /// เรนเดอร์ใบเสร็จทั้งใบเป็น ESC/POS Raster Bit Image ผ่าน Windows GDI ด้วยฟอนต์ Sarabun
    pub fn render_sarabun_receipt(data: &ReceiptData) -> Result<Vec<u8>, String> {
        ensure_fonts();

        let width: i32 = match data.paper_size {
            PaperSize::Mm80 => 576,
            PaperSize::Mm58 | PaperSize::Mm57 => 384,
        };
        let margin: i32 = match data.paper_size {
            PaperSize::Mm80 => 20,
            PaperSize::Mm58 | PaperSize::Mm57 => 12,
        };
        let row_bytes = (width as usize + 7) / 8;

        unsafe {
            let hdc_screen = CreateCompatibleDC(std::ptr::null_mut());
            if hdc_screen.is_null() {
                return Err("Failed to create GDI DC".to_string());
            }

            let fonts = ReceiptFonts::create(data.paper_size);

            // Pass 1: วัดความสูงรวมของสลิป
            let total_height = layout_receipt(hdc_screen, data, &fonts, width, margin, false);

            // Pass 2: สร้าง DIB Section ขนาดตามความสูงจริง
            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = width;
            bmi.bmiHeader.biHeight = -total_height; // Top-down bitmap
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB as u32;

            let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let hbitmap = CreateDIBSection(
                hdc_screen,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits_ptr,
                std::ptr::null_mut(),
                0,
            );

            if hbitmap.is_null() || bits_ptr.is_null() {
                fonts.cleanup();
                DeleteDC(hdc_screen);
                return Err("Failed to create DIB Section".to_string());
            }

            let old_bmp = SelectObject(hdc_screen, hbitmap);

            // เติมพื้นหลังสีขาว (0xFF)
            let buf_size = (width as usize) * (total_height as usize) * 4;
            let pixel_slice = std::slice::from_raw_parts_mut(bits_ptr as *mut u8, buf_size);
            pixel_slice.fill(0xFF);

            SetBkMode(hdc_screen, TRANSPARENT as i32);
            SetTextColor(hdc_screen, 0x00000000); // Black

            // Pass 3: วาดเนื้อหาทั้งหมดลงบน DIB
            layout_receipt(hdc_screen, data, &fonts, width, margin, true);

            // Pass 4: แปลง 32-bit BGRA เป็น 1-bit Monochrome Raster (MSB first)
            let mut raster = Vec::with_capacity(row_bytes * total_height as usize);
            for cy in 0..total_height as usize {
                for byte_idx in 0..row_bytes {
                    let mut b = 0u8;
                    for bit in 0..8 {
                        let cx = byte_idx * 8 + bit;
                        if cx < width as usize {
                            let off = (cy * width as usize + cx) * 4;
                            let blue = pixel_slice[off] as u32;
                            let green = pixel_slice[off + 1] as u32;
                            let red = pixel_slice[off + 2] as u32;
                            let lum = (red * 299 + green * 587 + blue * 114) / 1000;
                            // จุดสีดำ (เข้มกว่า 200) พิมพ์หมึก (bit 1)
                            if lum < 200 {
                                b |= 0x80 >> bit;
                            }
                        }
                    }
                    raster.push(b);
                }
            }

            // Cleanup GDI objects
            SelectObject(hdc_screen, old_bmp);
            DeleteObject(hbitmap);
            fonts.cleanup();
            DeleteDC(hdc_screen);

            // Pass 5: ประกอบคำสั่ง ESC/POS
            let mut escpos = Vec::with_capacity(raster.len() + 64);
            escpos.extend_from_slice(&[ESC, b'@']); // Initialize printer
            escpos.extend_from_slice(&[ESC, b'a', 0x00]); // Left align

            // พิมพ์เป็น chunk ย่อยละ 512 บรรทัดเพื่อป้องกัน buffer เครื่องพิมพ์ล้น
            let chunk_lines = 512usize;
            let tot_h = total_height as usize;
            for chunk_start in (0..tot_h).step_by(chunk_lines) {
                let lines = (tot_h - chunk_start).min(chunk_lines);
                escpos.extend_from_slice(&[
                    GS, b'v', b'0', 0x00,
                    (row_bytes % 256) as u8,
                    (row_bytes / 256) as u8,
                    (lines % 256) as u8,
                    (lines / 256) as u8,
                ]);
                let start_b = chunk_start * row_bytes;
                let end_b = start_b + lines * row_bytes;
                escpos.extend_from_slice(&raster[start_b..end_b]);
            }

            escpos.extend_from_slice(&[LF, LF]);
            // Cash drawer
            escpos.extend_from_slice(&[ESC, b'p', 0x00, 0x19, 0xFA]);
            // Cut paper
            escpos.extend_from_slice(&[ESC, b'd', 0x03, GS, b'V', 0x00]);

            Ok(escpos)
        }
    }
}

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
                match ch {
                    '“' | '”' => out.push(b'"'),
                    '‘' | '’' => out.push(b'\''),
                    '–' | '—' => out.push(b'-'),
                    '…' => {
                        out.extend_from_slice(b"...");
                    }
                    _ => out.push(b'?'),
                }
            }
        }
        out
    }
}

/// ตรวจสอบว่าเป็นสระ/วรรณยุกต์ไทยที่ไม่กินพื้นที่แนวนอน (Non-spacing combining marks) หรือไม่
pub fn is_thai_combining(c: char) -> bool {
    let u = c as u32;
    // สระบน/ล่าง และวรรณยุกต์ไทย (ั, ิ..ฺ, ็..๎)
    matches!(u, 0x0E31 | 0x0E34..=0x0E3A | 0x0E47..=0x0E4E)
}

/// คำนวณความกว้างที่แท้จริงของการแสดงผลบนกระดาษ (หักสระ/วรรณยุกต์ลอย/จมออก)
pub fn thai_display_width(text: &str) -> usize {
    text.chars().filter(|c| !is_thai_combining(*c)).count()
}

// ==========================================================
// EscPosBuilder — ประกอบ Raw Bytes ใบเสร็จ (Pure Rust ไม่มี I/O)
// ==========================================================

pub struct EscPosBuilder {
    bytes: Vec<u8>,
    width: usize,
    paper: PaperSize,
    codepage: u8,
}

impl EscPosBuilder {
    pub fn new(paper: PaperSize) -> Self {
        Self::with_codepage(paper, DEFAULT_CODEPAGE)
    }

    pub fn with_codepage(paper: PaperSize, codepage: u8) -> Self {
        let mut builder = Self {
            bytes: Vec::new(),
            width: paper.chars_per_line(),
            paper,
            codepage,
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

    /// ESC @ (Initialize) + ESC t (Codepage TIS-18 / TIS-620) + ESC a 0 (Align Left)
    pub fn push_init(&mut self) {
        self.raw(&[ESC, b'@']);
        self.raw(&[ESC, b't', self.codepage]);
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

    fn truncate_to_display_width(&self, text: &str, max_width: usize) -> String {
        let mut cur_width = 0;
        let mut out = String::new();
        for c in text.chars() {
            let is_combining = is_thai_combining(c);
            if !is_combining && cur_width >= max_width {
                break;
            }
            out.push(c);
            if !is_combining {
                cur_width += 1;
            }
        }
        out
    }

    /// บรรทัดซ้าย-ขวา (label ซ้าย, value ขวาชิดขอบ)
    pub fn push_kv(&mut self, label: &str, value: &str) {
        let val_width = thai_display_width(value);
        let max_label = self.width.saturating_sub(val_width + 1).max(1);
        let label = self.truncate_to_display_width(label, max_label);
        let label_width = thai_display_width(&label);
        let spaces = self.width.saturating_sub(label_width + val_width);
        let line = format!("{}{}{}", label, " ".repeat(spaces), value);
        self.push_line(&line);
    }

    /// บรรทัดสรุปยอดตัวใหญ่ (Double Height) เช่น "ยอดรวม  1,234.00"
    pub fn push_big_kv(&mut self, label: &str, value: &str) {
        let val_width = thai_display_width(value);
        let max_label = self.width.saturating_sub(val_width + 1).max(1);
        let label = self.truncate_to_display_width(label, max_label);
        let label_width = thai_display_width(&label);
        let spaces = self.width.saturating_sub(label_width + val_width);
        let line = format!("{}{}{}", label, " ".repeat(spaces), value);
        self.push_emphasis(0x20); // Double height
        self.push_line(&line);
        self.push_emphasis(0x00);
    }

    /// รายการสินค้า 1 รายการ — 80mm: ตารางชื่อ + (จำนวน x ราคา = ยอด)
    /// 58/57mm: ซ้อน 2 บรรทัดตามสเปก
    pub fn push_item_row(&mut self, name: &str, quantity: i32, unit_price: f64, line_total: f64) {
        let price = format!("{:.2}", unit_price);
        let total = format!("{:.2}", line_total);
        let total_width = thai_display_width(&total);

        // บรรทัดบน: ชื่อสินค้า
        self.push_line(&self.truncate_to_display_width(name, self.width));

        // บรรทัดล่าง: left "  qty x price" / right "line_total"
        let prefix = if self.paper == PaperSize::Mm80 { "  " } else { "" };
        let left = format!("{}{} x {}", prefix, quantity, price);
        let max_left = self.width.saturating_sub(total_width + 1);
        let left = self.truncate_to_display_width(&left, max_left);
        let left_width = thai_display_width(&left);
        let spaces = self.width.saturating_sub(left_width + total_width);
        let line = format!("{}{}{}", left, " ".repeat(spaces), total);
        self.push_line(&line);
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
    let quiet_zone = 4; // Quiet zone ตามมาตรฐาน QR Code (4 modules) เพื่อให้กล้องมือถือสแกนได้ทันที
    let total_modules = matrix_width + quiet_zone * 2;
    let scaled = total_modules * module_size;
    let row_bytes = (scaled + 7) / 8;

    // คำสั่ง ESC/POS: GS v 0 m xL xH yL yH
    // m = 0x00 (Normal mode, 1x1 dot scaling)
    // xL, xH = ความกว้างภาพเป็นจำนวนไบต์ (row_bytes = xL + xH * 256)
    // yL, yH = ความสูงภาพเป็นจำนวนจุดดอต (scaled = yL + yH * 256)
    let mut bytes = vec![GS, b'v', b'0', 0x00];
    bytes.extend_from_slice(&[
        (row_bytes % 256) as u8,
        (row_bytes / 256) as u8,
        (scaled % 256) as u8,
        (scaled / 256) as u8,
    ]);

    for y in 0..scaled {
        let mod_y = y / module_size;
        for byte_index in 0..row_bytes {
            let mut byte: u8 = 0;
            for bit in 0..8 {
                let x = byte_index * 8 + bit;
                if x < scaled {
                    let mod_x = x / module_size;
                    if mod_y >= quiet_zone
                        && mod_y < quiet_zone + matrix_width
                        && mod_x >= quiet_zone
                        && mod_x < quiet_zone + matrix_width
                    {
                        let src_y = mod_y - quiet_zone;
                        let src_x = mod_x - quiet_zone;
                        if colors[src_y * matrix_width + src_x] == Color::Dark {
                            byte |= 0x80 >> bit;
                        }
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
    /// ประกอบ Raw Bytes ของใบเสร็จทั้งใบ (เลือกโหมดฟอนต์ตาม data.receipt_font: "sarabun" หรือ "device")
    pub fn build_receipt_bytes(data: &ReceiptData) -> Vec<u8> {
        if data.receipt_font == "sarabun" {
            #[cfg(target_os = "windows")]
            {
                match gdi_raster::render_sarabun_receipt(data) {
                    Ok(bytes) => return bytes,
                    Err(e) => {
                        eprintln!("Sarabun GDI raster failed, fallback to text: {}", e);
                    }
                }
            }
        }
        Self::build_receipt_bytes_text(data)
    }

    /// ประกอบ Raw Bytes ในโหมด Text ดั้งเดิม (ESC/POS TIS-620)
    pub fn build_receipt_bytes_text(data: &ReceiptData) -> Vec<u8> {
        let cp = if data.codepage == 0 { DEFAULT_CODEPAGE } else { data.codepage };
        let mut b = EscPosBuilder::with_codepage(data.paper_size, cp);

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

        // QR PromptPay เฉพาะบิลขายที่มีเลข PromptPay และมียอดรับชำระ (หรือบิลทดสอบ TEST-PRINT)
        if data.order_type == "SALE" {
            if let Some(ref pp_id) = data.promptpay_id {
                if !pp_id.trim().is_empty() && (data.total_amount > 0.0 || data.order_no == "TEST-PRINT") {
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
            codepage: 26,
            receipt_font: "device".to_string(),
        }
    }

    #[test]
    fn test_receipt_sarabun_raster() {
        let mut data = sample_data(PaperSize::Mm80);
        data.receipt_font = "sarabun".to_string();
        data.promptpay_id = Some("0812345678".to_string());
        let bytes = PrintReceiptUseCase::build_receipt_bytes(&data);
        // ต้องเริ่มต้นด้วย ESC @
        assert_eq!(&bytes[0..2], &[0x1B, b'@']);
        #[cfg(target_os = "windows")]
        {
            // บน Windows ต้องเป็นโหมด Raster GS v 0 0
            assert!(bytes.windows(4).any(|w| w == [0x1D, b'v', b'0', 0x00]));
        }
        // จบด้วยตัดกระดาษ GS V 0
        assert!(bytes.ends_with(&[GS, b'V', 0x00]));
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
        // เริ่มด้วย ESC @ + ESC t 26 (TIS18) + ESC a 0
        assert_eq!(&bytes[0..6], &[0x1B, b'@', 0x1B, b't', 26, 0x1B]);
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
        // ขึ้นต้นด้วย GS v 0 m (โดย m = 0x00 สำหรับโหมดมาตรฐาน)
        assert_eq!(&bytes[0..4], &[0x1D, b'v', b'0', 0x00]);
        // Header: xL xH (bytes/row) yL yH (rows) เริ่มที่ byte ที่ 4
        let row_bytes = u16::from(bytes[4]) | (u16::from(bytes[5]) << 8);
        let rows = u16::from(bytes[6]) | (u16::from(bytes[7]) << 8);
        // จำนวน bytes ตรงกับ header (4 bytes "GS v 0 \0" + 4 bytes header + ข้อมูลภาพ)
        assert_eq!(bytes.len(), 8 + row_bytes as usize * rows as usize);
        // Module size 4 → ความสูงภาพต้องเป็นจำนวนเต็มของ 4
        assert_eq!(rows % 4, 0);
    }

    #[test]
    fn test_receipt_with_promptpay_qr() {
        let mut data = sample_data(PaperSize::Mm80);
        data.promptpay_id = Some("0812345678".to_string());
        let bytes = PrintReceiptUseCase::build_receipt_bytes(&data);
        // ต้องมีคำสั่ง GS v 0 0 (0x1D, 'v', '0', 0x00) สำหรับพิมพ์ QR Code
        assert!(bytes.windows(4).any(|w| w == [0x1D, b'v', b'0', 0x00]));
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