<script lang="ts">
  import Dropdown from './Dropdown.svelte';

  let {
    currentPage = $bindable(1),
    pageSize = $bindable(10),
    totalItems = 0,
    pageSizeOptions = [10, 25, 50, 75, 100],
    itemLabel = 'รายการ',
    showPageSize = true,
  }: {
    currentPage?: number;
    pageSize?: number;
    totalItems: number;
    pageSizeOptions?: number[];
    itemLabel?: string;
    showPageSize?: boolean;
  } = $props();

  let totalPages = $derived(Math.max(1, Math.ceil(totalItems / pageSize)));

  // Clamp current page when totalPages changes
  $effect(() => {
    if (currentPage > totalPages) {
      currentPage = totalPages;
    }
  });

  function setPage(page: number) {
    currentPage = Math.max(1, Math.min(totalPages, page));
  }

  function handlePageSizeChange(newSize: string | number) {
    pageSize = Number(newSize);
    currentPage = 1;
  }

  function getPageItems(current: number, total: number): (number | 'ellipsis')[] {
    if (total <= 7) {
      return Array.from({ length: total }, (_, i) => i + 1);
    }
    const pages = new Set<number>([1, total, current - 1, current, current + 1]);
    const sorted = [...pages].filter((p) => p >= 1 && p <= total).sort((a, b) => a - b);
    const items: (number | 'ellipsis')[] = [];
    let prev = 0;
    for (const p of sorted) {
      if (p - prev > 1) items.push('ellipsis');
      items.push(p);
      prev = p;
    }
    return items;
  }

  let startItem = $derived(totalItems > 0 ? (currentPage - 1) * pageSize + 1 : 0);
  let endItem = $derived(Math.min(currentPage * pageSize, totalItems));
</script>

<div class="pagination-bar">
  <div class="pagination-info">
    แสดง {startItem}–{endItem} จาก {totalItems} {itemLabel}
  </div>

  {#if totalPages > 1}
    <nav aria-label="pagination" class="pagination">
      <button
        type="button"
        class="page-btn"
        aria-label="ไปหน้าก่อนหน้า"
        disabled={currentPage === 1}
        onclick={() => setPage(currentPage - 1)}
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
          <path d="m15 18-6-6 6-6" />
        </svg>
      </button>

      {#each getPageItems(currentPage, totalPages) as item}
        {#if item === 'ellipsis'}
          <span class="page-ellipsis" aria-hidden="true">…</span>
        {:else}
          <button
            type="button"
            class="page-btn {item === currentPage ? 'page-btn-active' : ''}"
            aria-label={`ไปหน้าที่ ${item}`}
            aria-current={item === currentPage ? 'page' : undefined}
            onclick={() => setPage(item)}
          >
            {item}
          </button>
        {/if}
      {/each}

      <button
        type="button"
        class="page-btn"
        aria-label="ไปหน้าถัดไป"
        disabled={currentPage === totalPages}
        onclick={() => setPage(currentPage + 1)}
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
          <path d="m9 18 6-6-6-6" />
        </svg>
      </button>
    </nav>
  {/if}

  {#if showPageSize}
    <div class="pagination-size">
      <Dropdown
        id="page-size"
        label="รายการต่อหน้า:"
        options={pageSizeOptions.map((size) => ({
          value: String(size),
          label: String(size),
        }))}
        value={String(pageSize)}
        onchange={handlePageSizeChange}
        minWidth="80px"
      />
    </div>
  {/if}
</div>

<style>
  .pagination-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    margin-top: var(--space-lg);
    flex-wrap: wrap;
  }

  .pagination-info {
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.6;
    white-space: nowrap;
  }

  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    flex-wrap: wrap;
  }

  .page-btn {
    min-width: 36px;
    height: 36px;
    padding: 0 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    background-color: transparent;
    border: 1px solid transparent;
    transition: all 0.2s ease;
  }

  .page-btn:hover:not(:disabled) {
    background-color: #f4f7fa;
    border-color: var(--color-muted);
  }

  .page-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .page-btn-active {
    background-color: var(--color-primary);
    color: var(--color-surface);
    border-color: var(--color-primary);
  }

  .page-btn-active:hover:not(:disabled) {
    background-color: var(--color-primary);
    border-color: var(--color-primary);
  }

  .page-ellipsis {
    min-width: 36px;
    height: 36px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-primary);
    opacity: 0.5;
    font-size: 14px;
  }

  .pagination-size {
    display: flex;
    align-items: center;
  }
</style>
