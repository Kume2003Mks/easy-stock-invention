<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Modal from '$lib/components/Modal.svelte';
  import ErrorModal from '$lib/components/ErrorModal.svelte';
  import { parseAppError } from '$lib/utils/errorHandler';
  import type { Order } from '$lib/types';

  let {
    open = false,
    currencySymbol = '฿',
    onClose = () => {},
    onCompleted = (_order: Order) => {},
  }: {
    open?: boolean;
    currencySymbol?: string;
    onClose?: () => void;
    onCompleted?: (order: Order) => void;
  } = $props();

  interface ReturnRow {
    product_id: string | null;
    product_name: string;
    unit_price: number;
    sold: number;
    returned: number;
    returnQty: number;
  }

  let searchNo = $state('');
  let searching = $state(false);
  let originalOrder = $state<Order | null>(null);
  let rows = $state<ReturnRow[]>([]);
  let reason = $state('');
  let processing = $state(false);

  let errorModal = $state({ open: false, title: '', message: '', details: '' });

  // รีเซ็ตทุกครั้งที่เปิด
  $effect(() => {
    if (open) {
      searchNo = '';
      originalOrder = null;
      rows = [];
      reason = '';
      searching = false;
      processing = false;
    }
  });

  const returnTotal = $derived(
    rows.reduce((sum, r) => sum + r.returnQty * r.unit_price, 0)
  );
  const hasReturn = $derived(rows.some((r) => r.returnQty > 0));

  function formatMoney(n: number): string {
    return n.toLocaleString('th-TH', { minimumFractionDigits: 2 });
  }

  async function searchOrder() {
    const no = searchNo.trim();
    if (!no || searching) return;
    searching = true;
    originalOrder = null;
    rows = [];
    try {
      const found = (await invoke('get_order_by_no', { orderNo: no })) as Order;
      if (found.order_type !== 'SALE' || found.status !== 'COMPLETED') {
        errorModal = {
          open: true,
          title: 'ไม่สามารถรับคืนได้',
          message: 'คืนสินค้าได้เฉพาะบิลขายที่ชำระเงินแล้วเท่านั้น',
          details: '',
        };
        return;
      }
      originalOrder = found;
      rows = found.items.map((item) => ({
        product_id: item.product_id,
        product_name: item.product_name,
        unit_price: item.unit_price,
        sold: item.quantity,
        returned: item.returned_quantity,
        returnQty: 0,
      }));
    } catch (e) {
      const parsed = parseAppError(e, 'ค้นหาบิลไม่สำเร็จ');
      errorModal = { open: true, title: parsed.title, message: parsed.message, details: parsed.details ?? '' };
    } finally {
      searching = false;
    }
  }

  function maxReturn(row: ReturnRow): number {
    return row.sold - row.returned;
  }

  function setReturnQty(index: number, qty: number) {
    const row = rows[index];
    row.returnQty = Math.max(0, Math.min(qty, maxReturn(row)));
  }

  function returnAll(index: number) {
    setReturnQty(index, maxReturn(rows[index]));
  }

  async function confirmReturn() {
    if (!originalOrder || !hasReturn || processing) return;
    processing = true;
    try {
      const returnOrder = (await invoke('create_return_order', {
        payload: {
          originalOrderId: originalOrder.order_id,
          reason: reason.trim() || null,
          items: rows
            .filter((r) => r.returnQty > 0)
            .map((r) => ({
              productId: r.product_id,
              productName: r.product_name,
              quantity: r.returnQty,
              unitPrice: r.unit_price,
            })),
        },
      })) as Order;
      onClose();
      onCompleted(returnOrder);
    } catch (e) {
      const parsed = parseAppError(e, 'รับคืนสินค้าไม่สำเร็จ');
      errorModal = { open: true, title: parsed.title, message: parsed.message, details: parsed.details ?? '' };
    } finally {
      processing = false;
    }
  }
</script>

