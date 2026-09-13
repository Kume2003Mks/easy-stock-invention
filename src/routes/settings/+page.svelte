<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { beforeNavigate, goto } from "$app/navigation";
  import Dropdown from "$lib/components/Dropdown.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import ErrorModal from "$lib/components/ErrorModal.svelte";
  import { parseAppError } from "$lib/utils/errorHandler";
  import {
    SUPPORTED_CURRENCIES,
    getCurrencySymbol,
    getCurrencyName,
  } from "$lib/utils/currency";
  import { updateSystemCurrency } from "$lib/stores/settings";
  import type { AppSettings, SystemPrintersResponse } from "$lib/types";

  type SettingCategory = "store" | "printer" | "stock" | "system";

  // Navigation state: null = Category Hub (main list), string = Subpage view
  let activeCategory = $state<SettingCategory | null>(null);

  // Store settings state
  let storeName = $state("Easy Stock");
  let storeAddress = $state("");
  let storePhone = $state("");
  let storeEmail = $state("");

  // Currency settings
  let currency = $state("THB");
  const currencyDropdownOptions = SUPPORTED_CURRENCIES.map((c) => ({
    value: c.code,
    label: `${c.code} (${c.symbol}) - ${c.nameTh}`,
  }));
  let currentCurrencySymbol = $derived(getCurrencySymbol(currency));
  let currentCurrencyName = $derived(getCurrencyName(currency));

  // Stock & Sales settings
  let allowOutOfStockSale = $state(false);
  let lowStockThreshold = $state(10);
  let lowStockAlert = $state(true);
  let dailyReport = $state(false);

  // Toast notification state
  let toastMessage = $state<string | null>(null);
  let toastTimeout: ReturnType<typeof setTimeout> | null = null;

  function showToast(message: string) {
    if (toastTimeout) clearTimeout(toastTimeout);
    toastMessage = message;
    toastTimeout = setTimeout(() => {
      toastMessage = null;
    }, 3000);
  }

  // Printer settings (เครื่องพิมพ์ใบเสร็จ)
  let printBehavior = $state<"direct" | "preview" | "none">("direct");
  let autoPrintEnabled = $state(true);
  let receiptPreviewEnabled = $state(true);
  let paperSize = $state("80");
  let printerConnection = $state("none");
  let printerTarget = $state("");
  let promptpayId = $state("");
  let promptpayQrEnabled = $state(false);
  let promptpayAmountEnabled = $state(true);
  let printerCodepage = $state("26");
  let receiptFont = $state("sarabun");
  let receiptFooter = $state("ขอบคุณที่ใช้บริการ");
  let testingPrint = $state(false);
  let saving = $state(false);

  const receiptFontOptions = [
    { value: "sarabun", label: "ฟอนต์ Sarabun (ค่าเริ่มต้น)" },
    { value: "device", label: "ฟอนต์ในตัวเครื่องพิมพ์ (พิมพ์เร็ว)" },
  ];

  const codepageOptions = [
    { value: "26", label: "TIS-18 (Code 26) - ค่าเริ่มต้น" },
    { value: "21", label: "TIS-11 (Code 21) - ทั่วไป" },
    { value: "255", label: "CP874 (Code 255) - OEM" },
    { value: "20", label: "KU42 (Code 20) - แบบเดิม" },
  ];

  // System printers detection state
  let systemPrinters = $state<string[]>([]);
  let defaultSystemPrinter = $state<string | null>(null);
  let loadingPrinters = $state(false);
  let selectedUsbPrinter = $state("default");
  let isManualUsb = $state(false);

  let errorModal = $state({ open: false, title: "", message: "", details: "" });

  const paperSizeOptions = ["80", "58", "57"];
  const printerConnectionOptions = [
    {
      value: "none",
      label: "ไม่ระบุเครื่องพิมพ์ (ไม่มีเครื่องพิมพ์ / ไม่ใช้งาน)",
    },
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
      const res = (await invoke(
        "get_system_printers",
      )) as SystemPrintersResponse;
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
      if (printBehavior === "direct") {
        printBehavior = "preview";
      }
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
      errors.store_name = "กรุณากรอกชื่อร้านค้า";
    }
    formErrors = errors;
    return Object.keys(errors).length === 0;
  }

  function validateStockForm(): boolean {
    const errors: Record<string, string> = {};
    if (lowStockAlert) {
      const val = Number(lowStockThreshold);
      if (isNaN(val) || val < 1) {
        errors.low_stock_threshold =
          "ระดับสต็อกขั้นต่ำต้องเริ่มต้นจาก 1 ชิ้นขึ้นไป";
      }
    }
    formErrors = { ...formErrors, ...errors };
    return !errors.low_stock_threshold;
  }

  function validatePrinterForm(): boolean {
    const errors: Record<string, string> = {};
    if (promptpayQrEnabled) {
      const cleanId = promptpayId.replace(/[-\s]/g, "");
      if (!cleanId) {
        errors.promptpay_id = "กรุณากรอกหมายเลข PromptPay (จำเป็นเมื่อเปิดใช้งานพร้อมเพย์)";
      } else if (!/^\d+$/.test(cleanId)) {
        errors.promptpay_id = "หมายเลข PromptPay ต้องเป็นตัวเลขเท่านั้น";
      } else if (cleanId.length !== 10 && cleanId.length !== 13 && cleanId.length !== 15) {
        errors.promptpay_id = `หมายเลข PromptPay ต้องเป็นเบอร์โทร 10 หลัก หรือเลขบัตรประชาชน 13 หลัก (ปัจจุบันมี ${cleanId.length} หลัก)`;
      }
    }
    formErrors = { ...formErrors, ...errors };
    return !errors.promptpay_id;
  }

  // Saved Snapshot for Dirty State Tracking
  type SavedSnapshot = {
    storeName: string;
    storeAddress: string;
    storePhone: string;
    storeEmail: string;
    currency: string;
    allowOutOfStockSale: boolean;
    lowStockThreshold: number;
    lowStockAlert: boolean;
    dailyReport: boolean;
    printBehavior: "direct" | "preview" | "none";
    autoPrintEnabled: boolean;
    receiptPreviewEnabled: boolean;
    paperSize: string;
    printerConnection: string;
    printerTarget: string;
    selectedUsbPrinter: string;
    isManualUsb: boolean;
    promptpayId: string;
    promptpayQrEnabled: boolean;
    promptpayAmountEnabled: boolean;
    printerCodepage: string;
    receiptFont: string;
    receiptFooter: string;
  };

  let savedSnapshot = $state<SavedSnapshot | null>(null);

  function takeSnapshot(): SavedSnapshot {
    return {
      storeName,
      storeAddress,
      storePhone,
      storeEmail,
      currency,
      allowOutOfStockSale,
      lowStockThreshold,
      lowStockAlert,
      dailyReport,
      printBehavior,
      autoPrintEnabled,
      receiptPreviewEnabled,
      paperSize,
      printerConnection,
      printerTarget,
      selectedUsbPrinter,
      isManualUsb,
      promptpayId,
      promptpayQrEnabled,
      promptpayAmountEnabled,
      printerCodepage,
      receiptFont,
      receiptFooter,
    };
  }

  function isCategoryDirty(cat: SettingCategory): boolean {
    if (!savedSnapshot) return false;
    if (cat === "store") {
      return (
        storeName !== savedSnapshot.storeName ||
        storeAddress !== savedSnapshot.storeAddress ||
        storePhone !== savedSnapshot.storePhone ||
        storeEmail !== savedSnapshot.storeEmail
      );
    }
    if (cat === "printer") {
      return (
        printBehavior !== savedSnapshot.printBehavior ||
        autoPrintEnabled !== savedSnapshot.autoPrintEnabled ||
        receiptPreviewEnabled !== savedSnapshot.receiptPreviewEnabled ||
        paperSize !== savedSnapshot.paperSize ||
        printerConnection !== savedSnapshot.printerConnection ||
        printerTarget !== savedSnapshot.printerTarget ||
        selectedUsbPrinter !== savedSnapshot.selectedUsbPrinter ||
        isManualUsb !== savedSnapshot.isManualUsb ||
        promptpayId !== savedSnapshot.promptpayId ||
        promptpayQrEnabled !== savedSnapshot.promptpayQrEnabled ||
        promptpayAmountEnabled !== savedSnapshot.promptpayAmountEnabled ||
        printerCodepage !== savedSnapshot.printerCodepage ||
        receiptFont !== savedSnapshot.receiptFont ||
        receiptFooter !== savedSnapshot.receiptFooter
      );
    }
    if (cat === "stock") {
      return (
        allowOutOfStockSale !== savedSnapshot.allowOutOfStockSale ||
        lowStockAlert !== savedSnapshot.lowStockAlert ||
        Number(lowStockThreshold) !== Number(savedSnapshot.lowStockThreshold)
      );
    }
    if (cat === "system") {
      return currency !== savedSnapshot.currency;
    }
    return false;
  }

  function revertCategory(cat: SettingCategory) {
    if (!savedSnapshot) return;
    if (cat === "store") {
      storeName = savedSnapshot.storeName;
      storeAddress = savedSnapshot.storeAddress;
      storePhone = savedSnapshot.storePhone;
      storeEmail = savedSnapshot.storeEmail;
      clearFieldError("store_name");
    } else if (cat === "printer") {
      printBehavior = savedSnapshot.printBehavior;
      autoPrintEnabled = savedSnapshot.autoPrintEnabled;
      receiptPreviewEnabled = savedSnapshot.receiptPreviewEnabled;
      paperSize = savedSnapshot.paperSize;
      printerConnection = savedSnapshot.printerConnection;
      printerTarget = savedSnapshot.printerTarget;
      selectedUsbPrinter = savedSnapshot.selectedUsbPrinter;
      isManualUsb = savedSnapshot.isManualUsb;
      promptpayId = savedSnapshot.promptpayId;
      promptpayQrEnabled = savedSnapshot.promptpayQrEnabled;
      promptpayAmountEnabled = savedSnapshot.promptpayAmountEnabled;
      printerCodepage = savedSnapshot.printerCodepage;
      receiptFont = savedSnapshot.receiptFont;
      receiptFooter = savedSnapshot.receiptFooter;
      syncUsbSelection();
    } else if (cat === "stock") {
      allowOutOfStockSale = savedSnapshot.allowOutOfStockSale;
      lowStockAlert = savedSnapshot.lowStockAlert;
      lowStockThreshold = savedSnapshot.lowStockThreshold;
    } else if (cat === "system") {
      currency = savedSnapshot.currency;
    }
  }

  // Unsaved Changes Confirmation Modal State
  let unsavedModal = $state<{
    open: boolean;
    targetCategory: SettingCategory | null;
    targetUrl: string | null;
  }>({
    open: false,
    targetCategory: null,
    targetUrl: null,
  });

  function navigateToCategory(target: SettingCategory | null) {
    if (
      activeCategory &&
      activeCategory !== target &&
      isCategoryDirty(activeCategory)
    ) {
      unsavedModal = {
        open: true,
        targetCategory: target,
        targetUrl: null,
      };
      return;
    }
    activeCategory = target;
  }

  beforeNavigate(({ cancel, to }) => {
    if (activeCategory && isCategoryDirty(activeCategory)) {
      cancel();
      unsavedModal = {
        open: true,
        targetCategory: null,
        targetUrl: to?.url?.href || null,
      };
    }
  });

  async function handleDiscardAndLeave() {
    if (activeCategory) {
      revertCategory(activeCategory);
    }
    const { targetCategory, targetUrl } = unsavedModal;
    unsavedModal = { open: false, targetCategory: null, targetUrl: null };
    if (targetUrl) {
      await goto(targetUrl);
    } else {
      activeCategory = targetCategory;
    }
  }

  async function handleSaveAndLeave() {
    if (!activeCategory) return;
    const success = await saveCategory(activeCategory);
    if (success) {
      const { targetCategory, targetUrl } = unsavedModal;
      unsavedModal = { open: false, targetCategory: null, targetUrl: null };
      if (targetUrl) {
        await goto(targetUrl);
      } else {
        activeCategory = targetCategory;
      }
    }
  }

  function handleCancelUnsavedModal() {
    unsavedModal = { open: false, targetCategory: null, targetUrl: null };
  }

  onMount(async () => {
    try {
      loadSystemPrinters();

      const result = (await invoke("get_settings")) as AppSettings;

      storeName = result.store_name || "Easy Stock";
      storeAddress = result.store_address || "";
      storePhone = result.store_phone || "";
      storeEmail = result.store_email || "";
      currency = result.currency || "THB";
      const parsedThreshold = Number(result.low_stock_threshold);
      lowStockThreshold =
        !isNaN(parsedThreshold) && parsedThreshold >= 1 ? parsedThreshold : 10;
      lowStockAlert = result.low_stock_alert === "true";
      dailyReport = result.daily_report === "true";
      allowOutOfStockSale = result.allow_out_of_stock_sale === "true";
      if (result.print_behavior) {
        printBehavior = result.print_behavior as "direct" | "preview" | "none";
      } else if (
        result.auto_print_enabled === "true" &&
        result.receipt_preview_enabled === "false"
      ) {
        printBehavior = "direct";
      } else if (
        result.receipt_preview_enabled === "true" &&
        result.auto_print_enabled === "false"
      ) {
        printBehavior = "preview";
      } else if (
        result.receipt_preview_enabled === "false" &&
        result.auto_print_enabled === "false"
      ) {
        printBehavior = "none";
      } else {
        printBehavior = "direct";
      }
      autoPrintEnabled = printBehavior === "direct";
      receiptPreviewEnabled = printBehavior === "preview";
      paperSize = result.paper_size || "80";
      printerConnection = result.printer_connection || "none";
      printerTarget = result.printer_target || "";
      promptpayId = result.promptpay_id || "";
      promptpayQrEnabled = result.promptpay_qr_enabled
        ? result.promptpay_qr_enabled === "true"
        : Boolean(result.promptpay_id?.trim());
      promptpayAmountEnabled =
        result.promptpay_amount_enabled !== undefined
          ? result.promptpay_amount_enabled !== "false"
          : true;
      printerCodepage = result.printer_codepage || "26";
      receiptFont = result.receipt_font || "sarabun";
      receiptFooter =
        result.receipt_footer !== undefined
          ? result.receipt_footer
          : "ขอบคุณที่ใช้บริการ";

      syncUsbSelection();
      savedSnapshot = takeSnapshot();
      updateSystemCurrency(currency);
    } catch (e) {
      loadError = String(e);
    } finally {
      loading = false;
    }
  });

  async function saveCategory(cat: SettingCategory): Promise<boolean> {
    if (cat === "store") {
      if (!validateForm()) return false;
    }
    if (cat === "printer") {
      if (!validatePrinterForm()) return false;
    }
    if (cat === "stock") {
      if (!validateStockForm()) return false;
    }

    saving = true;
    try {
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
          print_behavior: printBehavior,
          auto_print_enabled: String(
            printerConnection !== "none" && printBehavior === "direct",
          ),
          receipt_preview_enabled: String(printBehavior === "preview"),
          paper_size: paperSize,
          printer_connection: printerConnection,
          printer_target: targetToSave,
          promptpay_id: promptpayId,
          promptpay_qr_enabled: String(promptpayQrEnabled),
          promptpay_amount_enabled: String(promptpayAmountEnabled),
          printer_codepage: printerCodepage,
          receipt_font: receiptFont,
          receipt_footer: receiptFooter,
        },
      });

      savedSnapshot = takeSnapshot();

      if (cat === "system") {
        updateSystemCurrency(currency);
      }

      const messages: Record<SettingCategory, string> = {
        store: "บันทึกข้อมูลร้านค้าเรียบร้อยแล้ว",
        printer: "บันทึกการตั้งค่าเครื่องพิมพ์เรียบร้อยแล้ว",
        stock: "บันทึกการตั้งค่าสต็อกและการขายเรียบร้อยแล้ว",
        system: "บันทึกการตั้งค่าระบบเรียบร้อยแล้ว",
      };

      showToast(messages[cat]);
      return true;
    } catch (e) {
      loadError = String(e);
      return false;
    } finally {
      saving = false;
    }
  }

  async function testPrint() {
    if (printerConnection === "none") {
      errorModal = {
        open: true,
        title: "ไม่ได้เชื่อมต่อเครื่องพิมพ์",
        message:
          "ระบบตั้งค่าอยู่ในโหมดไม่ระบุเครื่องพิมพ์ กรุณาเลือกประเภทการเชื่อมต่อเครื่องพิมพ์ (เช่น USB หรือ เครือข่าย) ก่อนทำการทดสอบ",
        details: "",
      };
      return;
    }

    if (printerConnection === "network" && !printerTarget.trim()) {
      errorModal = {
        open: true,
        title: "ยังไม่ได้ระบุที่อยู่เครื่องพิมพ์",
        message:
          "กรุณาระบุที่อยู่ IP และ Port ของเครื่องพิมพ์เครือข่าย (เช่น 192.168.1.200:9100) ก่อนทำการทดสอบ",
        details: "",
      };
      return;
    }

    if (promptpayQrEnabled && !validatePrinterForm()) {
      return;
    }

    testingPrint = true;
    try {
      const targetToTest =
        printerConnection === "usb"
          ? isManualUsb
            ? printerTarget
            : selectedUsbPrinter
          : printerConnection === "network"
            ? printerTarget
            : "";

      await invoke("print_test_receipt", {
        payload: {
          printer_connection: printerConnection,
          printer_target: targetToTest,
          paper_size: paperSize,
          promptpay_id: promptpayId,
          promptpay_qr_enabled: String(promptpayQrEnabled),
          promptpay_amount_enabled: String(promptpayAmountEnabled),
          store_name: storeName,
          store_address: storeAddress,
          store_phone: storePhone,
          receipt_footer: receiptFooter,
          printer_codepage: printerCodepage,
          receipt_font: receiptFont,
        },
      });
      errorModal = {
        open: false,
        title: "",
        message: "",
        details: "",
      };
      showToast("สั่งพิมพ์ใบเสร็จทดสอบเรียบร้อยแล้ว");
    } catch (e) {
      const parsed = parseAppError(e, "ทดสอบพิมพ์ไม่สำเร็จ");
      errorModal = {
        open: true,
        title: parsed.title,
        message: parsed.message,
        details: parsed.details ?? "",
      };
    } finally {
      testingPrint = false;
    }
  }

  const categoryTitles: Record<
    SettingCategory,
    { title: string; subtitle: string }
  > = {
    store: {
      title: "ข้อมูลร้านค้า",
      subtitle:
        "ชื่อร้านค้า, ที่อยู่, เบอร์โทรศัพท์, อีเมลสำหรับติดต่อและออกใบเสร็จ",
    },
    printer: {
      title: "เครื่องพิมพ์ใบเสร็จ",
      subtitle:
        "การเชื่อมต่อ USB / เครือข่าย, ขนาดกระดาษ, ฟอนต์ Sarabun, รหัสภาษาไทย, พร้อมเพย์ QR",
    },
    stock: {
      title: "สต็อกและการขาย",
      subtitle:
        "ระดับสต็อกขั้นต่ำ, การแจ้งเตือนสต็อกต่ำ, อนุญาตให้ขายสินค้าเมื่อหมดสต็อก",
    },
    system: {
      title: "การตั้งค่าระบบ",
      subtitle: "สกุลเงินที่ใช้ในระบบและการตั้งค่าพื้นฐาน",
    },
  };
