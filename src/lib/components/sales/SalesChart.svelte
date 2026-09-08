<script lang="ts">
  import type { SalesSummaryBreakdownItem } from '$lib/types/sales';

  let {
    items = [],
    periodType = 'daily',
    currencySymbol = '฿'
  }: {
    items: SalesSummaryBreakdownItem[];
    periodType: string;
    currencySymbol?: string;
  } = $props();

  let hoveredIndex = $state<number | null>(null);

  const thaiMonthsShort = [
    'ม.ค.', 'ก.พ.', 'มี.ค.', 'เม.ย.', 'พ.ค.', 'มิ.ย.',
    'ก.ค.', 'ส.ค.', 'ก.ย.', 'ต.ค.', 'พ.ย.', 'ธ.ค.'
  ];

  function formatLabel(period: string, type: string): string {
    if (!period) return '';
    if (type === 'hourly') {
      return period;
    }
    if (type === 'monthly') {
      const parts = period.split('-');
      if (parts.length === 2) {
        const m = parseInt(parts[1], 10) - 1;
        const y = parts[0];
        return `${thaiMonthsShort[m] ?? period} ${y.slice(-2)}`;
      }
      return period;
    }
    // Daily "YYYY-MM-DD"
    const parts = period.split('-');
    if (parts.length === 3) {
      const d = parseInt(parts[2], 10);
      const m = parseInt(parts[1], 10) - 1;
      return `${d} ${thaiMonthsShort[m] ?? ''}`;
    }
    return period;
  }

  function formatTooltipHeader(period: string, type: string): string {
    if (!period) return '';
    if (type === 'hourly') {
      return `${period} น.`;
    }
    if (type === 'monthly') {
      const parts = period.split('-');
      if (parts.length === 2) {
        const m = parseInt(parts[1], 10) - 1;
        const y = parts[0];
        return `${thaiMonthsShort[m] ?? period} ${y}`;
      }
      return period;
    }
    // Daily "YYYY-MM-DD"
    const parts = period.split('-');
    if (parts.length === 3) {
      const d = parseInt(parts[2], 10);
      const m = parseInt(parts[1], 10) - 1;
      const y = parts[0];
      return `${d} ${thaiMonthsShort[m] ?? ''} ${y}`;
    }
    return period;
  }

  function formatCurrency(val: number): string {
    return new Intl.NumberFormat('th-TH', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2
    }).format(val);
  }

  function formatAxisValue(val: number): string {
    if (val >= 1_000_000) {
      return (val / 1_000_000).toFixed(1).replace(/\.0$/, '') + 'M';
    }
    if (val >= 1_000) {
      return (val / 1_000).toFixed(1).replace(/\.0$/, '') + 'k';
    }
    return Math.round(val).toString();
  }

  // Dimension computations
  const svgWidth = 900;
  const svgHeight = 260;
  const padLeft = 60;
  const padRight = 30;
  const padTop = 30;
  const padBottom = 45;
  const chartW = svgWidth - padLeft - padRight;
  const chartH = svgHeight - padTop - padBottom;

  const maxVal = $derived.by(() => {
    if (!items || items.length === 0) return 1000;
    const max = Math.max(...items.map((i) => i.totalSales), 0);
    if (max <= 0) return 1000;
    // ปัดขึ้นให้เป็นเลขสวยงาม
    const magnitude = Math.pow(10, Math.floor(Math.log10(max)));
    const multiple = Math.ceil(max / magnitude);
    return multiple * magnitude;
  });

  const yTicks = $derived([
    maxVal,
    maxVal * 0.75,
    maxVal * 0.5,
    maxVal * 0.25,
    0
  ]);

  const barStep = $derived(
    items.length > 0 ? chartW / items.length : chartW
  );
  const barWidth = $derived(
    Math.min(Math.max(barStep * 0.55, 8), 44)
  );
</script>

