/**
 * Utility functions for currency symbols and formatting
 */

export function getCurrencySymbol(currency?: string): string {
  switch (currency?.toUpperCase()) {
    case 'THB':
      return '฿';
    case 'USD':
      return '$';
    case 'EUR':
      return '€';
    case 'JPY':
      return '¥';
    case 'CNY':
      return '¥';
    case 'GBP':
      return '£';
    case 'KRW':
      return '₩';
    case 'SGD':
      return 'S$';
    default:
      return currency ? `${currency} ` : '฿';
  }
}

export function formatCurrency(amount: number, currency = 'THB'): string {
  const symbol = getCurrencySymbol(currency);
  const num = Number(amount) || 0;
  return `${symbol}${num.toFixed(2)}`;
}
