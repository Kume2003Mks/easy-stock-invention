import type { Category } from './category';
import type { Supplier } from './supplier';

/**
 * สินค้าคงคลัง (ตรงกับ domain/entities.rs: Product)
 */
export interface Product {
  product_id: string;
  barcode: string | null;
  name: string;
  category_id: string | null;
  supplier_id: string | null;
  cost_price: number;
  selling_price: number;
  wholesale_price: number;
  current_stock: number;
}

/**
 * พารามิเตอร์สำหรับค้นหา/แบ่งหน้าสินค้า (ตรงกับ ProductQueryParams ฝั่ง Rust)
 */
export interface ProductQueryParams {
  page?: number;
  pageSize?: number;
  search?: string | null;
  categoryId?: string | null;
}

/**
 * ข้อมูลหน้าแสดงรายการสินค้า (ตรงกับ ProductsPageData ฝั่ง Rust)
 */
export interface ProductsPageData {
  products: Product[];
  totalItems: number;
  page: number;
  pageSize: number;
  totalPages: number;
  categories: Category[];
  suppliers: Supplier[];
  currency?: string;
}
