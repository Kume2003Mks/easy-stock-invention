<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import Modal from "$lib/components/Modal.svelte";
  import ErrorModal from "$lib/components/ErrorModal.svelte";
  import { parseAppError } from "$lib/utils/errorHandler";
  import type { Order, StoreSettings } from "$lib/types";

  let {
    open = false,
    order = null,
    promptpayAmountEnabled = undefined,
    onClose = () => {},
  }: {
    open?: boolean;
    order?: Order | null;
    promptpayAmountEnabled?: boolean;
    onClose?: () => void;
  } = $props();

  let store = $state<StoreSettings>({
    store_name: "Easy Stock",
    store_address: "",
    store_phone: "",
    receipt_footer: "ขอบคุณที่ใช้บริการ",
  });
  let receiptFont = $state("sarabun");
  let printing = $state(false);
  let printSuccess = $state(false);
  let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;

  let errorModal = $state({ open: false, title: "", message: "", details: "" });

  async function loadSettings() {
    try {
      const res = (await invoke("get_settings")) as any;
      store = res as StoreSettings;
      receiptFont = res.receipt_font || "sarabun";
    } catch {
      // ใช้ค่าเริ่มต้นหากโหลด settings ไม่ได้
    }
  }

  onMount(() => {
    loadSettings();
  });

  onDestroy(() => {
    if (autoCloseTimer) {
      clearTimeout(autoCloseTimer);
    }
  });

  function handleClose() {
    if (autoCloseTimer) {
      clearTimeout(autoCloseTimer);
      autoCloseTimer = null;
    }
    onClose();
  }

  // รีเซ็ตสถานะพิมพ์ทุกครั้งที่เปิดด้วยบิลใหม่ (โหมดตัวอย่างใบเสร็จ ต้องรอกดยืนยันเสมอ)
  $effect(() => {
    if (open) {
      printSuccess = false;
      if (autoCloseTimer) {
        clearTimeout(autoCloseTimer);
        autoCloseTimer = null;
      }
    } else {
      if (autoCloseTimer) {
        clearTimeout(autoCloseTimer);
        autoCloseTimer = null;
      }
    }
  });

  // รองรับการกดปุ่ม Enter เพื่อยืนยันการพิมพ์ทันที
  $effect(() => {
    if (!open) return;
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Enter" && !printing && !printSuccess) {
        e.preventDefault();
        printReceipt();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  });

  function formatMoney(n: number): string {
    return n.toLocaleString("th-TH", { minimumFractionDigits: 2 });
  }

  function paymentLabel(method: string): string {
    switch (method) {
      case "PROMPTPAY":
        return "พร้อมเพย์";
      case "TRANSFER":
        return "โอนเงิน";
      default:
        return "เงินสด";
    }
  }

  async function printReceipt() {
    if (!order || printing) return;
    printing = true;
    try {
      await invoke("print_receipt", {
        orderId: order.order_id,
        promptpayAmountEnabled,
      });
      printSuccess = true;

      // ปิด modal อัตโนมัติหลังพิมพ์สำเร็จ 1.2 วินาที
      if (autoCloseTimer) clearTimeout(autoCloseTimer);
      autoCloseTimer = setTimeout(() => {
        handleClose();
      }, 1200);
    } catch (e) {
      const parsed = parseAppError(e, "พิมพ์ใบเสร็จไม่สำเร็จ");
      errorModal = {
        open: true,
        title: parsed.title,
        message: parsed.message,
        details: parsed.details ?? "",
      };
    } finally {
      printing = false;
    }
  }
</script>

