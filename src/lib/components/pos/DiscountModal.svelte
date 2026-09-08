<script lang="ts">
  import Modal from '$lib/components/Modal.svelte';

  let {
    open = false,
    subtotal = 0,
    initialDiscount = 0,
    currencySymbol = '฿',
    onCancel = () => {},
    onConfirm = (_discount: number) => {},
  }: {
    open?: boolean;
    subtotal?: number;
    initialDiscount?: number;
    currencySymbol?: string;
    onCancel?: () => void;
    onConfirm?: (discount: number) => void;
  } = $props();

  let amount = $state('0');

  $effect(() => {
    if (open) {
      amount = String(initialDiscount);
    }
  });

  const parsed = $derived(Number(amount) || 0);

  function confirm() {
    if (parsed < 0 || parsed > subtotal) return;
    onConfirm(parsed);
  }
</script>

<Modal {open} title="ส่วนลดบิล" onClose={onCancel} maxWidth="420px">
  <div class="discount-content">
    <div class="subtotal-row">
      <span>ยอดรวมก่อนส่วนลด</span>
      <strong>{subtotal.toLocaleString('th-TH', { minimumFractionDigits: 2 })} {currencySymbol}</strong>
    </div>

    <div class="form-group">
      <label for="discount-amount">ส่วนลด ({currencySymbol})</label>
      <input
        id="discount-amount"
        type="number"
        class="input-field"
        min="0"
        max={subtotal}
        step="1"
        bind:value={amount}
        onkeydown={(e) => {
          if (e.key === 'Enter') confirm();
        }}
      />
    </div>

    <div class="quick-buttons">
      <button type="button" class="btn-outline quick-btn" onclick={() => (amount = '0')}>ไม่มีส่วนลด</button>
      <button type="button" class="btn-outline quick-btn" onclick={() => (amount = String(Math.round(subtotal * 0.05 * 100) / 100))}>5%</button>
      <button type="button" class="btn-outline quick-btn" onclick={() => (amount = String(Math.round(subtotal * 0.1 * 100) / 100))}>10%</button>
    </div>

    {#if parsed > subtotal}
      <p class="error-text">ส่วนลดต้องไม่เกินยอดรวมของบิล</p>
    {/if}

    <div class="actions">
      <button type="button" class="btn-outline" onclick={onCancel}>ยกเลิก</button>
      <button
        type="button"
        class="btn-primary"
        onclick={confirm}
        disabled={parsed < 0 || parsed > subtotal}
      >
        ตกลง
      </button>
    </div>
  </div>
</Modal>

<style>
  .discount-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .subtotal-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    font-size: 15px;
  }

  .form-group label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    opacity: 0.8;
    margin-bottom: 6px;
  }

  .quick-buttons {
    display: flex;
    gap: var(--space-sm);
  }

  .quick-btn {
    flex: 1;
    padding: 8px 12px;
    font-size: 14px;
  }

  .error-text {
    font-size: 13px;
    color: var(--color-danger);
    font-weight: 500;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
  }
</style>