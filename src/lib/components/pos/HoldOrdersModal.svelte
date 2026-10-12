<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import ErrorModal from '$lib/components/ErrorModal.svelte';
  import { parseAppError } from '$lib/utils/errorHandler';
  import type { Order } from '$lib/types';

  let {
    open = false,
    currencySymbol = '฿',
    onClose = () => {},
    onRecall = (_order: Order) => {},
  }: {
    open?: boolean;
    currencySymbol?: string;
    onClose?: () => void;
    onRecall?: (order: Order) => void;
  } = $props();

  let heldOrders = $state<Order[]>([]);
  let loading = $state(false);
  let loadError = $state('');
  let deletingId = $state<string | null>(null);
  let confirmDelete = $state<Order | null>(null);

  let errorModal = $state({ open: false, title: '', message: '', details: '' });

  async function load() {
    if (!open) return;
    loading = true;
    loadError = '';
    try {
      heldOrders = (await invoke('get_held_orders')) as Order[];
    } catch (e) {
      const parsed = parseAppError(e, 'โหลดบิลที่พักไม่สำเร็จ');
      loadError = parsed.message;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open) load();
  });

  function recall(order: Order) {
    onClose();
    onRecall(order);
  }

  function requestDelete(order: Order) {
    confirmDelete = order;
  }

  async function doDelete() {
    if (!confirmDelete) return;
    deletingId = confirmDelete.order_id;
    confirmDelete = null;
    try {
      await invoke('delete_held_order', { orderId: deletingId });
      await load();
    } catch (e) {
      const parsed = parseAppError(e, 'ลบบิลไม่สำเร็จ');
      errorModal = { open: true, title: parsed.title, message: parsed.message, details: parsed.details ?? '' };
    } finally {
      deletingId = null;
    }
  }

  function paymentLabel(order: Order): string {
    switch (order.payment_method) {
      case 'PROMPTPAY':
        return 'พร้อมเพย์';
      case 'TRANSFER':
        return 'โอนเงิน';
      default:
        return 'เงินสด';
    }
  }
</script>

<Modal {open} title="บิลที่พักไว้ (Hold Orders)" onClose={onClose} maxWidth="640px">
  <div class="held-content">
    {#if loading}
      <div class="state-box">กำลังโหลดบิลที่พัก...</div>
    {:else if loadError}
      <div class="state-box error">{loadError}</div>
    {:else if heldOrders.length === 0}
      <div class="state-box">
        <p>ไม่มีบิลที่พักไว้</p>
        <span>เมื่อกด "พักบิล" รายการจะแสดงที่นี่เพื่อรอเรียกคืนภายหลัง</span>
      </div>
    {:else}
      {#each heldOrders as order (order.order_id)}
        <div class="held-card">
          <div class="held-info">
            <div class="held-title">
              <strong>{order.hold_name || 'บิลพัก'}</strong>
              <span class="order-no">{order.order_no}</span>
            </div>
            <div class="held-meta">
              {order.order_date} · {order.items.length} รายการ · {paymentLabel(order)}
            </div>
            <div class="held-items">
              {#each order.items as item (item.item_id)}
                <span class="item-chip">{item.product_name} × {item.quantity}</span>
              {/each}
            </div>
          </div>
          <div class="held-actions">
            <span class="held-total">{order.total_amount.toLocaleString('th-TH', { minimumFractionDigits: 2 })} {currencySymbol}</span>
            <button type="button" class="btn-primary recall-btn" onclick={() => recall(order)}>
              เรียกคืน
            </button>
            <button
              type="button"
              class="btn-delete"
              title="ลบบิลที่พัก"
              disabled={deletingId === order.order_id}
              onclick={() => requestDelete(order)}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="3 6 5 6 21 6" />
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              </svg>
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</Modal>

<ConfirmModal
  open={confirmDelete !== null}
  title="ลบบิลที่พัก"
  message={`ต้องการลบบิลที่พัก "${confirmDelete?.hold_name || confirmDelete?.order_no || ''}" ใช่หรือไม่? การลบไม่สามารถย้อนกลับได้`}
  confirmText="ลบบิล"
  cancelText="ยกเลิก"
  variant="danger"
  onConfirm={doDelete}
  onCancel={() => (confirmDelete = null)}
/>

<ErrorModal
  open={errorModal.open}
  title={errorModal.title}
  message={errorModal.message}
  details={errorModal.details}
  onClose={() => (errorModal = { open: false, title: '', message: '', details: '' })}
/>

<style>
  .held-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .state-box {
    padding: var(--space-xl);
    text-align: center;
    color: var(--color-text-primary);
    opacity: 0.7;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .state-box.error {
    color: var(--color-danger);
    opacity: 1;
  }

  .held-card {
    display: flex;
    justify-content: space-between;
    gap: var(--space-md);
    padding: var(--space-md);
    border: var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .held-info {
    flex: 1;
    min-width: 0;
  }

  .held-title {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .order-no {
    font-size: 12px;
    color: var(--color-primary);
    font-weight: 500;
  }

  .held-meta {
    font-size: 12px;
    opacity: 0.6;
    margin: 4px 0;
  }

  .held-items {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 6px;
  }

  .item-chip {
    font-size: 11px;
    padding: 2px 8px;
    background-color: var(--color-background);
    border-radius: 100px;
    white-space: nowrap;
  }

  .held-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-shrink: 0;
  }

  .held-total {
    font-weight: 600;
    white-space: nowrap;
  }

  .recall-btn {
    padding: 8px 16px;
    font-size: 14px;
  }

  .btn-delete {
    width: 34px;
    height: 34px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    color: var(--color-danger);
    border: 1px solid transparent;
    transition: all 0.15s ease;
  }

  .btn-delete:hover {
    background-color: #fdecec;
    border-color: var(--color-danger);
  }

  .btn-delete:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>