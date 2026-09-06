<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Dropdown from "$lib/components/Dropdown.svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import ErrorModal from "$lib/components/ErrorModal.svelte";
  import { parseAppError } from "$lib/utils/errorHandler";
  // Store settings state
  let storeName = $state("Easy Stock");
  let storeAddress = $state("");
  let storePhone = $state("");
  let storeEmail = $state("");

  // Currency settings
  let currency = $state("THB");
  const currencyOptions = ["THB", "USD", "EUR", "JPY", "CNY"];

  // Stock & Sales settings
  let allowOutOfStockSale = $state(false);
  let lowStockThreshold = $state(10);
  let lowStockAlert = $state(true);
  let dailyReport = $state(false);

  // Save state
  let saved = $state(false);

  // Printer settings (เครื่องพิมพ์ใบเสร็จ)
  let autoPrintEnabled = $state(true);
  let receiptPreviewEnabled = $state(true);
  let paperSize = $state("80");
  let printerConnection = $state("none");
  let printerTarget = $state("");
  let promptpayId = $state("");
  let testingPrint = $state(false);

  // System printers detection state
  let systemPrinters = $state<string[]>([]);
  let defaultSystemPrinter = $state<string | null>(null);
  let loadingPrinters = $state(false);
  let selectedUsbPrinter = $state("default");
  let isManualUsb = $state(false);

  let errorModal = $state({ open: false, title: "", message: "", details: "" });

  const paperSizeOptions = ["80", "58", "57"];
  const printerConnectionOptions = [
    { value: "none", label: "ไม่ระบุเครื่องพิมพ์ (ไม่มีเครื่องพิมพ์ / ไม่ใช้งาน)" },
    { value: "usb", label: "ต่อตรงกับเครื่อง (USB / ไดรเวอร์ในระบบ)" },
    { value: "network", label: "เครือข่าย (Network TCP 9100)" },
  ];

  const usbPrinterOptions = $derived.by(() => {
    const opts = [
      {
        value: "default",
        label: `เครื่องพิมพ์เริ่มต้นของระบบ${defaultSystemPrinter ? ` (${defaultSystemPrinter})` : ""}`,
      },
      ...systemPrinters.map((p) => ({ value: p, label: p })),
      { value: "__manual__", label: "ระบุชื่อเครื่องพิมพ์เอง..." },
    ];
    return opts;
  });

  async function loadSystemPrinters() {
    loadingPrinters = true;
    try {
      const res = (await invoke("get_system_printers")) as {
        printers: string[];
        default_printer: string | null;
      };
      systemPrinters = res.printers || [];
      defaultSystemPrinter = res.default_printer || null;
      syncUsbSelection();
    } catch (e) {
      console.error("Failed to load printers:", e);
      systemPrinters = [];
    } finally {
      loadingPrinters = false;
    }
  }

  function syncUsbSelection() {
    if (!printerTarget || printerTarget === "default") {
      selectedUsbPrinter = "default";
      isManualUsb = false;
    } else if (systemPrinters.includes(printerTarget)) {
      selectedUsbPrinter = printerTarget;
      isManualUsb = false;
    } else {
      selectedUsbPrinter = "__manual__";
      isManualUsb = true;
    }
  }

  function handleUsbPrinterChange(val: string | number) {
    const strVal = String(val);
    selectedUsbPrinter = strVal;
    if (strVal === "__manual__") {
      isManualUsb = true;
      if (printerTarget === "default") printerTarget = "";
    } else {
      isManualUsb = false;
      printerTarget = strVal;
    }
  }

  function handleConnectionChange(val: string | number) {
    printerConnection = String(val);
    if (printerConnection === "none") {
      autoPrintEnabled = false;
    } else if (printerConnection === "usb") {
      if (!printerTarget || printerTarget === "") {
        printerTarget = "default";
        selectedUsbPrinter = "default";
      }
    }
  }

  // Loading / error state
  let loading = $state(true);
  let loadError = $state("");

  // Validation state
  let formErrors = $state<Record<string, string>>({});

  function clearFieldError(field: string) {
    if (formErrors[field]) {
      const { [field]: _removed, ...rest } = formErrors;
      formErrors = rest;
    }
  }

  function validateForm(): boolean {
    const errors: Record<string, string> = {};

    if (!storeName.trim()) {
      errors.store_name = "กรุณากรอกชื่อร้าน";
    }

    formErrors = errors;
    return Object.keys(errors).length === 0;
  }

  // Save confirmation state
  let showSaveConfirm = $state(false);

  onMount(async () => {
    try {
      loadSystemPrinters();

      const result = (await invoke("get_settings")) as {
        store_name: string;
        store_address: string;
        store_phone: string;
        store_email: string;
        currency: string;
        low_stock_threshold: string;
        low_stock_alert: string;
        daily_report: string;
        allow_out_of_stock_sale?: string;
        auto_print_enabled: string;
        receipt_preview_enabled: string;
        paper_size: string;
        printer_connection: string;
        printer_target: string;
        promptpay_id: string;
      };

      storeName = result.store_name;
      storeAddress = result.store_address;
      storePhone = result.store_phone;
      storeEmail = result.store_email;
      currency = result.currency;
      lowStockThreshold = Number(result.low_stock_threshold) || 10;
      lowStockAlert = result.low_stock_alert === "true";
      dailyReport = result.daily_report === "true";
      allowOutOfStockSale = result.allow_out_of_stock_sale === "true";
      autoPrintEnabled = result.auto_print_enabled !== "false";
      receiptPreviewEnabled = result.receipt_preview_enabled !== "false";
      paperSize = result.paper_size || "80";
      printerConnection = result.printer_connection || "none";
      printerTarget = result.printer_target || "";
      promptpayId = result.promptpay_id || "";

      syncUsbSelection();
    } catch (e) {
      loadError = String(e);
    } finally {
      loading = false;
    }
  });

  function requestSaveSettings() {
    if (!validateForm()) return;
    showSaveConfirm = true;
  }

  function cancelSave() {
    showSaveConfirm = false;
  }

  async function confirmSaveSettings() {
    try {
      showSaveConfirm = false;
      const targetToSave =
        printerConnection === "usb"
          ? isManualUsb
            ? printerTarget
            : selectedUsbPrinter
          : printerConnection === "network"
          ? printerTarget
          : "";

      await invoke("save_settings", {
        payload: {
          store_name: storeName,
          store_address: storeAddress,
          store_phone: storePhone,
          store_email: storeEmail,
          currency,
          low_stock_threshold: String(lowStockThreshold),
          low_stock_alert: String(lowStockAlert),
          daily_report: String(dailyReport),
          allow_out_of_stock_sale: String(allowOutOfStockSale),
          auto_print_enabled: String(printerConnection !== "none" && autoPrintEnabled),
          receipt_preview_enabled: String(receiptPreviewEnabled),
          paper_size: paperSize,
          printer_connection: printerConnection,
          printer_target: targetToSave,
          promptpay_id: promptpayId,
        },
      });
      saved = true;
      setTimeout(() => {
        saved = false;
      }, 3000);
    } catch (e) {
      loadError = String(e);
    }
  }

  async function testPrint() {
    if (printerConnection === "none") {
      errorModal = {
        open: true,
        title: "ไม่ได้เชื่อมต่อเครื่องพิมพ์",
        message: "ระบบตั้งค่าอยู่ในโหมดไม่ระบุเครื่องพิมพ์ กรุณาเลือกประเภทการเชื่อมต่อเครื่องพิมพ์ (เช่น USB หรือ เครือข่าย) ก่อนทำการทดสอบ",
        details: "",
      };
      return;
    }

    testingPrint = true;
    try {
      await invoke("print_test_receipt");
      errorModal = {
        open: false,
        title: "",
        message: "",
        details: "",
      };
      saved = true;
      setTimeout(() => {
        saved = false;
      }, 3000);
    } catch (e) {
      const parsed = parseAppError(e, "ทดสอบพิมพ์ไม่สำเร็จ");
      errorModal = { open: true, title: parsed.title, message: parsed.message, details: parsed.details ?? "" };
    } finally {
      testingPrint = false;
    }
  }