<Modal {open} title="รับคืนสินค้า (RB)" onClose={onClose} maxWidth="640px">
  <div class="return-content">
    <!-- ขั้นตอน 1: ค้นหาบิล -->
    <div class="search-row">
      <input
        type="text"
        class="input-field"
        placeholder="กรอกเลขที่บิล เช่น ORD-20260906-0001"
        bind:value={searchNo}
        onkeydown={(e) => {
          if (e.key === 'Enter') searchOrder();
        }}
      />
      <button type="button" class="btn-primary search-btn" onclick={searchOrder} disabled={searching}>
        {searching ? 'กำลังค้นหา...' : 'ค้นหาบิล'}
      </button>
    </div>

    {#if originalOrder}
      <!-- ข้อมูลบิลเดิม -->
      <div class="order-summary">
        <div class="summary-row"><span>เลขที่บิล</span><strong>{originalOrder.order_no}</strong></div>
        <div class="summary-row"><span>วันที่ขาย</span><span>{originalOrder.order_date}</span></div>
        <div class="summary-row"><span>ยอดขายรวม</span><strong>{formatMoney(originalOrder.total_amount)} {currencySymbol}</strong></div>
      </div>

      <!-- ขั้นตอน 2: เลือกรายการคืน -->
      <div class="items-header">
        <span>รายการสินค้า</span>
        <span>จำนวนคืน / คืนได้</span>
      </div>
      <div class="items-list">
        {#each rows as row, index (row.product_id ?? row.product_name)}
          {@const available = maxReturn(row)}
          <div class="return-item {available <= 0 ? 'disabled' : ''}">
            <div class="item-info">
              <span class="item-name">{row.product_name}</span>
              <span class="item-meta">
                ขาย {row.sold} ชิ้น · ราคา {formatMoney(row.unit_price)} {currencySymbol}
                {#if row.returned > 0}
                  · คืนไปแล้ว {row.returned} ชิ้น
                {/if}
              </span>
            </div>
            <div class="qty-controls">
              {#if available > 0}
                <button type="button" class="qty-btn" onclick={() => setReturnQty(index, row.returnQty - 1)} disabled={row.returnQty <= 0}>−</button>
                <input
                  type="number"
                  class="qty-input"
                  min="0"
                  max={available}
                  value={row.returnQty}
                  oninput={(e) => setReturnQty(index, Number((e.currentTarget as HTMLInputElement).value) || 0)}
                />
                <button type="button" class="qty-btn" onclick={() => setReturnQty(index, row.returnQty + 1)} disabled={row.returnQty >= available}>+</button>
                <button type="button" class="btn-outline all-btn" onclick={() => returnAll(index)}>คืนหมด</button>
              {:else}
                <span class="no-return">คืนครบแล้ว</span>
              {/if}
            </div>
          </div>
        {/each}
      </div>

      <!-- เหตุผล -->
      <div class="form-group">
        <label for="return-reason">เหตุผลการคืน (ไม่บังคับ)</label>
        <input
          id="return-reason"
          type="text"
          class="input-field"
          placeholder="เช่น สินค้าชำรุด / ลูกค้าเปลี่ยนใจ"
          bind:value={reason}
        />
      </div>

      <!-- สรุปยอดคืน + ยืนยัน -->
      <div class="return-footer">
        <div class="return-total">
          <span>ยอดคืนเงินรวม</span>
          <strong>{formatMoney(returnTotal)} {currencySymbol}</strong>
        </div>
        <div class="footer-actions">
          <button type="button" class="btn-outline" onclick={onClose} disabled={processing}>
            ยกเลิก
          </button>
          <button
            type="button"
            class="btn-primary confirm-btn"
            onclick={confirmReturn}
            disabled={!hasReturn || processing}
          >
            {processing ? 'กำลังบันทึก...' : 'ยืนยันรับคืน'}
          </button>
        </div>
      </div>
    {:else}
      <div class="empty-hint">
        ค้นหาบิลด้วยเลขที่บิลเพื่อเริ่มกระบวนการรับคืนสินค้า
      </div>
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
  .return-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .search-row {
    display: flex;
    gap: var(--space-sm);
  }

  .search-row .input-field {
    flex: 1;
  }

  .search-btn {
    flex-shrink: 0;
  }

  .order-summary {
    padding: var(--space-md);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .summary-row {
    display: flex;
    justify-content: space-between;
    font-size: 14px;
  }

  .items-header {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    opacity: 0.6;
  }

  .items-list {
    display: flex;
    flex-direction: column;
    max-height: 300px;
    overflow-y: auto;
    gap: var(--space-sm);
  }

  .return-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    border: var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .return-item.disabled {
    opacity: 0.5;
    background-color: var(--color-background);
  }

  .item-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .item-name {
    font-weight: 500;
  }

  .item-meta {
    font-size: 12px;
    opacity: 0.6;
  }

  .qty-controls {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .qty-btn {
    width: 32px;
    height: 32px;
    border: var(--border-subtle);
    border-radius: 8px;
    font-size: 16px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .qty-btn:hover:not(:disabled) {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .qty-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .qty-input {
    width: 56px;
    text-align: center;
    padding: 6px 4px;
    border: var(--border-subtle);
    border-radius: 8px;
    font-size: 15px;
    font-weight: 600;
  }

  .all-btn {
    padding: 6px 10px;
    font-size: 12px;
  }

  .no-return {
    font-size: 12px;
    opacity: 0.6;
  }

  .form-group label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    opacity: 0.8;
    margin-bottom: 6px;
  }

  .return-footer {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .return-total {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md);
    background-color: #fdecec;
    border-radius: var(--radius-md);
  }

  .return-total strong {
    color: var(--color-danger);
    font-size: 20px;
  }

  .footer-actions {
    display: flex;
    gap: var(--space-md);
  }

  .footer-actions .btn-outline {
    flex: 1;
  }

  .confirm-btn {
    flex: 2;
  }

  .confirm-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty-hint {
    text-align: center;
    padding: var(--space-xl);
    opacity: 0.6;
    font-size: 14px;
  }
</style>