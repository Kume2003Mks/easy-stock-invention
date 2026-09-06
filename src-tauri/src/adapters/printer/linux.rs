use std::fs::OpenOptions;
use std::io::Write;

use crate::domain::receipt::PrinterPort;

/// เครื่องพิมพ์ความร้อนบน Linux — เขียน Raw Bytes ตรงเข้า Device File
/// target เช่น "/dev/usb/lp0" (หากไม่ระบุจะใช้ lp0 เป็นค่าเริ่มต้น)
pub struct LinuxPrinter {
    device_path: String,
}

impl LinuxPrinter {
    pub fn new(device_path: String) -> Self {
        Self { device_path }
    }
}

impl PrinterPort for LinuxPrinter {
    fn send_raw(&self, bytes: &[u8]) -> Result<(), String> {
        let path = if self.device_path.starts_with("/dev/") {
            self.device_path.clone()
        } else {
            format!("/dev/usb/{}", self.device_path)
        };

        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)
            .map_err(|e| {
                format!(
                    "ERR_PRINTER_OFFLINE: เปิดอุปกรณ์พิมพ์ {} ไม่ได้ ({})",
                    path, e
                )
            })?;

        file.write_all(bytes)
            .and_then(|_| file.flush())
            .map_err(|e| {
                format!(
                    "ERR_PRINTER_OFFLINE: ส่งข้อมูลไปยังอุปกรณ์พิมพ์ไม่สำเร็จ ({})",
                    e
                )
            })?;

        Ok(())
    }
}