<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ErrorModal from '$lib/components/ErrorModal.svelte';
  import { parseAppError } from '$lib/utils/errorHandler';
  import type { Order } from '$lib/types';

  let {
    open = false,
    order = null,
    onClose = () => {},
  }: {
    open?: boolean;
    order?: Order | null;
    onClose?: () => void;
  } = $props();

  interface StoreSettings {
    store_name: string;
    store_address: string;
    store_phone: string;
  }

  let store = $state<StoreSettings>({ store_name: 'Easy Stock', store_address: '', store_phone: '' });
  let printing = $state(false);
  let printSuccess = $state(false);

  let errorModal = $state({ open: false, title: '', message: '', details: '' });

  onMount(async () => {
    try {
      store = (await invoke('get_settings')) as unknown as StoreSettings;
    } catch {
      // ใช้ค่าเริ่มต้นหากโหลด settings ไม่ได้
    }
  });

  // รีเซ็ตสถานะพิมพ์ทุกครั้งที่เปิดด้วยบิลใหม่
  $effect(() => {
    if (open) {
      printSuccess = false;
    }
  });

  function formatMoney(n: number): string {
    return n.toLocaleString('th-TH', { minimumFractionDigits: 2 });
  }

  function paymentLabel(method: string): string {
    switch (method) {
      case 'PROMPTPAY':
        return 'พร้อมเพย์';
      case 'TRANSFER':
        return 'โอนเงิน';
      default:
        return 'เงินสด';
    }
  }

  async function printReceipt() {
    if (!order || printing) return;
    printing = true;
    try {
      await invoke('print_receipt', { orderId: order.order_id });
      printSuccess = true;
    } catch (e) {
      const parsed = parseAppError(e, 'พิมพ์ใบเสร็จไม่สำเร็จ');
      errorModal = { open: true, title: parsed.title, message: parsed.message, details: parsed.details ?? '' };
    } finally {
      printing = false;
    }
  }
</script>

<Modal {open} title="ตัวอย่างใบเสร็จ" onClose={onClose} maxWidth="420px">
  <div class="preview-content">
    {#if order}
      <div class="receipt-paper">
        <div class="receipt-center receipt-store">{store.store_name}</div>
        {#if store.store_address}
          <div class="receipt-center receipt-muted">{store.store_address}</div>
        {/if}
        {#if store.store_phone}
          <div class="receipt-center receipt-muted">โทร. {store.store_phone}</div>
        {/if}
        <div class="receipt-sep"></div>
        {#if order.order_type === 'RETURN'}
          <div class="receipt-center receipt-return">** ใบเสร็จรับเงิน (คืนสินค้า) **</div>
        {/if}
        <div class="receipt-row"><span>เลขที่</span><span>{order.order_no}</span></div>
        <div class="receipt-row"><span>วันที่</span><span>{order.order_date}</span></div>
        <div class="receipt-sep"></div>

        {#each order.items as item (item.item_id)}
          <div class="receipt-item-name">{item.product_name}</div>
          <div class="receipt-row">
            <span>&nbsp;&nbsp;{item.quantity} x {formatMoney(item.unit_price)}</span>
            <span>{formatMoney(item.line_total)}</span>
          </div>
        {/each}
        <div class="receipt-sep"></div>

        <div class="receipt-row"><span>รวมย่อย</span><span>{formatMoney(order.subtotal)}</span></div>
        {#if order.discount_amount > 0}
          <div class="receipt-row"><span>ส่วนลด</span><span>-{formatMoney(order.discount_amount)}</span></div>
        {/if}
        <div class="receipt-row receipt-total"><span>ยอดรวม</span><span>{formatMoney(order.total_amount)}</span></div>
        <div class="receipt-row"><span>ชำระ ({paymentLabel(order.payment_method)})</span><span>{formatMoney(order.paid_amount)}</span></div>
        {#if order.change_amount > 0}
          <div class="receipt-row"><span>เงินทอน</span><span>{formatMoney(order.change_amount)}</span></div>
        {/if}
        {#if order.note}
          <div class="receipt-sep"></div>
          <div class="receipt-row"><span>หมายเหตุ</span><span>{order.note}</span></div>
        {/if}
        <div class="receipt-sep"></div>
        <div class="receipt-center receipt-muted">ขอบคุณที่ใช้บริการ</div>
      </div>

      <div class="preview-actions">
        <button type="button" class="btn-outline" onclick={onClose}>
          ปิด
        </button>
        <button
          type="button"
          class="btn-primary print-btn"
          onclick={printReceipt}
          disabled={printing}
        >
          {printing
            ? 'กำลังพิมพ์...'
            : printSuccess
              ? 'พิมพ์ซ้ำอีกครั้ง'
              : 'พิมพ์ใบเสร็จ'}
        </button>
      </div>

      {#if printSuccess}
        <div class="print-toast">สั่งพิมพ์ใบเสร็จเรียบร้อยแล้ว</div>
      {/if}
    {/if}
  </div>
</Modal>

<ErrorModal
  open={errorModal.open}
  title={errorModal.title}
  message={errorModal.message}
  details={errorModal.details}
  onClose={() => (errorModal = { open: false, title: '', message: '', details: '' })}
/>

<style>
  .preview-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
    position: relative;
  }

  .receipt-paper {
    background-color: #fffefb;
    border: 1px dashed var(--color-muted);
    border-radius: var(--radius-md);
    padding: var(--space-lg);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 13px;
    line-height: 1.7;
    color: #2e3440;
  }

  .receipt-center {
    text-align: center;
  }

  .receipt-store {
    font-size: 17px;
    font-weight: 700;
  }

  .receipt-muted {
    opacity: 0.65;
    font-size: 12px;
  }

  .receipt-return {
    color: var(--color-danger);
    font-weight: 600;
  }

  .receipt-sep {
    border-top: 1px dashed var(--color-muted);
    margin: 8px 0;
  }

  .receipt-row {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }

  .receipt-item-name {
    font-weight: 600;
    margin-top: 4px;
  }

  .receipt-total {
    font-size: 16px;
    font-weight: 700;
  }

  .preview-actions {
    display: flex;
    gap: var(--space-md);
  }

  .preview-actions .btn-outline {
    flex: 1;
  }

  .print-btn {
    flex: 2;
  }

  .print-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .print-toast {
    position: fixed;
    bottom: var(--space-xl);
    right: var(--space-xl);
    background-color: var(--color-accent-success);
    color: var(--color-surface);
    padding: 12px 24px;
    border-radius: var(--radius-md);
    font-weight: 500;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    animation: fadeIn 0.3s ease;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>