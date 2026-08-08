<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import ActionMenu from '$lib/components/ActionMenu.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import Pagination from '$lib/components/Pagination.svelte';
  import ErrorModal from '$lib/components/ErrorModal.svelte';
  import { parseAppError } from '$lib/utils/errorHandler';

  interface Category {
    category_id: string;
    name: string;
  }

  let categories = $state<Category[]>([]);
  let loading = $state(true);
  let loadError = $state('');

  // Error modal state
  let showErrorModal = $state(false);
  let errorModalTitle = $state('เกิดข้อผิดพลาด');
  let errorModalMessage = $state('');
  let errorModalDetails = $state('');

  function showError(err: unknown, fallbackTitle = 'เกิดข้อผิดพลาด') {
    const formatted = parseAppError(err, fallbackTitle);
    errorModalTitle = formatted.title;
    errorModalMessage = formatted.message;
    errorModalDetails = formatted.details ?? '';
    showErrorModal = true;
  }

  // Search & Pagination state
  let searchQuery = $state('');
  let currentPage = $state(1);
  let pageSize = $state(10);

  let filteredCategories = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return categories;
    return categories.filter(
      (c) =>
        c.name.toLowerCase().includes(q) ||
        c.category_id.toLowerCase().includes(q)
    );
  });

  let paginatedCategories = $derived.by(() => {
    const offset = (currentPage - 1) * pageSize;
    return filteredCategories.slice(offset, offset + pageSize);
  });

  // Add modal state
  let showAddModal = $state(false);

  // Save confirmation state
  let showSaveConfirm = $state(false);
  let pendingCategoryName = $state('');

  // New category form state
  let newCategory = $state({
    name: ''
  });

  // Validation errors
  let formErrors = $state<Record<string, string>>({});

  // Delete confirmation state
  let showDeleteConfirm = $state(false);
  let deleteTargetId = $state<string | null>(null);
  let deleteTargetName = $state('');

  function clearFieldError(field: string) {
    if (formErrors[field]) {
      const { [field]: _removed, ...rest } = formErrors;
      formErrors = rest;
    }
  }

  function validateForm(): boolean {
    const errors: Record<string, string> = {};

    if (!newCategory.name.trim()) {
      errors.name = 'กรุณากรอกชื่อหมวดหมู่';
    }

    formErrors = errors;
    return Object.keys(errors).length === 0;
  }

  async function loadCategories() {
    loading = true;
    loadError = '';
    try {
      const result = (await invoke('get_categories')) as Category[];
      categories = result ?? [];
    } catch (err) {
      console.error('Failed to load categories:', err);
      loadError = String(err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadCategories();
  });

  function openAddModal() {
    newCategory = { name: '' };
    formErrors = {};
    showAddModal = true;
  }

  function closeAddModal() {
    showAddModal = false;
    formErrors = {};
  }

  function addCategory() {
    if (!validateForm()) return;

    pendingCategoryName = newCategory.name.trim();
    showSaveConfirm = true;
  }

  function cancelSave() {
    showSaveConfirm = false;
    pendingCategoryName = '';
  }

  async function confirmSaveCategory() {
    try {
      const saved = (await invoke('create_category', {
        category: {
          category_id: '',
          name: newCategory.name.trim()
        }
      })) as Category;

      categories = [...categories, saved];
      cancelSave();
      closeAddModal();
    } catch (err) {
      console.error('Failed to create category:', err);
      cancelSave();
      showError(err, 'ไม่สามารถบันทึกหมวดหมู่ได้');
    }
  }

  function requestDeleteCategory(c: { category_id: string; name: string }) {
    deleteTargetId = c.category_id;
    deleteTargetName = c.name;
    showDeleteConfirm = true;
  }

  function cancelDelete() {
    showDeleteConfirm = false;
    deleteTargetId = null;
    deleteTargetName = '';
  }

  async function confirmDeleteCategory() {
    if (deleteTargetId) {
      try {
        await invoke('delete_category', { categoryId: deleteTargetId });
        categories = categories.filter((c) => c.category_id !== deleteTargetId);
      } catch (err) {
        console.error('Failed to delete category:', err);
        showError(err, 'ไม่สามารถลบหมวดหมู่ได้');
      }
    }
    cancelDelete();
  }
</script>

<header class="topbar">
  <h1>หมวดหมู่</h1>
  <button class="btn-primary" onclick={openAddModal}>+ เพิ่มหมวดหมู่</button>
</header>

<div class="content-area">
  <div class="card">
    <div class="table-actions">
      <input
        type="text"
        class="input-field search-input"
        placeholder="ค้นหาด้วยชื่อ หรือรหัสหมวดหมู่..."
        bind:value={searchQuery}
        oninput={() => (currentPage = 1)}
      />
    </div>

    <div class="table-wrapper">
      <table class="data-table">
        <thead>
          <tr>
            <th class="col-id">รหัสหมวดหมู่</th>
            <th class="col-name">ชื่อหมวดหมู่</th>
            <th class="col-actions">จัดการ</th>
          </tr>
        </thead>
        <tbody>
          {#if loading}
            <tr>
              <td colspan="3" class="loading-state">
                <div class="spinner"></div>
                <span>กำลังโหลดหมวดหมู่...</span>
              </td>
            </tr>
          {:else if loadError}
            <tr>
              <td colspan="3" class="error-state">
                <span>เกิดข้อผิดพลาดในการโหลดข้อมูล: {loadError}</span>
                <button class="btn-outline btn-sm" onclick={loadCategories}>
                  ลองใหม่อีกครั้ง
                </button>
              </td>
            </tr>
          {:else}
            {#each paginatedCategories as c}
              <tr>
                <td class="id-cell col-id">{c.category_id}</td>
                <td class="font-medium col-name">{c.name}</td>
                <td class="col-actions">
                  <ActionMenu
                    align="right"
                    items={[
                      {
                        label: 'แก้ไขหมวดหมู่',
                        icon: 'edit',
                        onclick: () => console.log('Edit category', c.category_id),
                      },
                      {
                        label: 'ลบหมวดหมู่',
                        icon: 'delete',
                        variant: 'danger',
                        onclick: () => requestDeleteCategory(c),
                      },
                    ]}
                  />
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan="3" class="empty-state">ไม่พบหมวดหมู่</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>

    <Pagination
      bind:currentPage
      bind:pageSize
      totalItems={filteredCategories.length}
      itemLabel="หมวดหมู่"
    />
  </div>
</div>

<Modal
  open={showAddModal}
  title="เพิ่มหมวดหมู่ใหม่"
  onClose={closeAddModal}
  maxWidth="420px"
>
  <form
    class="category-form"
    onsubmit={(e) => {
      e.preventDefault();
      addCategory();
    }}
  >
    <div class="form-group" class:has-error={!!formErrors.name}>
      <label for="category-name" class="form-label">ชื่อหมวดหมู่ *</label>
      <input
        id="category-name"
        type="text"
        class="input-field"
        class:input-error={!!formErrors.name}
        placeholder="เช่น เครื่องดื่ม"
        bind:value={newCategory.name}
        oninput={() => clearFieldError('name')}
      />
      {#if formErrors.name}
        <span class="error-text">{formErrors.name}</span>
      {/if}
    </div>

    <div class="form-actions">
      <button type="button" class="btn-outline" onclick={closeAddModal}>
        ยกเลิก
      </button>
      <button type="submit" class="btn-primary">บันทึกหมวดหมู่</button>
    </div>
  </form>
</Modal>

<ConfirmModal
  open={showSaveConfirm}
  title="ยืนยันการบันทึกหมวดหมู่"
  message={`ต้องการบันทึกหมวดหมู่ "${pendingCategoryName}" ใช่หรือไม่?`}
  confirmText="บันทึก"
  cancelText="ยกเลิก"
  variant="primary"
  onConfirm={confirmSaveCategory}
  onCancel={cancelSave}
/>

<ConfirmModal
  open={showDeleteConfirm}
  title="ยืนยันการลบหมวดหมู่"
  message={`ต้องการลบหมวดหมู่ "${deleteTargetName}" ใช่หรือไม่? การดำเนินการนี้ไม่สามารถย้อนกลับได้`}
  confirmText="ลบ"
  cancelText="ยกเลิก"
  variant="danger"
  onConfirm={confirmDeleteCategory}
  onCancel={cancelDelete}
/>

<ErrorModal
  open={showErrorModal}
  title={errorModalTitle}
  message={errorModalMessage}
  details={errorModalDetails}
  onClose={() => (showErrorModal = false)}
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

  .col-id {
    width: 320px;
    min-width: 280px;
    max-width: 340px;
  }

  .col-name {
    width: auto;
  }

  .font-medium {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .id-cell {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 13px;
    color: var(--color-text-secondary, #6b7280);
    letter-spacing: -0.2px;
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

  .category-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
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
</style>