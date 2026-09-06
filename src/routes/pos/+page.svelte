<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import ErrorModal from '$lib/components/ErrorModal.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import CheckoutModal from '$lib/components/pos/CheckoutModal.svelte';
  import HoldOrdersModal from '$lib/components/pos/HoldOrdersModal.svelte';
  import DiscountModal from '$lib/components/pos/DiscountModal.svelte';
  import ReturnModal from '$lib/components/pos/ReturnModal.svelte';
  import ReceiptPreviewModal from '$lib/components/pos/ReceiptPreviewModal.svelte';
  import Dropdown from '$lib/components/Dropdown.svelte';
  import { parseAppError } from '$lib/utils/errorHandler';
  import type {
    AppSettings,
    CartItem,
    CategoryForPos,
    Order,
    ProductForPos,
  } from '$lib/types';

  // ---------- ข้อมูลสินค้า ----------
  let products = $state<ProductForPos[]>([]);
  let categories = $state<CategoryForPos[]>([]);
  let loading = $state(true);
  let loadError = $state('');

  let search = $state('');
  let activeCategory = $state<string | number>('all');

  const categoryOptions = $derived([
    { value: 'all', label: 'ทั้งหมด' },
    { value: 'none', label: 'ไม่มีหมวดหมู่' },
    ...categories.map((c) => ({ value: c.category_id, label: c.name })),
  ]);

  // ---------- ตะกร้า ----------
  let cart = $state<CartItem[]>([]);
  let discountAmount = $state(0);

  // ---------- Modals ----------
  let showCheckout = $state(false);
  let showHold = $state(false);
  let showDiscount = $state(false);
  let showReturn = $state(false);
  let showClearConfirm = $state(false);
  let heldCount = $state(0);
  let previewOrder = $state<Order | null>(null);
  let receiptPreviewEnabled = $state(true);
  let allowOutOfStockSale = $state(false);

  let errorModal = $state({ open: false, title: '', message: '', details: '' });
  let searchInput = $state<HTMLInputElement | null>(null);

  function showError(err: unknown, title: string) {
    const parsed = parseAppError(err, title);
    errorModal = {
      open: true,
      title: parsed.title,
      message: parsed.message,
      details: parsed.details ?? '',
    };
  }

  async function loadProducts() {
    try {
      const data = (await invoke('get_products_data', {
        params: { page: 1, pageSize: 500 },
      })) as {
        products: ProductForPos[];
        categories: CategoryForPos[];
      };
      products = data.products;
      categories = data.categories;
      loadError = '';
    } catch (e) {
      loadError = parseAppError(e, 'โหลดข้อมูลสินค้าไม่สำเร็จ').message;
    } finally {
      loading = false;
    }
  }

  async function loadHeldCount() {
    try {
      const held = (await invoke('get_held_orders')) as Order[];
      heldCount = held.length;
    } catch {
      heldCount = 0;
    }
  }

  onMount(async () => {
    await Promise.all([loadProducts(), loadHeldCount()]);
    try {
      const settings = (await invoke('get_settings')) as AppSettings;
      receiptPreviewEnabled = settings.receipt_preview_enabled !== 'false';
      allowOutOfStockSale = settings.allow_out_of_stock_sale === 'true';
    } catch {
      receiptPreviewEnabled = true;
      allowOutOfStockSale = false;
    }
  });

  // ---------- Derived ----------
  const filteredProducts = $derived.by(() => {
    const term = search.trim().toLowerCase();
    const cat = String(activeCategory);
    return products.filter((p) => {
      const matchCat =
        cat === 'all' ||
        (cat === 'none' ? !p.category_id : p.category_id === cat);
      const matchTerm =
        !term ||
        p.name.toLowerCase().includes(term) ||
        (p.barcode ?? '').toLowerCase().includes(term);
      return matchCat && matchTerm;
    });
  });

  const cartSubtotal = $derived(
    Math.round(cart.reduce((sum, i) => sum + i.unit_price * i.quantity, 0) * 100) / 100
  );
  const cartTotal = $derived(
    Math.round(Math.max(0, cartSubtotal - discountAmount) * 100) / 100
  );

  function formatMoney(n: number): string {
    return n.toLocaleString('th-TH', { minimumFractionDigits: 2 });
  }

  // ---------- ตะกร้า ----------
  function addToCart(product: ProductForPos) {
    if (!allowOutOfStockSale && product.current_stock <= 0) return;
    const existing = cart.find((i) => i.product_id === product.product_id);
    if (existing) {
      if (!allowOutOfStockSale && existing.quantity >= product.current_stock) return;
      existing.quantity += 1;
    } else {
      cart.push({
        product_id: product.product_id,
        product_name: product.name,
        quantity: 1,
        unit_price: product.selling_price,
        stock: product.current_stock,
      });
    }
  }

  function changeQty(index: number, delta: number) {
    const item = cart[index];
    const next = item.quantity + delta;
    if (next <= 0) {
      cart.splice(index, 1);
      return;
    }
    if (!allowOutOfStockSale && next > item.stock) return;
    item.quantity = next;
  }

  function removeItem(index: number) {
    cart.splice(index, 1);
  }

  function clearCart() {
    cart = [];
    discountAmount = 0;
    showClearConfirm = false;
  }

  // ---------- พักบิล ----------
  async function holdBill() {
    if (cart.length === 0) return;
    const holdName = `พัก ${new Date().toLocaleTimeString('th-TH', {
      hour: '2-digit',
      minute: '2-digit',
    })}`;
    try {
      await invoke('hold_order', {
        payload: {
          holdName,
          subtotal: cartSubtotal,
          discountAmount,
          items: cart.map((i) => ({
            productId: i.product_id,
            productName: i.product_name,
            quantity: i.quantity,
            unitPrice: i.unit_price,
          })),
        },
      });
      clearCart();
      await loadHeldCount();
    } catch (e) {
      showError(e, 'พักบิลไม่สำเร็จ');
    }
  }

  async function recallHeld(order: Order) {
    cart = order.items.map((item) => {
      const product = products.find((p) => p.product_id === item.product_id);
      return {
        product_id: item.product_id ?? '',
        product_name: item.product_name,
        quantity: item.quantity,
        unit_price: item.unit_price,
        stock: product?.current_stock ?? item.quantity,
      };
    });
    discountAmount = order.discount_amount;
    // ลบบิลที่พักออกจากฐานข้อมูล (รายการย้ายเข้าตะกร้าแล้ว — พักใหม่ได้ภายหลัง)
    try {
      await invoke('delete_held_order', { orderId: order.order_id });
    } catch {
      // หากลบไม่สำเร็จให้รายการยังอยู่ในรายการพัก
    }
    await loadHeldCount();
  }

  // ---------- ชำระเงินสำเร็จ ----------
  async function handleSaleCompleted(order: Order) {
    cart = [];
    discountAmount = 0;
    await loadProducts();
    if (receiptPreviewEnabled) {
      previewOrder = order;
    }
  }

  // ---------- Keyboard Shortcuts ----------
  function handleKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const isTyping =
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable;
    const modalOpen =
      showCheckout ||
      showHold ||
      showDiscount ||
      showReturn ||
      showClearConfirm ||
      previewOrder !== null;

    if (e.key === 'F2') {
      e.preventDefault();
      searchInput?.focus();
      return;
    }
    if (modalOpen || isTyping) return;

    // Shift+F4 — พักบิล
    if (e.shiftKey && e.key === 'F4') {
      e.preventDefault();
      holdBill();
      return;
    }
    // Space / Enter — เปิดหน้าชำระเงิน
    if (e.key === ' ' || e.key === 'Enter') {
      e.preventDefault();
      if (cart.length > 0) showCheckout = true;
    }
  }

  // สแกนบาร์โค้ด — Enter เพิ่มสินค้าทันที
  function handleBarcodeEnter() {
    const term = search.trim().toLowerCase();
    if (!term) return;
    const product =
      products.find((p) => (p.barcode ?? '').toLowerCase() === term) ??
      filteredProducts[0];
    if (product) {
      addToCart(product);
      search = '';
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<header class="topbar">
  <h1>หน้าร้าน POS</h1>
  <div class="topbar-actions">
    <button type="button" class="btn-outline" onclick={() => (showReturn = true)}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" /></svg>
      รับคืนสินค้า (RB)
    </button>
    <button type="button" class="btn-outline" onclick={() => (showHold = true)}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></svg>
      บิลที่พัก
      {#if heldCount > 0}
        <span class="held-badge">{heldCount}</span>
      {/if}
    </button>
  </div>
</header>

<div class="pos-layout">
  <!-- ฝั่งซ้าย: สินค้า -->
  <section class="product-panel">
    <div class="search-row">
      <input
        bind:this={searchInput}
        bind:value={search}
        onkeydown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            e.stopPropagation();
            handleBarcodeEnter();
          }
        }}
        type="text"
        class="input-field search-input"
        placeholder="สแกนบาร์โค้ด / ค้นหาสินค้า (F2)"
      />
      <Dropdown
        id="pos-category-filter"
        label="หมวดหมู่:"
        options={categoryOptions}
        bind:value={activeCategory}
        minWidth="160px"
      />
    </div>

    {#if loading}
      <div class="state-box">กำลังโหลดสินค้า...</div>
    {:else if loadError}
      <div class="state-box error">{loadError}</div>
    {:else if filteredProducts.length === 0}
      <div class="state-box">ไม่พบสินค้าที่ค้นหา</div>
    {:else}
      <div class="product-grid">
        {#each filteredProducts as product (product.product_id)}
          <button
            type="button"
            class="product-card {product.current_stock <= 0 && !allowOutOfStockSale ? 'out-of-stock' : ''} {product.current_stock <= 0 && allowOutOfStockSale ? 'stock-allowed' : ''}"
            disabled={!allowOutOfStockSale && product.current_stock <= 0}
            onclick={() => addToCart(product)}
          >
            <span class="product-name">{product.name}</span>
            <span class="product-price">{formatMoney(product.selling_price)} ฿</span>
            <span class="product-stock {product.current_stock <= 0 ? (allowOutOfStockSale ? 'out-allowed' : 'low') : (product.current_stock <= 5 ? 'low' : '')}">
              {product.current_stock <= 0
                ? (allowOutOfStockSale ? `หมด (คงเหลือ ${product.current_stock})` : 'สินค้าหมด')
                : `คงเหลือ ${product.current_stock}`}
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </section>

  <!-- ฝั่งขวา: ตะกร้า -->
  <aside class="cart-panel">
    <div class="cart-header">
      <span>ตะกร้าสินค้า</span>
      <span class="cart-count">{cart.length} รายการ</span>
    </div>

    <div class="cart-items">
      {#if cart.length === 0}
        <div class="cart-empty">
          <p>ยังไม่มีสินค้าในตะกร้า</p>
          <span>คลิกสินค้าฝั่งซ้าย หรือสแกนบาร์โค้ดเพื่อเพิ่มรายการ</span>
        </div>
      {:else}
        {#each cart as item, index (item.product_id)}
          <div class="cart-item">
            <div class="cart-item-info">
              <span class="cart-item-name">{item.product_name}</span>
              <span class="cart-item-price">{formatMoney(item.unit_price)} ฿</span>
            </div>
            <div class="cart-item-controls">
              <button type="button" class="qty-btn" onclick={() => changeQty(index, -1)}>−</button>
              <span class="qty-display">{item.quantity}</span>
              <button
                type="button"
                class="qty-btn"
                onclick={() => changeQty(index, 1)}
                disabled={!allowOutOfStockSale && item.quantity >= item.stock}
              >+</button>
            </div>
            <div class="cart-item-right">
              <span class="cart-item-total">{formatMoney(item.unit_price * item.quantity)} ฿</span>
              <button type="button" class="remove-btn" title="ลบรายการ" onclick={() => removeItem(index)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18" /><path d="m6 6 12 12" /></svg>
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="cart-summary">
      <button type="button" class="discount-row" onclick={() => (showDiscount = true)}>
        <span>ส่วนลด</span>
        <span class="discount-value">
          -{formatMoney(discountAmount)} ฿ <small>แก้ไข</small>
        </span>
      </button>
      <div class="summary-row">
        <span>รวมย่อย</span>
        <span>{formatMoney(cartSubtotal)} ฿</span>
      </div>
      <div class="summary-total">
        <span>ยอดสุทธิ</span>
        <strong>{formatMoney(cartTotal)} ฿</strong>
      </div>

      <div class="cart-actions-grid">
        <button
          type="button"
          class="btn-hold"
          disabled={cart.length === 0}
          title="พักบิลนี้ (Shift + F4)"
          onclick={holdBill}
        >
          พักบิล
        </button>
        <button
          type="button"
          class="btn-clear"
          disabled={cart.length === 0}
          title="ล้างตะกร้า"
          onclick={() => (showClearConfirm = true)}
        >
          ล้างตะกร้า
        </button>
      </div>

      <button
        type="button"
        class="btn-pay"
        disabled={cart.length === 0}
        onclick={() => (showCheckout = true)}
      >
        <span>ชำระเงิน (Space / Enter)</span>
        <span class="pay-amount">{formatMoney(cartTotal)} ฿</span>
      </button>
    </div>
  </aside>
</div>

<CheckoutModal
  open={showCheckout}
  total={cartTotal}
  {cart}
  discountAmount={discountAmount}
  onClose={() => (showCheckout = false)}
  onCompleted={handleSaleCompleted}
/>

<HoldOrdersModal
  open={showHold}
  onClose={() => (showHold = false)}
  onRecall={recallHeld}
/>

<DiscountModal
  open={showDiscount}
  subtotal={cartSubtotal}
  initialDiscount={discountAmount}
  onCancel={() => (showDiscount = false)}
  onConfirm={(amount) => {
    discountAmount = amount;
    showDiscount = false;
  }}
/>

<ReturnModal
  open={showReturn}
  onClose={() => (showReturn = false)}
  onCompleted={(order) => {
    loadProducts();
    if (receiptPreviewEnabled) {
      previewOrder = order;
    }
  }}
/>

<ReceiptPreviewModal
  open={previewOrder !== null}
  order={previewOrder}
  onClose={() => (previewOrder = null)}
/>

<ConfirmModal
  open={showClearConfirm}
  title="ล้างตะกร้า"
  message="ต้องการล้างสินค้าทั้งหมดในตะกร้าใช่หรือไม่?"
  confirmText="ล้างตะกร้า"
  cancelText="ยกเลิก"
  variant="danger"
  onConfirm={clearCart}
  onCancel={() => (showClearConfirm = false)}
/>

<ErrorModal
  open={errorModal.open}
  title={errorModal.title}
  message={errorModal.message}
  details={errorModal.details}
  onClose={() => (errorModal = { open: false, title: '', message: '', details: '' })}
/>

<style>
  .topbar {
    padding: var(--space-lg) var(--space-xl) 0 var(--space-xl);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .topbar h1 {
    font-size: 28px;
    margin-bottom: 0;
  }

  .topbar-actions {
    display: flex;
    gap: var(--space-md);
  }

  .topbar-actions .btn-outline {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 10px 16px;
    font-size: 14px;
  }

  .held-badge {
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 100px;
    background-color: var(--color-primary);
    color: var(--color-surface);
    font-size: 12px;
    font-weight: 600;
  }

  .pos-layout {
    flex: 1;
    display: flex;
    gap: var(--space-lg);
    padding: var(--space-lg) var(--space-xl) var(--space-xl) var(--space-xl);
    min-height: 0;
  }

  .product-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    min-width: 0;
  }

  .search-row {
    display: flex;
    gap: var(--space-md);
    align-items: center;
  }

  .search-input {
    font-size: 16px;
    flex: 1;
    min-width: 200px;
  }

  .product-grid {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: var(--space-md);
    overflow-y: auto;
    align-content: start;
    padding-bottom: var(--space-sm);
  }

  .product-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: var(--space-md);
    background-color: var(--color-surface);
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    text-align: left;
    transition: all 0.15s ease;
  }

  .product-card:hover:not(:disabled) {
    border-color: var(--color-primary);
    box-shadow: 0 4px 12px rgba(94, 129, 172, 0.15);
  }

  .product-card:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background-color: var(--color-background);
  }

  .product-name {
    font-weight: 500;
    font-size: 14px;
    line-height: 1.4;
    min-height: 2.8em;
  }

  .product-price {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-primary);
  }

  .product-stock {
    font-size: 11px;
    color: var(--color-accent-success);
    font-weight: 500;
  }

  .product-stock.low {
    color: var(--color-danger);
  }

  .product-stock.out-allowed {
    color: var(--color-accent-warning, #D08770);
    font-weight: 600;
  }

  .product-card.stock-allowed {
    border-style: dashed;
  }

  .state-box {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.6;
  }

  .state-box.error {
    color: var(--color-danger);
    opacity: 1;
  }

  /* ---------- ตะกร้า ---------- */
  .cart-panel {
    width: 400px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background-color: var(--color-surface);
    border: var(--border-subtle);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .cart-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-lg);
    border-bottom: var(--border-subtle);
    font-weight: 600;
  }

  .cart-count {
    font-size: 13px;
    font-weight: 500;
    opacity: 0.6;
  }

  .cart-items {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .cart-empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    text-align: center;
    opacity: 0.5;
    padding: var(--space-xl);
  }

  .cart-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border: var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .cart-item-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .cart-item-name {
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .cart-item-price {
    font-size: 12px;
    opacity: 0.6;
  }

  .cart-item-controls {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .qty-btn {
    width: 28px;
    height: 28px;
    border: var(--border-subtle);
    border-radius: 8px;
    font-size: 15px;
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
    opacity: 0.3;
    cursor: not-allowed;
  }

  .qty-display {
    min-width: 28px;
    text-align: center;
    font-weight: 600;
  }

  .cart-item-right {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
    min-width: 72px;
  }

  .cart-item-total {
    font-weight: 600;
    font-size: 14px;
  }

  .remove-btn {
    color: var(--color-danger);
    opacity: 0.6;
    padding: 2px;
  }

  .remove-btn:hover {
    opacity: 1;
  }

  .cart-summary {
    border-top: var(--border-subtle);
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .discount-row {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    font-size: 14px;
  }

  .discount-row:hover .discount-value small {
    opacity: 1;
  }

  .discount-value small {
    font-size: 11px;
    color: var(--color-primary);
    opacity: 0.7;
    margin-left: 4px;
  }

  .summary-row {
    display: flex;
    justify-content: space-between;
    font-size: 14px;
    opacity: 0.8;
  }

  .summary-total {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    margin-top: var(--space-sm);
  }

  .summary-total strong {
    font-size: 22px;
    color: var(--color-primary);
  }

  .cart-actions-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-sm);
    margin-top: var(--space-sm);
  }

  .btn-hold {
    padding: 12px;
    border: var(--border-subtle);
    border-color: var(--color-primary);
    color: var(--color-primary);
    border-radius: var(--radius-md);
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .btn-hold:hover:not(:disabled) {
    background-color: #ebf1f7;
  }

  .btn-hold:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-clear {
    padding: 12px;
    border: var(--border-subtle);
    color: var(--color-danger);
    border-color: var(--color-danger);
    border-radius: var(--radius-md);
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .btn-clear:hover:not(:disabled) {
    background-color: #fdecec;
  }

  .btn-clear:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-pay {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    background-color: var(--color-primary);
    color: var(--color-surface);
    border-radius: var(--radius-md);
    font-weight: 600;
    font-size: 15px;
    transition: opacity 0.2s;
    margin-top: var(--space-sm);
  }

  .btn-pay:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-pay:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .pay-amount {
    font-size: 18px;
  }
</style>
