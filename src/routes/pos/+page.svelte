<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import ErrorModal from "$lib/components/ErrorModal.svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import CheckoutModal from "$lib/components/pos/CheckoutModal.svelte";
  import HoldOrdersModal from "$lib/components/pos/HoldOrdersModal.svelte";
  import DiscountModal from "$lib/components/pos/DiscountModal.svelte";
  import ReturnModal from "$lib/components/pos/ReturnModal.svelte";
  import ReceiptPreviewModal from "$lib/components/pos/ReceiptPreviewModal.svelte";
  import Dropdown from "$lib/components/Dropdown.svelte";
  import Pagination from "$lib/components/Pagination.svelte";
  import { parseAppError } from "$lib/utils/errorHandler";
  import { getCurrencySymbol } from "$lib/utils/currency";
  import { currencyStore, fetchSystemCurrency } from "$lib/stores/settings";
  import type {
    AppSettings,
    CartItem,
    CategoryForPos,
    Order,
    ProductForPos,
    ProductsPageData,
  } from "$lib/types";

  // ---------- ข้อมูลสินค้า & แบ่งหน้าหลังบ้าน ----------
  let products = $state<ProductForPos[]>([]);
  let categories = $state<CategoryForPos[]>([]);
  let currency = $state("THB");
  let currencySymbol = $derived(getCurrencySymbol(currency));
  let lowStockThreshold = $state(10);
  let lowStockAlert = $state(true);
  let loading = $state(true);
  let loadError = $state("");
  let totalItems = $state(0);
  let currentPage = $state(1);
  let pageSize = $state(30);

  let search = $state("");
  let activeCategory = $state<string | number>("all");
  let isMounted = false;
  let searchDebounceTimer: ReturnType<typeof setTimeout> | undefined;

  const categoryOptions = $derived([
    { value: "all", label: "ทั้งหมด" },
    { value: "none", label: "ไม่มีหมวดหมู่" },
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
  let previewPromptpayAmountEnabled = $state<boolean | undefined>(undefined);
  let printBehavior = $state<"direct" | "preview" | "none">("direct");
  let printerConnection = $state("none");
  let receiptPreviewEnabled = $state(false);
  let allowOutOfStockSale = $state(false);

  let errorModal = $state({ open: false, title: "", message: "", details: "" });
  let searchInput = $state<HTMLInputElement | null>(null);

  // Toast notification state
  let toastMessage = $state<{ title: string; subtitle?: string } | null>(null);
  let toastTimeout: ReturnType<typeof setTimeout> | null = null;

  function showToast(title: string, subtitle?: string) {
    if (toastTimeout) clearTimeout(toastTimeout);
    toastMessage = { title, subtitle };
    toastTimeout = setTimeout(() => {
      toastMessage = null;
    }, 2500);
  }

  function showError(err: unknown, title: string) {
    const parsed = parseAppError(err, title);
    errorModal = {
      open: true,
      title: parsed.title,
      message: parsed.message,
      details: parsed.details ?? "",
    };
  }

  async function loadProducts() {
    loading = true;
    try {
      const data = (await invoke("get_products_data", {
        params: {
          page: currentPage,
          pageSize: pageSize,
          search: search.trim() || null,
          categoryId: activeCategory === "all" ? null : String(activeCategory),
        },
      })) as ProductsPageData;
      products = data.products ?? [];
      totalItems = Number(data.totalItems ?? (data as any).total_items ?? 0);
      if (data.categories && data.categories.length > 0) {
        categories = data.categories;
      }
      loadError = "";
    } catch (e) {
      loadError = parseAppError(e, "โหลดข้อมูลสินค้าไม่สำเร็จ").message;
    } finally {
      loading = false;
    }
  }

  async function loadHeldCount() {
    try {
      const held = (await invoke("get_held_orders")) as Order[];
      heldCount = held.length;
    } catch {
      heldCount = 0;
    }
  }

  // Effect สำหรับโหลดข้อมูลเมื่อเปลี่ยนหน้าหรือขนาดหน้า
  $effect(() => {
    const _p = currentPage;
    const _s = pageSize;
    if (!isMounted) return;
    loadProducts();
  });

  onMount(() => {
    const unsub = currencyStore.subscribe((c) => {
      if (c) currency = c;
    });

    (async () => {
      await Promise.all([
        loadProducts(),
        loadHeldCount(),
        fetchSystemCurrency(),
      ]);
      isMounted = true;
      try {
        const settings = (await invoke("get_settings")) as AppSettings;
        printerConnection = settings.printer_connection || "none";
        if (settings.print_behavior) {
          printBehavior = settings.print_behavior as "direct" | "preview" | "none";
        } else if (
          settings.auto_print_enabled === "true" &&
          settings.receipt_preview_enabled === "false"
        ) {
          printBehavior = "direct";
        } else if (
          settings.receipt_preview_enabled === "true" &&
          settings.auto_print_enabled === "false"
        ) {
          printBehavior = "preview";
        } else if (
          settings.receipt_preview_enabled === "false" &&
          settings.auto_print_enabled === "false"
        ) {
          printBehavior = "none";
        } else {
          printBehavior = "direct";
        }
        receiptPreviewEnabled = printBehavior === "preview";
        allowOutOfStockSale = settings.allow_out_of_stock_sale === "true";
        const parsedThreshold = Number(settings.low_stock_threshold);
        lowStockThreshold =
          !isNaN(parsedThreshold) && parsedThreshold >= 1
            ? parsedThreshold
            : 10;
        lowStockAlert = settings.low_stock_alert !== "false";
      } catch {
        printBehavior = "direct";
        receiptPreviewEnabled = false;
        allowOutOfStockSale = false;
      }
    })();

    return () => {
      unsub();
      if (toastTimeout) clearTimeout(toastTimeout);
    };
  });

  function handleSearchInput() {
    if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
    searchDebounceTimer = setTimeout(() => {
      if (currentPage !== 1) {
        currentPage = 1;
      } else {
        loadProducts();
      }
    }, 250);
  }

  function handleCategoryChange(val: string | number) {
    activeCategory = val;
    if (currentPage !== 1) {
      currentPage = 1;
    } else {
      loadProducts();
    }
  }

  const cartSubtotal = $derived(
    Math.round(
      cart.reduce((sum, i) => sum + i.unit_price * i.quantity, 0) * 100,
    ) / 100,
  );
  const cartTotal = $derived(
    Math.round(Math.max(0, cartSubtotal - discountAmount) * 100) / 100,
  );

  function formatMoney(n: number): string {
    return n.toLocaleString("th-TH", { minimumFractionDigits: 2 });
  }

  // ---------- ตะกร้า ----------
  function addToCart(product: ProductForPos) {
    if (!allowOutOfStockSale && product.current_stock <= 0) return;
    const existing = cart.find((i) => i.product_id === product.product_id);
    if (existing) {
      if (!allowOutOfStockSale && existing.quantity >= product.current_stock)
        return;
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
    const holdName = `พัก ${new Date().toLocaleTimeString("th-TH", {
      hour: "2-digit",
      minute: "2-digit",
    })}`;
    try {
      await invoke("hold_order", {
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
      showError(e, "พักบิลไม่สำเร็จ");
    }
  }

  async function recallHeld(order: Order) {
    cart = order.items.map((item) => {
      const product = products.find((p) => p.product_id === item.product_id);
      return {
        product_id: item.product_id ?? "",
        product_name: item.product_name,
        quantity: item.quantity,
        unit_price: item.unit_price,
        stock: product?.current_stock ?? item.quantity,
      };
    });
    discountAmount = order.discount_amount;
    // ลบบิลที่พักออกจากฐานข้อมูล (รายการย้ายเข้าตะกร้าแล้ว — พักใหม่ได้ภายหลัง)
    try {
      await invoke("delete_held_order", { orderId: order.order_id });
    } catch {
      // หากลบไม่สำเร็จให้รายการยังอยู่ในรายการพัก
    }
    await loadHeldCount();
  }

  // ---------- ชำระเงินสำเร็จ ----------
  async function handleSaleCompleted(order: Order, promptpayAmountEnabled?: boolean) {
    cart = [];
    discountAmount = 0;
    const orderNo = order?.order_no ? `บิล ${order.order_no}` : "";
    const totalText =
      order?.total_amount != null
        ? ` · ยอดชำระ ${formatMoney(order.total_amount)} ${currencySymbol}`
        : "";
    const changeText =
      order?.change_amount && order.change_amount > 0
        ? ` · เงินทอน ${formatMoney(order.change_amount)} ${currencySymbol}`
        : "";
    const printNote =
      printBehavior === "direct" && printerConnection !== "none"
        ? " · กำลังพิมพ์ใบเสร็จ..."
        : "";
    showToast(
      "ชำระเงินเรียบร้อยแล้ว",
      orderNo ? `${orderNo}${totalText}${changeText}${printNote}` : undefined,
    );
    await loadProducts();
    if (printBehavior === "preview") {
      previewOrder = order;
      previewPromptpayAmountEnabled = promptpayAmountEnabled;
    }
  }

  // ---------- Keyboard Shortcuts ----------
  function handleKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const isTyping =
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.isContentEditable;
    const modalOpen =
      showCheckout ||
      showHold ||
      showDiscount ||
      showReturn ||
      showClearConfirm ||
      previewOrder !== null;

    if (e.key === "F2") {
      e.preventDefault();
      searchInput?.focus();
      return;
    }
    if (modalOpen || isTyping) return;

    // Shift+F4 — พักบิล
    if (e.shiftKey && e.key === "F4") {
      e.preventDefault();
      holdBill();
      return;
    }
    // Space / Enter — เปิดหน้าชำระเงิน
    if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      if (cart.length > 0) showCheckout = true;
    }
  }

  // สแกนบาร์โค้ด — Enter เพิ่มสินค้าทันที
  async function handleBarcodeEnter() {
    const term = search.trim();
    if (!term) return;

    // 1. ตรวจสอบในรายการสินค้าที่โหลดอยู่ในหน้าปัจจุบันก่อน
    let product = products.find(
      (p) => (p.barcode ?? "").toLowerCase() === term.toLowerCase(),
    );

    // 2. ถ้าไม่พบในหน้าปัจจุบัน ให้ค้นหาจากหลังบ้านโดยตรงตามบาร์โค้ด/คำค้นหา
    if (!product) {
      try {
        const data = (await invoke("get_products_data", {
          params: { page: 1, pageSize: 1, search: term },
        })) as ProductsPageData;
        if (data.products && data.products.length > 0) {
          product = data.products[0];
        }
      } catch (err) {
        console.error("Barcode lookup error:", err);
      }
    }

    // 3. หากยังไม่พบคงเหลือ ให้เลือกรายการแรกที่มีในหน้าปัจจุบัน
    if (!product && products.length > 0) {
      product = products[0];
    }

    if (product) {
      addToCart(product);
      search = "";
      if (currentPage !== 1) {
        currentPage = 1;
      } else {
        loadProducts();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<header class="topbar">
  <h1>หน้าร้าน POS</h1>
  <div class="topbar-actions">
    <button
      type="button"
      class="btn-outline"
      onclick={() => (showReturn = true)}
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        ><polyline points="1 4 1 10 7 10" /><path
          d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"
        /></svg
      >
      รับคืนสินค้า (RB)
    </button>
    <button type="button" class="btn-outline" onclick={() => (showHold = true)}>
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        ><circle cx="12" cy="12" r="10" /><polyline
          points="12 6 12 12 16 14"
        /></svg
      >
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
        oninput={handleSearchInput}
        onkeydown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            e.stopPropagation();
            if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
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
        onchange={handleCategoryChange}
        minWidth="160px"
      />
    </div>

    {#if loading && products.length === 0}
      <div class="state-box">กำลังโหลดสินค้า...</div>
    {:else if loadError && products.length === 0}
      <div class="state-box error">{loadError}</div>
    {:else if products.length === 0}
      <div class="state-box">ไม่พบสินค้าที่ค้นหา</div>
    {:else}
      <div class="product-grid" class:is-loading={loading}>
        {#each products as product (product.product_id)}
          <button
            type="button"
            class="product-card {product.current_stock <= 0 &&
            !allowOutOfStockSale
              ? 'out-of-stock'
              : ''} {product.current_stock <= 0 && allowOutOfStockSale
              ? 'stock-allowed'
              : ''}"
            disabled={!allowOutOfStockSale && product.current_stock <= 0}
            onclick={() => addToCart(product)}
          >
            <span class="product-name">{product.name}</span>
            <span class="product-price"
              >{formatMoney(product.selling_price)} {currencySymbol}</span
            >
            <span
              class="product-stock {product.current_stock <= 0
                ? allowOutOfStockSale
                  ? 'out-allowed'
                  : 'low'
                : lowStockAlert && product.current_stock <= lowStockThreshold
                  ? 'low'
                  : ''}"
            >
              {#if product.current_stock <= 0}
                {allowOutOfStockSale
                  ? `หมด (คงเหลือ ${product.current_stock})`
                  : "สินค้าหมด"}
              {:else if lowStockAlert && product.current_stock <= lowStockThreshold}
                ใกล้หมด (คงเหลือ {product.current_stock})
              {:else}
                คงเหลือ {product.current_stock}
              {/if}
            </span>
          </button>
        {/each}
      </div>

      <div class="pos-pagination-wrapper">
        <Pagination
          id="pos-products"
          bind:currentPage
          bind:pageSize
          pageSizeOptions={[30, 50, 100]}
          {totalItems}
          itemLabel="รายการ"
        />
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
              <span class="cart-item-price"
                >{formatMoney(item.unit_price)} {currencySymbol}</span
              >
            </div>
            <div class="cart-item-controls">
              <button
                type="button"
                class="qty-btn"
                onclick={() => changeQty(index, -1)}>−</button
              >
              <span class="qty-display">{item.quantity}</span>
              <button
                type="button"
                class="qty-btn"
                onclick={() => changeQty(index, 1)}
                disabled={!allowOutOfStockSale && item.quantity >= item.stock}
                >+</button
              >
            </div>
            <div class="cart-item-right">
              <span class="cart-item-total"
                >{formatMoney(item.unit_price * item.quantity)}
                {currencySymbol}</span
              >
              <button
                type="button"
                class="remove-btn"
                title="ลบรายการ"
                onclick={() => removeItem(index)}
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  ><path d="M18 6 6 18" /><path d="m6 6 12 12" /></svg
                >
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="cart-summary">
      <button
        type="button"
        class="discount-row"
        onclick={() => (showDiscount = true)}
      >
        <span>ส่วนลด</span>
        <span class="discount-value">
          -{formatMoney(discountAmount)}
          {currencySymbol} <small>แก้ไข</small>
        </span>
      </button>
      <div class="summary-row">
        <span>รวมย่อย</span>
        <span>{formatMoney(cartSubtotal)} {currencySymbol}</span>
      </div>
      <div class="summary-total">
        <span>ยอดสุทธิ</span>
        <strong>{formatMoney(cartTotal)} {currencySymbol}</strong>
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
        <span class="pay-amount">{formatMoney(cartTotal)} {currencySymbol}</span
        >
      </button>
    </div>
  </aside>
</div>

<CheckoutModal
  open={showCheckout}
  total={cartTotal}
  {cart}
  {discountAmount}
  {currency}
  {currencySymbol}
  onClose={() => (showCheckout = false)}
  onCompleted={handleSaleCompleted}
/>

<HoldOrdersModal
  open={showHold}
  {currencySymbol}
  onClose={() => (showHold = false)}
  onRecall={recallHeld}
/>

<DiscountModal
  open={showDiscount}
  subtotal={cartSubtotal}
  initialDiscount={discountAmount}
  {currencySymbol}
  onCancel={() => (showDiscount = false)}
  onConfirm={(amount) => {
    discountAmount = amount;
    showDiscount = false;
  }}
/>

<ReturnModal
  open={showReturn}
  {currencySymbol}
  onClose={() => (showReturn = false)}
  onCompleted={(order) => {
    loadProducts();
    const orderNo = order?.order_no ? `บิล ${order.order_no}` : "";
    const totalText =
      order?.total_amount != null
        ? ` · ยอดคืน ${formatMoney(order.total_amount)} ${currencySymbol}`
        : "";
    const printNote =
      printBehavior === "direct" && printerConnection !== "none"
        ? " · กำลังพิมพ์ใบเสร็จ..."
        : "";
    showToast(
      "บันทึกการคืนสินค้าเรียบร้อยแล้ว",
      orderNo ? `${orderNo}${totalText}${printNote}` : undefined,
    );
    if (printBehavior === "preview") {
      previewOrder = order;
    }
  }}
/>

<ReceiptPreviewModal
  open={previewOrder !== null}
  order={previewOrder}
  promptpayAmountEnabled={previewPromptpayAmountEnabled}
  onClose={() => {
    previewOrder = null;
    previewPromptpayAmountEnabled = undefined;
  }}
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
  onClose={() =>
    (errorModal = { open: false, title: "", message: "", details: "" })}
/>

<!-- Toast Notification -->
{#if toastMessage}
  <aside class="pos-toast" role="status" aria-live="polite">
    <div class="toast-icon">
      <svg
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
        <polyline points="22 4 12 14.01 9 11.01" />
      </svg>
    </div>
    <div class="toast-content">
      <div class="toast-title">{toastMessage.title}</div>
      {#if toastMessage.subtitle}
        <div class="toast-subtitle">{toastMessage.subtitle}</div>
      {/if}
    </div>
    <button
      type="button"
      class="toast-close"
      aria-label="ปิดการแจ้งเตือน"
      onclick={() => (toastMessage = null)}
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
  </aside>
{/if}

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
    padding-right: var(--space-sm);
    transition: opacity 0.15s ease;
  }

  .product-grid.is-loading {
    opacity: 0.6;
    pointer-events: none;
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
    color: var(--color-accent-warning, #d08770);
    font-weight: 600;
  }

  .product-card.stock-allowed {
    border-style: dashed;
  }

  .pos-pagination-wrapper {
    flex-shrink: 0;
    padding-top: var(--space-xs);
  }

  .pos-pagination-wrapper :global(.pagination-bar) {
    margin-top: 0;
    padding-top: var(--space-sm);
    border-top: var(--border-subtle);
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

  /* Toast Notification */
  .pos-toast {
    position: fixed;
    bottom: var(--space-xl);
    right: var(--space-xl);
    background-color: var(--color-surface);
    border: 1px solid var(--color-muted);
    border-left: 4px solid var(--color-accent-success);
    color: var(--color-text-primary);
    padding: 12px 16px;
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(46, 52, 64, 0.12);
    animation: toastSlideUp 0.25s cubic-bezier(0.16, 1, 0.3, 1);
    z-index: 3000;
    display: flex;
    align-items: center;
    gap: 12px;
    max-width: 400px;
  }

  .toast-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background-color: rgba(163, 190, 140, 0.2);
    color: #43694f;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .toast-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .toast-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .toast-subtitle {
    font-size: 13px;
    color: #4c566a;
    word-break: break-word;
  }

  .toast-close {
    color: #7b889b;
    padding: 4px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    background: transparent;
    border: none;
    transition:
      background-color 0.15s,
      color 0.15s;
    flex-shrink: 0;
  }

  .toast-close:hover {
    background-color: var(--color-background);
    color: var(--color-text-primary);
  }

  @keyframes toastSlideUp {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
