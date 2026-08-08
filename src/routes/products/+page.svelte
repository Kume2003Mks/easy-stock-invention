<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Dropdown from "$lib/components/Dropdown.svelte";
  import ActionMenu from "$lib/components/ActionMenu.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import Pagination from "$lib/components/Pagination.svelte";
  import ErrorModal from "$lib/components/ErrorModal.svelte";
  import { parseAppError } from "$lib/utils/errorHandler";

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

  // Error modal state
  let showErrorModal = $state(false);
  let errorModalTitle = $state("เกิดข้อผิดพลาด");
  let errorModalMessage = $state("");
  let errorModalDetails = $state("");

  function showError(err: unknown, fallbackTitle = "เกิดข้อผิดพลาด") {
    const formatted = parseAppError(err, fallbackTitle);
    errorModalTitle = formatted.title;
    errorModalMessage = formatted.message;
    errorModalDetails = formatted.details ?? "";
    showErrorModal = true;
  }

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
      cancelSave();
      showError(err, "ไม่สามารถบันทึกสินค้าได้");
    }
  }

  // Edit product modal state
  let showEditModal = $state(false);
  let showEditConfirm = $state(false);
  let editProduct = $state({
    product_id: "",
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
  let editFormErrors = $state<Record<string, string>>({});

  function clearEditFieldError(field: string) {
    if (editFormErrors[field]) {
      const { [field]: _removed, ...rest } = editFormErrors;
      editFormErrors = rest;
    }
  }

  function validateEditForm(): boolean {
    const errors: Record<string, string> = {};
    if (!editProduct.name.trim()) {
      errors.name = "กรุณากรอกชื่อสินค้า";
    }
    if (!editProduct.barcode.trim()) {
      errors.barcode = "กรุณากรอกบาร์โค้ด";
    }
    if (!editProduct.category_id) {
      errors.category_id = "กรุณาเลือกหมวดหมู่";
    }
    editFormErrors = errors;
    return Object.keys(errors).length === 0;
  }

  function openEditModal(p: Product) {
    editProduct = {
      product_id: p.product_id,
      barcode: p.barcode ?? "",
      name: p.name,
      category_id: p.category_id ?? "",
      supplier_id: p.supplier_id ?? "",
      cost_price: p.cost_price,
      selling_price: p.selling_price,
      wholesale_price: p.wholesale_price,
      current_stock: p.current_stock,
      reorder_level: p.reorder_level,
    };
    editFormErrors = {};
    showEditModal = true;
  }

  function closeEditModal() {
    showEditModal = false;
    editFormErrors = {};
  }

  function saveEditProduct() {
    if (!validateEditForm()) return;
    showEditConfirm = true;
  }

  function cancelEditConfirm() {
    showEditConfirm = false;
  }

  async function confirmSaveEditProduct() {
    try {
      const payload: Product = {
        product_id: editProduct.product_id,
        barcode: editProduct.barcode.trim() || null,
        name: editProduct.name.trim(),
        category_id: editProduct.category_id || null,
        supplier_id: editProduct.supplier_id || null,
        cost_price: Number(editProduct.cost_price) || 0,
        selling_price: Number(editProduct.selling_price) || 0,
        wholesale_price: Number(editProduct.wholesale_price) || 0,
        current_stock: Number(editProduct.current_stock) || 0,
        reorder_level: Number(editProduct.reorder_level) || 10,
      };

      const updated = (await invoke("update_product", {
        product: payload,
      })) as Product;

      products = products.map((p) =>
        p.product_id === updated.product_id ? updated : p,
      );
      cancelEditConfirm();
      closeEditModal();
    } catch (err) {
      console.error("Failed to update product:", err);
      cancelEditConfirm();
      showError(err, "ไม่สามารถแก้ไขสินค้าได้");
    }
  }

  // Adjust stock modal state
  let showAdjustModal = $state(false);
  let showAdjustConfirm = $state(false);
  let adjustTargetProduct = $state<Product | null>(null);
  let adjustType = $state<"IN" | "OUT" | "ADJUST">("IN");
  let adjustQuantity = $state<number>(1);
  let adjustReference = $state("");
  let adjustError = $state("");

  let calculatedNewStock = $derived.by(() => {
    if (!adjustTargetProduct) return 0;
    const current = adjustTargetProduct.current_stock;
    const qty = Number(adjustQuantity) || 0;
    if (adjustType === "IN") {
      return current + qty;
    } else if (adjustType === "OUT") {
      return current - qty;
    } else if (adjustType === "ADJUST") {
      return qty;
    }
    return current;
  });

  function openAdjustModal(p: Product) {
    adjustTargetProduct = p;
    adjustType = "IN";
    adjustQuantity = 1;
    adjustReference = "";
    adjustError = "";
    showAdjustModal = true;
  }

  function closeAdjustModal() {
    showAdjustModal = false;
    adjustTargetProduct = null;
    adjustError = "";
  }

  function validateAdjust(): boolean {
    adjustError = "";
    if (!adjustTargetProduct) return false;
    const qty = Number(adjustQuantity);
    if (isNaN(qty)) {
      adjustError = "กรุณากรอกจำนวนตัวเลข";
      return false;
    }
    if (adjustType === "IN" && qty <= 0) {
      adjustError = "จำนวนรับเข้าต้องมากกว่า 0";
      return false;
    }
    if (adjustType === "OUT") {
      if (qty <= 0) {
        adjustError = "จำนวนจ่ายออกต้องมากกว่า 0";
        return false;
      }
      if (qty > adjustTargetProduct.current_stock) {
        adjustError = `สต็อกมีเพียง ${adjustTargetProduct.current_stock} ชิ้น ไม่สามารถจ่ายออกเกินสต็อกที่มีได้`;
        return false;
      }
    }
    if (adjustType === "ADJUST" && qty < 0) {
      adjustError = "จำนวนสต็อกต้องไม่ติดลบ";
      return false;
    }
    return true;
  }

  function requestAdjustStock() {
    if (!validateAdjust()) return;
    showAdjustConfirm = true;
  }

  function cancelAdjustConfirm() {
    showAdjustConfirm = false;
  }

  async function confirmAdjustStock() {
    if (!adjustTargetProduct) return;
    try {
      const updated = (await invoke("adjust_stock", {
        productId: adjustTargetProduct.product_id,
        transactionType: adjustType,
        quantity: Number(adjustQuantity),
        referenceNo: adjustReference.trim() || null,
      })) as Product;

      products = products.map((p) =>
        p.product_id === updated.product_id ? updated : p,
      );
      cancelAdjustConfirm();
      closeAdjustModal();
    } catch (err) {
      console.error("Failed to adjust stock:", err);
      cancelAdjustConfirm();
      showError(err, "ไม่สามารถปรับปรุงสต็อกได้");
    }
  }

  // Pagination state
  let currentPage = $state(1); // 1-based
  let pageSize = $state(10); // default 10

  // Search state
  let searchQuery = $state("");

  // Category filter state
  let selectedCategory = $state("all");

  // Filtered products based on search + category
  let filteredProducts = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    return products.filter((p) => {
      // Category filter
      if (selectedCategory !== "all") {
        if (selectedCategory === "none") {
          if (p.category_id) return false;
        } else if (p.category_id !== selectedCategory) {
          return false;
        }
      }
      // Search filter
      if (q) {
        return (
          p.name.toLowerCase().includes(q) ||
          (p.barcode && p.barcode.toLowerCase().includes(q)) ||
          (p.category_id &&
            getCategoryName(p.category_id).toLowerCase().includes(q)) ||
          (!p.category_id &&
            ("ไม่มีหมวดหมู่".includes(q) ||
              "ไม่ระบุหมวดหมู่".includes(q) ||
              "ไม่มี".includes(q))) ||
          (p.supplier_id &&
            getSupplierName(p.supplier_id).toLowerCase().includes(q))
        );
      }
      return true;
    });
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
        showError(err, "ไม่สามารถลบสินค้าได้");
      }
    }
    cancelDeleteProduct();
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
        oninput={() => (currentPage = 1)}
      />
      <Dropdown
        id="category-filter"
        label="หมวดหมู่:"
        options={[
          { value: "all", label: "ทั้งหมด" },
          { value: "none", label: "ไม่มีหมวดหมู่" },
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
                        onclick: () => openEditModal(p),
                      },
                      {
                        label: "ปรับปรุงสต็อก",
                        icon: "adjust",
                        onclick: () => openAdjustModal(p),
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

    <Pagination
      bind:currentPage
      bind:pageSize
      totalItems={filteredProducts.length}
      itemLabel="รายการ"
    />
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
          options={[
            { value: "none", label: "ไม่มีหมวดหมู่" },
            ...categories.map((c) => ({
              value: c.category_id,
              label: c.name,
            })),
          ]}
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

<Modal
  open={showEditModal}
  title="แก้ไขข้อมูลสินค้า"
  onClose={closeEditModal}
  maxWidth="640px"
>
  <form
    class="product-form"
    onsubmit={(e) => {
      e.preventDefault();
      saveEditProduct();
    }}
  >
    <div class="form-grid">
      <div class="form-group" class:has-error={!!editFormErrors.barcode}>
        <label for="edit-product-barcode" class="form-label">บาร์โค้ด *</label>
        <input
          id="edit-product-barcode"
          type="text"
          class="input-field"
          class:input-error={!!editFormErrors.barcode}
          placeholder="เช่น 8850123456789"
          bind:value={editProduct.barcode}
          oninput={() => clearEditFieldError("barcode")}
        />
        {#if editFormErrors.barcode}
          <span class="error-text">{editFormErrors.barcode}</span>
        {/if}
      </div>

      <div class="form-group" class:has-error={!!editFormErrors.name}>
        <label for="edit-product-name" class="form-label">ชื่อสินค้า *</label>
        <input
          id="edit-product-name"
          type="text"
          class="input-field"
          class:input-error={!!editFormErrors.name}
          placeholder="เช่น Premium Coffee Beans 500g"
          bind:value={editProduct.name}
          oninput={() => clearEditFieldError("name")}
        />
        {#if editFormErrors.name}
          <span class="error-text">{editFormErrors.name}</span>
        {/if}
      </div>

      <div class="form-group" class:has-error={!!editFormErrors.category_id}>
        <label for="edit-product-category" class="form-label">หมวดหมู่ *</label>
        <Dropdown
          id="edit-product-category"
          options={categories.map((c) => ({
            value: c.category_id,
            label: c.name,
          }))}
          bind:value={editProduct.category_id}
          onchange={() => clearEditFieldError("category_id")}
          placeholder="เลือกหมวดหมู่"
          minWidth="100%"
          hasError={!!editFormErrors.category_id}
        />
        {#if editFormErrors.category_id}
          <span class="error-text">{editFormErrors.category_id}</span>
        {/if}
      </div>

      <div class="form-group">
        <label for="edit-product-supplier" class="form-label"
          >ผู้จัดจำหน่าย</label
        >
        <Dropdown
          id="edit-product-supplier"
          options={suppliers.map((s) => ({
            value: s.supplier_id,
            label: s.name,
          }))}
          bind:value={editProduct.supplier_id}
          placeholder="เลือกผู้จัดจำหน่าย"
          clearable={true}
          minWidth="100%"
        />
      </div>

      <div class="form-group">
        <label for="edit-product-cost" class="form-label">ต้นทุน (บาท)</label>
        <input
          id="edit-product-cost"
          type="number"
          min="0"
          step="0.01"
          class="input-field"
          placeholder="0.00"
          bind:value={editProduct.cost_price}
        />
      </div>

      <div class="form-group">
        <label for="edit-product-selling" class="form-label"
          >ราคาขาย (บาท)</label
        >
        <input
          id="edit-product-selling"
          type="number"
          min="0"
          step="0.01"
          class="input-field"
          placeholder="0.00"
          bind:value={editProduct.selling_price}
        />
      </div>

      <div class="form-group">
        <label for="edit-product-wholesale" class="form-label"
          >ราคาส่ง (บาท)</label
        >
        <input
          id="edit-product-wholesale"
          type="number"
          min="0"
          step="0.01"
          class="input-field"
          placeholder="0.00"
          bind:value={editProduct.wholesale_price}
        />
      </div>

      <div class="form-group">
        <label for="edit-product-reorder" class="form-label"
          >ระดับสต็อกขั้นต่ำ</label
        >
        <input
          id="edit-product-reorder"
          type="number"
          min="0"
          step="1"
          class="input-field"
          placeholder="10"
          bind:value={editProduct.reorder_level}
        />
      </div>
    </div>

    <div class="form-actions">
      <button type="button" class="btn-outline" onclick={closeEditModal}>
        ยกเลิก
      </button>
      <button type="submit" class="btn-primary">บันทึกการแก้ไข</button>
    </div>
  </form>
</Modal>

<ConfirmModal
  open={showEditConfirm}
  title="ยืนยันการแก้ไขข้อมูลสินค้า"
  message={`ต้องการบันทึกการแก้ไขสินค้า "${editProduct.name}" ใช่หรือไม่?`}
  confirmText="บันทึก"
  cancelText="ยกเลิก"
  variant="primary"
  onConfirm={confirmSaveEditProduct}
  onCancel={cancelEditConfirm}
/>

<Modal
  open={showAdjustModal}
  title="ปรับปรุงสต็อกสินค้า"
  onClose={closeAdjustModal}
  maxWidth="540px"
>
  {#if adjustTargetProduct}
    <div class="adjust-container">
      <div class="product-summary-card">
        <div class="summary-details">
          <span class="summary-name">{adjustTargetProduct.name}</span>
          <span class="summary-meta">
            บาร์โค้ด: {adjustTargetProduct.barcode || "-"} | หมวดหมู่: {getCategoryName(
              adjustTargetProduct.category_id,
            )}
          </span>
        </div>
        <div class="summary-stock">
          <span class="stock-label">สต็อกปัจจุบัน</span>
          <span
            class="badge {adjustTargetProduct.current_stock >
            adjustTargetProduct.reorder_level
              ? 'badge-success'
              : 'badge-warning'}"
          >
            {adjustTargetProduct.current_stock} ชิ้น
          </span>
        </div>
      </div>

      <div class="adjust-form">
        <div class="form-group">
          <span class="form-label">ประเภทการปรับปรุง</span>
          <div class="type-selector">
            <button
              type="button"
              class="type-pill {adjustType === 'IN'
                ? 'type-pill-active in'
                : ''}"
              onclick={() => {
                adjustType = "IN";
                adjustError = "";
              }}
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
                <path d="M12 5v14M5 12l7-7 7 7" />
              </svg>
              <span>รับเข้า (+)</span>
            </button>

            <button
              type="button"
              class="type-pill {adjustType === 'OUT'
                ? 'type-pill-active out'
                : ''}"
              onclick={() => {
                adjustType = "OUT";
                adjustError = "";
              }}
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
                <path d="M12 19V5M5 12l7 7 7-7" />
              </svg>
              <span>จ่ายออก (-)</span>
            </button>

            <button
              type="button"
              class="type-pill {adjustType === 'ADJUST'
                ? 'type-pill-active adjust'
                : ''}"
              onclick={() => {
                adjustType = "ADJUST";
                adjustQuantity = adjustTargetProduct?.current_stock ?? 0;
                adjustError = "";
              }}
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
                <path d="M4 12h16" />
              </svg>
              <span>ปรับยอด (=)</span>
            </button>
          </div>
        </div>

        <div class="form-group" class:has-error={!!adjustError}>
          <label for="adjust-qty" class="form-label">
            {adjustType === "ADJUST"
              ? "จำนวนสต็อกจริงที่นับได้"
              : "จำนวนที่ต้องการปรับปรุง"} *
          </label>
          <div class="qty-input-wrapper">
            <button
              type="button"
              class="qty-btn"
              onclick={() => {
                const min = adjustType === "ADJUST" ? 0 : 1;
                adjustQuantity = Math.max(min, Number(adjustQuantity) - 1);
                adjustError = "";
              }}
            >
              -
            </button>
            <input
              id="adjust-qty"
              type="number"
              min={adjustType === "ADJUST" ? "0" : "1"}
              step="1"
              class="input-field qty-input"
              class:input-error={!!adjustError}
              bind:value={adjustQuantity}
              oninput={() => (adjustError = "")}
            />
            <button
              type="button"
              class="qty-btn"
              onclick={() => {
                adjustQuantity = Number(adjustQuantity) + 1;
                adjustError = "";
              }}
            >
              +
            </button>
          </div>
          {#if adjustError}
            <span class="error-text">{adjustError}</span>
          {/if}
        </div>

        <div class="preview-box">
          <div class="preview-item">
            <span class="preview-label">สต็อกเดิม:</span>
            <span class="preview-val"
              >{adjustTargetProduct.current_stock} ชิ้น</span
            >
          </div>
          <div class="preview-arrow">➔</div>
          <div class="preview-item">
            <span class="preview-label">สต็อกหลังปรับปรุง:</span>
            <span
              class="preview-val-new"
              class:val-negative={calculatedNewStock < 0}
            >
              {calculatedNewStock} ชิ้น
            </span>
          </div>
          <div class="preview-diff">
            ({adjustType === "IN"
              ? `+${adjustQuantity}`
              : adjustType === "OUT"
                ? `-${adjustQuantity}`
                : `${calculatedNewStock - adjustTargetProduct.current_stock >= 0 ? "+" : ""}${calculatedNewStock - adjustTargetProduct.current_stock}`})
          </div>
        </div>

        <div class="form-group">
          <label for="adjust-ref" class="form-label"
            >หมายเหตุ / เอกสารอ้างอิง</label
          >
          <input
            id="adjust-ref"
            type="text"
            class="input-field"
            placeholder="เช่น ตรวจนับสต็อกประจำรอบ, สินค้าชำรุด, PO-2026-08-01"
            bind:value={adjustReference}
          />
        </div>

        <div class="form-actions">
          <button type="button" class="btn-outline" onclick={closeAdjustModal}>
            ยกเลิก
          </button>
          <button
            type="button"
            class="btn-primary"
            onclick={requestAdjustStock}
          >
            บันทึกการปรับปรุง
          </button>
        </div>
      </div>
    </div>
  {/if}
</Modal>

<ConfirmModal
  open={showAdjustConfirm}
  title="ยืนยันการปรับปรุงสต็อกสินค้า"
  message={`ต้องการปรับปรุงสต็อกสินค้า "${adjustTargetProduct?.name}" ${
    adjustType === "IN"
      ? `เพิ่มขึ้น ${adjustQuantity} ชิ้น (จาก ${adjustTargetProduct?.current_stock} เป็น ${calculatedNewStock} ชิ้น)`
      : adjustType === "OUT"
        ? `ลดลง ${adjustQuantity} ชิ้น (จาก ${adjustTargetProduct?.current_stock} เป็น ${calculatedNewStock} ชิ้น)`
        : `เป็น ${calculatedNewStock} ชิ้น (จากเดิม ${adjustTargetProduct?.current_stock} ชิ้น)`
  } ใช่หรือไม่?`}
  confirmText="ยืนยันการปรับปรุง"
  cancelText="ยกเลิก"
  variant="primary"
  onConfirm={confirmAdjustStock}
  onCancel={cancelAdjustConfirm}
/>

<ErrorModal
  open={showErrorModal}
  title={errorModalTitle}
  message={errorModalMessage}
  details={errorModalDetails}
  onClose={() => (showErrorModal = false)}
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

  /* Adjust stock modal styles */
  .adjust-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .product-summary-card {
    background-color: var(--color-background);
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--space-md);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md);
    flex-wrap: wrap;
  }

  .summary-details {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .summary-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .summary-meta {
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.65;
  }

  .summary-stock {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }

  .stock-label {
    font-size: 12px;
    color: var(--color-text-primary);
    opacity: 0.6;
  }

  .adjust-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .type-selector {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 8px;
  }

  .type-pill {
    padding: 10px 8px;
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    background-color: var(--color-surface);
    color: var(--color-text-primary);
    font-size: 13px;
    font-weight: 500;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    transition: all 0.2s ease;
    cursor: pointer;
  }

  .type-pill:hover {
    background-color: var(--color-background);
  }

  .type-pill-active.in {
    background-color: #e8f2e2;
    border-color: #a3be8c;
    color: #2e7d32;
    font-weight: 600;
  }

  .type-pill-active.out {
    background-color: #ffebee;
    border-color: #ef9a9a;
    color: #c62828;
    font-weight: 600;
  }

  .type-pill-active.adjust {
    background-color: #ebf1f7;
    border-color: var(--color-primary);
    color: var(--color-primary);
    font-weight: 600;
  }

  .qty-input-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .qty-btn {
    width: 44px;
    height: 44px;
    border-radius: var(--radius-md);
    border: var(--border-subtle);
    background-color: var(--color-surface);
    color: var(--color-text-primary);
    font-size: 18px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    cursor: pointer;
  }

  .qty-btn:hover {
    background-color: var(--color-background);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .qty-input {
    text-align: center;
    font-size: 16px;
    font-weight: 600;
  }

  .preview-box {
    background-color: var(--color-background);
    border: 1px dashed var(--color-muted);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
  }

  .preview-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .preview-label {
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.7;
  }

  .preview-val {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .preview-arrow {
    color: var(--color-primary);
    font-size: 14px;
  }

  .preview-val-new {
    font-size: 15px;
    font-weight: 700;
    color: var(--color-primary);
  }

  .preview-val-new.val-negative {
    color: var(--color-danger);
  }

  .preview-diff {
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.65;
    font-weight: 500;
  }

  @media (max-width: 480px) {
    .type-selector {
      grid-template-columns: 1fr;
    }
  }
</style>