</script>

<div class="settings-container">
  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>กำลังโหลดข้อมูลการตั้งค่า...</p>
    </div>
  {:else if loadError}
    <div class="error-state">
      <p>เกิดข้อผิดพลาด: {loadError}</p>
    </div>
  {:else}
    <!-- VIEW 1: CATEGORY HUB (Main Windows 11 style list) -->
    {#if activeCategory === null}
      <header class="hub-header">
        <div class="hub-title-group">
          <h1>ตั้งค่า</h1>
          <p class="hub-subtitle">
            จัดการข้อมูลร้านค้า เครื่องพิมพ์ใบเสร็จ สต็อกสินค้า และระบบ
          </p>
        </div>
      </header>

      <!-- Store Overview Hero Banner -->
      <div class="store-hero-card">
        <div class="store-hero-icon">
          <svg
            width="32"
            height="32"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
            <polyline points="9 22 9 12 15 12 15 22" />
          </svg>
        </div>
        <div class="store-hero-info">
          <h2 class="store-hero-name">{storeName || "Easy Stock"}</h2>
          <div class="store-hero-badges">
            <span class="badge">
              <span class="badge-dot"></span>
              {printerConnection === "none"
                ? "โหมดไม่ระบุเครื่องพิมพ์"
                : printerConnection === "usb"
                  ? `เครื่องพิมพ์ USB (${paperSize} มม.)`
                  : `เครื่องพิมพ์เครือข่าย (${paperSize} มม.)`}
            </span>
            <span class="badge badge-secondary">
              สกุลเงิน: {currency} ({currentCurrencySymbol})
            </span>
            {#if allowOutOfStockSale}
              <span class="badge badge-accent">อนุญาตขายสินค้าหมด</span>
            {/if}
          </div>
        </div>
      </div>

      <!-- Categories List (Matching Attached Design) -->
      <div class="category-list-section">
        <div class="category-list">
          <!-- 1. Store Profile -->
          <button
            type="button"
            class="category-item-card"
            onclick={() => navigateToCategory("store")}
          >
            <div class="category-icon-box">
              <svg
                width="22"
                height="22"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
                <polyline points="9 22 9 12 15 12 15 22" />
              </svg>
            </div>
            <div class="category-text-block">
              <span class="category-title">ข้อมูลร้านค้า</span>
              <span class="category-desc"
                >ชื่อร้าน, ที่อยู่, เบอร์โทรศัพท์,
                อีเมลสำหรับติดต่อและออกใบเสร็จ</span
              >
            </div>
            {#if isCategoryDirty("store")}
              <span class="dirty-indicator" title="มีการแก้ไขที่ยังไม่บันทึก"
                >• แก้ไขแล้ว</span
              >
            {/if}
            <div class="category-chevron">
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </div>
          </button>

          <!-- 2. Receipt Printer -->
          <button
            type="button"
            class="category-item-card"
            onclick={() => navigateToCategory("printer")}
          >
            <div class="category-icon-box">
              <svg
                width="22"
                height="22"
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
            </div>
            <div class="category-text-block">
              <span class="category-title">เครื่องพิมพ์ใบเสร็จ</span>
              <span class="category-desc"
                >การเชื่อมต่อ USB/เครือข่าย, ขนาดกระดาษ, ฟอนต์ Sarabun,
                รหัสภาษาไทย, พร้อมเพย์ QR, ข้อความท้ายใบเสร็จ</span
              >
            </div>
            {#if isCategoryDirty("printer")}
              <span class="dirty-indicator" title="มีการแก้ไขที่ยังไม่บันทึก"
                >• แก้ไขแล้ว</span
              >
            {/if}
            <div class="category-chevron">
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </div>
          </button>

          <!-- 3. Stock & Sales -->
          <button
            type="button"
            class="category-item-card"
            onclick={() => navigateToCategory("stock")}
          >
            <div class="category-icon-box">
              <svg
                width="22"
                height="22"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path
                  d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"
                />
                <path d="m3.3 7 8.7 5 8.7-5" />
                <path d="M12 22V12" />
              </svg>
            </div>
            <div class="category-text-block">
              <span class="category-title">สต็อกและการขาย</span>
              <span class="category-desc"
                >ระดับสต็อกขั้นต่ำ, การแจ้งเตือนสต็อกต่ำ,
                อนุญาตให้ขายสินค้าเมื่อหมดสต็อก</span
              >
            </div>
            {#if isCategoryDirty("stock")}
              <span class="dirty-indicator" title="มีการแก้ไขที่ยังไม่บันทึก"
                >• แก้ไขแล้ว</span
              >
            {/if}
            <div class="category-chevron">
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </div>
          </button>

          <!-- 4. System Settings -->
          <button
            type="button"
            class="category-item-card"
            onclick={() => navigateToCategory("system")}
          >
            <div class="category-icon-box">
              <svg
                width="22"
                height="22"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <circle cx="12" cy="12" r="3" />
                <path
                  d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
                />
              </svg>
            </div>
            <div class="category-text-block">
              <span class="category-title">การตั้งค่าระบบ</span>
              <span class="category-desc"
                >สกุลเงินที่ใช้ในระบบ ({currency} - {currentCurrencySymbol})
                และการตั้งค่าพื้นฐาน</span
              >
            </div>
            {#if isCategoryDirty("system")}
              <span class="dirty-indicator" title="มีการแก้ไขที่ยังไม่บันทึก"
                >• แก้ไขแล้ว</span
              >
            {/if}
            <div class="category-chevron">
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </div>
          </button>
        </div>
      </div>

      <!-- VIEW 2: DRILL-DOWN SUBPAGE (Per-category Settings) -->
    {:else}
      <div class="subpage-view">
        <!-- Subpage Top Bar & Breadcrumb -->
        <div class="subpage-header">
          <div class="breadcrumb-nav">
            <button
              type="button"
              class="btn-back"
              onclick={() => navigateToCategory(null)}
              title="กลับสู่หน้ารวมหมวดหมู่"
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <line x1="19" y1="12" x2="5" y2="12" />
                <polyline points="12 19 5 12 12 5" />
              </svg>
              <span>ตั้งค่า</span>
            </button>
            <span class="breadcrumb-separator">/</span>
            <span class="breadcrumb-current"
              >{categoryTitles[activeCategory].title}</span
            >
            {#if isCategoryDirty(activeCategory)}
              <span class="dirty-pill">ยังไม่บันทึก</span>
            {/if}
          </div>
        </div>

        <!-- Quick Switcher Tabs -->
        <div class="category-tabs-bar">
          <button
            type="button"
            class="tab-pill"
            class:active={activeCategory === "store"}
            onclick={() => navigateToCategory("store")}
          >
            ข้อมูลร้านค้า
            {#if isCategoryDirty("store")}<span class="tab-dot"></span>{/if}
          </button>
          <button
            type="button"
            class="tab-pill"
            class:active={activeCategory === "printer"}
            onclick={() => navigateToCategory("printer")}
          >
            เครื่องพิมพ์ใบเสร็จ
            {#if isCategoryDirty("printer")}<span class="tab-dot"></span>{/if}
          </button>
          <button
            type="button"
            class="tab-pill"
            class:active={activeCategory === "stock"}
            onclick={() => navigateToCategory("stock")}
          >
            สต็อกและการขาย
            {#if isCategoryDirty("stock")}<span class="tab-dot"></span>{/if}
          </button>
          <button
            type="button"
            class="tab-pill"
            class:active={activeCategory === "system"}
            onclick={() => navigateToCategory("system")}
          >
            การตั้งค่าระบบ
            {#if isCategoryDirty("system")}<span class="tab-dot"></span>{/if}
          </button>
        </div>

        <!-- Subpage Content Container -->
        <div class="subpage-content">
          <!-- SUBPAGE 1: STORE PROFILE -->
          {#if activeCategory === "store"}
            <div class="fluent-section-card">
              <div class="section-card-header">
                <div class="section-card-title-group">
                  <h3>ข้อมูลร้านค้าพื้นฐาน</h3>
                  <p>
                    ข้อมูลเหล่านี้จะถูกแสดงบนหัวใบเสร็จรับเงิน
                    และข้อมูลติดต่อของร้าน
                  </p>
                </div>
              </div>

              <div class="fluent-rows-group">
                <div class="fluent-row">
                  <div class="fluent-row-info">
                    <label
                      for="store-name"
                      class:label-error={!!formErrors.store_name}
                    >
                      ชื่อร้านค้า *
                    </label>
                    <span class="fluent-row-desc"
                      >ชื่อหลักที่จะพิมพ์บนใบเสร็จและหน้าจอ POS</span
                    >
                  </div>
                  <div class="fluent-row-control">
                    <input
                      id="store-name"
                      type="text"
                      class="input-field input-modern"
                      class:input-error={!!formErrors.store_name}
                      bind:value={storeName}
                      placeholder="เช่น Easy Stock"
                      oninput={() => clearFieldError("store_name")}
                    />
                    {#if formErrors.store_name}
                      <span class="error-text">{formErrors.store_name}</span>
                    {/if}
                  </div>
                </div>

                <div class="fluent-row">
                  <div class="fluent-row-info">
                    <label for="store-address">ที่อยู่ร้านค้า</label>
                    <span class="fluent-row-desc"
                      >ที่ตั้งร้านค้า สาขา หรือรายละเอียดที่ต้องการระบุบนใบเสร็จ</span
                    >
                  </div>
                  <div class="fluent-row-control">
                    <textarea
                      id="store-address"
                      class="input-field input-modern"
                      bind:value={storeAddress}
                      placeholder="เช่น 123/45 ถนนสุขุมวิท แขวงคลองเตย เขตคลองเตย กรุงเทพฯ 10110"
                      rows="3"
                    ></textarea>
                  </div>
                </div>

                <div class="fluent-row">
                  <div class="fluent-row-info">
                    <label for="store-phone">เบอร์โทรศัพท์ติดต่อ</label>
                    <span class="fluent-row-desc"
                      >เบอร์โทรศัพท์สำหรับลูกค้าสอบถามข้อมูล</span
                    >
                  </div>
                  <div class="fluent-row-control">
                    <input
                      id="store-phone"
                      type="tel"
                      class="input-field input-modern"
                      bind:value={storePhone}
                      placeholder="เช่น 02-123-4567 หรือ 081-234-5678"
                    />
                  </div>
                </div>

                <div class="fluent-row">
                  <div class="fluent-row-info">
                    <label for="store-email">อีเมลติดต่อ</label>
                    <span class="fluent-row-desc"
                      >อีเมลสำหรับติดต่อธุรกิจหรือส่งใบเสร็จดิจิทัล</span
                    >
                  </div>
                  <div class="fluent-row-control">
                    <input
                      id="store-email"
                      type="email"
                      class="input-field input-modern"
                      bind:value={storeEmail}
                      placeholder="เช่น contact@example.com"
                    />
                  </div>
                </div>
              </div>
            </div>

            <!-- SUBPAGE 2: RECEIPT PRINTER -->
          {:else if activeCategory === "printer"}
            {#if printerConnection === "none"}
              <div class="printer-none-notice">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <circle cx="12" cy="12" r="10" />
                  <line x1="12" y1="8" x2="12" y2="12" />
                  <line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
                <div>
                  <strong>โหมดไม่ระบุเครื่องพิมพ์</strong>
                  <p>
                    ระบบปิดการพิมพ์อัตโนมัติ
                    โดยยังสามารถดูตัวอย่างใบเสร็จบนหน้าจอ POS ได้ตามปกติ
                  </p>
                </div>
              </div>
            {/if}

            <!-- Printer Behavior Card -->
            <div class="fluent-section-card">
              <div class="section-card-header">
                <div class="section-card-title-group">
                  <h3>พฤติกรรมการพิมพ์ใบเสร็จ</h3>
                  <p>
                    กำหนดรูปแบบการทำงานของระบบเมื่อรับชำระเงินหรือทำรายการขายสำเร็จ
                  </p>
                </div>
              </div>

              <div class="print-behavior-list">
                <!-- Option 1: Direct Print -->
                <label
                  class="choice-card"
                  class:selected={printBehavior === "direct"}
                  class:disabled={printerConnection === "none"}
                >
                  <div class="choice-radio">
                    <input
                      type="radio"
                      name="print-behavior"
                      value="direct"
                      checked={printBehavior === "direct"}
                      disabled={printerConnection === "none"}
                      onchange={() => (printBehavior = "direct")}
                    />
                    <div class="radio-indicator"></div>
                  </div>
                  <div class="choice-content">
                    <div class="choice-header">
                      <span class="choice-title"
                        >พิมพ์ทันทีอัตโนมัติ (Fast Checkout)</span
                      >
                      {#if printerConnection === "none"}
                        <span class="choice-badge badge-warning"
                          >ต้องเชื่อมต่อเครื่องพิมพ์</span
                        >
                      {:else}
                        <span class="choice-badge badge-primary">แนะนำ</span>
                      {/if}
                    </div>
                    <p class="choice-desc">
                      สั่งพิมพ์ใบเสร็จออกเครื่องพิมพ์ทันทีเมื่อรับชำระเงินสำเร็จ
                      โดยไม่เปิดหน้าต่างพรีวิว
                      เหมาะสำหรับร้านค้าที่ต้องการความรวดเร็วในการขายหน้าร้าน
                    </p>
                  </div>
                </label>

                <!-- Option 2: Preview & Confirm -->
                <label
                  class="choice-card"
                  class:selected={printBehavior === "preview"}
                >
                  <div class="choice-radio">
                    <input
                      type="radio"
                      name="print-behavior"
                      value="preview"
                      checked={printBehavior === "preview"}
                      onchange={() => (printBehavior = "preview")}
                    />
                    <div class="radio-indicator"></div>
                  </div>
                  <div class="choice-content">
                    <div class="choice-header">
                      <span class="choice-title"
                        >แสดงตัวอย่างใบเสร็จก่อนพิมพ์ (Preview & Confirm)</span
                      >
                    </div>
                    <p class="choice-desc">
                      เปิดหน้าต่างพรีวิวใบเสร็จเพื่อตรวจสอบรายการ ยอดเงิน
                      และเงินทอนก่อน แล้วจึงกด Enter หรือกดยืนยันเพื่อสั่งพิมพ์
                    </p>
                  </div>
                </label>

                <!-- Option 3: Manual / No Auto Print -->
                <label
                  class="choice-card"
                  class:selected={printBehavior === "none"}
                >
                  <div class="choice-radio">
                    <input
                      type="radio"
                      name="print-behavior"
                      value="none"
                      checked={printBehavior === "none"}
                      onchange={() => (printBehavior = "none")}
                    />
                    <div class="radio-indicator"></div>
                  </div>
                  <div class="choice-content">
                    <div class="choice-header">
                      <span class="choice-title"
                        >ไม่พิมพ์ใบเสร็จอัตโนมัติ (Manual / No Print)</span
                      >
                      <span class="choice-badge badge-muted">ประหยัดกระดาษ</span
                      >
                    </div>
                    <p class="choice-desc">
                      บันทึกการขายเสร็จสิ้นทันทีโดยไม่เปิดหน้าต่างและไม่สั่งพิมพ์กระดาษ
                      เหมาะสำหรับร้านที่ไม่ต้องการพิมพ์ทุกออเดอร์
                      (สามารถพิมพ์ย้อนหลังได้จากประวัติบิล)
                    </p>
                  </div>
                </label>
              </div>
            </div>

            <!-- Printer Hardware Card -->
            <div class="fluent-section-card">
              <div class="section-card-header">
                <div class="section-card-title-group">
                  <h3>การเชื่อมต่อและฮาร์ดแวร์เครื่องพิมพ์</h3>
                  <p>
                    เลือกวิธีการเชื่อมต่อเครื่องพิมพ์ใบเสร็จความร้อน (Thermal
                    Printer)
                  </p>
                </div>
              </div>

              <div class="fluent-rows-group">
                <div class="fluent-row">
                  <div class="fluent-row-info">
                    <span class="fluent-row-title">ประเภทการเชื่อมต่อ</span>
                    <span class="fluent-row-desc"
                      >เลือกโหมดการเชื่อมต่อกับเครื่องพิมพ์ในระบบ</span
                    >
                  </div>
                  <div class="fluent-row-control">
                    <Dropdown
                      id="printer-connection"
                      label=""
                      options={printerConnectionOptions}
                      bind:value={printerConnection}
                      onchange={handleConnectionChange}
                      minWidth="100%"
                    />
                  </div>
                </div>

                {#if printerConnection === "usb"}
                  <div class="fluent-row">
                    <div class="fluent-row-info">
                      <span class="fluent-row-title">เครื่องพิมพ์ในระบบ</span>
                      <span class="fluent-row-desc"
                        >เลือกไดรเวอร์เครื่องพิมพ์ที่เชื่อมต่อผ่านสาย USB</span
                      >
                      <button
                        type="button"
                        class="btn-text-action"
                        onclick={loadSystemPrinters}
                        disabled={loadingPrinters}
                        title="ค้นหาเครื่องพิมพ์ใหม่"
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
                          <path
                            d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"
                          />
                          <polyline points="21 3 21 8 16 8" />
                        </svg>
                        <span
                          >{loadingPrinters
                            ? "กำลังค้นหา..."
                            : "ค้นหาเครื่องพิมพ์ใหม่"}</span
                        >
                      </button>
                    </div>
                    <div class="fluent-row-control">
                      <Dropdown
                        id="usb-printer-select"
                        label=""
                        options={usbPrinterOptions}
                        bind:value={selectedUsbPrinter}
                        onchange={handleUsbPrinterChange}
                        minWidth="100%"
                      />
                      {#if isManualUsb}
                        <input
                          id="printer-target-manual"
                          type="text"
                          class="input-field input-modern"
                          style="margin-top: 8px;"
                          bind:value={printerTarget}
                          placeholder="ระบุชื่อเครื่องพิมพ์ เช่น POS-80"
                        />
                      {/if}
                    </div>
                  </div>
                {:else if printerConnection === "network"}
                  <div class="fluent-row">
                    <div class="fluent-row-info">
                      <span class="fluent-row-title"
                        >ที่อยู่เครื่องพิมพ์เครือข่าย (IP:Port)</span
                      >
                      <span class="fluent-row-desc"
                        >ระบุ IP Address และพอร์ต TCP 9100 ของเครื่องพิมพ์</span
                      >
                    </div>
                    <div class="fluent-row-control">
                      <input
                        id="printer-target"
                        type="text"
                        class="input-field input-modern"
                        bind:value={printerTarget}
                        placeholder="192.168.1.200:9100"
                      />
                    </div>
                  </div>
                {/if}

                {#if printerConnection !== "none"}
                  <div class="fluent-row">
                    <div class="fluent-row-info">
                      <span class="fluent-row-title">ขนาดกระดาษความร้อน</span>
                      <span class="fluent-row-desc"
                        >ความกว้างของม้วนกระดาษใบเสร็จ</span
                      >
                    </div>
                    <div class="fluent-row-control">
                      <Dropdown
                        id="paper-size"
                        label=""
                        options={paperSizeOptions.map((s) => ({
                          value: s,
                          label: `${s} มม.`,
                        }))}
                        bind:value={paperSize}
                        minWidth="100%"
                      />
                    </div>
                  </div>

                  <div class="fluent-row">
                    <div class="fluent-row-info">
                      <span class="fluent-row-title">รูปแบบฟอนต์ใบเสร็จ</span>
                      <span class="fluent-row-desc">
                        {receiptFont === "sarabun"
                          ? "โหมดกราฟิกความคมชัดสูง เรนเดอร์สระและวรรณยุกต์ไทยเรียงตัวสวยงาม 100%"
                          : "โหมดข้อความดั้งเดิม พิมพ์เร็วที่สุดผ่านชิปฮาร์ดแวร์เครื่องพิมพ์"}
                      </span>
                    </div>
                    <div class="fluent-row-control">
                      <Dropdown
                        id="receipt-font"
                        label=""
                        options={receiptFontOptions}
                        bind:value={receiptFont}
                        minWidth="100%"
                      />
                    </div>
                  </div>

                  {#if receiptFont === "device"}
                    <div class="fluent-row">
                      <div class="fluent-row-info">
                        <span class="fluent-row-title"
                          >รหัสภาษาไทย (Code Page)</span
                        >
                        <span class="fluent-row-desc"
                          >ตารางรหัสภาษาไทยของฮาร์ดแวร์เครื่องพิมพ์</span
                        >
                      </div>
                      <div class="fluent-row-control">
                        <Dropdown
                          id="printer-codepage"
                          label=""
                          options={codepageOptions}
                          bind:value={printerCodepage}
                          minWidth="100%"
                        />
                      </div>
                    </div>
                  {/if}
                {/if}
              </div>
            </div>

            <!-- Receipt Footer Card -->
            <div class="fluent-section-card">
              <div class="section-card-header">
                <div class="section-card-title-group">
                  <h3>ข้อความท้ายใบเสร็จ</h3>
                  <p>
                    กำหนดข้อความขอบคุณ หรือเงื่อนไขท้ายใบเสร็จรับเงิน (เช่น สินค้าซื้อแล้วไม่รับเปลี่ยนคืน)
                  </p>
                </div>
              </div>

              <div class="fluent-rows-group">
                <div class="fluent-row">
                  <div class="fluent-row-info">
                    <label for="receipt-footer">ข้อความท้ายใบเสร็จ</label>
                    <span class="fluent-row-desc"
                      >ข้อความที่พิมพ์อยู่ส่วนล่างสุดของใบเสร็จ (รองรับการขึ้นบรรทัดใหม่)</span
                    >
                  </div>
                  <div class="fluent-row-control">
                    <textarea
                      id="receipt-footer"
                      class="input-field input-modern"
                      bind:value={receiptFooter}
                      placeholder="เช่น ขอบคุณที่ใช้บริการ&#10;สินค้าซื้อแล้วไม่รับเปลี่ยนหรือคืน"
                      rows="3"
                    ></textarea>
                  </div>
                </div>
              </div>
            </div>

            <!-- PromptPay QR on Receipt -->
            {#if printerConnection !== "none"}
              <div class="fluent-section-card">
                <div class="section-card-header">
                  <div class="section-card-title-group">
                    <h3>พร้อมเพย์ QR Code บนใบเสร็จ</h3>
                    <p>
                      พิมพ์ QR Code
                      สำหรับให้ลูกค้าสแกนชำระเงินท้ายใบเสร็จรับเงิน
                    </p>
                  </div>
                </div>

                <div class="fluent-rows-group">
                  <div class="fluent-row fluent-row-action">
                    <div class="fluent-row-info">
                      <span class="fluent-row-title"
                        >พิมพ์ QR พร้อมเพย์บนใบเสร็จ</span
                      >
                      <span class="fluent-row-desc"
                        >สร้าง QR Code พร้อมเพย์ตามยอดบิลโดยอัตโนมัติ</span
                      >
                    </div>
                    <div class="fluent-row-control">
                      <label class="toggle-switch">
                        <input
                          type="checkbox"
                          bind:checked={promptpayQrEnabled}
                          onchange={() => {
                            if (!promptpayQrEnabled) {
                              clearFieldError("promptpay_id");
                            }
                          }}
                        />
                        <span class="toggle-slider"></span>
                      </label>
                    </div>
                  </div>

                  {#if promptpayQrEnabled}
                    <div class="fluent-row">
                      <div class="fluent-row-info">
                        <label for="promptpay-id" class:label-error={!!formErrors.promptpay_id}>
                          หมายเลข PromptPay <span class="required-star">*</span>
                        </label>
                        <span class="fluent-row-desc"
                          >เบอร์โทรศัพท์ 10 หลัก หรือ เลขประจำตัวประชาชน 13 หลัก</span
                        >
                      </div>
                      <div class="fluent-row-control">
                        <input
                          id="promptpay-id"
                          type="text"
                          class="input-field input-modern"
                          class:input-error={!!formErrors.promptpay_id}
                          bind:value={promptpayId}
                          oninput={() => clearFieldError("promptpay_id")}
                          placeholder="เบอร์โทรศัพท์ 10 หลัก"
                          required
                        />
                        {#if formErrors.promptpay_id}
                          <span class="error-text">{formErrors.promptpay_id}</span>
                        {/if}
                      </div>
                    </div>

                    <div class="fluent-row fluent-row-action">
                      <div class="fluent-row-info">
                        <span class="fluent-row-title"
                          >ระบุยอดเงินใน QR Code ตามยอดบิล</span
                        >
                        <span class="fluent-row-desc"
                          >{promptpayAmountEnabled
                            ? "ระบุยอดเงินที่ต้องชำระใน QR Code โดยอัตโนมัติ (ลูกค้าสแกนแล้วยอดเงินจะขึ้นทันที)"
                            : "ไม่ระบุยอดเงินใน QR Code (ลูกค้าเป็นผู้กรอกยอดเงินเองเมื่อสแกน)"}</span
                        >
                      </div>
                      <div class="fluent-row-control">
                        <label class="toggle-switch">
                          <input
                            type="checkbox"
                            bind:checked={promptpayAmountEnabled}
                          />
                          <span class="toggle-slider"></span>
                        </label>
                      </div>
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Test Print Card -->
              <div class="fluent-section-card">
                <div class="section-card-header">
                  <div class="section-card-title-group">
                    <h3>ทดสอบการพิมพ์</h3>
                    <p>
                      พิมพ์ใบเสร็จทดสอบเพื่อตรวจสอบการเชื่อมต่อและความคมชัดของตัวอักษร
                    </p>
                  </div>
                  <button
                    type="button"
                    class="btn-outline btn-test-print"
                    onclick={testPrint}
                    disabled={testingPrint}
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
                    >
                      <polyline points="6 9 6 2 18 2 18 9" />
                      <path
                        d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"
                      />
                      <rect x="6" y="14" width="12" height="8" />
                    </svg>
                    <span
                      >{testingPrint
                        ? "กำลังพิมพ์..."
                        : "ทดสอบพิมพ์ใบเสร็จ"}</span
                    >
                  </button>
                </div>
              </div>
            {/if}

            <!-- SUBPAGE 3: STOCK & SALES -->
          {:else if activeCategory === "stock"}
            <div class="fluent-section-card">
              <div class="section-card-header">
                <div class="section-card-title-group">
                  <h3>กฎการขายและการตัดสต็อก</h3>
                  <p>
                    กำหนดเงื่อนไขการขายเมื่อสินค้าหมดและการแจ้งเตือนสต็อกใกล้หมด
                  </p>
                </div>
              </div>

              <div class="fluent-rows-group">
                <div class="fluent-row fluent-row-action">
                  <div class="fluent-row-info">
                    <span class="fluent-row-title"
                      >อนุญาตให้ขายสินค้าได้เมื่อสินค้าหมดสต๊อก</span
                    >
                    <span class="fluent-row-desc"
                      >เมื่อเปิดใช้งาน ระบบจะอนุญาตให้ขายต่อไปได้แม้สต็อกจะเหลือ
                      0 หรือติดลบ</span
                    >
                  </div>
                  <div class="fluent-row-control">
                    <label class="toggle-switch">
                      <input
                        type="checkbox"
                        bind:checked={allowOutOfStockSale}
                      />
                      <span class="toggle-slider"></span>
                    </label>
                  </div>
                </div>

                <div class="fluent-row fluent-row-action">
                  <div class="fluent-row-info">
                    <span class="fluent-row-title"
                      >เปิดการแจ้งเตือนสต็อกต่ำ</span
                    >
                    <span class="fluent-row-desc"
                      >แสดงการแจ้งเตือนเมื่อจำนวนสินค้าในคลังลดลงถึงเกณฑ์ขั้นต่ำ</span
                    >
                  </div>
                  <div class="fluent-row-control">
                    <label class="toggle-switch">
                      <input type="checkbox" bind:checked={lowStockAlert} />
                      <span class="toggle-slider"></span>
                    </label>
                  </div>
                </div>

                {#if lowStockAlert}
                  <div class="fluent-row">
                    <div class="fluent-row-info">
                      <label
                        for="low-stock-threshold"
                        class:label-error={!!formErrors.low_stock_threshold}
                        >ระดับสต็อกขั้นต่ำสำหรับแจ้งเตือน (ชิ้น)</label
                      >
                      <span class="fluent-row-desc"
                        >จำนวนชิ้นที่ระบบจะเริ่มแจ้งเตือนว่าสินค้าใกล้หมด</span
                      >
                    </div>
                    <div class="fluent-row-control">
                      <input
                        id="low-stock-threshold"
                        type="number"
                        min="1"
                        step="1"
                        class="input-field input-modern"
                        class:input-error={!!formErrors.low_stock_threshold}
                        bind:value={lowStockThreshold}
                        onkeydown={(e) => {
                          if (e.key === "-" || e.key === "e" || e.key === "+") {
                            e.preventDefault();
                          }
                        }}
                        oninput={(e) => {
                          clearFieldError("low_stock_threshold");
                          const target = e.currentTarget as HTMLInputElement;
                          if (target.value !== "" && Number(target.value) < 1) {
                            lowStockThreshold = 1;
                          }
                        }}
                        onblur={() => {
                          if (!lowStockThreshold || lowStockThreshold < 1) {
                            lowStockThreshold = 1;
                            clearFieldError("low_stock_threshold");
                          }
                        }}
                        placeholder="10"
                      />
                      {#if formErrors.low_stock_threshold}
                        <span class="error-text"
                          >{formErrors.low_stock_threshold}</span
                        >
                      {/if}
                    </div>
                  </div>
                {/if}
              </div>
            </div>

            <!-- SUBPAGE 4: SYSTEM SETTINGS -->
          {:else if activeCategory === "system"}
            <div class="fluent-section-card">
              <div class="section-card-header">
                <div class="section-card-title-group">
                  <h3>การตั้งค่าทั่วไปและสกุลเงิน</h3>
                  <p>กำหนดสกุลเงินหลักที่ใช้แสดงผลราคาและคำนวณเงินในระบบ</p>
                </div>
              </div>

              <div class="fluent-rows-group">
                <div class="fluent-row">
                  <div class="fluent-row-info">
                    <span class="fluent-row-title">สกุลเงินของระบบ</span>
                    <span class="fluent-row-desc"
                      >สกุลเงินที่ใช้สำหรับแสดงราคาสินค้า รายงาน และยอดชำระเงิน</span
                    >
                  </div>
                  <div class="fluent-row-control">
                    <Dropdown
                      id="currency"
                      label=""
                      options={currencyDropdownOptions}
                      bind:value={currency}
                      minWidth="100%"
                    />
                  </div>
                </div>

                <!-- Currency Visual Preview Card -->
                <div class="fluent-row currency-preview-row">
                  <div class="currency-preview-card">
                    <div class="currency-preview-badge">
                      <span class="preview-symbol">{currentCurrencySymbol}</span
                      >
                    </div>
                    <div class="currency-preview-details">
                      <div class="currency-preview-header">
                        <strong class="currency-preview-title"
                          >{currency} ({currentCurrencySymbol})</strong
                        >
                        <span class="currency-preview-subtitle"
                          >{currentCurrencyName}</span
                        >
                      </div>
                      <div class="currency-preview-examples">
                        <span class="example-tag">
                          ตัวอย่างราคา: <strong
                            >{currentCurrencySymbol}1,250.00</strong
                          >
                        </span>
                        <span class="example-tag">
                          หรือ <strong>1,250.00 {currentCurrencySymbol}</strong>
                        </span>
                      </div>
                      <p class="currency-preview-hint">
                        หน้าร้าน POS, รายงานสรุปยอดขาย,
                        และสินค้าคงคลังจะแสดงผลด้วยสัญลักษณ์ "{currentCurrencySymbol}"
                        ตามการตั้งค่านี้
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          {/if}

          <!-- Bottom Action Bar inside Subpage -->
          <div class="subpage-bottom-bar">
            <button
              type="button"
              class="btn-outline"
              onclick={() => navigateToCategory(null)}
            >
              ย้อนกลับ
            </button>
            <button
              type="button"
              class="btn-primary"
              onclick={() => saveCategory(activeCategory!)}
              disabled={saving}
            >
              {saving
                ? "กำลังบันทึก..."
                : `บันทึก${categoryTitles[activeCategory].title}`}
            </button>
          </div>
        </div>
      </div>
    {/if}
  {/if}

  <!-- Toast Notification -->
  {#if toastMessage}
    <div class="save-toast">
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="20 6 9 17 4 12" />
      </svg>
      <span>{toastMessage}</span>
    </div>
  {/if}
</div>

<!-- UNSAVED CHANGES CONFIRMATION MODAL -->
<Modal
  open={unsavedModal.open}
  title="บันทึกการเปลี่ยนแปลงหรือไม่?"
  onClose={handleCancelUnsavedModal}
  maxWidth="460px"
>
  <div class="unsaved-modal-body">
    <div class="unsaved-modal-icon">
      <svg
        width="28"
        height="28"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
    </div>
    <div class="unsaved-modal-text">
      <p>
        คุณมีการแก้ไขข้อมูลที่ยังไม่ได้บันทึก
        ต้องการบันทึกก่อนออกจากหน้านี้หรือไม่?
      </p>
    </div>
  </div>
  <div class="unsaved-modal-actions">
    <button
      type="button"
      class="btn-outline btn-discard"
      onclick={handleDiscardAndLeave}
    >
      ไม่บันทึก
    </button>
    <button
      type="button"
      class="btn-outline"
      onclick={handleCancelUnsavedModal}
    >
      ยกเลิก
    </button>
    <button
      type="button"
      class="btn-primary"
      onclick={handleSaveAndLeave}
      disabled={saving}
    >
      {saving ? "กำลังบันทึก..." : "บันทึกข้อมูล"}
    </button>
  </div>
</Modal>

<!-- ERROR MODAL -->
<ErrorModal
  open={errorModal.open}
  title={errorModal.title}
  message={errorModal.message}
  details={errorModal.details}
  onClose={() =>
    (errorModal = { open: false, title: "", message: "", details: "" })}
/>

<style>
  .settings-container {
    padding: var(--space-xl);
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    max-width: 960px;
    margin: 0 auto;
    width: 100%;
  }

  /* Hub Header */
  .hub-header {
    margin-bottom: var(--space-lg);
  }

  .hub-title-group h1 {
    font-size: 28px;
    margin-bottom: 4px;
    color: var(--color-text-primary);
  }

  .hub-subtitle {
    font-size: 14px;
    color: var(--color-text-primary);
    opacity: 0.7;
    margin: 0;
  }

  /* Store Overview Hero Banner */
  .store-hero-card {
    display: flex;
    align-items: center;
    gap: var(--space-lg);
    background: linear-gradient(
      135deg,
      var(--color-surface) 0%,
      rgba(94, 129, 172, 0.08) 100%
    );
    border: var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: 24px;
    margin-bottom: var(--space-lg);
  }

  .store-hero-icon {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    background-color: rgba(94, 129, 172, 0.12);
    color: var(--color-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .store-hero-info {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .store-hero-name {
    font-size: 20px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .store-hero-badges {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 500;
    padding: 4px 10px;
    border-radius: 20px;
    background-color: rgba(94, 129, 172, 0.12);
    color: var(--color-primary);
  }

  .badge-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--color-accent-success);
  }

  .badge-secondary {
    background-color: rgba(46, 52, 64, 0.06);
    color: var(--color-text-primary);
    opacity: 0.85;
  }

  .badge-accent {
    background-color: rgba(163, 190, 140, 0.2);
    color: #4c6b38;
  }

  /* CATEGORY LIST - MATCHING USER IMAGE SPEC */
  .category-list-section {
    display: flex;
    flex-direction: column;
  }

  .category-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .category-item-card {
    display: flex;
    align-items: center;
    gap: 16px;
    width: 100%;
    background-color: var(--color-surface);
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 16px 20px;
    text-align: left;
    cursor: pointer;
    transition: all 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .category-item-card:hover {
    background-color: rgba(94, 129, 172, 0.05);
    border-color: rgba(94, 129, 172, 0.4);
  }

  .category-icon-box {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: 10px;
    background-color: rgba(94, 129, 172, 0.08);
    color: var(--color-primary);
    flex-shrink: 0;
  }

  .category-text-block {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
    min-width: 0;
  }

  .category-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .category-desc {
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.65;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dirty-indicator {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-danger);
    background-color: rgba(191, 97, 106, 0.1);
    padding: 3px 8px;
    border-radius: 12px;
    flex-shrink: 0;
  }

  .category-chevron {
    color: var(--color-text-primary);
    opacity: 0.4;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition:
      transform 0.18s ease,
      opacity 0.18s ease;
  }

  .category-item-card:hover .category-chevron {
    transform: translateX(3px);
    opacity: 0.8;
  }

  /* SUBPAGE VIEW */
  .subpage-view {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .subpage-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-md);
  }

  .breadcrumb-nav {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
  }

  .btn-back {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: 500;
    color: var(--color-primary);
    padding: 6px 12px;
    border-radius: var(--radius-md);
    transition: background-color 0.15s ease;
  }

  .btn-back:hover {
    background-color: rgba(94, 129, 172, 0.15);
  }

  .breadcrumb-separator {
    color: var(--color-text-primary);
    opacity: 0.4;
  }

  .breadcrumb-current {
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .dirty-pill {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-danger);
    background-color: rgba(191, 97, 106, 0.12);
    padding: 2px 8px;
    border-radius: 10px;
  }

  /* Quick Switcher Tabs */
  .category-tabs-bar {
    display: flex;
    gap: 6px;
    overflow-x: auto;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--color-muted);
  }

  .tab-pill {
    position: relative;
    padding: 8px 16px;
    border-radius: 20px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    opacity: 0.75;
    background: transparent;
    transition: all 0.15s ease;
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .tab-pill:hover {
    opacity: 1;
    background-color: rgba(94, 129, 172, 0.08);
  }

  .tab-pill.active {
    opacity: 1;
    color: var(--color-primary);
    background-color: rgba(94, 129, 172, 0.12);
    font-weight: 600;
  }

  .tab-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--color-danger);
  }

  /* Subpage Content & Fluent Cards */
  .subpage-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .fluent-section-card {
    background-color: var(--color-surface);
    border: var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding-bottom: 12px;
    border-bottom: 1px solid rgba(216, 222, 233, 0.6);
  }

  .section-card-title-group h3 {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 4px 0;
  }

  .section-card-title-group p {
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.65;
    margin: 0;
  }

  .fluent-rows-group {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .fluent-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 24px;
    padding: 4px 0;
  }

  .fluent-row-action {
    padding: 8px 0;
  }

  .fluent-row-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
  }

  .fluent-row-info label,
  .fluent-row-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .fluent-row-desc {
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.65;
    line-height: 1.4;
  }

  .fluent-row-control {
    width: 320px;
    max-width: 50%;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }

  .input-modern {
    width: 100%;
  }

  .subpage-bottom-bar {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
    padding-top: var(--space-md);
    border-top: 1px solid var(--color-muted);
  }

  /* PRINT BEHAVIOR SINGLE CHOICE CARDS */
  .print-behavior-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 4px;
  }

  .choice-card {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 14px 18px;
    border: 1.5px solid var(--color-muted);
    border-radius: var(--radius-md);
    background-color: var(--color-surface);
    cursor: pointer;
    transition: all 0.18s cubic-bezier(0.16, 1, 0.3, 1);
    user-select: none;
  }

  .choice-card:hover:not(.disabled) {
    border-color: rgba(94, 129, 172, 0.6);
    background-color: rgba(94, 129, 172, 0.03);
  }

  .choice-card.selected {
    border-color: var(--color-primary);
    background-color: rgba(94, 129, 172, 0.06);
  }

  .choice-card.disabled {
    opacity: 0.55;
    cursor: not-allowed;
    background-color: rgba(216, 222, 233, 0.2);
  }

  .choice-radio {
    margin-top: 2px;
    position: relative;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .choice-radio input[type="radio"] {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
    pointer-events: none;
  }

  .radio-indicator {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 2px solid var(--color-muted);
    background-color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .choice-card:hover:not(.disabled) .radio-indicator {
    border-color: var(--color-primary);
  }

  .choice-card.selected .radio-indicator {
    border-color: var(--color-primary);
  }

  .choice-card.selected .radio-indicator::after {
    content: "";
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--color-primary);
  }

  .choice-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .choice-header {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .choice-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .choice-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
    line-height: 1.2;
  }

  .badge-primary {
    background-color: rgba(94, 129, 172, 0.16);
    color: var(--color-primary);
  }

  .badge-muted {
    background-color: rgba(100, 116, 139, 0.12);
    color: #475569;
  }

  .badge-warning {
    background-color: rgba(235, 203, 139, 0.3);
    color: #9c6e00;
  }

  .choice-desc {
    font-size: 12.5px;
    color: var(--color-text-primary);
    opacity: 0.7;
    line-height: 1.45;
    margin: 0;
  }

  /* TOGGLE SWITCH COMPONENT */
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 46px;
    height: 26px;
    flex-shrink: 0;
    cursor: pointer;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
    position: absolute;
  }

  .toggle-slider {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: #d8dee9;
    border-radius: 26px;
    transition: all 0.22s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .toggle-slider:before {
    position: absolute;
    content: "";
    height: 20px;
    width: 20px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    border-radius: 50%;
    transition: all 0.22s cubic-bezier(0.16, 1, 0.3, 1);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .toggle-switch input:checked + .toggle-slider {
    background-color: var(--color-primary);
  }

  .toggle-switch input:checked + .toggle-slider:before {
    transform: translateX(20px);
  }

  .toggle-switch input:disabled + .toggle-slider {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Notice and Actions */
  .printer-none-notice {
    display: flex;
    align-items: flex-start;
    gap: var(--space-md);
    padding: var(--space-md);
    background-color: rgba(94, 129, 172, 0.08);
    border: 1px solid rgba(94, 129, 172, 0.2);
    border-radius: var(--radius-md);
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

  .btn-text-action {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: none;
    border: none;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-primary);
    cursor: pointer;
    padding: 2px 6px;
    margin-top: 4px;
    border-radius: 6px;
    transition: all 0.15s ease;
    align-self: flex-start;
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

  .btn-test-print {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    font-size: 13px;
  }

  .label-error {
    color: var(--color-danger);
  }

  .required-star {
    color: var(--color-danger);
    font-weight: 700;
    margin-left: 2px;
  }

  .input-error {
    border-color: var(--color-danger) !important;
  }

  .error-text {
    font-size: 12px;
    color: var(--color-danger);
    font-weight: 500;
    margin-top: 4px;
  }

  /* Unsaved Modal Content */
  .unsaved-modal-body {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    margin-bottom: var(--space-lg);
  }

  .unsaved-modal-icon {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background-color: rgba(191, 97, 106, 0.12);
    color: var(--color-danger);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .unsaved-modal-text p {
    margin: 0;
    font-size: 15px;
    line-height: 1.5;
    color: var(--color-text-primary);
  }

  .unsaved-modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .btn-discard {
    color: var(--color-danger);
    border-color: rgba(191, 97, 106, 0.3);
  }

  .btn-discard:hover {
    background-color: rgba(191, 97, 106, 0.08);
    border-color: var(--color-danger);
  }

  /* Toast Notification */
  .save-toast {
    position: fixed;
    bottom: var(--space-xl);
    right: var(--space-xl);
    background-color: var(--color-accent-success);
    color: var(--color-surface);
    padding: 12px 20px;
    border-radius: var(--radius-md);
    font-weight: 500;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    animation: fadeIn 0.3s ease;
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 8px;
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

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .loading-state,
  .error-state {
    padding: 60px var(--space-xl);
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: var(--color-text-primary);
    opacity: 0.7;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(94, 129, 172, 0.2);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @media (max-width: 640px) {
    .fluent-row {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .fluent-row-control {
      width: 100%;
      max-width: 100%;
      align-items: flex-start;
    }
  }

  /* Currency Preview Card */
  .currency-preview-row {
    padding: 8px;
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .currency-preview-card {
    display: flex;
    align-items: center;
    gap: 16px;
    width: 100%;
    padding: 14px 16px;
    background: #ffffff;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  }

  .currency-preview-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 52px;
    height: 52px;
    min-width: 52px;
    background: linear-gradient(135deg, #e8f0fe 0%, #d2e3fc 100%);
    border: 1px solid #aecbfa;
    border-radius: var(--radius-md);
    color: var(--color-primary);
  }

  .preview-symbol {
    font-size: 26px;
    font-weight: 700;
    line-height: 1;
  }

  .currency-preview-details {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }

  .currency-preview-header {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .currency-preview-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .currency-preview-subtitle {
    font-size: 14px;
    color: var(--color-muted);
  }

  .currency-preview-examples {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .example-tag {
    display: inline-flex;
    align-items: center;
    padding: 3px 10px;
    background-color: var(--color-background);
    border: 1px solid var(--color-border);
    border-radius: 100px;
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .example-tag strong {
    margin-left: 4px;
    color: var(--color-primary);
  }

  .currency-preview-hint {
    margin: 0;
    font-size: 12px;
    color: var(--color-muted);
  }
</style>