<div class="chart-container">
  {#if items.length === 0}
    <div class="empty-state">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="18" height="18" rx="3" />
        <path d="M7 16v-4" />
        <path d="M12 16V8" />
        <path d="M17 16v-6" />
      </svg>
      <p>ไม่มีข้อมูลยอดขายในช่วงเวลานี้</p>
    </div>
  {:else}
    <div class="svg-wrapper">
      <svg
        viewBox="0 0 {svgWidth} {svgHeight}"
        preserveAspectRatio="xMidYMid meet"
        class="sales-svg"
      >
        <!-- Y-Axis Grid Lines & Labels -->
        {#each yTicks as tick}
          {@const y = padTop + (1 - tick / maxVal) * chartH}
          <line
            x1={padLeft}
            y1={y}
            x2={svgWidth - padRight}
            y2={y}
            stroke="var(--color-muted)"
            stroke-dasharray={tick === 0 ? 'none' : '3 3'}
            stroke-width="1"
            opacity="0.7"
          />
          <text
            x={padLeft - 10}
            y={y + 4}
            text-anchor="end"
            font-size="11"
            fill="#7E889B"
            font-family="var(--font-body)"
          >
            {formatAxisValue(tick)}
          </text>
        {/each}

        <!-- Bars -->
        {#each items as item, i}
          {@const x = padLeft + i * barStep + (barStep - barWidth) / 2}
          {@const barH = (item.totalSales / maxVal) * chartH}
          {@const y = padTop + chartH - barH}
          {@const isHovered = hoveredIndex === i}

          <g
            class="bar-group"
            onmouseenter={() => (hoveredIndex = i)}
            onmouseleave={() => (hoveredIndex = null)}
            role="presentation"
          >
            <!-- Invisible wide target for easy hover -->
            <rect
              x={padLeft + i * barStep}
              y={padTop}
              width={barStep}
              height={chartH + 20}
              fill="transparent"
              class="hover-zone"
            />

            <!-- Background highlight column on hover -->
            {#if isHovered}
              <rect
                x={padLeft + i * barStep + 2}
                y={padTop}
                width={barStep - 4}
                height={chartH}
                fill="var(--color-primary)"
                opacity="0.06"
                rx="6"
              />
            {/if}

            <!-- Data Bar -->
            <rect
              {x}
              y={Math.max(y, padTop)}
              width={barWidth}
              height={Math.max(barH, item.totalSales > 0 ? 3 : 0)}
              rx="4"
              ry="4"
              fill={isHovered ? '#4C6E9B' : 'var(--color-primary)'}
              class="bar-rect"
            />

            <!-- X-axis tick & label -->
            <text
              x={x + barWidth / 2}
              y={padTop + chartH + 18}
              text-anchor="middle"
              font-size={items.length > 20 ? '10' : '11'}
              font-weight={isHovered ? '600' : '400'}
              fill={isHovered ? 'var(--color-primary)' : 'var(--color-text-primary)'}
              font-family="var(--font-body)"
            >
              {formatLabel(item.period, periodType)}
            </text>
          </g>
        {/each}
      </svg>

      <!-- Interactive Floating Tooltip -->
      {#if hoveredIndex !== null && items[hoveredIndex]}
        {@const it = items[hoveredIndex]}
        {@const leftPercent = ((padLeft + hoveredIndex * barStep + barStep / 2) / svgWidth) * 100}
        {@const tooltipTransform =
          hoveredIndex <= 1
            ? 'translateX(-10%)'
            : hoveredIndex >= items.length - 2
              ? 'translateX(-90%)'
              : 'translateX(-50%)'}
        <div
          class="tooltip"
          style="left: {leftPercent}%; transform: {tooltipTransform};"
        >
          <div class="tooltip-header">{formatTooltipHeader(it.period, periodType)}</div>
          <div class="tooltip-row">
            <span class="tooltip-label">ยอดขาย:</span>
            <span class="tooltip-val highlight">{currencySymbol}{formatCurrency(it.totalSales)}</span>
          </div>
          <div class="tooltip-row">
            <span class="tooltip-label">จำนวนบิล:</span>
            <span class="tooltip-val">{it.orderCount} บิล</span>
          </div>
          <div class="tooltip-row">
            <span class="tooltip-label">เฉลี่ยต่อบิล:</span>
            <span class="tooltip-val">{currencySymbol}{formatCurrency(it.averageOrderValue)}</span>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .chart-container {
    width: 100%;
    position: relative;
  }

  .svg-wrapper {
    position: relative;
    width: 100%;
    overflow: hidden;
  }

  .sales-svg {
    width: 100%;
    height: auto;
    display: block;
  }

  .bar-rect {
    transition: fill 0.15s ease, height 0.3s ease;
  }

  .hover-zone {
    cursor: pointer;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl) var(--space-md);
    color: #8C96A5;
    gap: 8px;
    font-size: 14px;
  }

  .empty-state svg {
    opacity: 0.5;
  }

  .tooltip {
    position: absolute;
    top: 10px;
    transform: translateX(-50%);
    background: #2E3440;
    color: #ECEFF4;
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 12px;
    pointer-events: none;
    z-index: 20;
    white-space: nowrap;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .tooltip-header {
    font-weight: 600;
    margin-bottom: 4px;
    color: #D8DEE9;
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
    padding-bottom: 3px;
  }

  .tooltip-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    line-height: 1.5;
  }

  .tooltip-label {
    color: #A3BE8C;
  }

  .tooltip-val {
    font-weight: 500;
  }

  .tooltip-val.highlight {
    color: #88C0D0;
    font-weight: 600;
  }
</style>
