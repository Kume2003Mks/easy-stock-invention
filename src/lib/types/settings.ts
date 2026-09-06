/**
 * ข้อมูลการตั้งค่าระบบ (ตรงกับ SettingsPayload ฝั่ง Rust)
 */
export interface AppSettings {
  store_name: string;
  store_address: string;
  store_phone: string;
  store_email: string;
  currency: string;
  low_stock_threshold: string;
  low_stock_alert: string;
  daily_report: string;
  allow_out_of_stock_sale?: string;
  auto_print_enabled: string;
  receipt_preview_enabled: string;
  paper_size: string;
  printer_connection: string;
  printer_target: string;
  promptpay_id: string;
  promptpay_qr_enabled?: string;
  printer_codepage?: string;
  receipt_font?: string;
}

/**
 * ข้อมูลร้านค้าย่อ สำหรับพิมพ์ใบเสร็จ
 */
export interface StoreSettings {
  store_name: string;
  store_address: string;
  store_phone: string;
}

/**
 * ผลลัพธ์รายชื่อเครื่องพิมพ์จากระบบ
 */
export interface SystemPrintersResponse {
  printers: string[];
  default_printer: string | null;
}
