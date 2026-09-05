# Thermal Receipt & Auto-Print Architecture (Native ESC/POS)

## 1. Architectural Strategy & Decision
ระบบออกใบเสร็จอัตโนมัติหลังชำระเงิน (Auto-Print) ถูกกำหนดให้ขับเคลื่อนผ่าน **Backend Native ESC/POS (Rust)** แทนการใช้ Frontend Webview (`window.print()`) ด้วยเหตุผลทางสถาปัตยกรรมดังนี้:
* **Silent Background Printing 100%:** สั่งพิมพ์งานเบื้องหลังได้ทันทีโดยไม่ต้องพึ่งพา Print Dialog ของเบราว์เซอร์/OS และไม่ขัดจังหวะการขายหน้าร้านของแคชเชียร์
* **Backend-Driven with App Settings Control:** 
  * Backend เป็นผู้จัดเตรียมข้อมูลและสั่งพิมพ์อัตโนมัติเมื่อ Transaction ของ SQLite (`orders`, `order_items`, `activity_logs`) ถูก Commit สำเร็จ
  * มีการตรวจสอบการตั้งค่าของแอปพลิเคชัน (`settings` table เช่น `auto_print_enabled: bool`, `printer_target: string`, `paper_size: string`) ก่อนพิมพ์เสมอ หากผู้ใช้ปิดการพิมพ์อัตโนมัติ ระบบจะข้ามขั้นตอนการส่งพิมพ์โดยไม่เกิดข้อผิดพลาด
  * รองรับคำสั่งพิมพ์ย้อนหลัง (Manual Reprint) จาก Frontend ผ่าน IPC Command กรณีต้องการพิมพ์บิลซ้ำ
* **Hardware-Level Control:** ควบคุมใบมีดตัดกระดาษอัตโนมัติ (Auto-Cutter: `GS V`), ฟีดกระดาษ และการสั่งเด้งลิ้นชักเก็บเงิน (Cash Drawer: `ESC p`) ได้แม่นยำ
* **Sub-second Performance:** ประกอบ Raw Bytes และยิงตรงเข้าพอร์ตระดับ Native เร็วในระดับมิลลิวินาที ผ่าน Tokio Background Task โดยไม่บล็อก Response การขาย
* **Pixel-Perfect Clarity:** บิลและ QR Code คมชัดระดับหัวพิมพ์ความร้อน (1:1 Native Pixel Ratio) หมดปัญหา Margin เพี้ยนหรือภาพแตกจากการซูมของ Webview

---

## 2. Paper Size & Layout Specifications
ระบบรองรับหน้ากว้างกระดาษความร้อน 3 ขนาดหลัก ผ่านการคำนวณ Character Grid และ Margin ในระดับ Rust Engine:

| ขนาดกระดาษ | หน้ากว้างพิมพ์จริง | ตัวอักษร/บรรทัด (Font A / Font B) | รูปแบบ Layout รายการสินค้า |
| :--- | :--- | :--- | :--- |
| **80 mm** | ~72 mm (576 dots) | 42 – 48 chars (Font A) | **ตาราง 4 คอลัมน์:** ชื่อสินค้า, จำนวน, ราคา, ยอดรวม |
| **58 mm** | ~48 mm (384 dots) | 30 – 32 chars (Font B) | **ซ้อน 2 บรรทัด:** ชื่อสินค้า (บรรทัดบน) / จำนวน x ราคา + ยอดรวม (บรรทัดล่าง) |
| **57 mm** | ~46–48 mm (384 dots) | 30 – 32 chars (Font B) | **ซ้อน 2 บรรทัด:** โครงสร้างเดียวกับ 58mm โดยตัดระยะขอบข้าง (Margin) ออก |

---

## 3. QR Code Rendering Pipeline (Universal Compatibility)
เพื่อหลีกเลี่ยงข้อจำกัดของเครื่องพิมพ์บางรุ่น (โดยเฉพาะเครื่องพิมพ์ความร้อนแบบ OEM/จีน) ที่ไม่รองรับคำสั่งเฉพาะทาง (`GS ( k`):
* **Raster Bit Image (`GS v 0`):** Rust Backend จะสร้าง Matrix ขาว-ดำ ผ่าน Crate `qrcode` แล้วแปลงเป็น Raster Bitmap Stream 1 บิตต่อพิกเซล
* **Adaptive Module Size:** ปรับขนาดจุดพิกเซลตามขนาดกระดาษอัตโนมัติ (ขนาด 80mm ใช้ Module Size 4–5 / ขนาด 58mm และ 57mm ใช้ Module Size 3–4) เพื่อให้กล้องมือถือสแกน PromptPay หรือ URL ยืนยันบิลได้ง่ายและไม่ล้นขอบกระดาษ

