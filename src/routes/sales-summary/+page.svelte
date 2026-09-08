<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import ErrorModal from "$lib/components/ErrorModal.svelte";
  import DatePicker from "$lib/components/DatePicker.svelte";
  import SalesChart from "$lib/components/sales/SalesChart.svelte";
  import { parseAppError } from "$lib/utils/errorHandler";
  import { getCurrencySymbol } from "$lib/utils/currency";
  import { currencyStore, fetchSystemCurrency } from "$lib/stores/settings";
  import type {
    SalesSummaryFilter,
    SalesSummaryResult,
    SalesSummaryBreakdownItem,
  } from "$lib/types";

  type PresetPeriod = "today" | "yesterday" | "7days" | "thisMonth" | "thisYear" | "custom";

  let activePreset = $state<PresetPeriod>("today");
  let startDate = $state("");
  let endDate = $state("");
  let loading = $state(false);
  let exporting = $state(false);
  let salesData = $state<SalesSummaryResult | null>(null);
  let currency = $state("THB");
  let currencySymbol = $derived(getCurrencySymbol(currency));
  let saveSuccessMessage = $state("");
  let successTimer: ReturnType<typeof setTimeout> | undefined;

  let errorModal = $state({
    open: false,
    title: "",
    message: "",
    details: "",
  });

  function showError(err: unknown, title: string) {
    const parsed = parseAppError(err, title);
    errorModal = {
      open: true,
      title: parsed.title,
      message: parsed.message,
      details: parsed.details ?? "",
    };
  }

  function formatLocalDate(d: Date): string {
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
  }

  function applyPreset(preset: PresetPeriod) {
    activePreset = preset;
    const now = new Date();
    const todayStr = formatLocalDate(now);

    if (preset === "today") {
      startDate = todayStr;
      endDate = todayStr;
    } else if (preset === "yesterday") {
      const yesterday = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1);
      const yesterdayStr = formatLocalDate(yesterday);
      startDate = yesterdayStr;
      endDate = yesterdayStr;
    } else if (preset === "7days") {
      const past = new Date(now.getTime() - 6 * 24 * 60 * 60 * 1000);
      startDate = formatLocalDate(past);
      endDate = todayStr;
    } else if (preset === "thisMonth") {
      const firstDay = new Date(now.getFullYear(), now.getMonth(), 1);
      const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0);
      startDate = formatLocalDate(firstDay);
      endDate = formatLocalDate(lastDay);
    } else if (preset === "thisYear") {
      startDate = `${now.getFullYear()}-01-01`;
      endDate = `${now.getFullYear()}-12-31`;
    }

    if (preset !== "custom") {
      loadSalesSummary();
    }
  }

  async function loadSalesSummary() {
    if (!startDate || !endDate) return;
    if (endDate < startDate) {
      showError(
        new Error("วันที่สิ้นสุดต้องไม่น้อยกว่าวันที่เริ่มต้น"),
        "ช่วงวันที่ไม่ถูกต้อง",
      );
      return;
    }
    loading = true;
    try {
      const filter: SalesSummaryFilter = {
        startDate,
        endDate,
      };
      salesData = await invoke<SalesSummaryResult>("get_sales_summary", {
        filter,
      });
    } catch (err) {
      showError(err, "ไม่สามารถโหลดข้อมูลสรุปยอดขายได้");
    } finally {
      loading = false;
    }
  }

  function formatCurrency(val: number): string {
    return new Intl.NumberFormat("th-TH", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(val);
  }

  function formatDisplayDate(period: string, type: string): string {
    if (!period) return "-";
    const thaiMonthsFull = [
      "มกราคม",
      "กุมภาพันธ์",
      "มีนาคม",
      "เมษายน",
      "พฤษภาคม",
      "มิถุนายน",
      "กรกฎาคม",
      "สิงหาคม",
      "กันยายน",
      "ตุลาคม",
      "พฤศจิกายน",
      "ธันวาคม",
    ];
    if (type === "hourly") {
      return `${period} น.`;
    }
    if (type === "monthly") {
      const parts = period.split("-");
      if (parts.length === 2) {
        const m = parseInt(parts[1], 10) - 1;
        const y = parts[0];
        return `${thaiMonthsFull[m] ?? period} ${y}`;
      }
      return period;
    }
    // Daily "YYYY-MM-DD"
    const parts = period.split("-");
    if (parts.length === 3) {
      const d = parseInt(parts[2], 10);
      const m = parseInt(parts[1], 10) - 1;
      const y = parts[0];
      return `${d} ${thaiMonthsFull[m] ?? ""} ${y}`;
    }
    return period;
  }

  async function exportToCsv() {
    if (!salesData || salesData.breakdown.length === 0 || exporting) return;

    const headers = [
      "ช่วงเวลา/วันที่",
      "จำนวนบิล (บิล)",
      `ยอดขายรวม (${currency})`,
      `ยอดเฉลี่ยต่อบิล (${currency})`,
    ];
    const rows = salesData.breakdown.map((item) => [
      `"${item.period}"`,
      item.orderCount,
      item.totalSales.toFixed(2),
      item.averageOrderValue.toFixed(2),
    ]);

    // Total row
    rows.push([
      '"รวมทั้งสิ้น"',
      salesData.kpi.totalOrders,
      salesData.kpi.totalSales.toFixed(2),
      salesData.kpi.averageOrderValue.toFixed(2),
    ]);

    // Prepend UTF-8 BOM for Thai characters in Excel
    const csvContent =
      "\uFEFF" +
      [headers.join(","), ...rows.map((r) => r.join(","))].join("\r\n");
    const defaultFilename = `sales_summary_${startDate}_to_${endDate}.csv`;

    exporting = true;
    try {
      const savedPath = await invoke<string | null>("export_csv_file", {
        defaultFilename,
        content: csvContent,
      });

      if (savedPath) {
        saveSuccessMessage = `บันทึกไฟล์เรียบร้อยแล้ว: ${savedPath}`;
        if (successTimer) clearTimeout(successTimer);
        successTimer = setTimeout(() => {
          saveSuccessMessage = "";
        }, 6000);
      }
    } catch (err) {
      // Fallback สำหรับกรณีรันนอก Tauri
      try {
        const blob = new Blob([csvContent], {
          type: "text/csv;charset=utf-8;",
        });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.setAttribute("href", url);
        link.setAttribute("download", defaultFilename);
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
      } catch {
        showError(err, "บันทึกไฟล์ไม่สำเร็จ");
      }
    } finally {
      exporting = false;
    }
  }

  // Chart data: if single day and hourlyBreakdown exists, show hourly in chart; otherwise breakdown
  const isSingleDay = $derived(Boolean(startDate && startDate === endDate));

  const chartItems = $derived.by((): SalesSummaryBreakdownItem[] => {
    if (!salesData) return [];
    if (
      isSingleDay &&
      salesData.hourlyBreakdown &&
      salesData.hourlyBreakdown.length > 0
    ) {
      return salesData.hourlyBreakdown;
    }
    return salesData.breakdown;
  });

  const chartPeriodType = $derived(
    isSingleDay &&
      salesData?.hourlyBreakdown &&
      salesData.hourlyBreakdown.length > 0
      ? "hourly"
      : (salesData?.periodType ?? "daily"),
  );

  const isDateRangeInvalid = $derived(
    Boolean(startDate && endDate && endDate < startDate),
  );

  onMount(() => {
    const unsub = currencyStore.subscribe((c) => {
      if (c) currency = c;
    });
    fetchSystemCurrency();
    applyPreset("today");
    return () => {
      unsub();
    };
  });
</script>

<svelte:head>
  <title>สรุปยอดขาย - Easy Stock</title>
</svelte:head>

<!-- Header Topbar (ตรงตามโครงสร้างของหน้า สินค้าคงคลัง, หมวดหมู่, ผู้จัดจำหน่าย) -->
<header class="topbar">
  <h1>สรุปยอดขาย</h1>
  <div class="topbar-actions">
    <button
      class="btn-primary export-btn"
      onclick={exportToCsv}
      disabled={loading ||
        exporting ||
        !salesData ||
        salesData.breakdown.length === 0}
      title="เลือกโฟลเดอร์ปลายทางและส่งออกรายงานเป็นไฟล์ CSV"
    >
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="7 10 12 15 17 10" />
        <line x1="12" y1="15" x2="12" y2="3" />
      </svg>
      <span>{exporting ? "กำลังบันทึก..." : "ส่งออก CSV"}</span>
    </button>
  </div>
</header>

<!-- Content Area (ตรงตามโครงสร้างของหน้า สินค้าคงคลัง, หมวดหมู่, ผู้จัดจำหน่าย) -->
<div class="content-area">
  {#if saveSuccessMessage}
    <div class="success-banner">
      <div class="success-content">
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          class="success-icon"
        >
          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
          <polyline points="22 4 12 14.01 9 11.01" />
        </svg>
        <span class="success-text">{saveSuccessMessage}</span>
      </div>
      <button
        class="banner-close"
        onclick={() => (saveSuccessMessage = "")}
        aria-label="ปิดการแจ้งเตือน"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>
  {/if}

  <div class="card summary-card">
    <!-- Top Filter Controls (ตรงตามตำแหน่ง .table-actions เหมือนหน้า สินค้าคงคลัง) -->
    <div class="table-actions">
      <div class="preset-group">
        <button
          class="preset-btn {activePreset === 'today' ? 'active' : ''}"
          onclick={() => applyPreset("today")}
        >
          วันนี้
        </button>
        <button
          class="preset-btn {activePreset === 'yesterday' ? 'active' : ''}"
          onclick={() => applyPreset("yesterday")}
        >
          เมื่อวาน
        </button>
        <button
          class="preset-btn {activePreset === '7days' ? 'active' : ''}"
          onclick={() => applyPreset("7days")}
        >
          7 วันล่าสุด
        </button>
        <button
          class="preset-btn {activePreset === 'thisMonth' ? 'active' : ''}"
          onclick={() => applyPreset("thisMonth")}
        >
          เดือนนี้
        </button>
        <button
          class="preset-btn {activePreset === 'thisYear' ? 'active' : ''}"
          onclick={() => applyPreset("thisYear")}
        >
          ปีนี้
        </button>
        <button
          class="preset-btn {activePreset === 'custom' ? 'active' : ''}"
          onclick={() => applyPreset("custom")}
        >
          กำหนดเอง
        </button>
      </div>

      {#if activePreset === "custom"}
        <div class="custom-date-inline">
          <label class="date-field">
            <span class="field-label">ตั้งแต่วันที่:</span>
            <DatePicker
              bind:value={startDate}
              max={endDate || undefined}
            />
          </label>
          <label class="date-field">
            <span class="field-label">ถึงวันที่:</span>
            <DatePicker
              bind:value={endDate}
              min={startDate || undefined}
              hasError={isDateRangeInvalid}
            />
          </label>
          <button
            class="btn-primary apply-btn"
            onclick={loadSalesSummary}
            disabled={loading || !startDate || !endDate || isDateRangeInvalid}
            title={isDateRangeInvalid
              ? "วันที่สิ้นสุดต้องไม่น้อยกว่าวันที่เริ่มต้น"
              : "ค้นหาข้อมูลสรุปยอดขาย"}
          >
            {#if loading}
              กำลังโหลด...
            {:else}
              ค้นหาข้อมูล
            {/if}
          </button>

          {#if isDateRangeInvalid}
            <div class="date-error-badge" role="alert">
              <svg
                width="15"
                height="15"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <circle cx="12" cy="12" r="10" />
                <line x1="12" y1="8" x2="12" y2="12" />
                <line x1="12" y1="16" x2="12.01" y2="16" />
              </svg>
              <span>วันที่สิ้นสุดต้องไม่น้อยกว่าวันที่เริ่มต้น</span>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    {#if loading && !salesData}
      <div class="loading-container">
        <div class="spinner"></div>
        <p>กำลังประมวลผลข้อมูลสรุปยอดขาย...</p>
      </div>
    {:else}
      <!-- 1. KPI Cards Grid -->
      <div class="kpi-grid">
        <!-- Card 1: ยอดขายรวมสุทธิ -->
        <div class="kpi-box">
          <div class="kpi-header">
            <span class="kpi-label">ยอดขายรวมสุทธิ</span>
            <span class="kpi-icon-wrapper sales-icon">
              <span class="kpi-symbol-icon">{currencySymbol}</span>
            </span>
          </div>
          <div class="kpi-amount">
            <span class="currency-symbol">{currencySymbol}</span>{formatCurrency(
              salesData?.kpi.totalSales ?? 0,
            )}
          </div>
          <div class="kpi-subtext">รายได้สุทธิหลังหักส่วนลดและการคืนสินค้า</div>
        </div>

        <!-- Card 2: จำนวนบิลทั้งหมด -->
        <div class="kpi-box">
          <div class="kpi-header">
            <span class="kpi-label">จำนวนบิลทั้งหมด</span>
            <span class="kpi-icon-wrapper orders-icon">
              <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path
                  d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
                />
                <polyline points="14 2 14 8 20 8" />
                <line x1="16" y1="13" x2="8" y2="13" />
                <line x1="16" y1="17" x2="8" y2="17" />
                <polyline points="10 9 9 9 8 9" />
              </svg>
            </span>
          </div>
          <div class="kpi-amount">
            {salesData?.kpi.totalOrders ?? 0} <span class="unit">บิล</span>
          </div>
          <div class="kpi-subtext">จำนวนรายการขายที่สำเร็จในช่วงนี้</div>
        </div>

        <!-- Card 3: ยอดเฉลี่ยต่อบิล -->
        <div class="kpi-box">
          <div class="kpi-header">
            <span class="kpi-label">ยอดเฉลี่ยต่อบิล</span>
            <span class="kpi-icon-wrapper avg-icon">
              <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <circle cx="12" cy="12" r="10" />
                <polyline points="12 6 12 12 16 14" />
              </svg>
            </span>
          </div>
          <div class="kpi-amount">
            <span class="currency-symbol">{currencySymbol}</span>{formatCurrency(
              salesData?.kpi.averageOrderValue ?? 0,
            )}
          </div>
          <div class="kpi-subtext">มูลค่าเฉลี่ยต่อบิล (Basket Size)</div>
        </div>
      </div>

      <div class="section-divider"></div>

      <!-- 2. Chart Section -->
      <div class="chart-section">
        <div class="section-header">
          <div class="section-title-wrap">
            <h2>แนวโน้มยอดขาย</h2>
            <span class="badge badge-primary">
              {#if chartPeriodType === "hourly"}
                {#if activePreset === "today"}
                  แจกแจงรายชั่วโมง (วันนี้)
                {:else if activePreset === "yesterday"}
                  แจกแจงรายชั่วโมง (เมื่อวาน)
                {:else}
                  แจกแจงรายชั่วโมง
                {/if}
              {:else if chartPeriodType === "monthly"}
                แจกแจงรายเดือน
              {:else}
                แจกแจงรายวัน
              {/if}
            </span>
          </div>
          <span class="period-caption">
            ช่วงเวลา: {startDate} ถึง {endDate}
          </span>
        </div>

        <div class="chart-body">
          <SalesChart items={chartItems} periodType={chartPeriodType} {currencySymbol} />
        </div>
      </div>

      <div class="section-divider"></div>

      <!-- 3. Data Breakdown Table Section -->
      <div class="table-section">
        <div class="section-header">
          <div class="section-title-wrap">
            <h2>ตารางแจกแจงยอดขาย</h2>
            <span class="row-count">
              ({salesData?.breakdown.length ?? 0} รายการ)
            </span>
          </div>
        </div>

        <div class="table-wrapper">
          <table class="data-table">
            <thead>
              <tr>
                <th>ช่วงเวลา / วันที่</th>
                <th class="text-right">จำนวนบิล</th>
                <th class="text-right">ยอดขายรวม ({currencySymbol})</th>
                <th class="text-right">ยอดเฉลี่ยต่อบิล ({currencySymbol})</th>
              </tr>
            </thead>
            <tbody>
              {#if !salesData || salesData.breakdown.length === 0}
                <tr>
                  <td colspan="4" class="no-data-cell">
                    ไม่มีรายการยอดขายในช่วงเวลานี้
                  </td>
                </tr>
              {:else}
                {#each salesData.breakdown as row}
                  <tr>
                    <td class="font-medium">
                      {formatDisplayDate(row.period, salesData.periodType)}
                      <span class="raw-period">({row.period})</span>
                    </td>
                    <td class="text-right">{row.orderCount} บิล</td>
                    <td class="text-right font-semibold text-primary">
                      {currencySymbol}{formatCurrency(row.totalSales)}
                    </td>
                    <td class="text-right text-muted">
                      {currencySymbol}{formatCurrency(row.averageOrderValue)}
                    </td>
                  </tr>
                {/each}
              {/if}
            </tbody>
            {#if salesData && salesData.breakdown.length > 0}
              <tfoot>
                <tr class="total-row">
                  <td>รวมทั้งสิ้น ({salesData.breakdown.length} ช่วงเวลา)</td>
                  <td class="text-right font-bold"
                    >{salesData.kpi.totalOrders} บิล</td
                  >
                  <td class="text-right font-bold text-highlight">
                    {currencySymbol}{formatCurrency(salesData.kpi.totalSales)}
                  </td>
                  <td class="text-right font-bold">
                    {currencySymbol}{formatCurrency(salesData.kpi.averageOrderValue)}
                  </td>
                </tr>
              </tfoot>
            {/if}
          </table>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Error Modal (Zero Native Popups) -->
<ErrorModal
  open={errorModal.open}
  title={errorModal.title}
  message={errorModal.message}
  details={errorModal.details}
  onClose={() => (errorModal.open = false)}
/>

<style>
  /* Header Topbar (ตรงตามหน้า สินค้าคงคลัง, หมวดหมู่, ผู้จัดจำหน่าย ทุกพิกเซล) */
  .topbar {
    padding: var(--space-xl) var(--space-xl) 0 var(--space-xl);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .topbar h1 {
    margin-bottom: 0;
  }

  .topbar-actions {
    display: flex;
    gap: var(--space-md);
    align-items: center;
  }

  /* Content Area */
  .content-area {
    padding: var(--space-xl);
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  /* Summary Card (Cohesive Card แบบเดียวกับหน้า สินค้าคงคลัง) */
  .summary-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  /* Filter Toolbar inside the card (.table-actions style) */
  .table-actions {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex-wrap: wrap;
  }

  /* Preset Pill Group */
  .preset-group {
    display: inline-flex;
    background-color: var(--color-background);
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 3px;
    gap: 2px;
  }

  .preset-btn {
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
    transition: all 0.15s ease;
  }

  .preset-btn:hover {
    color: var(--color-primary);
  }

  .preset-btn.active {
    background-color: var(--color-surface);
    color: var(--color-primary);
    font-weight: 600;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  }

  .custom-date-inline {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .date-field {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .field-label {
    font-size: 13px;
    color: #4c566a;
    font-weight: 500;
    white-space: nowrap;
  }

  .apply-btn {
    padding: 8px 18px;
    font-size: 13px;
    height: 38px;
    transition: all 0.2s ease;
  }

  .apply-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
    background-color: #94a3b8;
  }

  .date-error-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background-color: #fff5f5;
    color: var(--color-danger, #bf616a);
    border: 1px solid #fca5a5;
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 500;
    animation: fadeIn 0.2s ease-in-out;
    white-space: nowrap;
  }

  .date-error-badge svg {
    flex-shrink: 0;
    color: var(--color-danger, #bf616a);
  }

  .export-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    font-size: 14px;
    font-weight: 500;
  }

  .export-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Success Notification Banner */
  .success-banner {
    background-color: #e8f2e2;
    border: 1px solid #c4dcb5;
    border-radius: var(--radius-md);
    padding: 12px 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    animation: fadeIn 0.2s ease-in-out;
  }

  .success-content {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: #2e5824;
    word-break: break-all;
  }

  .success-icon {
    color: #5c8c43;
    flex-shrink: 0;
  }

  .success-text {
    font-weight: 500;
  }

  .banner-close {
    background: transparent;
    border: none;
    cursor: pointer;
    color: #4c566a;
    display: flex;
    align-items: center;
    padding: 4px;
    border-radius: 4px;
    transition: background-color 0.15s;
    flex-shrink: 0;
  }

  .banner-close:hover {
    background-color: rgba(0, 0, 0, 0.06);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* KPI Grid inside Card */
  .kpi-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-md);
  }

  @media (max-width: 960px) {
    .kpi-grid {
      grid-template-columns: 1fr;
    }
  }

  .kpi-box {
    background-color: #f8fafc;
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 20px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    min-height: 120px;
    transition: border-color 0.2s;
  }

  .kpi-box:hover {
    border-color: #bac7d5;
  }

  .kpi-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .kpi-label {
    font-size: 13px;
    font-weight: 500;
    color: #7e889b;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .kpi-icon-wrapper {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sales-icon {
    background-color: #ebf1f7;
    color: var(--color-primary);
  }

  .kpi-symbol-icon {
    font-size: 18px;
    font-weight: 700;
    line-height: 1;
  }

  .orders-icon {
    background-color: #e8f2e2;
    color: var(--color-accent-success);
  }

  .avg-icon {
    background-color: #f0edf5;
    color: #88c0d0;
  }

  .kpi-amount {
    font-size: 28px;
    font-weight: 700;
    color: var(--color-text-primary);
    font-family: var(--font-heading);
    margin: 8px 0 2px 0;
  }

  .currency-symbol {
    font-size: 20px;
    font-weight: 500;
    color: var(--color-primary);
    margin-right: 2px;
  }

  .unit {
    font-size: 16px;
    font-weight: 400;
    color: #7e889b;
  }

  .kpi-subtext {
    font-size: 12px;
    color: #98a2b3;
  }

  /* Section Divider */
  .section-divider {
    height: 1px;
    background-color: #e5e9f0;
    margin: 4px 0;
  }

  /* Section Header */
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-md);
    flex-wrap: wrap;
    gap: 8px;
  }

  .section-title-wrap {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .section-title-wrap h2 {
    font-size: 18px;
    margin-bottom: 0;
  }

  .badge {
    padding: 3px 10px;
    border-radius: 100px;
    font-size: 12px;
    font-weight: 600;
  }

  .badge-primary {
    background-color: #ebf1f7;
    color: var(--color-primary);
  }

  .period-caption {
    font-size: 12px;
    color: #7e889b;
  }

  .row-count {
    font-size: 13px;
    color: #7e889b;
    font-weight: 400;
  }

  .chart-body {
    padding-top: 4px;
  }

  /* Data Table Customizations */
  .text-right {
    text-align: right !important;
  }

  .font-medium {
    font-weight: 500;
  }

  .font-semibold {
    font-weight: 600;
  }

  .font-bold {
    font-weight: 700;
  }

  .text-primary {
    color: var(--color-primary);
  }

  .text-muted {
    color: #7e889b;
  }

  .text-highlight {
    color: #3b5b82;
  }

  .raw-period {
    font-size: 11px;
    color: #98a2b3;
    margin-left: 6px;
    font-weight: 400;
  }

  .no-data-cell {
    text-align: center;
    padding: var(--space-xl);
    color: #98a2b3;
  }

  .total-row {
    background-color: #f8fafc;
    border-top: 2px solid var(--color-muted);
  }

  .total-row td {
    padding: var(--space-md) var(--space-lg);
    font-size: 14px;
  }

  /* Loading Container */
  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 64px 0;
    gap: 16px;
    color: #7e889b;
    font-size: 14px;
  }

  .spinner {
    width: 36px;
    height: 36px;
    border: 3px solid #e5e9f0;
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
