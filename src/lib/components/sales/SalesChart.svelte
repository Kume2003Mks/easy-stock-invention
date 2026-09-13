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
  let containerWidth = $state(900);
  let scrollContainer = $state<HTMLDivElement | null>(null);
  let canScrollLeft = $state(false);
  let canScrollRight = $state(false);
  let isDragging = $state(false);
  let hasDragged = $state(false);
  let startX = 0;
  let scrollLeftStart = 0;

  const yAxisWidth = 56;
  const svgHeight = 260;
  const padTop = 30;
  const padBottom = 45;
  const chartH = svgHeight - padTop - padBottom; // 185px
  const minItemWidth = 68; // Minimum space per column so labels never overlap

  // Condition: horizontally scrollable if more than 12 items
  const isScrollable = $derived(items.length > 12);

  const safeContainerWidth = $derived(containerWidth > 0 ? containerWidth : 900);
  const availableContentWidth = $derived(Math.max(safeContainerWidth - yAxisWidth, 300));

  const contentWidth = $derived.by(() => {
    if (!isScrollable) {
      return availableContentWidth;
    }
    const needed = items.length * minItemWidth + 28;
    return Math.max(availableContentWidth, needed);
  });

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

  const maxVal = $derived.by(() => {
    if (!items || items.length === 0) return 1000;
    const max = Math.max(...items.map((i) => i.totalSales), 0);
    if (max <= 0) return 1000;
    // Round up to clean multiple
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

  const padContentLeft = 12;
  const padContentRight = 16;
  const chartInnerW = $derived(Math.max(contentWidth - padContentLeft - padContentRight, 100));

  const barStep = $derived(
    items.length > 0 ? chartInnerW / items.length : chartInnerW
  );

  const barWidth = $derived(
    Math.min(Math.max(barStep * 0.45, 12), 38)
  );

  function updateScrollState() {
    if (!scrollContainer) return;
    const { scrollLeft, scrollWidth, clientWidth } = scrollContainer;
    canScrollLeft = scrollLeft > 4;
    canScrollRight = scrollLeft < scrollWidth - clientWidth - 4;
  }

  function scrollByDirection(direction: 'left' | 'right') {
    if (!scrollContainer) return;
    const offset = 320;
    scrollContainer.scrollBy({
      left: direction === 'left' ? -offset : offset,
      behavior: 'smooth'
    });
  }

  function scrollToStart() {
    if (!scrollContainer) return;
    scrollContainer.scrollTo({ left: 0, behavior: 'smooth' });
  }

  function scrollToEnd() {
    if (!scrollContainer) return;
    scrollContainer.scrollTo({ left: scrollContainer.scrollWidth, behavior: 'smooth' });
  }

  function onPointerDown(e: MouseEvent) {
    if (!isScrollable || !scrollContainer) return;
    if (e.button !== 0) return;
    isDragging = true;
    hasDragged = false;
    startX = e.pageX;
    scrollLeftStart = scrollContainer.scrollLeft;
  }

  function onPointerMove(e: MouseEvent) {
    if (!isDragging || !scrollContainer) return;
    const dx = e.pageX - startX;
    if (Math.abs(dx) > 4) {
      hasDragged = true;
      hoveredIndex = null;
    }
    if (hasDragged) {
      scrollContainer.scrollLeft = scrollLeftStart - dx;
      updateScrollState();
    }
  }

  function onPointerUp() {
    if (isDragging) {
      isDragging = false;
      setTimeout(() => {
        hasDragged = false;
      }, 50);
    }
  }

  $effect(() => {
    // Re-check scroll buttons when items change or container resizes
    if (items.length && scrollContainer) {
      requestAnimationFrame(updateScrollState);
    }
  });
</script>

<svelte:window
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
/>

<div class="chart-container" bind:clientWidth={containerWidth}>
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
    <!-- Optional Toolbar when items > 12 -->
    {#if isScrollable}
      <div class="chart-toolbar">
        <div class="toolbar-info">
          <span class="toolbar-badge">{items.length} รายการ</span>
          <span class="toolbar-hint">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="8 4 4 8 8 12" />
              <polyline points="16 12 20 16 16 20" />
              <line x1="4" y1="8" x2="20" y2="8" />
              <line x1="4" y1="16" x2="20" y2="16" />
            </svg>
            เลื่อนแนวนอนเพื่อดูข้อมูล
          </span>
        </div>
        <div class="nav-btn-group">
          <button
            type="button"
            class="nav-btn"
            title="เลื่อนไปซ้ายสุด (ข้อมูลเริ่มต้น)"
            disabled={!canScrollLeft}
            onclick={scrollToStart}
            aria-label="เลื่อนไปซ้ายสุด"
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="11 17 6 12 11 7" />
              <polyline points="18 17 13 12 18 7" />
            </svg>
          </button>
          <button
            type="button"
            class="nav-btn"
            title="เลื่อนซ้าย"
            disabled={!canScrollLeft}
            onclick={() => scrollByDirection('left')}
            aria-label="เลื่อนซ้าย"
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="15 18 9 12 15 6" />
            </svg>
          </button>
          <button
            type="button"
            class="nav-btn"
            title="เลื่อนขวา"
            disabled={!canScrollRight}
            onclick={() => scrollByDirection('right')}
            aria-label="เลื่อนขวา"
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="9 18 15 12 9 6" />
            </svg>
          </button>
          <button
            type="button"
            class="nav-btn"
            title="เลื่อนไปขวาสุด (ข้อมูลล่าสุด)"
            disabled={!canScrollRight}
            onclick={scrollToEnd}
            aria-label="เลื่อนไปขวาสุด"
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="13 17 18 12 13 7" />
              <polyline points="6 17 11 12 6 7" />
            </svg>
          </button>
        </div>
      </div>
    {/if}

    <!-- Chart Stage: Fixed Y-axis + Scrollable or Stretched Bars -->
    <div class="chart-stage">
      <!-- Fixed Y-Axis Column -->
      <div class="y-axis-col" class:has-shadow={canScrollLeft}>
        <svg width={yAxisWidth} height={svgHeight} class="y-axis-svg">
          {#each yTicks as tick}
            {@const y = padTop + (1 - tick / maxVal) * chartH}
            <text
              x={yAxisWidth - 10}
              y={y + 4}
              text-anchor="end"
              font-size="11"
              fill="#7E889B"
              font-family="var(--font-body)"
            >
              {formatAxisValue(tick)}
            </text>
          {/each}
        </svg>
      </div>

      <!-- Main Chart Area (Scrollable if items > 12) -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="chart-scroll-wrapper"
        class:is-scrollable={isScrollable}
        class:is-dragging={isDragging}
        bind:this={scrollContainer}
        onscroll={updateScrollState}
        onpointerdown={onPointerDown}
      >
        <div
          class="chart-content"
          style="width: {contentWidth}px; min-width: 100%; height: {svgHeight}px;"
        >
          <svg
            viewBox="0 0 {contentWidth} {svgHeight}"
            style="width: {contentWidth}px; height: {svgHeight}px;"
            class="sales-svg"
          >
            <!-- Horizontal Grid Lines -->
            {#each yTicks as tick}
              {@const y = padTop + (1 - tick / maxVal) * chartH}
              <line
                x1="0"
                y1={y}
                x2={contentWidth}
                y2={y}
                stroke={tick === 0 ? '#BAC7D5' : '#E5E9F0'}
                stroke-dasharray={tick === 0 ? 'none' : '3 3'}
                stroke-width={tick === 0 ? '1.5' : '1'}
              />
            {/each}

            <!-- Bars & X-axis Labels -->
            {#each items as item, i}
              {@const x = padContentLeft + i * barStep + (barStep - barWidth) / 2}
              {@const barH = (item.totalSales / maxVal) * chartH}
              {@const y = padTop + chartH - barH}
              {@const isHovered = hoveredIndex === i}

              <g
                class="bar-group"
                onmouseenter={() => {
                  if (!hasDragged) hoveredIndex = i;
                }}
                onmouseleave={() => (hoveredIndex = null)}
                role="presentation"
              >
                <!-- Invisible Wide Target For Hover -->
                <rect
                  x={padContentLeft + i * barStep}
                  y={padTop}
                  width={barStep}
                  height={chartH + 28}
                  fill="transparent"
                  class="hover-zone"
                />

                <!-- Background Highlight Column on Hover -->
                {#if isHovered}
                  <rect
                    x={padContentLeft + i * barStep + 1}
                    y={padTop}
                    width={barStep - 2}
                    height={chartH}
                    fill="var(--color-primary, #5E81AC)"
                    opacity="0.08"
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
                  fill={isHovered ? '#4C6E9B' : 'var(--color-primary, #5E81AC)'}
                  class="bar-rect"
                />

                <!-- X-axis Label -->
                <text
                  x={x + barWidth / 2}
                  y={padTop + chartH + 20}
                  text-anchor="middle"
                  font-size="11"
                  font-weight={isHovered ? '600' : '400'}
                  fill={isHovered ? 'var(--color-primary, #5E81AC)' : 'var(--color-text-primary, #2E3440)'}
                  font-family="var(--font-body)"
                >
                  {formatLabel(item.period, periodType)}
                </text>
              </g>
            {/each}
          </svg>

          <!-- Interactive Floating Tooltip -->
          {#if hoveredIndex !== null && items[hoveredIndex] && !isDragging}
            {@const it = items[hoveredIndex]}
            {@const targetX = padContentLeft + hoveredIndex * barStep + barStep / 2}
            {@const currentScroll = scrollContainer?.scrollLeft ?? 0}
            {@const viewportW = scrollContainer?.clientWidth ?? availableContentWidth}
            {@const tooltipX = Math.max(currentScroll + 90, Math.min(targetX, currentScroll + viewportW - 90))}
            <div
              class="tooltip"
              style="left: {tooltipX}px; top: 8px;"
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
      </div>
    </div>
  {/if}
</div>

<style>
  .chart-container {
    width: 100%;
    position: relative;
  }

  /* Toolbar for scrolling controls */
  .chart-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 4px 10px 4px;
    font-size: 12px;
    color: #7e889b;
    flex-wrap: wrap;
    gap: 8px;
  }

  .toolbar-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .toolbar-badge {
    font-weight: 600;
    color: var(--color-primary, #5e81ac);
    background: #ebf1f7;
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 11px;
  }

  .toolbar-hint {
    display: flex;
    align-items: center;
    gap: 4px;
    color: #8c96a5;
    font-size: 12px;
  }

  .nav-btn-group {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: #f4f7fa;
    padding: 3px;
    border-radius: 8px;
    border: 1px solid #e5e9f0;
  }

  .nav-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    background: transparent;
    border-radius: 5px;
    color: #4c566a;
    cursor: pointer;
    transition: all 0.15s ease;
    padding: 0;
  }

  .nav-btn:hover:not(:disabled) {
    background: #ffffff;
    color: var(--color-primary, #5e81ac);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  }

  .nav-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  /* Stage Layout */
  .chart-stage {
    display: flex;
    position: relative;
    border-radius: var(--radius-md, 12px);
    background: #ffffff;
    overflow: hidden;
  }

  /* Fixed Y-Axis Column */
  .y-axis-col {
    width: 56px;
    flex-shrink: 0;
    background: #ffffff;
    border-right: 1px solid #e5e9f0;
    z-index: 5;
    transition: box-shadow 0.2s ease;
  }

  .y-axis-col.has-shadow {
    box-shadow: 4px 0 12px rgba(46, 52, 64, 0.08);
  }

  .y-axis-svg {
    display: block;
  }

  /* Scrollable / Flexible Chart Area */
  .chart-scroll-wrapper {
    flex: 1;
    min-width: 0;
    overflow-x: hidden;
    overflow-y: hidden;
    position: relative;
  }

  .chart-scroll-wrapper.is-scrollable {
    overflow-x: auto;
    cursor: grab;
  }

  .chart-scroll-wrapper.is-dragging {
    cursor: grabbing;
    user-select: none;
  }

  /* Sleek Scrollbar */
  .chart-scroll-wrapper::-webkit-scrollbar {
    height: 7px;
  }

  .chart-scroll-wrapper::-webkit-scrollbar-track {
    background: #f4f7fa;
    border-radius: 4px;
  }

  .chart-scroll-wrapper::-webkit-scrollbar-thumb {
    background: #cbd5e1;
    border-radius: 4px;
  }

  .chart-scroll-wrapper::-webkit-scrollbar-thumb:hover {
    background: #5e81ac;
  }

  .chart-content {
    position: relative;
  }

  .sales-svg {
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
    padding: var(--space-xl, 32px) var(--space-md, 16px);
    color: #8c96a5;
    gap: 8px;
    font-size: 14px;
  }

  .empty-state svg {
    opacity: 0.5;
  }

  /* Floating Tooltip */
  .tooltip {
    position: absolute;
    transform: translateX(-50%);
    background: #2e3440;
    color: #eceff4;
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 12px;
    pointer-events: none;
    z-index: 20;
    white-space: nowrap;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.18);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .tooltip-header {
    font-weight: 600;
    margin-bottom: 4px;
    color: #d8dee9;
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
    color: #a3be8c;
  }

  .tooltip-val {
    font-weight: 500;
  }

  .tooltip-val.highlight {
    color: #88c0d0;
    font-weight: 600;
  }
</style>
