<script lang="ts">
  import Dropdown from '$lib/components/Dropdown.svelte';

  // Store settings state
  let storeName = $state('Easy Stock');
  let storeAddress = $state('');
  let storePhone = $state('');
  let storeEmail = $state('');

  // Currency settings
  let currency = $state('THB');
  const currencyOptions = ['THB', 'USD', 'EUR', 'JPY', 'CNY'];

  // Low stock threshold
  let lowStockThreshold = $state(10);

  // Notification settings
  let lowStockAlert = $state(true);
  let dailyReport = $state(false);

  // Save state
  let saved = $state(false);

  function saveSettings() {
    // TODO: Connect to backend later
    saved = true;
    setTimeout(() => { saved = false; }, 3000);
  }
</script>

<header class="topbar">
  <h1>ตั้งค่า</h1>
  <button class="btn-primary" onclick={saveSettings}>บันทึก</button>
</header>

<div class="content-area">
  <div class="settings-grid">
    <!-- Store Information -->
    <div class="card">
      <h2 class="section-title">ข้อมูลร้านค้า</h2>
      <div class="form-group">
        <label for="store-name">ชื่อร้าน</label>
        <input
          id="store-name"
          type="text"
          class="input-field"
          bind:value={storeName}
          placeholder="ชื่อร้านค้า"
        />
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
      <div class="form-group">
        <label for="low-stock">ระดับสต็อกขั้นต่ำ (แจ้งเตือน)</label>
        <input
          id="low-stock"
          type="number"
          class="input-field"
          bind:value={lowStockThreshold}
          min="0"
        />
      </div>
      <div class="form-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={lowStockAlert} />
          แจ้งเตือนเมื่อสินค้าใกล้หมด
        </label>
      </div>
      <div class="form-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={dailyReport} />
          ส่งรายงานสรุปประจำวัน
        </label>
      </div>
    </div>
  </div>

  {#if saved}
    <div class="save-toast">บันทึกการตั้งค่าเรียบร้อยแล้ว</div>
  {/if}
</div>

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

  .form-group textarea {
    resize: vertical;
    min-height: 80px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    cursor: pointer;
    font-size: 15px;
    opacity: 1;
  }

  .checkbox-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    accent-color: var(--color-primary);
    cursor: pointer;
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
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>