<Modal {open} title="ตัวอย่างใบเสร็จ" onClose={handleClose} maxWidth="420px">
  <div class="preview-content">
    {#if order}
      <div
        class="receipt-paper {receiptFont === 'sarabun'
          ? 'font-sarabun'
          : 'font-device'}"
      >
        <div class="receipt-center receipt-store">{store.store_name}</div>
        {#if store.store_address}
          <div class="receipt-center receipt-muted">{store.store_address}</div>
        {/if}
        {#if store.store_phone}
          <div class="receipt-center receipt-muted">
            โทร. {store.store_phone}
          </div>
        {/if}
        <div class="receipt-sep"></div>
        {#if order.order_type === "RETURN"}
          <div class="receipt-center receipt-return">
            ** ใบเสร็จรับเงิน (คืนสินค้า) **
          </div>
        {/if}
        <div class="receipt-row">
          <span>เลขที่</span><span>{order.order_no}</span>
        </div>
        <div class="receipt-row">
          <span>วันที่</span><span>{order.order_date}</span>
        </div>
        <div class="receipt-sep"></div>

        {#each order.items as item (item.item_id)}
          <div class="receipt-item-name">{item.product_name}</div>
          <div class="receipt-row">
            <span
              >&nbsp;&nbsp;{item.quantity} x {formatMoney(
                item.unit_price,
              )}</span
            >
            <span>{formatMoney(item.line_total)}</span>
          </div>
        {/each}
        <div class="receipt-sep"></div>

        <div class="receipt-row">
          <span>รวมย่อย</span><span>{formatMoney(order.subtotal)}</span>
        </div>
        {#if order.discount_amount > 0}
          <div class="receipt-row">
            <span>ส่วนลด</span><span>-{formatMoney(order.discount_amount)}</span
            >
          </div>
        {/if}
        <div class="receipt-row receipt-total">
          <span>ยอดรวม</span><span>{formatMoney(order.total_amount)}</span>
        </div>
        <div class="receipt-row">
          <span>ชำระ ({paymentLabel(order.payment_method)})</span><span
            >{formatMoney(order.paid_amount)}</span
          >
        </div>
        {#if order.change_amount > 0}
          <div class="receipt-row">
            <span>เงินทอน</span><span>{formatMoney(order.change_amount)}</span>
          </div>
        {/if}
        {#if order.note}
          <div class="receipt-sep"></div>
          <div class="receipt-row">
            <span>หมายเหตุ</span><span>{order.note}</span>
          </div>
        {/if}
        {#if store.receipt_footer !== undefined}
          {#if store.receipt_footer.trim()}
            <div class="receipt-sep"></div>
            {#each store.receipt_footer.split("\n") as line}
              {#if line.trim()}
                <div class="receipt-center receipt-muted">{line.trim()}</div>
              {/if}
            {/each}
          {/if}
        {:else}
          <div class="receipt-sep"></div>
          <div class="receipt-center receipt-muted">ขอบคุณที่ใช้บริการ</div>
        {/if}
      </div>

      <!-- Action Buttons -->
      <div class="preview-actions">
        <button type="button" class="btn-outline" onclick={handleClose}>
          ไม่พิมพ์ใบเสร็จ
        </button>
        <button
          type="button"
          class="btn-primary print-btn"
          class:btn-success={printSuccess}
          onclick={printReceipt}
          disabled={printing}
        >
          {#if printing}
            <span class="btn-inner">
              <svg
                class="spin"
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
                <polyline points="21 3 21 8 16 8" />
              </svg>
              <span>กำลังพิมพ์...</span>
            </span>
          {:else if printSuccess}
            <span class="btn-inner">
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
              <span>สั่งพิมพ์ใบเสร็จเรียบร้อยแล้ว</span>
            </span>
          {:else}
            <span class="btn-inner">
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="6 9 6 2 18 2 18 9" />
                <path
                  d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"
                />
                <rect x="6" y="14" width="12" height="8" />
              </svg>
              <span>ยืนยันพิมพ์ใบเสร็จ (Enter)</span>
            </span>
          {/if}
        </button>
      </div>

      {#if printSuccess}
        <div class="print-status-banner">
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="20 6 9 17 4 12" />
          </svg>
          <span>สั่งพิมพ์เรียบร้อยแล้ว (กำลังปิดหน้าต่างอัตโนมัติ...)</span>
        </div>
      {/if}
    {/if}
  </div>
</Modal>

<ErrorModal
  open={errorModal.open}
  title={errorModal.title}
  message={errorModal.message}
  details={errorModal.details}
  onClose={() =>
    (errorModal = { open: false, title: "", message: "", details: "" })}
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
    font-size: 13px;
    line-height: 1.7;
    color: #2e3440;
    overflow-wrap: break-word;
    word-break: break-word;
  }

  .receipt-paper.font-sarabun {
    font-family:
      "Sarabun",
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      Roboto,
      sans-serif;
  }

  .receipt-paper.font-device {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
      monospace;
  }

  .receipt-center {
    text-align: center;
  }

  .receipt-store {
    font-size: 17px;
    font-weight: 700;
    overflow-wrap: break-word;
    word-break: break-word;
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
    min-width: 80px;
  }

  .print-btn {
    flex: 2;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .print-btn.btn-success {
    background-color: var(--color-accent-success);
    color: var(--color-surface);
  }

  .btn-inner {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 500;
  }

  .print-status-banner {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 8px 16px;
    border-radius: var(--radius-md);
    background-color: rgba(163, 190, 140, 0.18);
    color: #3b5726;
    font-size: 13px;
    font-weight: 500;
    animation: fadeIn 0.25s ease;
  }

  .spin {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
