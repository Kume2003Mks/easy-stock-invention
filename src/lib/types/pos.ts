/**
 * Shared types สำหรับระบบหน้าร้าน POS (ตรงกับ domain/entities.rs ฝั่ง Rust)
 */

export interface OrderItem {
  item_id: string;
  order_id: string;
  product_id: string | null;
  product_name: string;
  quantity: number;
  unit_price: number;
  line_total: number;
  returned_quantity: number;
}

export interface Order {
  order_id: string;
  order_no: string;
  order_type: 'SALE' | 'RETURN';
  status: 'HELD' | 'COMPLETED' | 'VOIDED';
  subtotal: number;
  discount_amount: number;
  total_amount: number;
  payment_method: 'CASH' | 'PROMPTPAY' | 'TRANSFER';
  paid_amount: number;
  change_amount: number;
  hold_name: string | null;
  note: string | null;
  original_order_id: string | null;
  order_date: string;
  items: OrderItem[];
}

/** สินค้าย่อสำหรับตะกร้า (snapshot) */
export interface CartItem {
  product_id: string;
  product_name: string;
  quantity: number;
  unit_price: number;
  /** สต็อกคงเหลือล่าสุด (สำหรับจำกัดจำนวนที่กดเพิ่ม) */
  stock: number;
}

export interface ProductForPos {
  product_id: string;
  barcode: string | null;
  name: string;
  category_id: string | null;
  selling_price: number;
  current_stock: number;
}

export interface CategoryForPos {
  category_id: string;
  name: string;
}
