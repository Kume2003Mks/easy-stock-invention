<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Dropdown from "$lib/components/Dropdown.svelte";
  import ActionMenu from "$lib/components/ActionMenu.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";

  interface Product {
    product_id: string;
    barcode: string | null;
    name: string;
    category_id: string | null;
    supplier_id: string | null;
    cost_price: number;
    selling_price: number;
    wholesale_price: number;
    current_stock: number;
    reorder_level: number;
  }

  interface Category {
    category_id: string;
    name: string;
  }

  interface Supplier {
    supplier_id: string;
    name: string;
    contact_info: string | null;
  }

  interface ProductsPageData {
    products: Product[];
    categories: Category[];
    suppliers: Supplier[];
  }

  let products = $state<Product[]>([]);
  let categories = $state<Category[]>([]);
  let suppliers = $state<Supplier[]>([]);

  let loading = $state(true);
  let loadError = $state("");

  // Add product modal state
  let showAddModal = $state(false);

  // Save confirmation state
  let showSaveConfirm = $state(false);
  let pendingProductName = $state("");
  let pendingProductBarcode = $state("");
  let pendingProductCategory = $state("");
  let pendingProductSupplier = $state("");
  let pendingProductCost = $state(0);
  let pendingProductSelling = $state(0);
  let pendingProductWholesale = $state(0);
  let pendingProductStock = $state(0);
  let pendingProductReorder = $state(10);

  // New product form state
  let newProduct = $state({
    barcode: "",
    name: "",
    category_id: "",
    supplier_id: "",
    cost_price: 0,
    selling_price: 0,
    wholesale_price: 0,
    current_stock: 0,
    reorder_level: 10,
  });

  // Validation error state
  let formErrors = $state<Record<string, string>>({});

  function clearFieldError(field: string) {
    if (formErrors[field]) {
      const { [field]: _removed, ...rest } = formErrors;
      formErrors = rest;
    }
  }

  function validateForm(): boolean {
    const errors: Record<string, string> = {};

    if (!newProduct.name.trim()) {
      errors.name = "กรุณากรอกชื่อสินค้า";
    }
    if (!newProduct.barcode.trim()) {
      errors.barcode = "กรุณากรอกบาร์โค้ด";
    }
    if (!newProduct.category_id) {
      errors.category_id = "กรุณาเลือกหมวดหมู่";
    }

    formErrors = errors;
    return Object.keys(errors).length === 0;
  }

  async function loadProductsData() {
    loading = true;
    loadError = "";
    try {
      const data = (await invoke("get_products_data")) as ProductsPageData;
      products = data.products ?? [];
      categories = data.categories ?? [];
      suppliers = data.suppliers ?? [];
    } catch (err) {
      console.error("Failed to load products data:", err);
      loadError = String(err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadProductsData();
  });

  function openAddModal() {
    // Reset form
    newProduct = {
      barcode: "",
      name: "",
      category_id: "",
      supplier_id: "",
      cost_price: 0,
      selling_price: 0,
      wholesale_price: 0,
      current_stock: 0,
      reorder_level: 10,
    };
    formErrors = {};
    showAddModal = true;
  }

  function closeAddModal() {
    showAddModal = false;
    formErrors = {};
  }

  function addProduct() {
    // Validate required fields
    if (!validateForm()) return;

    // Store pending data for confirmation
    pendingProductName = newProduct.name.trim();
    pendingProductBarcode = newProduct.barcode.trim();
    pendingProductCategory = getCategoryName(newProduct.category_id);
    pendingProductSupplier = newProduct.supplier_id
      ? getSupplierName(newProduct.supplier_id)
      : "-";
    pendingProductCost = Number(newProduct.cost_price) || 0;
    pendingProductSelling = Number(newProduct.selling_price) || 0;
    pendingProductWholesale = Number(newProduct.wholesale_price) || 0;
    pendingProductStock = Number(newProduct.current_stock) || 0;
    pendingProductReorder = Number(newProduct.reorder_level) || 10;

    showSaveConfirm = true;
  }

  function cancelSave() {
    showSaveConfirm = false;
  }

  async function confirmSaveProduct() {
    try {
      const payload: Product = {
        product_id: "",
        barcode: newProduct.barcode.trim() || null,
        name: newProduct.name.trim(),
        category_id: newProduct.category_id || null,
        supplier_id: newProduct.supplier_id || null,
        cost_price: Number(newProduct.cost_price) || 0,
        selling_price: Number(newProduct.selling_price) || 0,
        wholesale_price: Number(newProduct.wholesale_price) || 0,
        current_stock: Number(newProduct.current_stock) || 0,
        reorder_level: Number(newProduct.reorder_level) || 10,
      };

      const saved = (await invoke("create_product", {
        product: payload,
      })) as Product;

      products = [...products, saved];
      cancelSave();
      closeAddModal();
    } catch (err) {
      console.error("Failed to create product:", err);
      alert(`ไม่สามารถบันทึกสินค้าได้: ${err}`);
    }
  }

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
          (p.barcode && p.barcode.toLowerCase().includes(q)) ||
          (p.category_id &&
            getCategoryName(p.category_id).toLowerCase().includes(q)) ||
          (p.supplier_id &&
            getSupplierName(p.supplier_id).toLowerCase().includes(q))
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

  function getCategoryName(category_id: string | null): string {
    if (!category_id) return "-";
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

  // Export state
  let showExportModal = $state(false);
  let exportScope = $state("all");
  let exportCategory = $state("");
  let exportFormat = $state("csv");
  let showExportConfirm = $state(false);

  function openExportModal() {
    exportScope = "all";
    exportCategory = "";
    exportFormat = "csv";
    showExportModal = true;
  }

  function closeExportModal() {
    showExportModal = false;
  }

  function requestExport() {
    showExportConfirm = true;
  }

  function cancelExport() {
    showExportConfirm = false;
  }

  function confirmExport() {
    console.log("Export products", {
      scope: exportScope,
      category: exportCategory,
      format: exportFormat,
    });
    showExportConfirm = false;
    closeExportModal();
  }

  // Delete confirmation state
  let showDeleteConfirm = $state(false);
  let deleteTargetId = $state<string | null>(null);
  let deleteTargetName = $state("");

  function requestDeleteProduct(p: { product_id: string; name: string }) {
    deleteTargetId = p.product_id;
    deleteTargetName = p.name;
    showDeleteConfirm = true;
  }

  function cancelDeleteProduct() {
    showDeleteConfirm = false;
    deleteTargetId = null;
    deleteTargetName = "";
  }

  async function confirmDeleteProduct() {
    if (deleteTargetId) {
      try {
        await invoke("delete_product", { productId: deleteTargetId });
        products = products.filter((p) => p.product_id !== deleteTargetId);
      } catch (err) {
        console.error("Failed to delete product:", err);
        alert(`ไม่สามารถลบสินค้าได้: ${err}`);
      }
    }
    cancelDeleteProduct();
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
  <div class="topbar-actions">
    <button class="btn-outline" onclick={openExportModal}>ส่งออกสินค้า</button>
    <button class="btn-primary" onclick={openAddModal}>+ เพิ่มสินค้า</button>
  </div>
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
          { value: "all", label: "ทั้งหมด" },
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
            <th class="col-name">ชื่อสินค้า</th>
            <th>หมวดหมู่</th>
            <th>ผู้จัดจำหน่าย</th>
            <th>ต้นทุน</th>
            <th>ราคาขาย</th>
            <th>ราคาส่ง</th>
            <th>คงเหลือ</th>
            <th class="col-actions">จัดการ</th>
          </tr>
        </thead>
        <tbody>
          {#if loading}
            <tr>
              <td colspan="9" class="loading-state">
                <div class="spinner"></div>
                <span>กำลังโหลดข้อมูลสินค้า...</span>
              </td>
            </tr>
          {:else if loadError}
            <tr>
              <td colspan="9" class="error-state">
                <span>เกิดข้อผิดพลาดในการโหลดข้อมูล: {loadError}</span>
                <button class="btn-outline btn-sm" onclick={loadProductsData}>
                  ลองใหม่อีกครั้ง
                </button>
              </td>
            </tr>
          {:else}
            {#each paginatedProducts as p}
              <tr>
                <td>{p.barcode || "-"}</td>
                <td class="font-medium col-name">{p.name}</td>
                <td>{getCategoryName(p.category_id)}</td>
                <td>{getSupplierName(p.supplier_id)}</td>
                <td>฿{p.cost_price.toFixed(2)}</td>
                <td>฿{p.selling_price.toFixed(2)}</td>
                <td>฿{p.wholesale_price.toFixed(2)}</td>
                <td>
                  <span
                    class="badge {p.current_stock > p.reorder_level
                      ? 'badge-success'
                      : 'badge-warning'}"
                  >
                    {p.current_stock}
                  </span>
                </td>
                <td class="col-actions">
                  <ActionMenu
                    align="right"
                    items={[
                      {
                        label: "แก้ไขข้อมูล",
                        icon: "edit",
                        onclick: () =>
                          console.log("Edit product", p.product_id),
                      },
                      {
                        label: "ปรับปรุงสต็อก",
                        icon: "adjust",
                        onclick: () =>
                          console.log("Adjust stock", p.product_id),
                      },
                      {
                        label: "ประวัติสินค้า",
                        icon: "history",
                        onclick: () =>
                          console.log("Product history", p.product_id),
                      },
                      {
                        label: "ลบสินค้า",
                        icon: "delete",
                        variant: "danger",
                        onclick: () => requestDeleteProduct(p),
                      },
                    ]}
                  />
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan="9" class="empty-state">ไม่พบสินค้า</td>
              </tr>
            {/each}
          {/if}
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

<Modal
  open={showAddModal}
  title="เพิ่มสินค้าใหม่"
  onClose={closeAddModal}
  maxWidth="640px"
>
  <form
    class="product-form"
    onsubmit={(e) => {
      e.preventDefault();
      addProduct();
    }}
  >
    <div class="form-grid">
      <div class="form-group" class:has-error={!!formErrors.barcode}>
        <label for="product-barcode" class="form-label">บาร์โค้ด *</label>
        <input
          id="product-barcode"
          type="text"
          class="input-field"
          class:input-error={!!formErrors.barcode}
          placeholder="เช่น 8850123456789"
          bind:value={newProduct.barcode}
          oninput={() => clearFieldError("barcode")}
        />
        {#if formErrors.barcode}
          <span class="error-text">{formErrors.barcode}</span>
        {/if}
      </div>

      <div class="form-group" class:has-error={!!formErrors.name}>
        <label for="product-name" class="form-label">ชื่อสินค้า *</label>
        <input
          id="product-name"
          type="text"
          class="input-field"
          class:input-error={!!formErrors.name}
          placeholder="เช่น Premium Coffee Beans 500g"
          bind:value={newProduct.name}
          oninput={() => clearFieldError("name")}
        />
        {#if formErrors.name}
          <span class="error-text">{formErrors.name}</span>
        {/if}
      </div>

      <div class="form-group" class:has-error={!!formErrors.category_id}>
        <label for="product-category" class="form-label">หมวดหมู่ *</label>
        <Dropdown
          id="product-category"
          options={categories.map((c) => ({
            value: c.category_id,
            label: c.name,
          }))}
          bind:value={newProduct.category_id}
          onchange={() => clearFieldError("category_id")}
          placeholder="เลือกหมวดหมู่"
          minWidth="100%"
          hasError={!!formErrors.category_id}
        />
        {#if formErrors.category_id}
          <span class="error-text">{formErrors.category_id}</span>
        {/if}
      </div>

      <div class="form-group">
        <label for="product-supplier" class="form-label">ผู้จัดจำหน่าย</label>
        <Dropdown
          id="product-supplier"
          options={suppliers.map((s) => ({
            value: s.supplier_id,
            label: s.name,
          }))}
          bind:value={newProduct.supplier_id}
          placeholder="เลือกผู้จัดจำหน่าย"
          clearable={true}
          minWidth="100%"
        />
      </div>

      <div class="form-group">
        <label for="product-cost" class="form-label">ต้นทุน (บาท)</label>
        <input
          id="product-cost"
          type="number"
          min="0"
          step="0.01"
          class="input-field"
          placeholder="0.00"
          bind:value={newProduct.cost_price}
        />
      </div>

      <div class="form-group">
        <label for="product-selling" class="form-label">ราคาขาย (บาท)</label>
        <input
          id="product-selling"
          type="number"
          min="0"
          step="0.01"
          class="input-field"
          placeholder="0.00"
          bind:value={newProduct.selling_price}
        />
      </div>

      <div class="form-group">
        <label for="product-wholesale" class="form-label">ราคาส่ง (บาท)</label>
        <input
          id="product-wholesale"
          type="number"
          min="0"
          step="0.01"
          class="input-field"
          placeholder="0.00"
          bind:value={newProduct.wholesale_price}
        />
      </div>

      <div class="form-group">
        <label for="product-stock" class="form-label">สต็อกเริ่มต้น</label>
        <input
          id="product-stock"
          type="number"
          min="0"
          step="1"
          class="input-field"
          placeholder="0"
          bind:value={newProduct.current_stock}
        />
      </div>

      <div class="form-group">
        <label for="product-reorder" class="form-label">ระดับสต็อกขั้นต่ำ</label
        >
        <input
          id="product-reorder"
          type="number"
          min="0"
          step="1"
          class="input-field"
          placeholder="10"
          bind:value={newProduct.reorder_level}
        />
      </div>
    </div>

    <div class="form-actions">
      <button type="button" class="btn-outline" onclick={closeAddModal}>
        ยกเลิก
      </button>
      <button type="submit" class="btn-primary">บันทึกสินค้า</button>
    </div>
  </form>
</Modal>

<Modal
  open={showExportModal}
  title="ส่งออกรายการสินค้า"
  onClose={closeExportModal}
  maxWidth="480px"
>
  <div class="export-form">
    <div class="form-group">
      <label for="export-scope" class="form-label">เลือกรายการที่ส่งออก</label>
      <Dropdown
        id="export-scope"
        options={[
          { value: "all", label: "ทุกรายการ" },
          { value: "low_stock", label: "สินค้าสต็อกต่ำ" },
          { value: "category", label: "ตามหมวดหมู่" },
        ]}
        bind:value={exportScope}
        minWidth="100%"
      />
    </div>
    {#if exportScope === "category"}
      <div class="form-group">
        <label for="export-category" class="form-label">หมวดหมู่</label>
        <Dropdown
          id="export-category"
          options={categories.map((c) => ({
            value: c.category_id,
            label: c.name,
          }))}
          bind:value={exportCategory}
          placeholder="เลือกหมวดหมู่"
          minWidth="100%"
        />
      </div>
    {/if}
    <div class="form-group">
      <label for="export-format" class="form-label">รูปแบบไฟล์</label>
      <Dropdown
        id="export-format"
        options={[
          { value: "csv", label: "CSV" },
          { value: "excel", label: "Excel" },
          { value: "pdf", label: "PDF" },
        ]}
        bind:value={exportFormat}
        minWidth="100%"
      />
    </div>
    <div class="form-actions">
      <button type="button" class="btn-outline" onclick={closeExportModal}>
        ยกเลิก
      </button>
      <button type="button" class="btn-primary" onclick={requestExport}>
        ส่งออก
      </button>
    </div>
  </div>
</Modal>

<ConfirmModal
  open={showExportConfirm}
  title="ยืนยันการส่งออกรายการสินค้า"
  message={`ต้องการส่งออกรายการสินค้า${
    exportScope === "all"
      ? "ทุกรายการ"
      : exportScope === "low_stock"
        ? "เฉพาะสินค้าสต็อกต่ำ"
        : `ตามหมวดหมู่ ${
            categories.find((c) => c.category_id === exportCategory)?.name ??
            exportCategory
          }`
  } ในรูปแบบ ${exportFormat.toUpperCase()} ใช่หรือไม่?`}
  confirmText="ส่งออก"
  cancelText="ยกเลิก"
  variant="primary"
  onConfirm={confirmExport}
  onCancel={cancelExport}
/>

<ConfirmModal
  open={showSaveConfirm}
  title="ยืนยันการบันทึกสินค้า"
  message={`ต้องการบันทึกสินค้า "${pendingProductName}" ใช่หรือไม่?`}
  confirmText="บันทึก"
  cancelText="ยกเลิก"
  variant="primary"
  onConfirm={confirmSaveProduct}
  onCancel={cancelSave}
/>

<ConfirmModal
  open={showDeleteConfirm}
  title="ยืนยันการลบสินค้า"
  message={`ต้องการลบสินค้า "${deleteTargetName}" ใช่หรือไม่? การดำเนินการนี้ไม่สามารถย้อนกลับได้`}
  confirmText="ลบ"
  cancelText="ยกเลิก"
  variant="danger"
  onConfirm={confirmDeleteProduct}
  onCancel={cancelDeleteProduct}
/>

<style>
  .product-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-md);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    opacity: 0.85;
  }

  .form-group.has-error .form-label {
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

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
    padding-top: var(--space-md);
    border-top: var(--border-subtle);
  }

  @media (max-width: 480px) {
    .form-grid {
      grid-template-columns: 1fr;
    }
  }

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

  .export-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .content-area {
    padding: var(--space-xl);
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
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
    flex: 1;
    min-height: 180px;
  }

  .data-table {
    min-width: 800px;
  }

  .font-medium {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  /* Make the product name column wider */
  .col-name {
    min-width: 280px;
    width: 30%;
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

  .empty-state {
    text-align: center;
    padding: var(--space-xl);
    color: var(--color-text-primary);
    opacity: 0.6;
  }

  .loading-state {
    text-align: center;
    padding: var(--space-xl);
    color: var(--color-text-primary);
    opacity: 0.7;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 3px solid rgba(0, 0, 0, 0.1);
    border-radius: 50%;
    border-top-color: var(--color-primary);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error-state {
    text-align: center;
    padding: var(--space-xl);
    color: var(--color-danger);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
  }

  .btn-sm {
    padding: 4px 12px;
    font-size: 13px;
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
