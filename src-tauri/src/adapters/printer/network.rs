use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

use crate::domain::receipt::PrinterPort;

/// เครื่องพิมพ์ความร้อนผ่านเครือข่าย (Wi-Fi/LAN) — เขียน Raw Bytes ตรงเข้า TCP พอร์ต 9100
pub struct NetworkPrinter {
    address: String,
}

impl NetworkPrinter {
    pub fn new(address: String) -> Self {
        Self { address }
    }
}

impl PrinterPort for NetworkPrinter {
    fn send_raw(&self, bytes: &[u8]) -> Result<(), String> {
        // แยก host:port หากผู้ใช้ไม่ระบุพอร์ต ใช้ค่าเริ่มต้น 9100
        let addr = if self.address.contains(':') {
            self.address.clone()
        } else {
            format!("{}:9100", self.address)
        };

        let stream = TcpStream::connect(&addr)
            .map_err(|e| format!("ERR_PRINTER_OFFLINE: เชื่อมต่อเครื่องพิมพ์ {} ไม่ได้ ({})", addr, e))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| format!("ERR_PRINTER_OFFLINE: ตั้งค่า timeout ไม่สำเร็จ ({})", e))?;

        let mut writer = stream;
        writer
            .write_all(bytes)
            .and_then(|_| writer.flush())
            .map_err(|e| format!("ERR_PRINTER_OFFLINE: ส่งข้อมูลไปยังเครื่องพิมพ์ไม่สำเร็จ ({})", e))?;

        Ok(())
    }
}