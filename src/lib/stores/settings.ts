import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '$lib/types';
import { getCurrencySymbol, getCurrencyName } from '$lib/utils/currency';

export const currencyStore = writable<string>('THB');

export const currencySymbolStore = derived(currencyStore, ($curr) => getCurrencySymbol($curr));
export const currencyNameStore = derived(currencyStore, ($curr) => getCurrencyName($curr));

let fetchPromise: Promise<string> | null = null;

export async function fetchSystemCurrency(force = false): Promise<string> {
  if (!force && fetchPromise) {
    return fetchPromise;
  }

  fetchPromise = (async () => {
    try {
      const settings = (await invoke('get_settings')) as AppSettings;
      const curr = settings?.currency || 'THB';
      currencyStore.set(curr);
      return curr;
    } catch (e) {
      console.error('Failed to fetch system currency setting:', e);
      return get(currencyStore) || 'THB';
    } finally {
      fetchPromise = null;
    }
  })();

  return fetchPromise;
}

export function updateSystemCurrency(newCurrency: string) {
  if (newCurrency) {
    currencyStore.set(newCurrency);
  }
}
