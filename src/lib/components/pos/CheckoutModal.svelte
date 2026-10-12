<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Modal from '$lib/components/Modal.svelte';
  import ErrorModal from '$lib/components/ErrorModal.svelte';
  import { parseAppError } from '$lib/utils/errorHandler';
  import type { CartItem, Order } from '$lib/types';
  import type { PromptPayQrResponse } from '$lib/types/settings';

  let {
    open = false,
    total = 0,
    cart = [] as CartItem[],
    discountAmount = 0,
    currency = 'THB',
    currencySymbol = '฿',
    onClose = () => {},
    onCompleted = (_order: Order, _promptpayAmountEnabled?: boolean) => {},
  }: {
    open?: boolean;
    total?: number;
    cart?: CartItem[];
    discountAmount?: number;
    currency?: string;
    currencySymbol?: string;
    onClose?: () => void;
    onCompleted?: (order: Order, promptpayAmountEnabled?: boolean) => void;
  } = $props();

  type PaymentMethod = 'CASH' | 'PROMPTPAY' | 'TRANSFER';

  let paymentMethod = $state<PaymentMethod>('CASH');
  let cashInput = $state('');
  let processing = $state(false);

  // PromptPay states
  let promptpayData = $state<PromptPayQrResponse | null>(null);
  let loadingQr = $state(false);
  let promptpayAmountEnabled = $state(true);
  let initialSettingLoaded = $state(false);

  let errorModal = $state({ open: false, title: '', message: '', details: '' });

  const quickCash = [20, 50, 100, 200, 500, 1000];

  const normalizedTotal = $derived(Math.round(total * 100) / 100);
  const paid = $derived(
    paymentMethod === 'CASH'
      ? Math.round((Number(cashInput) || 0) * 100) / 100
      : normalizedTotal
  );
  const change = $derived(
    Math.max(0, Math.round((paid - normalizedTotal) * 100) / 100)
  );
  const canPay = $derived(
    paymentMethod === 'CASH'
      ? paid >= normalizedTotal - 0.001
      : paymentMethod === 'PROMPTPAY'
        ? Boolean(promptpayData?.promptpayId?.trim()) && !loadingQr
        : true
  );

  function formatMoney(n: number): string {
    return n.toLocaleString('th-TH', { minimumFractionDigits: 2 });
  }

  async function loadPromptPayQr(withAmount?: boolean) {
    loadingQr = true;
    try {
      const forceAmount =
        withAmount !== undefined
          ? withAmount
          : initialSettingLoaded
            ? promptpayAmountEnabled
            : undefined;

      const res = await invoke<PromptPayQrResponse>('get_promptpay_qr', {
        amount: normalizedTotal,
        forceAmount,
      });

      promptpayData = res;
      if (!initialSettingLoaded) {
        promptpayAmountEnabled = res.promptpayAmountEnabled;
        initialSettingLoaded = true;
      }
    } catch (e) {
      console.error('Failed to load PromptPay QR:', e);
    } finally {
      loadingQr = false;
    }
  }

  function handleTogglePromptpayAmount(enabled: boolean) {
    if (promptpayAmountEnabled === enabled && promptpayData) return;
    promptpayAmountEnabled = enabled;
    loadPromptPayQr(enabled);
  }

  function selectPaymentMethod(method: PaymentMethod) {
    paymentMethod = method;
    if (method === 'PROMPTPAY' && !promptpayData) {
      loadPromptPayQr();
    }
  }

  // รีเซ็ตทุกครั้งที่เปิด modal
  $effect(() => {
    if (open) {
      paymentMethod = 'CASH';
      cashInput = '';
      processing = false;
      promptpayData = null;
      initialSettingLoaded = false;
    }
  });

  // รองรับการกด Enter เพื่อยืนยันการชำระเงินเมื่อไม่ใช่เงินสด
  $effect(() => {
    if (!open) return;
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Enter' && paymentMethod !== 'CASH' && canPay && !processing) {
        e.preventDefault();
        confirmPayment();
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });

  async function confirmPayment() {
    if (!canPay || processing) return;
    if (paymentMethod === 'PROMPTPAY' && !promptpayData?.promptpayId?.trim()) {
      errorModal = {
        open: true,
        title: 'จำเป็นต้องระบุหมายเลข PromptPay',
        message: 'กรุณาตั้งค่าหมายเลข PromptPay ในหน้าตั้งค่าก่อนทำรายการชำระเงินด้วยพร้อมเพย์',
        details: '',
      };
      return;
    }
    processing = true;
    try {
      const order = (await invoke('create_order', {
        payload: {
          subtotal: Math.round(cart.reduce((sum, i) => sum + i.unit_price * i.quantity, 0) * 100) / 100,
          discountAmount: Math.round(discountAmount * 100) / 100,
          paymentMethod,
          paidAmount: paid,
          promptpayAmountEnabled: paymentMethod === 'PROMPTPAY' ? promptpayAmountEnabled : undefined,
          items: cart.map((i) => ({
            productId: i.product_id,
            productName: i.product_name,
            quantity: i.quantity,
            unitPrice: Math.round(i.unit_price * 100) / 100,
          })),
        },
      })) as Order;
      onClose();
      onCompleted(order, paymentMethod === 'PROMPTPAY' ? promptpayAmountEnabled : undefined);
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
      <strong>{formatMoney(normalizedTotal)} {currencySymbol}</strong>
    </div>

    <div class="method-row">
      <button
        type="button"
        class="method-btn {paymentMethod === 'CASH' ? 'active' : ''}"
        onclick={() => selectPaymentMethod('CASH')}
      >
        เงินสด
      </button>
      <button
        type="button"
        class="method-btn {paymentMethod === 'PROMPTPAY' ? 'active' : ''}"
        onclick={() => selectPaymentMethod('PROMPTPAY')}
      >
        พร้อมเพย์
      </button>
      <button
        type="button"
        class="method-btn {paymentMethod === 'TRANSFER' ? 'active' : ''}"
        onclick={() => selectPaymentMethod('TRANSFER')}
      >
        โอนเงิน
      </button>
    </div>

    {#if paymentMethod === 'CASH'}
      <div class="cash-section">
        <div class="form-group">
          <label for="cash-input">รับเงินมา ({currencySymbol})</label>
          <input
            id="cash-input"
            type="text"
            inputmode="decimal"
            class="input-field cash-input"
            placeholder="0.00"
            bind:value={cashInput}
            oninput={(e) => {
              const val = e.currentTarget.value;
              const sanitized = val.replace(/[^0-9.]/g, '').replace(/(\..*?)\..*/g, '$1');
              if (val !== sanitized) {
                cashInput = sanitized;
              }
            }}
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
              onclick={() => (cashInput = String(amount))}
            >
              {amount}
            </button>
          {/each}
          <button
            type="button"
            class="btn-outline quick-btn exact"
            onclick={() => (cashInput = normalizedTotal.toFixed(2))}
          >
            ยอดตรง
          </button>
        </div>

        <div class="change-row {canPay ? 'ready' : ''}">
          <span>เงินทอน</span>
          <strong>{formatMoney(change)} {currencySymbol}</strong>
        </div>
      </div>
    {:else if paymentMethod === 'PROMPTPAY'}
      <div class="promptpay-section">
        {#if loadingQr && !promptpayData}
          <div class="qr-loading">
            <div class="loading-spinner"></div>
            <span>กำลังสร้าง QR Code พร้อมเพย์...</span>
          </div>
        {:else if !promptpayData?.promptpayId}
          <div class="qr-alert-warning">
            <div class="alert-icon">⚠️</div>
            <div class="alert-content">
              <strong>ยังไม่ได้ตั้งค่าหมายเลข PromptPay (จำเป็นต้องระบุ)</strong>
              <p>ระบบกำหนดให้ต้องมีหมายเลข PromptPay จึงจะสามารถเช็คบิลด้วยพร้อมเพย์ได้ กรุณาระบุหมายเลขโทรศัพท์ในหน้า <em>ตั้งค่า &gt; เครื่องพิมพ์ใบเสร็จ</em> ก่อนทำรายการ หรือเลือกวิธีชำระเงินอื่น</p>
            </div>
          </div>
        {:else}
          <!-- Choice to specify amount or not -->
          <div class="amount-choice-container">
            <div class="choice-pills">
              <button
                type="button"
                class="choice-pill"
                class:active={promptpayAmountEnabled}
                onclick={() => handleTogglePromptpayAmount(true)}
              >
                <div class="pill-radio" class:checked={promptpayAmountEnabled}>
                  <div class="radio-inner"></div>
                </div>
                <div class="pill-text">
                  <span class="pill-title">กำหนดราคาตามบิล</span>
                  <span class="pill-subtitle">{formatMoney(normalizedTotal)} {currencySymbol}</span>
                </div>
              </button>

              <button
                type="button"
                class="choice-pill"
                class:active={!promptpayAmountEnabled}
                onclick={() => handleTogglePromptpayAmount(false)}
              >
                <div class="pill-radio" class:checked={!promptpayAmountEnabled}>
                  <div class="radio-inner"></div>
                </div>
                <div class="pill-text">
                  <span class="pill-title">ไม่กำหนดราคา</span>
                  <span class="pill-subtitle muted">ลูกค้ากรอกยอดเอง</span>
                </div>
              </button>
            </div>
          </div>

          <!-- QR Card Display -->
          <div class="qr-card">
            <div class="qr-card-header">
              <span class="qr-thai-label">THAI QR PAYMENT</span>
              <span class="qr-id-label">พร้อมเพย์: {promptpayData.promptpayId}</span>
            </div>

            <div class="qr-card-body">
              {#if promptpayData.qrSvg}
                <div class="qr-svg-wrapper">
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  {@html promptpayData.qrSvg}
                </div>
              {:else}
                <div class="qr-placeholder">ไม่สามารถสร้าง QR Code ได้</div>
              {/if}
            </div>

            <div
              class="qr-card-footer"
              class:is-dynamic={promptpayAmountEnabled}
              class:is-static={!promptpayAmountEnabled}
            >
              {#if promptpayAmountEnabled}
                <span class="footer-badge">✓ ระบุยอดเงิน</span>
                <span class="footer-text">สแกนแล้วจะขึ้นยอด <strong>{formatMoney(normalizedTotal)} {currencySymbol}</strong> อัตโนมัติ</span>
              {:else}
                <span class="footer-badge static">ℹ ไม่ระบุยอด</span>
                <span class="footer-text">ลูกค้าพิมพ์ยอด <strong>{formatMoney(normalizedTotal)} {currencySymbol}</strong> ในแอปธนาคารเอง</span>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {:else}
      <div class="qr-note">
        ยืนยันการชำระผ่านการโอนเงินยอด <strong>{formatMoney(normalizedTotal)} {currencySymbol}</strong>
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
        {#if processing}
          กำลังบันทึก...
        {:else if paymentMethod === 'PROMPTPAY' && !promptpayData?.promptpayId}
          จำเป็นต้องตั้งค่าพร้อมเพย์ก่อน
        {:else}
          ยืนยันชำระเงิน {formatMoney(normalizedTotal)} {currencySymbol}
        {/if}
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
    border: 1px solid var(--color-border);
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
    cursor: pointer;
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
    border: 1px solid var(--color-border);
    margin-top: var(--space-md);
  }

  .change-row.ready {
    background-color: #e8f2e2;
    border-color: #a3be8c;
  }

  .change-row.ready strong {
    color: #4a7c39;
  }

  /* PromptPay Section */
  .promptpay-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .qr-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 40px 20px;
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    color: var(--color-text-secondary, #666);
    font-size: 14px;
  }

  .loading-spinner {
    width: 28px;
    height: 28px;
    border: 3px solid #d8dee9;
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .qr-alert-warning {
    display: flex;
    gap: 12px;
    padding: 16px;
    background-color: #fff9e6;
    border: 1px solid #ffd166;
    border-radius: var(--radius-md);
    color: #7a5800;
  }

  .qr-alert-warning .alert-icon {
    font-size: 24px;
    line-height: 1;
  }

  .qr-alert-warning .alert-content strong {
    display: block;
    font-size: 14px;
    margin-bottom: 4px;
  }

  .qr-alert-warning .alert-content p {
    margin: 0;
    font-size: 13px;
    line-height: 1.4;
  }

  /* Choice Pills (เลือกกำหนดราคา หรือ ไม่กำหนดราคา) */
  .amount-choice-container {
    background-color: var(--color-background);
    padding: 6px;
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
  }

  .choice-pills {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }

  .choice-pill {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: calc(var(--radius-md) - 2px);
    border: 1.5px solid transparent;
    background-color: transparent;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .choice-pill.active {
    background-color: var(--color-surface);
    border-color: var(--color-primary);
    box-shadow: 0 2px 6px rgba(94, 129, 172, 0.1);
  }

  .pill-radio {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid #cbd5e1;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all 0.15s ease;
  }

  .pill-radio.checked {
    border-color: var(--color-primary);
  }

  .radio-inner {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: transparent;
    transition: all 0.15s ease;
  }

  .pill-radio.checked .radio-inner {
    background-color: var(--color-primary);
  }

  .pill-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .pill-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
  }

  .pill-subtitle {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-primary);
  }

  .pill-subtitle.muted {
    color: #64748b;
  }

  /* QR Card */
  .qr-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .qr-card-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 16px;
    background-color: #f8fafc;
    border-bottom: 1px solid var(--color-border);
  }

  .qr-thai-label {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.5px;
    color: #1e3a8a;
  }

  .qr-id-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text);
  }

  .qr-card-body {
    padding: 16px;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #ffffff;
  }

  .qr-svg-wrapper {
    width: 190px;
    height: 190px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .qr-svg-wrapper :global(svg) {
    width: 100%;
    height: 100%;
    display: block;
  }

  .qr-placeholder {
    padding: 40px 20px;
    color: #ef4444;
    font-size: 13px;
  }

  .qr-card-footer {
    width: 100%;
    padding: 8px 14px;
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    border-top: 1px solid var(--color-border);
    background-color: #f8fafc;
  }

  .qr-card-footer.is-dynamic {
    background-color: #f0fdf4;
    border-top-color: #bbf7d0;
  }

  .qr-card-footer.is-static {
    background-color: #f8fafc;
  }

  .footer-badge {
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    background-color: #dcfce7;
    color: #166534;
    flex-shrink: 0;
  }

  .footer-badge.static {
    background-color: #e2e8f0;
    color: #475569;
  }

  .footer-text {
    color: var(--color-text);
    line-height: 1.3;
  }

  .qr-note {
    padding: var(--space-lg);
    text-align: center;
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
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