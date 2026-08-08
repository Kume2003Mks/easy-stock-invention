export interface FormattedError {
  title: string;
  message: string;
  details?: string;
}

export function parseAppError(err: unknown, defaultTitle = 'เกิดข้อผิดพลาด'): FormattedError {
  const rawMsg = err instanceof Error ? err.message : String(err ?? '');

  if (
    rawMsg.includes('UNIQUE constraint failed: Products.barcode') ||
    rawMsg.includes('Products.barcode')
  ) {
    return {
      title: 'บาร์โค้ดซ้ำในระบบ',
      message: 'บาร์โค้ดนี้ถูกใช้งานโดยสินค้าอื่นแล้ว กรุณาตรวจสอบหรือเปลี่ยนบาร์โค้ดใหม่',
      details: rawMsg,
    };
  }

  if (
    rawMsg.includes('UNIQUE constraint failed: Categories.name') ||
    rawMsg.includes('Categories.name')
  ) {
    return {
      title: 'ชื่อหมวดหมู่ซ้ำในระบบ',
      message: 'มีหมวดหมู่ชื่อนี้อยู่ในระบบแล้ว กรุณากรอกชื่อหมวดหมู่อื่น',
      details: rawMsg,
    };
  }

  if (
    rawMsg.includes('UNIQUE constraint failed: Suppliers.name') ||
    rawMsg.includes('Suppliers.name')
  ) {
    return {
      title: 'ชื่อผู้จัดจำหน่ายซ้ำในระบบ',
      message: 'มีผู้จัดจำหน่ายชื่อนี้อยู่ในระบบแล้ว กรุณากรอกชื่ออื่น',
      details: rawMsg,
    };
  }

  if (rawMsg.includes('UNIQUE constraint failed')) {
    return {
      title: 'ข้อมูลซ้ำในระบบ',
      message: 'มีข้อมูลนี้อยู่ในระบบแล้ว กรุณาตรวจสอบความถูกต้องอีกครั้ง',
      details: rawMsg,
    };
  }

  if (rawMsg.includes('FOREIGN KEY constraint failed')) {
    return {
      title: 'ไม่สามารถดำเนินการได้',
      message: 'รายการนี้กำลังถูกอ้างอิงหรือใช้งานอยู่ในส่วนอื่นของระบบ จึงไม่สามารถดำเนินการหรือลบได้',
      details: rawMsg,
    };
  }

  if (
    rawMsg.includes('จำนวนสต็อกไม่เพียงพอ') ||
    rawMsg.includes('สต็อกไม่เพียงพอ') ||
    rawMsg.includes('สต็อกมีเพียง')
  ) {
    return {
      title: 'สต็อกไม่เพียงพอ',
      message: rawMsg.replace('Validation error:', '').trim(),
      details: rawMsg,
    };
  }

  if (rawMsg.includes('Validation error:')) {
    const cleanMsg = rawMsg.replace('Validation error:', '').trim();
    return {
      title: 'ข้อมูลไม่ถูกต้อง',
      message: cleanMsg,
      details: rawMsg,
    };
  }

  if (rawMsg.includes('Database error:')) {
    const cleanMsg = rawMsg.replace('Database error:', '').trim();
    return {
      title: 'ข้อผิดพลาดฐานข้อมูล',
      message: cleanMsg,
      details: rawMsg,
    };
  }

  return {
    title: defaultTitle,
    message: rawMsg || 'เกิดข้อผิดพลาดในการดำเนินการ กรุณาลองใหม่อีกครั้ง',
    details: rawMsg,
  };
}
