use crate::domain::receipt::PrinterPort;

/// เครื่องพิมพ์ที่ต่อตรงกับ Windows — พิมพ์ Raw Bytes ผ่าน WinSpool (Silent Print ไม่มี Dialog)
pub struct WindowsPrinter {
    name: String,
}

impl WindowsPrinter {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[cfg(target_os = "windows")]
impl PrinterPort for WindowsPrinter {
    fn send_raw(&self, bytes: &[u8]) -> Result<(), String> {
        use std::ptr;
        use windows_sys::Win32::Foundation::HANDLE;
        use windows_sys::Win32::Graphics::Gdi::DEVMODEW;
        use windows_sys::Win32::Graphics::Printing::{
            ClosePrinter, EndDocPrinter, EndPagePrinter, OpenPrinterW, PRINTER_ACCESS_USE,
            PRINTER_DEFAULTSW, StartDocPrinterW, StartPagePrinter, WritePrinter,
        };

        // หากระบุเป็น "default" หรือค่าว่าง ให้ดึงชื่อเครื่องพิมพ์หลักของระบบอัตโนมัติ
        let target_name = if self.name.trim().is_empty() || self.name.trim().eq_ignore_ascii_case("default") {
            get_default_printer_name().unwrap_or_else(|| self.name.clone())
        } else {
            self.name.clone()
        };

        unsafe {
            let mut wide_name: Vec<u16> = target_name
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let mut datatype: Vec<u16> = "RAW\0".encode_utf16().collect();
            let mut doc_name: Vec<u16> = "Receipt\0".encode_utf16().collect();

            let defaults = PRINTER_DEFAULTSW {
                pDatatype: datatype.as_mut_ptr(),
                pDevMode: ptr::null_mut::<DEVMODEW>(),
                DesiredAccess: PRINTER_ACCESS_USE,
            };

            let mut handle: HANDLE = ptr::null_mut();
            if OpenPrinterW(wide_name.as_mut_ptr(), &mut handle, &defaults) == 0 {
                return Err(format!(
                    "ERR_PRINTER_OFFLINE: เปิดเครื่องพิมพ์ '{}' ไม่ได้ (ตรวจสอบชื่อเครื่องพิมพ์และการเชื่อมต่อ)",
                    target_name
                ));
            }

            let doc = windows_sys::Win32::Graphics::Printing::DOC_INFO_1W {
                pDocName: doc_name.as_mut_ptr(),
                pOutputFile: ptr::null_mut(),
                pDatatype: datatype.as_mut_ptr(),
            };

            let job_id = StartDocPrinterW(handle, 1, &doc);
            if job_id == 0 {
                let _ = ClosePrinter(handle);
                return Err("ERR_PRINTER_OFFLINE: เริ่มงานพิมพ์ (StartDoc) ไม่สำเร็จ".to_string());
            }

            let result = (|| -> Result<(), String> {
                if StartPagePrinter(handle) == 0 {
                    return Err("ERR_PRINTER_OFFLINE: เริ่มหน้าพิมพ์ (StartPage) ไม่สำเร็จ".to_string());
                }
                let mut written: u32 = 0;
                if WritePrinter(
                    handle,
                    bytes.as_ptr() as *const core::ffi::c_void,
                    bytes.len() as u32,
                    &mut written,
                ) == 0
                {
                    return Err("ERR_PRINTER_OFFLINE: ส่งข้อมูลไปยังเครื่องพิมพ์ไม่สำเร็จ".to_string());
                }
                if EndPagePrinter(handle) == 0 {
                    return Err("ERR_PRINTER_OFFLINE: จบหน้าพิมพ์ (EndPage) ไม่สำเร็จ".to_string());
                }
                Ok(())
            })();

            let _ = EndDocPrinter(handle);
            let _ = ClosePrinter(handle);
            result
        }
    }
}

/// ดึงชื่อเครื่องพิมพ์เริ่มต้นของระบบ Windows
#[cfg(target_os = "windows")]
pub fn get_default_printer_name() -> Option<String> {
    use windows_sys::Win32::Graphics::Printing::GetDefaultPrinterW;
    unsafe {
        let mut buf_size: u32 = 0;
        let _ = GetDefaultPrinterW(std::ptr::null_mut(), &mut buf_size);
        if buf_size == 0 {
            return None;
        }
        let mut buffer: Vec<u16> = vec![0u16; buf_size as usize];
        if GetDefaultPrinterW(buffer.as_mut_ptr(), &mut buf_size) == 0 {
            return None;
        }
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        String::from_utf16(&buffer[..len]).ok()
    }
}

/// ดึงรายชื่อเครื่องพิมพ์ทั้งหมดที่ติดตั้งใน Windows
#[cfg(target_os = "windows")]
pub fn list_system_printers() -> Vec<String> {
    use windows_sys::Win32::Graphics::Printing::{
        EnumPrintersW, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL, PRINTER_INFO_4W,
    };
    unsafe {
        let mut bytes_needed: u32 = 0;
        let mut count: u32 = 0;
        let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;

        let _ = EnumPrintersW(
            flags,
            std::ptr::null_mut(),
            4,
            std::ptr::null_mut(),
            0,
            &mut bytes_needed,
            &mut count,
        );

        if bytes_needed == 0 {
            return Vec::new();
        }

        let mut buffer: Vec<u8> = vec![0u8; bytes_needed as usize];
        if EnumPrintersW(
            flags,
            std::ptr::null_mut(),
            4,
            buffer.as_mut_ptr(),
            bytes_needed,
            &mut bytes_needed,
            &mut count,
        ) == 0
        {
            return Vec::new();
        }

        let p_info = buffer.as_ptr() as *const PRINTER_INFO_4W;
        let mut printers = Vec::with_capacity(count as usize);
        for i in 0..count as isize {
            let info = &*p_info.offset(i);
            if !info.pPrinterName.is_null() {
                let mut len = 0;
                while *info.pPrinterName.offset(len) != 0 {
                    len += 1;
                }
                let slice = std::slice::from_raw_parts(info.pPrinterName, len as usize);
                if let Ok(name) = String::from_utf16(slice) {
                    if !name.trim().is_empty() {
                        printers.push(name);
                    }
                }
            }
        }
        printers.sort();
        printers
    }
}

#[cfg(not(target_os = "windows"))]
impl WindowsPrinter {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_default_printer_name() -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn list_system_printers() -> Vec<String> {
    Vec::new()
}