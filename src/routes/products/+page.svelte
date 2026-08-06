<script lang="ts">
  import Dropdown from '$lib/components/Dropdown.svelte';

  let products = $state([
    {
      product_id: "P001",
      barcode: "8850123456789",
      name: "Premium Coffee Beans 500g",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 150,
      selling_price: 250,
      wholesale_price: 200,
      current_stock: 45,
      reorder_level: 20,
    },
    {
      product_id: "P002",
      barcode: "8850123456790",
      name: "Oat Milk Barista Edition",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 80,
      selling_price: 120,
      wholesale_price: 100,
      current_stock: 12,
      reorder_level: 15,
    },
    {
      product_id: "P003",
      barcode: "8850123456791",
      name: "Ceramic Mug Set",
      category_id: "C02",
      supplier_id: "S02",
      cost_price: 300,
      selling_price: 550,
      wholesale_price: 450,
      current_stock: 8,
      reorder_level: 10,
    },
    {
      product_id: "P004",
      barcode: "8850123456792",
      name: "Espresso Blend 1kg",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 280,
      selling_price: 420,
      wholesale_price: 360,
      current_stock: 30,
      reorder_level: 15,
    },
    {
      product_id: "P005",
      barcode: "8850123456793",
      name: "Cold Brew Concentrate",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 120,
      selling_price: 190,
      wholesale_price: 160,
      current_stock: 5,
      reorder_level: 20,
    },
    {
      product_id: "P006",
      barcode: "8850123456794",
      name: "Paper Cups 12oz (50pk)",
      category_id: "C02",
      supplier_id: "S02",
      cost_price: 90,
      selling_price: 150,
      wholesale_price: 120,
      current_stock: 60,
      reorder_level: 25,
    },
    {
      product_id: "P007",
      barcode: "8850123456795",
      name: "Lid Inserts (50pk)",
      category_id: "C02",
      supplier_id: "S02",
      cost_price: 40,
      selling_price: 80,
      wholesale_price: 60,
      current_stock: 75,
      reorder_level: 30,
    },
    {
      product_id: "P008",
      barcode: "8850123456796",
      name: "Caramel Syrup 1L",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 110,
      selling_price: 180,
      wholesale_price: 150,
      current_stock: 18,
      reorder_level: 10,
    },
    {
      product_id: "P009",
      barcode: "8850123456797",
      name: "Vanilla Syrup 1L",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 110,
      selling_price: 180,
      wholesale_price: 150,
      current_stock: 22,
      reorder_level: 10,
    },
    {
      product_id: "P010",
      barcode: "8850123456798",
      name: "Hazelnut Syrup 1L",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 110,
      selling_price: 180,
      wholesale_price: 150,
      current_stock: 9,
      reorder_level: 10,
    },
    {
      product_id: "P011",
      barcode: "8850123456799",
      name: "Whole Milk 1L",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 45,
      selling_price: 70,
      wholesale_price: 58,
      current_stock: 40,
      reorder_level: 20,
    },
    {
      product_id: "P012",
      barcode: "8850123456800",
      name: "Almond Milk 1L",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 65,
      selling_price: 95,
      wholesale_price: 80,
      current_stock: 15,
      reorder_level: 12,
    },
    {
      product_id: "P013",
      barcode: "8850123456801",
      name: "Soy Milk 1L",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 55,
      selling_price: 85,
      wholesale_price: 70,
      current_stock: 28,
      reorder_level: 12,
    },
    {
      product_id: "P014",
      barcode: "8850123456802",
      name: "Dark Chocolate Bar 70%",
      category_id: "C03",
      supplier_id: "S02",
      cost_price: 60,
      selling_price: 120,
      wholesale_price: 95,
      current_stock: 35,
      reorder_level: 15,
    },
    {
      product_id: "P015",
      barcode: "8850123456803",
      name: "Milk Chocolate Bar",
      category_id: "C03",
      supplier_id: "S02",
      cost_price: 55,
      selling_price: 110,
      wholesale_price: 88,
      current_stock: 42,
      reorder_level: 15,
    },
    {
      product_id: "P016",
      barcode: "8850123456804",
      name: "Matcha Powder 200g",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 350,
      selling_price: 520,
      wholesale_price: 450,
      current_stock: 7,
      reorder_level: 10,
    },
    {
      product_id: "P017",
      barcode: "8850123456805",
      name: "Chai Tea Bags (25pk)",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 75,
      selling_price: 130,
      wholesale_price: 105,
      current_stock: 20,
      reorder_level: 10,
    },
    {
      product_id: "P018",
      barcode: "8850123456806",
      name: "Earl Grey Tea Bags (25pk)",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 70,
      selling_price: 125,
      wholesale_price: 100,
      current_stock: 25,
      reorder_level: 10,
    },
    {
      product_id: "P019",
      barcode: "8850123456807",
      name: "Glass Tumbler 16oz",
      category_id: "C02",
      supplier_id: "S02",
      cost_price: 120,
      selling_price: 250,
      wholesale_price: 200,
      current_stock: 14,
      reorder_level: 8,
    },
    {
      product_id: "P020",
      barcode: "8850123456808",
      name: "Stainless Bottle 500ml",
      category_id: "C02",
      supplier_id: "S02",
      cost_price: 180,
      selling_price: 350,
      wholesale_price: 280,
      current_stock: 11,
      reorder_level: 8,
    },
    {
      product_id: "P021",
      barcode: "8850123456809",
      name: "Croissant (Frozen 20pk)",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 200,
      selling_price: 320,
      wholesale_price: 260,
      current_stock: 16,
      reorder_level: 10,
    },
    {
      product_id: "P022",
      barcode: "8850123456810",
      name: "Bagel (Frozen 12pk)",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 150,
      selling_price: 240,
      wholesale_price: 195,
      current_stock: 19,
      reorder_level: 10,
    },
    {
      product_id: "P023",
      barcode: "8850123456811",
      name: "Muffin Mix 2kg",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 130,
      selling_price: 210,
      wholesale_price: 170,
      current_stock: 6,
      reorder_level: 8,
    },
    {
      product_id: "P024",
      barcode: "8850123456812",
      name: "Brown Sugar 1kg",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 50,
      selling_price: 90,
      wholesale_price: 72,
      current_stock: 33,
      reorder_level: 15,
    },
    {
      product_id: "P025",
      barcode: "8850123456813",
      name: "Honey 500g",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 140,
      selling_price: 220,
      wholesale_price: 180,
      current_stock: 13,
      reorder_level: 8,
    },
    {
      product_id: "P026",
      barcode: "8850123456814",
      name: "Cinnamon Powder 100g",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 85,
      selling_price: 140,
      wholesale_price: 115,
      current_stock: 21,
      reorder_level: 10,
    },
    {
      product_id: "P027",
      barcode: "8850123456815",
      name: "Nutmeg Powder 100g",
      category_id: "C03",
      supplier_id: "S01",
      cost_price: 95,
      selling_price: 155,
      wholesale_price: 125,
      current_stock: 17,
      reorder_level: 10,
    },
    {
      product_id: "P028",
      barcode: "8850123456816",
      name: "Whipped Cream Canister",
      category_id: "C01",
      supplier_id: "S01",
      cost_price: 70,
      selling_price: 120,
      wholesale_price: 98,
      current_stock: 24,
      reorder_level: 12,
    },
    {
      product_id: "P029",
      barcode: "8850123456817",
      name: "Ice Cubes Tray (2pk)",
      category_id: "C02",
      supplier_id: "S02",
      cost_price: 30,
      selling_price: 65,
      wholesale_price: 50,
      current_stock: 50,
      reorder_level: 20,
    },
    {
      product_id: "P030",
      barcode: "8850123456818",
      name: "Coffee Grinder Burr",
      category_id: "C02",
      supplier_id: "S02",
      cost_price: 1200,
      selling_price: 1800,
      wholesale_price: 1500,
      current_stock: 3,
      reorder_level: 2,
    },
  ]);

  let categories = $state([
    { category_id: "C01", name: "เครื่องดื่ม" },
    { category_id: "C02", name: "สินค้าทั่วไป" },
    { category_id: "C03", name: "ขนมขบเคี้ยว" },
  ]);

  let suppliers = $state([
    {
      supplier_id: "S01",
      name: "บริษัท โกลบอล เทรด",
      contact_info: "contact@globaltrade.com",
    },
    {
      supplier_id: "S02",
      name: "ช่างฝีมือท้องถิ่น",
      contact_info: "02-123-4567",
    },
  ]);

  // Pagination state
  let currentPage = $state(1); // 1-based
  let pageSize = $state(10); // default 10
  const pageSizeOptions = [10, 25, 50, 75, 100];

  // Search state
  let searchQuery = $state("");

  // Category filter state
  let selectedCategory = $state("all");

  // Filtered products based on search + category
  let filteredProducts = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    return products.filter((p) => {
      // Category filter
      if (selectedCategory !== "all" && p.category_id !== selectedCategory) {
        return false;
      }
      // Search filter
      if (q) {
        return (
          p.name.toLowerCase().includes(q) ||
          p.barcode.toLowerCase().includes(q) ||
          getCategoryName(p.category_id).toLowerCase().includes(q) ||
          getSupplierName(p.supplier_id).toLowerCase().includes(q)
        );
      }
      return true;
    });
  });

  // Total pages (recomputed when filters/totals/pageSize change)
  let totalPages = $derived(
    Math.max(1, Math.ceil(filteredProducts.length / pageSize)),
  );

  // Clamp current page when totalPages changes
  $effect(() => {
    if (currentPage > totalPages) {
      currentPage = totalPages;
    }
  });

  // Paginated slice — convert 1-based page to 0-based offset in exactly one place
  let paginatedProducts = $derived.by(() => {
    const offset = (currentPage - 1) * pageSize;
    return filteredProducts.slice(offset, offset + pageSize);
  });

  function getCategoryName(category_id: string): string {
    return (
      categories.find((c) => c.category_id === category_id)?.name ?? category_id
    );
  }

  function getSupplierName(supplier_id: string | null): string {
    if (!supplier_id) return "-";
    return (
      suppliers.find((s) => s.supplier_id === supplier_id)?.name ?? supplier_id
    );
  }

  function setPageSize(size: number) {
    pageSize = size;
    currentPage = 1; // reset to first page when page size changes
  }

  function setCategoryFilter() {
    currentPage = 1; // reset to first page when filter changes
  }

  // Generate pagination items with ellipsis for collapsed ranges
  function getPageItems(
    current: number,
    total: number,
  ): (number | "ellipsis")[] {
    if (total <= 7) {
      return Array.from({ length: total }, (_, i) => i + 1);
    }
    const pages = new Set<number>([
      1,
      total,
      current - 1,
      current,
      current + 1,
    ]);
    const sorted = [...pages]
      .filter((p) => p >= 1 && p <= total)
      .sort((a, b) => a - b);
    const items: (number | "ellipsis")[] = [];
    let prev = 0;
    for (const p of sorted) {
      if (p - prev > 1) items.push("ellipsis");
      items.push(p);
      prev = p;
    }
    return items;
  }
