/**
 * Utility functions for currency symbols, names, and formatting
 */

export interface CurrencyInfo {
  code: string;
  symbol: string;
  nameTh: string;
}

export const SUPPORTED_CURRENCIES: CurrencyInfo[] = [
  { code: 'THB', symbol: '฿', nameTh: 'บาทไทย' },
  { code: 'USD', symbol: '$', nameTh: 'ดอลลาร์สหรัฐ' },
  { code: 'EUR', symbol: '€', nameTh: 'ยูโร' },
  { code: 'JPY', symbol: '¥', nameTh: 'เยนญี่ปุ่น' },
  { code: 'CNY', symbol: '¥', nameTh: 'หยวนจีน' },
  { code: 'GBP', symbol: '£', nameTh: 'ปอนด์สเตอร์ลิง' },
  { code: 'KRW', symbol: '₩', nameTh: 'วอนเกาหลีใต้' },
  { code: 'SGD', symbol: 'S$', nameTh: 'ดอลลาร์สิงคโปร์' },
  { code: 'LAK', symbol: '₭', nameTh: 'กีบลาว' },
  { code: 'MMK', symbol: 'K', nameTh: 'จ๊าดพม่า' },
  { code: 'MYR', symbol: 'RM', nameTh: 'ริงกิตมาเลเซีย' },
  { code: 'VND', symbol: '₫', nameTh: 'ดงเวียดนาม' },
  { code: 'AUD', symbol: 'A$', nameTh: 'ดอลลาร์ออสเตรเลีย' },
  { code: 'CAD', symbol: 'C$', nameTh: 'ดอลลาร์แคนาดา' },
  { code: 'CHF', symbol: 'CHF', nameTh: 'ฟรังก์สวิส' },
];

export function getCurrencyInfo(currency?: string): CurrencyInfo {
  const upper = currency?.trim().toUpperCase() || 'THB';
  const found = SUPPORTED_CURRENCIES.find((c) => c.code === upper);
  return (
    found || {
      code: upper,
      symbol: upper ? `${upper} ` : '฿',
      nameTh: upper,
    }
  );
}

export function getCurrencySymbol(currency?: string): string {
  return getCurrencyInfo(currency).symbol;
}

export function getCurrencyName(currency?: string): string {
  return getCurrencyInfo(currency).nameTh;
}

export function formatMoney(amount: number): string {
  const num = Number(amount) || 0;
  return num.toLocaleString('th-TH', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

export function formatCurrency(amount: number, currency = 'THB'): string {
  const symbol = getCurrencySymbol(currency);
  return `${symbol}${formatMoney(amount)}`;
}
