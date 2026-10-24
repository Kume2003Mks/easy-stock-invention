---
trigger: model_decision
---

# Full Tax Invoice & A4 Receipt Architecture (Copier / Office Printer)

## 1. Architectural Strategy & Legal Standards
มาตรฐานการออก **"ใบเสร็จรับเงิน"** หรือ **"ใบเสร็จรับเงิน/ใบกำกับภาษีเต็มรูป"** ขนาด A4 สำหรับเครื่องถ่ายเอกสารหรือเครื่องพิมพ์สำนักงาน ตาม **ประมวลรัษฎากร มาตรา 86/4** และประกาศอธิบดีกรมสรรพากร (ฉบับที่ 194–197):
* **Strict Legal Compliance:** บังคับ 8 รายการสำคัญ, ระบุสำนักงานใหญ่/สาขา ทั้งผู้ขายและผู้ซื้อ, กำกับยอดเงินด้วยตัวอักษรไทย (Baht Text)
* **Decoupled Printers:** แยกการตั้งค่าเครื่องพิมพ์สลิปความร้อน (Thermal POS) และเครื่องพิมพ์ A4 ออกจากกันใน Settings
* **Non-blocking Hybrid Print:** พรีวิวแบบ Vector 1:1 บน Svelte UI (`@media print` CSS), พิมพ์ผ่าน WinSpool, และ Export PDF ได้
* **Audit Trail & Snapshot:** บันทึก Snapshot ข้อมูลลูกค้า ณ วันที่ออกเอกสาร และป้องกันการออกซ้ำซ้อนจาก Order เดียวกัน

---

## 2. เปรียบเทียบ: สลิปความร้อน (Thermal POS) vs เครื่องถ่ายเอกสาร (A4 Copier)

| มิติ | Thermal POS (80/58mm) | Full A4 Copier / Laser |
| :--- | :--- | :--- |
| **ประเภทเอกสาร** | บิลสลิป / ใบกำกับภาษีอย่างย่อ (ม.86/6) | ใบเสร็จรับเงิน / ใบกำกับภาษีเต็มรูป (ม.86/4) |
| **กระดาษ & การพิมพ์** | ม้วนความร้อน (ESC/POS Raw Stream) | แผ่น A4 (Vector HTML/CSS, WinSpool, PDF) |
| **ข้อมูลผู้ซื้อ** | ไม่บังคับระบุ | **บังคับ:** ชื่อ, ที่อยู่, เลข 13 หลัก, สนง.ใหญ่/สาขา |
| **การแจกแจงภาษี** | รวมในยอดขาย (VAT Included) | **บังคับแยก:** ยอดก่อนภาษี, VAT 7%, ยอดสุทธิ |
| **ตัวอักษรกำกับ** | ไม่มี | **บังคับมี:** บาทข้อความภาษาไทย |
| **ชุดเอกสาร** | 1 ใบ | **พิมพ์ 2 แผ่น/ชุด:** ต้นฉบับ (ลูกค้า) + สำเนา (ร้านค้า) |
| **ส่วนลงนาม** | ไม่มี | ช่องลงนาม 3 ฝ่าย (ผู้รับเงิน, ผู้มีอำนาจ, ผู้รับสินค้า) |

---

## 3. รายการสำคัญตามกฎหมาย (Mandatory Elements)