---

## 4. Implementation in Clean Architecture (Rust Backend)

การออกแบบระบบพิมพ์ถูกจัดวางตามแนวคิด Clean Architecture 4 เลเยอร์:

```text
[Frontend: Svelte UI]
│
├── invoke('create_order', payload) ──► Checkout Loop
└── invoke('reprint_receipt', { order_id }) ──► Manual Reprint
│
▼
[Interface Adapters: commands.rs]
│
▼
[Use Cases Layer: Checkout / PrintReceiptUseCase]
│  ├── 1. ตรวจสอบ Setting ใน App (auto_print_enabled)
│  ├── 2. ดึงข้อมูล Order + Company Header จาก Database
│  └── 3. ส่งข้อมูลให้ EscPosBuilder (Use Case Formatter)
│       ├── จัดวางฟอนต์ตามขนาดกระดาษ (Font A / Font B)
│       ├── จัด Format คอลัมน์รายการสินค้า
│       └── เรนเดอร์ QR Code เป็น ESC/POS Raster Bit Image (GS v 0)
▼
[Infrastructure Layer: Printer Port & Adapters]
├── Network Adapter (Cross-OS TCP Socket Port 9100)
├── Windows Adapter (WinSpool Raw Print via #[cfg(target_os = "windows")])
└── Linux Adapter (/dev/usb/lp0 หรือ CUPS via #[cfg(target_os = "linux")])
```

* **Domain Layer:**
  * กำหนด Entity และ Data Contract บริสุทธิ์ เช่น `ReceiptData`, `ReceiptItem`, `PaperSize` และ Trait `PrinterPort` (Pure Rust โดยไม่มี I/O)
* **Use Cases Layer (`use_cases/print_receipt/`):**
  * `PrintReceiptUseCase`: ดึงค่าคอนฟิกจาก `SettingsRepository` หาก `auto_print_enabled == true` จึงทำการสั่งพิมพ์
  * `EscPosBuilder`: มี Logic การจัดข้อความ, ตารางบิล, คำนวณความกว้างบรรทัด และการแปลง Bit Image เป็น Pure Rust (สามารถเขียน **Unit Test** ตรวจสอบ Raw Bytes ที่สร้างขึ้นได้อย่างอิสระโดยไม่ต้องต่อเครื่องพิมพ์จริง)
* **Interface Adapters Layer (`adapters/commands.rs`):**
  * มีคำสั่ง `reprint_receipt` สำหรับกรณีต้องการพิมพ์บิลย้อนหลัง และ `update_printer_settings` สำหรับเปิด/ปิดและเปลี่ยนขนาดกระดาษ
  * หากการพิมพ์ล้มเหลว (เช่น กระดาษหมด หรือเครื่องพิมพ์ออฟไลน์) จะแปลงเป็น Custom Error Code เช่น `ERR_PRINTER_OFFLINE` โดยไม่ทำให้ Transaction การขายที่ Commit ไปแล้วล้มเหลว
* **Infrastructure Layer (`adapters/printer/`):**
  * รองรับเฉพาะ Desktop Target ตามข้อกำหนดโปรเจกต์ (**Windows** และ **Linux**) รวมทั้ง Network/Wi-Fi (TCP Port 9100) ผ่าน Conditional Compilation (`#[cfg(target_os = "...")]`)

---

## 5. Frontend Integration & Settings Management
* **Printer Settings State:** หน้าบ้านสามารถตั้งค่าผ่านหน้า Settings (บันทึกและอ่านผ่าน IPC):
  ```typescript
  export interface PrinterSettings {
    autoPrintEnabled: boolean; // เปิด/ปิดการพิมพ์อัตโนมัติหลัง Checkout
    paperSize: '80' | '58' | '57'; // ขนาดกระดาษ
    connectionType: 'network' | 'usb';
    targetAddress: string; // เช่น "192.168.1.200:9100" หรือ Printer Name
  }
  ```
* **Decoupled Checkout Loop:** 
  * เมื่อแคชเชียร์กดยืนยันชำระเงิน Frontend เรียกเพียง `invoke('create_order', payload)`
  * Backend จะตรวจสอบ Setting เอง หากเปิดใช้งาน Auto-Print Backend จะ Spawn Tokio Task ส่งงานเข้าเครื่องพิมพ์ทันที
  * Frontend ได้รับ Response ยืนยันสำเร็จทันที เคลียร์ตะกร้าสินค้า และพร้อมรับลูกค้ารายต่อไปได้ทันที (Non-blocking UI)
  * กรณีต้องการพิมพ์บิลซ้ำ แคชเชียร์สามารถกดปุ่ม "พิมพ์บิลซ้ำ" เพื่อเรียก `invoke('reprint_receipt', { orderId })` ได้