</script>

<header class="topbar">
  <h1>สินค้าคงคลัง</h1>
    <button class="btn-primary">+ เพิ่มสินค้า</button>
</header>

<div class="content-area">
  <div class="card">
    <div class="table-actions">
      <input
        type="text"
        class="input-field search-input"
        placeholder="ค้นหาด้วยชื่อ, บาร์โค้ด, หมวดหมู่ หรือผู้จัดจำหน่าย..."
        bind:value={searchQuery}
      />
      <Dropdown
        id="category-filter"
        label="หมวดหมู่:"
        options={[
          { value: 'all', label: 'ทั้งหมด' },
          ...categories.map((c) => ({ value: c.category_id, label: c.name })),
        ]}
        bind:value={selectedCategory}
        onchange={() => setCategoryFilter()}
        minWidth="150px"
      />
    </div>

    <div class="table-wrapper">
      <table class="data-table">
        <thead>
          <tr>
            <th>บาร์โค้ด</th>
            <th>ชื่อสินค้า</th>
            <th>หมวดหมู่</th>
            <th>ผู้จัดจำหน่าย</th>
            <th>ต้นทุน</th>
            <th>ราคาขาย</th>
            <th>คงเหลือ</th>
            <th>จัดการ</th>
          </tr>
        </thead>
        <tbody>
          {#each paginatedProducts as p}
            <tr>
              <td>{p.barcode}</td>
              <td class="font-medium">{p.name}</td>
              <td>{getCategoryName(p.category_id)}</td>
              <td>{getSupplierName(p.supplier_id)}</td>
              <td>฿{p.cost_price.toFixed(2)}</td>
              <td>฿{p.selling_price.toFixed(2)}</td>
              <td>
                <span
                  class="badge {p.current_stock > p.reorder_level
                    ? 'badge-success'
                    : 'badge-warning'}"
                >
                  {p.current_stock}
                </span>
              </td>
              <td>
                <button class="action-btn">แก้ไข</button>
              </td>
            </tr>
          {:else}
            <tr>
              <td colspan="8" class="empty-state">ไม่พบสินค้า</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="pagination-bar">
      <div class="pagination-info">
        แสดง {paginatedProducts.length > 0
          ? (currentPage - 1) * pageSize + 1
          : 0}–{Math.min(currentPage * pageSize, filteredProducts.length)} จาก {filteredProducts.length}
        รายการ
      </div>

      {#if totalPages > 1}
        <nav aria-label="pagination" class="pagination">
          <button
            class="page-btn"
            aria-label="ไปหน้าก่อนหน้า"
            disabled={currentPage === 1}
            onclick={() => (currentPage = Math.max(1, currentPage - 1))}
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
            {#if item === "ellipsis"}
              <span class="page-ellipsis" aria-hidden="true">…</span>
            {:else}
              <button
                class="page-btn {item === currentPage ? 'page-btn-active' : ''}"
                aria-label={`ไปหน้าที่ ${item}`}
                aria-current={item === currentPage ? "page" : undefined}
                onclick={() => (currentPage = item)}
              >
                {item}
              </button>
            {/if}
          {/each}

          <button
            class="page-btn"
            aria-label="ไปหน้าถัดไป"
            disabled={currentPage === totalPages}
            onclick={() =>
              (currentPage = Math.min(totalPages, currentPage + 1))}
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

      <Dropdown
        id="page-size"
        label="รายการต่อหน้า:"
        options={pageSizeOptions.map((size) => ({
          value: String(size),
          label: String(size),
        }))}
        bind:value={pageSize}
        onchange={() => setPageSize(Number(pageSize))}
        minWidth="80px"
      />
    </div>
  </div>
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

  .table-actions {
    margin-bottom: var(--space-lg);
    display: flex;
    gap: var(--space-md);
    align-items: center;
    flex-wrap: wrap;
  }

  .search-input {
    max-width: 400px;
    flex: 1;
    min-width: 200px;
  }

  /* Prevent table from overflowing the card */
  .table-wrapper {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  .data-table {
    min-width: 800px;
  }

  .font-medium {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .badge {
    padding: 4px 10px;
    border-radius: 100px;
    font-size: 13px;
    font-weight: 600;
  }

  .badge-success {
    background-color: #e8f2e2;
    color: var(--color-accent-success);
  }

  .badge-warning {
    background-color: #fff3e0;
    color: #f57c00;
  }

  .action-btn {
    color: var(--color-primary);
    font-weight: 500;
    font-size: 14px;
  }

  .action-btn:hover {
    text-decoration: underline;
  }

  .empty-state {
    text-align: center;
    padding: var(--space-xl);
    color: var(--color-text-primary);
    opacity: 0.6;
  }

  /* Pagination bar — space between info, pagination, and page size selector */
  .pagination-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    margin-top: var(--space-lg);
    flex-wrap: wrap;
  }

  /* Pagination */
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

  .pagination-info {
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.6;
    white-space: nowrap;
  }
</style>
