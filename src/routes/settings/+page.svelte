<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Dropdown from "$lib/components/Dropdown.svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";

  // Store settings state
  let storeName = $state("Easy Stock");
  let storeAddress = $state("");
  let storePhone = $state("");
  let storeEmail = $state("");

  // Currency settings
  let currency = $state("THB");
  const currencyOptions = ["THB", "USD", "EUR", "JPY", "CNY"];

  // Low stock threshold
  let lowStockThreshold = $state(10);

  // Notification settings
  let lowStockAlert = $state(true);
  let dailyReport = $state(false);

  // Save state
  let saved = $state(false);

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
      const result = (await invoke("get_settings")) as {
        store_name: string;
        store_address: string;
        store_phone: string;
        store_email: string;
        currency: string;
        low_stock_threshold: string;
        low_stock_alert: string;
        daily_report: string;
      };

      storeName = result.store_name;
      storeAddress = result.store_address;
      storePhone = result.store_phone;
      storeEmail = result.store_email;
      currency = result.currency;
      lowStockThreshold = Number(result.low_stock_threshold) || 10;
      lowStockAlert = result.low_stock_alert === "true";
      dailyReport = result.daily_report === "true";
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
</style>
