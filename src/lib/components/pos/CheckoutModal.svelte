<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Modal from '$lib/components/Modal.svelte';
  import ErrorModal from '$lib/components/ErrorModal.svelte';
  import { parseAppError } from '$lib/utils/errorHandler';
  import type { CartItem, Order } from '$lib/types';

  let {
    open = false,
    total = 0,
    cart = [] as CartItem[],
    discountAmount = 0,
    onClose = () => {},
    onCompleted = (_order: Order) => {},
  }: {
    open?: boolean;
    total?: number;
    cart?: CartItem[];
    discountAmount?: number;
    onClose?: () => void;
    onCompleted?: (order: Order) => void;
  } = $props();

  type PaymentMethod = 'CASH' | 'PROMPTPAY' | 'TRANSFER';

  let paymentMethod = $state<PaymentMethod>('CASH');
  let cashInput = $state('');
  let processing = $state(false);

  let errorModal = $state({ open: false, title: '', message: '', details: '' });

  const quickCash = [20, 50, 100, 200, 500, 1000];

  // รีเซ็ตทุกครั้งที่เปิด modal
  $effect(() => {
    if (open) {
      paymentMethod = 'CASH';
      cashInput = '';
      processing = false;
    }
  });

  const paid = $derived(
    paymentMethod === 'CASH' ? Number(cashInput) || 0 : total
  );
  const change = $derived(Math.max(0, paid - total));
  const canPay = $derived(paymentMethod !== 'CASH' || paid >= total);

  function formatMoney(n: number): string {
    return n.toLocaleString('th-TH', { minimumFractionDigits: 2 });
  }

  async function confirmPayment() {
    if (!canPay || processing) return;
    processing = true;
    try {
      const order = (await invoke('create_order', {
        payload: {
          subtotal: cart.reduce((sum, i) => sum + i.unit_price * i.quantity, 0),
          discountAmount,
          paymentMethod,
          paidAmount: paid,
          items: cart.map((i) => ({
            productId: i.product_id,
            productName: i.product_name,
            quantity: i.quantity,
            unitPrice: i.unit_price,
          })),
        },
      })) as Order;
      onClose();
      onCompleted(order);
    } catch (e) {
      const parsed = parseAppError(e, 'ชำระเงินไม่สำเร็จ');
      errorModal = { open: true, title: parsed.title, message: parsed.message, details: parsed.details ?? '' };
    } finally {
      processing = false;
    }
  }
</script>

<Modal {open} title="ชำระเงิน" onClose={onClose} maxWidth="480px">
  <div class="checkout-content">
    <div class="total-banner">
      <span>ยอดที่ต้องชำระ</span>
      <strong>{formatMoney(total)} ฿</strong>
    </div>

    <div class="method-row">
      <button
        type="button"
        class="method-btn {paymentMethod === 'CASH' ? 'active' : ''}"
        onclick={() => (paymentMethod = 'CASH')}
      >
        เงินสด
      </button>
      <button
        type="button"
        class="method-btn {paymentMethod === 'PROMPTPAY' ? 'active' : ''}"
        onclick={() => (paymentMethod = 'PROMPTPAY')}
      >
        พร้อมเพย์
      </button>
      <button
        type="button"
        class="method-btn {paymentMethod === 'TRANSFER' ? 'active' : ''}"
        onclick={() => (paymentMethod = 'TRANSFER')}
      >
        โอนเงิน
      </button>
    </div>

    {#if paymentMethod === 'CASH'}
      <div class="cash-section">
        <div class="form-group">
          <label for="cash-input">รับเงินมา (บาท)</label>
          <input
            id="cash-input"
            type="number"
            class="input-field cash-input"
            min="0"
            step="1"
            placeholder={String(total)}
            bind:value={cashInput}
            onkeydown={(e) => {
              if (e.key === 'Enter') confirmPayment();
            }}
          />
        </div>

        <div class="quick-cash">
          {#each quickCash as amount (amount)}
            <button
              type="button"
              class="btn-outline quick-btn"
              onclick={() => (cashInput = String((Number(cashInput) || 0) + amount))}
            >
              +{amount}
            </button>
          {/each}
          <button type="button" class="btn-outline quick-btn exact" onclick={() => (cashInput = String(total))}>
            ยอดตรง
          </button>
        </div>

        <div class="change-row {paid >= total ? 'ready' : ''}">
          <span>เงินทอน</span>
          <strong>{formatMoney(change)} ฿</strong>
        </div>
      </div>
    {:else}
      <div class="qr-note">
        ยืนยันการชำระผ่าน{paymentMethod === 'PROMPTPAY' ? ' PromptPay' : ' การโอนเงิน'}ยอด {formatMoney(total)} ฿
      </div>
    {/if}

    <div class="actions">
      <button type="button" class="btn-outline" onclick={onClose} disabled={processing}>
        ยกเลิก
      </button>
      <button
        type="button"
        class="btn-primary pay-btn"
        onclick={confirmPayment}
        disabled={!canPay || processing}
      >
        {processing ? 'กำลังบันทึก...' : `ยืนยันชำระเงิน ${formatMoney(total)} ฿`}
      </button>
    </div>
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
  .checkout-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .total-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-lg);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .total-banner strong {
    font-size: 28px;
    color: var(--color-primary);
  }

  .method-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-sm);
  }

  .method-btn {
    padding: 12px 8px;
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    font-size: 14px;
    font-weight: 500;
    transition: all 0.15s ease;
    background-color: var(--color-surface);
  }

  .method-btn.active {
    border-color: var(--color-primary);
    background-color: #ebf1f7;
    color: var(--color-primary);
    font-weight: 600;
  }

  .form-group label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    opacity: 0.8;
    margin-bottom: 6px;
  }

  .cash-input {
    font-size: 22px;
    font-weight: 600;
  }

  .quick-cash {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-sm);
    margin-top: var(--space-sm);
  }

  .quick-btn {
    padding: 8px 4px;
    font-size: 14px;
  }

  .quick-btn.exact {
    grid-column: span 2;
  }

  .change-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md);
    border-radius: var(--radius-md);
    background-color: var(--color-background);
    margin-top: var(--space-md);
  }

  .change-row.ready {
    background-color: #e8f2e2;
  }

  .change-row.ready strong {
    color: var(--color-accent-success);
  }

  .qr-note {
    padding: var(--space-lg);
    text-align: center;
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    font-size: 15px;
  }

  .actions {
    display: flex;
    gap: var(--space-md);
  }

  .actions .btn-outline {
    flex: 1;
  }

  .pay-btn {
    flex: 2;
  }

  .pay-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>