| รายการ | แหล่งข้อมูลในระบบ | เงื่อนไขและการแสดงผล |
| :--- | :--- | :--- |
| **1. ข้อมูลผู้ขาย** | `App_Settings` | ชื่อเต็มบุคคล/นิติบุคคล, เลข 13 หลัก, `"สำนักงานใหญ่"` หรือ `"สาขาที่ 00000"` พร้อมที่อยู่ |
| **2. คำแสดงเอกสาร** | คำนวณจากสถานะ VAT | `"ใบเสร็จรับเงิน/ใบกำกับภาษี"` (จด VAT) หรือ `"ใบเสร็จรับเงิน"` + ระบุ `"ต้นฉบับ"` / `"สำเนา"` |
| **3. เลขที่ & เล่มที่** | `Tax_Invoices` | Running Sequence (เช่น `TAX-202610-0001`) และเล่มที่ (`book_no`) |
| **4. วัน เดือน ปี** | `issued_date` | รูปแบบ `DD/MM/YYYY` |
| **5. ข้อมูลผู้ซื้อ** | `Customers` / Snapshot | ชื่อเต็ม, ที่อยู่ตาม ภ.พ.20, เลข 13 หลัก (ตรวจ Mod 11), `"สำนักงานใหญ่"` หรือ `"สาขาที่ xxxxx"` |
| **6. รายการสินค้า** | `Order_Items` | ลำดับ, รหัส/ชื่อสินค้า, จำนวน, หน่วยนับ, ราคา/หน่วย, ส่วนลด, จำนวนเงิน |
| **7. สรุปยอดและภาษี** | `Tax_Invoices` | รวมเป็นเงิน, ส่วนลด, ยอดก่อนภาษี, VAT 7%, ยอดรวมสุทธิ |
| **8. ยอดเงินตัวอักษร** | `domain::baht_text` | ข้อความภาษาไทยกำกับ เช่น `(หนึ่งพันสองร้อยห้าสิบบาทถ้วน)` |
| **+ ส่วนลงนาม** | A4 Template | ช่องลายเซ็น 3 ฝ่าย: ผู้รับเงิน, ผู้มีอำนาจลงนาม, ผู้รับสินค้า/บริการ |

---

## 4. โครงสร้างฐานข้อมูล (Database Schema)

```sql
-- 1. Master Data ลูกค้า/นิติบุคคล
CREATE TABLE IF NOT EXISTS Customers (
    customer_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    tax_id TEXT,
    branch_type TEXT DEFAULT 'HEAD' CHECK(branch_type IN ('HEAD', 'BRANCH')),
    branch_code TEXT DEFAULT '00000',
    address TEXT NOT NULL,
    phone TEXT,
    email TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_customers_tax_id ON Customers(tax_id);

-- 2. ข้อมูลใบกำกับภาษีเต็มรูป (Immutable Snapshot)
CREATE TABLE IF NOT EXISTS Tax_Invoices (
    invoice_id TEXT PRIMARY KEY,
    invoice_no TEXT NOT NULL UNIQUE,
    book_no TEXT DEFAULT '01',
    order_id TEXT NOT NULL,
    customer_id TEXT,
    customer_name TEXT NOT NULL,
    customer_tax_id TEXT,
    customer_branch_type TEXT NOT NULL DEFAULT 'HEAD',
    customer_branch_code TEXT NOT NULL DEFAULT '00000',
    customer_address TEXT NOT NULL,
    subtotal REAL NOT NULL,
    discount_amount REAL NOT NULL DEFAULT 0.0,
    pre_tax_amount REAL NOT NULL,
    vat_rate REAL NOT NULL DEFAULT 0.07,
    vat_amount REAL NOT NULL,
    total_amount REAL NOT NULL,
    total_in_words TEXT NOT NULL,
    issued_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    issued_by TEXT,
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK(status IN ('ACTIVE', 'CANCELED')),
    canceled_reason TEXT,
    FOREIGN KEY (order_id) REFERENCES Orders(order_id) ON UPDATE CASCADE ON DELETE RESTRICT,
    FOREIGN KEY (customer_id) REFERENCES Customers(customer_id) ON UPDATE CASCADE ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_tax_invoices_order_id ON Tax_Invoices(order_id);
```

