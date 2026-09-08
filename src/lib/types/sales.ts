/**
 * Data contracts สำหรับหน้ารายงานสรุปยอดขาย (Sales Summary)
 * ตรงกับ SalesSummaryFilter / SalesSummaryResult ฝั่ง Rust
 */

export interface SalesSummaryFilter {
  startDate?: string;
  endDate?: string;
  groupBy?: 'daily' | 'monthly' | 'hourly';
}

export interface SalesSummaryKpi {
  totalSales: number;
  totalOrders: number;
  averageOrderValue: number;
}

export interface SalesSummaryBreakdownItem {
  period: string;
  totalSales: number;
  orderCount: number;
  averageOrderValue: number;
}

export interface SalesSummaryResult {
  kpi: SalesSummaryKpi;
  breakdown: SalesSummaryBreakdownItem[];
  hourlyBreakdown: SalesSummaryBreakdownItem[] | null;
  periodType: 'daily' | 'monthly' | 'hourly' | string;
}
