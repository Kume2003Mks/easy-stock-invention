/**
 * ผู้จัดจำหน่าย (ตรงกับ domain/entities.rs: Supplier)
 */
export interface Supplier {
  supplier_id: string;
  name: string;
  contact_info: string | null;
}
