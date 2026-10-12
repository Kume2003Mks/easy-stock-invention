/**
 * ธุรกรรมการเคลื่อนไหวสต็อก (ตรงกับ domain/entities.rs: StockTransaction)
 */
export interface StockTransaction {
  transaction_id: string;
  product_id: string | null;
  transaction_type: 'IN' | 'OUT' | 'ADJUST' | string;
  quantity: number;
  reference_no: string | null;
  transaction_date: string;
}
