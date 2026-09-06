pub mod linux;
pub mod network;
#[cfg(target_os = "windows")]
pub mod windows;

use crate::domain::receipt::PrinterPort;

/// ประเภทการเชื่อมต่อเครื่องพิมพ์ตามค่า setting `printer_connection`
#[derive(Debug, PartialEq, Eq)]
pub enum PrinterConnection {
    /// ไม่ระบุเครื่องพิมพ์ / ไม่ใช้งานเครื่องพิมพ์
    None,
    /// เครื่องพิมพ์ผ่านเครือข่าย (TCP Socket พอร์ต 9100) — target เช่น "192.168.1.200:9100"
    Network,
    /// เครื่องพิมพ์ที่ต่อตรงกับเครื่อง (Windows: ชื่อ Printer / Linux: path device)
    Usb,
}

impl PrinterConnection {
    pub fn from_setting(value: &str) -> PrinterConnection {
        match value.trim().to_lowercase().as_str() {
            "none" | "disabled" | "" => PrinterConnection::None,
            "usb" => PrinterConnection::Usb,
            _ => PrinterConnection::Network,
        }
    }
}

/// ดึงรายชื่อเครื่องพิมพ์ทั้งหมดที่ติดตั้งในระบบ
pub fn get_installed_printers() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        windows::list_system_printers()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

/// ดึงชื่อเครื่องพิมพ์เริ่มต้นของระบบ
pub fn get_default_printer() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        windows::get_default_printer_name()
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// สร้างพอร์ตเครื่องพิมพ์ตามการตั้งค่า (Infrastructure Layer Factory)
pub fn create_port(
    connection: &PrinterConnection,
    target: &str,
) -> Result<Box<dyn PrinterPort>, String> {
    match connection {
        PrinterConnection::None => {
            Err("ERR_PRINTER_DISABLED: ไม่ได้ระบุเครื่องพิมพ์ (อยู่ในโหมดไม่ใช้งานเครื่องพิมพ์)".to_string())
        }
        PrinterConnection::Network => {
            let trimmed = target.trim();
            if trimmed.is_empty() {
                return Err("ERR_PRINTER_OFFLINE: ยังไม่ได้ระบุที่อยู่เครื่องพิมพ์เครือข่าย (เช่น 192.168.1.200:9100)".to_string());
            }
            Ok(Box::new(network::NetworkPrinter::new(trimmed.to_string())))
        }
        PrinterConnection::Usb => {
            let trimmed = target.trim();
            let target_str = if trimmed.is_empty() { "default" } else { trimmed };
            #[cfg(target_os = "windows")]
            {
                Ok(Box::new(windows::WindowsPrinter::new(target_str.to_string())))
            }
            #[cfg(target_os = "linux")]
            {
                Ok(Box::new(linux::LinuxPrinter::new(target_str.to_string())))
            }
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            {
                let _ = target_str;
                Err("ระบบปฏิบัติการนี้ยังไม่รองรับการพิมพ์แบบ USB".to_string())
            }
        }
    }
}