</script>

<header class="topbar">
  <h1>ตั้งค่า</h1>
  <button class="btn-primary" onclick={requestSaveSettings} disabled={loading}>
    บันทึก
  </button>
</header>

<div class="content-area">
  {#if loading}
    <div class="loading-state">กำลังโหลดการตั้งค่า...</div>
  {:else if loadError}
    <div class="error-state">เกิดข้อผิดพลาด: {loadError}</div>
  {:else}
    <div class="settings-grid">
      <!-- Store Information -->
      <div class="card">
        <h2 class="section-title">ข้อมูลร้านค้า</h2>
        <div class="form-group">
          <label for="store-name" class:label-error={!!formErrors.store_name}>
            ชื่อร้าน *
          </label>
          <input
            id="store-name"
            type="text"
            class="input-field"
            class:input-error={!!formErrors.store_name}
            bind:value={storeName}
            placeholder="ชื่อร้านค้า"
            oninput={() => clearFieldError("store_name")}
          />
          {#if formErrors.store_name}
            <span class="error-text">{formErrors.store_name}</span>
          {/if}
        </div>
        <div class="form-group">
          <label for="store-address">ที่อยู่</label>
          <textarea
            id="store-address"
            class="input-field"
            bind:value={storeAddress}
            placeholder="ที่อยู่ร้านค้า"
            rows="3"
          ></textarea>
        </div>
        <div class="form-group">
          <label for="store-phone">เบอร์โทรศัพท์</label>
          <input
            id="store-phone"
            type="tel"
            class="input-field"
            bind:value={storePhone}
            placeholder="เบอร์โทรศัพท์"
          />
        </div>
        <div class="form-group">
          <label for="store-email">อีเมล</label>
          <input
            id="store-email"
            type="email"
            class="input-field"
            bind:value={storeEmail}
            placeholder="อีเมลติดต่อ"
          />
        </div>
      </div>

      <!-- Printer Settings -->
      <div class="card">
        <h2 class="section-title">เครื่องพิมพ์ใบเสร็จ</h2>

        {#if printerConnection === "none"}
          <div class="printer-none-notice">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="8" x2="12" y2="12" />
              <line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
            <div>
              <strong>โหมดไม่ระบุเครื่องพิมพ์</strong>
              <p>สำหรับร้านค้าที่ไม่มีเครื่องพิมพ์ หรือยังไม่ต้องการพิมพ์ใบเสร็จ ระบบจะปิดการพิมพ์อัตโนมัติ โดยยังสามารถดูตัวอย่างใบเสร็จบนหน้าจอ POS ได้ตามปกติ</p>
            </div>
          </div>
        {/if}

        <div class="checkbox-group">
          <label class="checkbox-label" class:disabled-label={printerConnection === "none"}>
            <input
              type="checkbox"
              bind:checked={autoPrintEnabled}
              disabled={printerConnection === "none"}
            />
            <span>พิมพ์ใบเสร็จอัตโนมัติหลังชำระเงิน</span>
          </label>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={receiptPreviewEnabled} />
            <span>แสดงตัวอย่างใบเสร็จก่อนพิมพ์</span>
          </label>
        </div>

        <div class="form-group">
          <Dropdown
            id="printer-connection"
            label="ประเภทการเชื่อมต่อ"
            options={printerConnectionOptions}
            bind:value={printerConnection}
            onchange={handleConnectionChange}
            minWidth="100%"
          />
        </div>

        {#if printerConnection === "usb"}
          <div class="form-group">
            <div class="label-with-action">
              <label for="usb-printer-select">เลือกเครื่องพิมพ์ที่ติดตั้งในเครื่อง</label>
              <button
                type="button"
                class="btn-text-action"
                onclick={loadSystemPrinters}
                disabled={loadingPrinters}
                title="ค้นหาเครื่องพิมพ์ที่เชื่อมต่อใหม่"
              >
                <svg
                  width="13"
                  height="13"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class:spin={loadingPrinters}
                >
                  <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
                  <polyline points="21 3 21 8 16 8" />
                </svg>
                <span>{loadingPrinters ? "กำลังค้นหา..." : "ค้นหาใหม่"}</span>
              </button>
            </div>
            <Dropdown
              id="usb-printer-select"
              label=""
              options={usbPrinterOptions}
              bind:value={selectedUsbPrinter}
              onchange={handleUsbPrinterChange}
              minWidth="100%"
            />
          </div>

          {#if isManualUsb}
            <div class="form-group">
              <label for="printer-target">ชื่อเครื่องพิมพ์ที่ติดตั้งในระบบ</label>
              <input
                id="printer-target"
                type="text"
                class="input-field"
                bind:value={printerTarget}
                placeholder="เช่น EPSON TM-T82X-II หรือ POS-80"
              />
            </div>
          {/if}
        {:else if printerConnection === "network"}
          <div class="form-group">
            <label for="printer-target">ที่อยู่เครื่องพิมพ์เครือข่าย (IP:Port)</label>
            <input
              id="printer-target"
              type="text"
              class="input-field"
              bind:value={printerTarget}
              placeholder="192.168.1.200:9100"
            />
          </div>
        {/if}

        {#if printerConnection !== "none"}
          <div class="form-group">
            <Dropdown
              id="paper-size"
              label="ขนาดกระดาษ (มม.)"
              options={paperSizeOptions.map((s) => ({ value: s, label: `${s} มม.` }))}
              bind:value={paperSize}
              minWidth="100%"
            />
          </div>
        {/if}

        <div class="form-group">
          <label for="promptpay-id">เลข PromptPay สำหรับ QR บนใบเสร็จ (ไม่บังคับ)</label>
          <input
            id="promptpay-id"
            type="text"
            class="input-field"
            bind:value={promptpayId}
            placeholder="เบอร์โทร 10 หลัก / เลขบัตรประชาชน 13 หลัก"
          />
        </div>

        {#if printerConnection !== "none"}
          <button type="button" class="btn-outline" onclick={testPrint} disabled={testingPrint}>
            {testingPrint ? "กำลังพิมพ์..." : "ทดสอบการพิมพ์"}
          </button>
        {/if}
      </div>

      <!-- Stock & Sales Settings -->
      <div class="card">
        <h2 class="section-title">การตั้งค่าสต็อกและการขาย</h2>
        <div class="checkbox-group">
          <label class="checkbox-label checkbox-label-block">
            <input type="checkbox" bind:checked={allowOutOfStockSale} />
            <div class="checkbox-text">
              <span class="checkbox-title">อนุญาตให้ขายสินค้าได้เมื่อสินค้าหมดสต๊อก</span>
              <p class="checkbox-desc">เมื่อเปิดใช้งาน ระบบจะอนุญาตให้ขายสินค้าต่อไปได้แม้สต็อกจะเหลือ 0 หรือติดลบ</p>
            </div>
          </label>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={lowStockAlert} />
            <span>เปิดการแจ้งเตือนสต็อกต่ำ</span>
          </label>
        </div>
        {#if lowStockAlert}
          <div class="form-group">
            <label for="low-stock-threshold">ระดับสต็อกขั้นต่ำสำหรับแจ้งเตือน (ชิ้น)</label>
            <input
              id="low-stock-threshold"
              type="number"
              min="0"
              class="input-field"
              bind:value={lowStockThreshold}
              placeholder="10"
            />
          </div>
        {/if}
      </div>

      <!-- System Settings -->
      <div class="card">
        <h2 class="section-title">การตั้งค่าระบบ</h2>
        <div class="form-group">
          <Dropdown
            id="currency"
            label="สกุลเงิน"
            options={currencyOptions.map((c) => ({ value: c, label: c }))}
            bind:value={currency}
            minWidth="100%"
          />
        </div>
      </div>
    </div>
  {/if}

  {#if saved}
    <div class="save-toast">บันทึกการตั้งค่าเรียบร้อยแล้ว</div>
  {/if}
</div>

<ConfirmModal
  open={showSaveConfirm}
  title="ยืนยันการบันทึกการตั้งค่า"
  message={`ต้องการบันทึกการตั้งค่า ใช่หรือไม่?`}
  confirmText="บันทึก"
  cancelText="ยกเลิก"
  variant="primary"
  onConfirm={confirmSaveSettings}
  onCancel={cancelSave}
/>

<ErrorModal
  open={errorModal.open}
  title={errorModal.title}
  message={errorModal.message}
  details={errorModal.details}
  onClose={() => (errorModal = { open: false, title: "", message: "", details: "" })}
/>

<style>
  .topbar {
    padding: var(--space-xl) var(--space-xl) 0 var(--space-xl);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .topbar h1 {
    margin-bottom: 0;
  }

  .content-area {
    padding: var(--space-xl);
    flex: 1;
    overflow-y: auto;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: var(--space-lg);
  }

  .section-title {
    font-size: 18px;
    margin-bottom: var(--space-lg);
    color: var(--color-text-primary);
  }

  .checkbox-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    margin-bottom: var(--space-lg);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 15px;
    cursor: pointer;
  }

  .checkbox-label input[type='checkbox'] {
    width: 18px;
    height: 18px;
    accent-color: var(--color-primary);
    cursor: pointer;
  }

  .checkbox-label-block {
    align-items: flex-start;
  }

  .checkbox-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .checkbox-title {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .checkbox-desc {
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.65;
    margin: 0;
    line-height: 1.4;
  }

  .form-group {
    margin-bottom: var(--space-md);
  }

  .form-group label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    opacity: 0.8;
    margin-bottom: 6px;
  }

  .form-group label.label-error {
    color: var(--color-danger);
    opacity: 1;
  }

  .input-error {
    border-color: var(--color-danger) !important;
  }

  .input-error:focus {
    box-shadow: 0 0 0 3px rgba(191, 97, 106, 0.15);
    border-color: var(--color-danger);
  }

  .error-text {
    font-size: 12px;
    color: var(--color-danger);
    font-weight: 500;
  }

  .form-group textarea {
    resize: vertical;
    min-height: 80px;
  }

  .loading-state,
  .error-state {
    padding: var(--space-xl);
    text-align: center;
    font-size: 15px;
    color: var(--color-text-primary);
    opacity: 0.7;
  }

  .error-state {
    color: var(--color-danger);
    opacity: 1;
  }

  .save-toast {
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
  .printer-none-notice {
    display: flex;
    align-items: flex-start;
    gap: var(--space-md);
    padding: var(--space-md);
    background-color: rgba(94, 129, 172, 0.08);
    border: 1px solid rgba(94, 129, 172, 0.2);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-md);
    color: var(--color-text-primary);
  }

  .printer-none-notice svg {
    color: var(--color-primary);
    flex-shrink: 0;
    margin-top: 2px;
  }

  .printer-none-notice strong {
    display: block;
    font-size: 14px;
    margin-bottom: 2px;
  }

  .printer-none-notice p {
    margin: 0;
    font-size: 13px;
    opacity: 0.75;
    line-height: 1.4;
  }

  .label-with-action {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 6px;
  }

  .label-with-action label {
    margin-bottom: 0;
  }

  .btn-text-action {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: none;
    border: none;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-primary);
    cursor: pointer;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    transition: all 0.15s ease;
  }

  .btn-text-action:hover:not(:disabled) {
    background-color: rgba(94, 129, 172, 0.1);
  }

  .btn-text-action:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-text-action svg.spin {
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

  .disabled-label {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