### การตั้งค่าเพิ่มเติมใน `App_Settings`:
* `company_tax_id`: เลขผู้เสียภาษี 13 หลักร้านค้า
* `company_branch_type` / `company_branch_code`: `'HEAD'` หรือ `'BRANCH'` / รหัสสาขา 5 หลัก (`'00000'`)
* `vat_system_enabled`: `'true'` (จด VAT 7%) / `'false'`
* `vat_pricing_mode`: `'inclusive'` (รวมภาษี) / `'exclusive'` (แยกภาษี)
* `a4_printer_target`: ชื่อไดรเวอร์เครื่องถ่ายเอกสาร A4 ในระบบ
* `a4_print_copies`: จำนวนชุด (ค่าเริ่มต้น `2` = ต้นฉบับ + สำเนา)
* `tax_invoice_prefix` / `tax_invoice_book_no`: คำนำหน้า (`TAX-`) / เล่มที่ (`01`)

---

## 5. Clean Architecture Layers

```text
[Frontend: Svelte 5 UI]
├── A4InvoiceModal.svelte (พรีวิว A4 เสมือนจริง + สั่งพิมพ์/Export PDF)
├── CustomerSelectorModal.svelte (ค้นหา/เพิ่มลูกค้า พร้อมตรวจ Checksum 13 หลัก)
└── A4InvoicePrintTemplate.svelte (HTML/CSS Template รองรับ @page A4)
      ▼
[Interface Adapters: commands/tax_invoice.rs]
├── invoke('create_tax_invoice', payload)
├── invoke('get_tax_invoice_by_order', { order_id })
├── invoke('search_customers', { query })
└── invoke('print_a4_invoice', { invoice_id })
      ▼
[Use Cases Layer: use_cases/tax_invoice/]
├── CreateTaxInvoiceUseCase (ตรวจการออกซ้ำ, Snapshot ข้อมูล, คำนวณ VAT 7%)
└── PrintA4DocumentUseCase (ส่งคำสั่งเข้า Spooler หรือเตรียม Data สำหรับ Print Window)
      ▼
[Domain Layer: domain/tax_invoice/]
├── TaxInvoice, Customer, InvoiceItem (Pure Entities)
├── format_baht_text(amount: f64) -> String (Pure Algorithm แปลงตัวเลขเป็นบาทข้อความ)
└── vat_calculator (คำนวณ Inclusive / Exclusive แม่นยำระดับทศนิยม 2 ตำแหน่ง)
```

---

## 6. Layout สเปกกระดาษ A4 (`@media print`)

* **ขนาด & ระยะขอบ:** A4 Portrait (`210mm x 297mm`), Margin บน/ล่าง `12mm`, ซ้าย/ขวา `15mm`
* **ฟอนต์:** `Sarabun` (หัวข้อ 16–20pt Bold, ตาราง/เนื้อหา 10–11pt Regular, ยอดรวม 12–14pt Bold)
* **ระบบพิมพ์ 2 หน้าเป็นชุด (Multi-Copy):**
  * หน้า 1: ป้ายมุมขวาบน `ต้นฉบับ / ORIGINAL (สำหรับลูกค้า)`
  * หน้า 2: ป้ายมุมขวาบน `สำเนา / COPY (สำหรับร้านค้า/ลงบัญชี)`

```css
@media print {
  @page { size: A4 portrait; margin: 12mm 15mm; }
  body { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
  .page-break { page-break-after: always; break-after: page; }
}
```

---

## 7. กฎการตรวจสอบ & UI Workflow (Validation & Interaction)

1. **ตรวจเลข 13 หลัก (Checksum Modulo 11):**
   * $\text{Check Digit} = (11 - (\sum_{i=1}^{12} (\text{digit}_i \times (14 - i)) \pmod{11})) \pmod{10}$
2. **ป้องกันการออกเอกสารซ้ำซ้อน (Duplicate Prevention):**
   * หาก Order เคยออกใบกำกับภาษีแล้ว ให้เปลี่ยนเป็นปุ่ม **"พิมพ์เอกสารซ้ำ (Reprint A4)"** พร้อมประทับตรา `"สำเนา / REPRINT"`
3. **Zero Native Popups:**
   * ใช้เฉพาะ Svelte Modal Components (`ErrorModal`, `ConfirmModal`, `CustomerSelectorModal`) ห้ามใช้ `alert()` หรือ `confirm()` โดยเด็ดขาด
