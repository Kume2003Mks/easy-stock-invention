<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import ActionMenu from '$lib/components/ActionMenu.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import Pagination from '$lib/components/Pagination.svelte';
  import ErrorModal from '$lib/components/ErrorModal.svelte';
  import { parseAppError } from '$lib/utils/errorHandler';

  interface Supplier {
    supplier_id: string;
    name: string;
    contact_info: string | null;
  }

  let suppliers = $state<Supplier[]>([]);
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

  let filteredSuppliers = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return suppliers;
    return suppliers.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        (s.contact_info && s.contact_info.toLowerCase().includes(q)) ||
        s.supplier_id.toLowerCase().includes(q)
    );
  });

  let paginatedSuppliers = $derived.by(() => {
    const offset = (currentPage - 1) * pageSize;
    return filteredSuppliers.slice(offset, offset + pageSize);
  });

  // Add modal state
  let showAddModal = $state(false);

  // Save confirmation state
  let showSaveConfirm = $state(false);
  let pendingSupplierName = $state('');
  let pendingSupplierContact = $state('');

  // New supplier form state
  let newSupplier = $state({
    name: '',
    contact_info: ''
  });

  // Validation errors
  let formErrors = $state<Record<string, string>>({});

  // Delete confirmation state
  let showDeleteConfirm = $state(false);
  let deleteTargetId = $state<string | null>(null);
  let deleteTargetName = $state('');

  // Edit supplier modal state
  let showEditModal = $state(false);
  let showEditConfirm = $state(false);
  let editSupplier = $state({
    supplier_id: '',
    name: '',
    contact_info: ''
  });
  let editFormErrors = $state<Record<string, string>>({});

  function openEditModal(s: Supplier) {
    editSupplier = {
      supplier_id: s.supplier_id,
      name: s.name,
      contact_info: s.contact_info || ''
    };
    editFormErrors = {};
    showEditModal = true;
  }

  function closeEditModal() {
    showEditModal = false;
    editFormErrors = {};
  }

  function validateEditForm(): boolean {
    const errors: Record<string, string> = {};
    if (!editSupplier.name.trim()) {
      errors.name = 'กรุณากรอกชื่อผู้จัดจำหน่าย';
    }
    editFormErrors = errors;
    return Object.keys(errors).length === 0;
  }

  function requestSaveEditSupplier() {
    if (!validateEditForm()) return;
    showEditConfirm = true;
  }

  function cancelEditConfirm() {
    showEditConfirm = false;
  }

  async function confirmSaveEditSupplier() {
    try {
      const updated = (await invoke('update_supplier', {
        supplier: {
          supplier_id: editSupplier.supplier_id,
          name: editSupplier.name.trim(),
          contact_info: editSupplier.contact_info.trim() || null
        }
      })) as Supplier;

      suppliers = suppliers.map((s) =>
        s.supplier_id === updated.supplier_id ? updated : s
      );
      cancelEditConfirm();
      closeEditModal();
    } catch (err) {
      console.error('Failed to update supplier:', err);
      cancelEditConfirm();
      showError(err, 'ไม่สามารถแก้ไขผู้จัดจำหน่ายได้');
    }
  }

  function clearFieldError(field: string) {
    if (formErrors[field]) {
      const { [field]: _removed, ...rest } = formErrors;
      formErrors = rest;
    }
  }

  function validateForm(): boolean {
    const errors: Record<string, string> = {};

    if (!newSupplier.name.trim()) {
      errors.name = 'กรุณากรอกชื่อผู้จัดจำหน่าย';
    }

    formErrors = errors;
    return Object.keys(errors).length === 0;
  }

  async function loadSuppliers() {
    loading = true;
    loadError = '';
    try {
      const result = (await invoke('get_suppliers')) as Supplier[];
      suppliers = result ?? [];
    } catch (err) {
      console.error('Failed to load suppliers:', err);
      loadError = String(err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadSuppliers();
  });

  function openAddModal() {
    newSupplier = { name: '', contact_info: '' };
    formErrors = {};
    showAddModal = true;
  }

  function closeAddModal() {
    showAddModal = false;
    formErrors = {};
  }

  function addSupplier() {
    if (!validateForm()) return;

    pendingSupplierName = newSupplier.name.trim();
    pendingSupplierContact = newSupplier.contact_info.trim();
    showSaveConfirm = true;
  }

  function cancelSave() {
    showSaveConfirm = false;
    pendingSupplierName = '';
    pendingSupplierContact = '';
  }

  async function confirmSaveSupplier() {
    try {
      const saved = (await invoke('create_supplier', {
        supplier: {
          supplier_id: '',
          name: newSupplier.name.trim(),
          contact_info: newSupplier.contact_info.trim() || null
        }
      })) as Supplier;

      suppliers = [...suppliers, saved];
      cancelSave();
      closeAddModal();
    } catch (err) {
      console.error('Failed to create supplier:', err);
      cancelSave();
      showError(err, 'ไม่สามารถบันทึกผู้จัดจำหน่ายได้');
    }
  }

  function requestDeleteSupplier(s: { supplier_id: string; name: string }) {
    deleteTargetId = s.supplier_id;
    deleteTargetName = s.name;
    showDeleteConfirm = true;
  }

  function cancelDelete() {
    showDeleteConfirm = false;
    deleteTargetId = null;
    deleteTargetName = '';
  }

  async function confirmDeleteSupplier() {
    if (deleteTargetId) {
      try {
        await invoke('delete_supplier', { supplierId: deleteTargetId });
        suppliers = suppliers.filter((s) => s.supplier_id !== deleteTargetId);
      } catch (err) {
        console.error('Failed to delete supplier:', err);
        showError(err, 'ไม่สามารถลบผู้จัดจำหน่ายได้');
      }
    }
    cancelDelete();
  }
</script>

<header class="topbar">
  <h1>ผู้จัดจำหน่าย</h1>
  <button class="btn-primary" onclick={openAddModal}>+ เพิ่มผู้จัดจำหน่าย</button>
</header>

<div class="content-area">
  <div class="card">
    <div class="table-actions">
      <input
        type="text"
        class="input-field search-input"
        placeholder="ค้นหาด้วยชื่อ, ข้อมูลติดต่อ หรือรหัสผู้จัดจำหน่าย..."
        bind:value={searchQuery}
        oninput={() => (currentPage = 1)}
      />
    </div>

    <div class="table-wrapper">
      <table class="data-table">
        <thead>
          <tr>
            <th class="col-id">รหัสผู้จัดจำหน่าย</th>
            <th class="col-name">ชื่อ</th>
            <th class="col-contact">ข้อมูลติดต่อ</th>
            <th class="col-actions">จัดการ</th>
          </tr>
        </thead>
        <tbody>
          {#if loading}
            <tr>
              <td colspan="4" class="loading-state">
                <div class="spinner"></div>
                <span>กำลังโหลดผู้จัดจำหน่าย...</span>
              </td>
            </tr>
          {:else if loadError}
            <tr>
              <td colspan="4" class="error-state">
                <span>เกิดข้อผิดพลาดในการโหลดข้อมูล: {loadError}</span>
                <button class="btn-outline btn-sm" onclick={loadSuppliers}>
                  ลองใหม่อีกครั้ง
                </button>
              </td>
            </tr>
          {:else}
            {#each paginatedSuppliers as s}
              <tr>
                <td class="id-cell col-id">{s.supplier_id}</td>
                <td class="font-medium col-name">{s.name}</td>
                <td class="col-contact">{s.contact_info || '-'}</td>
                <td class="col-actions">
                  <ActionMenu
                    align="right"
                    items={[
                      {
                        label: 'แก้ไขข้อมูล',
                        icon: 'edit',
                        onclick: () => openEditModal(s),
                      },
                      {
                        label: 'ลบผู้จัดจำหน่าย',
                        icon: 'delete',
                        variant: 'danger',
                        onclick: () => requestDeleteSupplier(s),
                      },
                    ]}
                  />
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan="4" class="empty-state">ไม่พบผู้จัดจำหน่าย</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>

    <Pagination
      bind:currentPage
      bind:pageSize
      totalItems={filteredSuppliers.length}
      itemLabel="ผู้จัดจำหน่าย"
    />
  </div>
</div>

<Modal
  open={showAddModal}
  title="เพิ่มผู้จัดจำหน่ายใหม่"
  onClose={closeAddModal}
  maxWidth="520px"
>
  <form
    class="supplier-form"
    onsubmit={(e) => {
      e.preventDefault();
      addSupplier();
    }}
  >
    <div class="form-grid">
      <div class="form-group" class:has-error={!!formErrors.name}>
        <label for="supplier-name" class="form-label">ชื่อผู้จัดจำหน่าย *</label>
        <input
          id="supplier-name"
          type="text"
          class="input-field"
          class:input-error={!!formErrors.name}
          placeholder="เช่น บริษัท โกลบอล เทรด"
          bind:value={newSupplier.name}
          oninput={() => clearFieldError('name')}
        />
        {#if formErrors.name}
          <span class="error-text">{formErrors.name}</span>
        {/if}
      </div>

      <div class="form-group">
        <label for="supplier-contact" class="form-label">ข้อมูลติดต่อ</label>
        <input
          id="supplier-contact"
          type="text"
          class="input-field"
          placeholder="เช่น contact@email.com หรือ 02-123-4567"
          bind:value={newSupplier.contact_info}
        />
      </div>
    </div>

    <div class="form-actions">
      <button type="button" class="btn-outline" onclick={closeAddModal}>
        ยกเลิก
      </button>
      <button type="submit" class="btn-primary">บันทึกผู้จัดจำหน่าย</button>
    </div>
  </form>
</Modal>

<ConfirmModal
  open={showSaveConfirm}
  title="ยืนยันการบันทึกผู้จัดจำหน่าย"
  message={pendingSupplierContact
    ? `ต้องการบันทึกผู้จัดจำหน่าย "${pendingSupplierName}" (ติดต่อ: ${pendingSupplierContact}) ใช่หรือไม่?`
    : `ต้องการบันทึกผู้จัดจำหน่าย "${pendingSupplierName}" ใช่หรือไม่?`}
  confirmText="บันทึก"
  cancelText="ยกเลิก"
  variant="primary"
  onConfirm={confirmSaveSupplier}
  onCancel={cancelSave}
/>

<Modal
  open={showEditModal}
  title="แก้ไขข้อมูลผู้จัดจำหน่าย"
  onClose={closeEditModal}
  maxWidth="520px"
>
  <form
    class="supplier-form"
    onsubmit={(e) => {
      e.preventDefault();
      requestSaveEditSupplier();
    }}
  >
    <div class="form-grid">
      <div class="form-group" class:has-error={!!editFormErrors.name}>
        <label for="edit-supplier-name" class="form-label">ชื่อผู้จัดจำหน่าย *</label>
        <input
          id="edit-supplier-name"
          type="text"
          class="input-field"
          class:input-error={!!editFormErrors.name}
          placeholder="เช่น บริษัท โกลบอล เทรด"
          bind:value={editSupplier.name}
          oninput={() => {
            if (editFormErrors.name) {
              const { name: _n, ...rest } = editFormErrors;
              editFormErrors = rest;
            }
          }}
        />
        {#if editFormErrors.name}
          <span class="error-text">{editFormErrors.name}</span>
        {/if}
      </div>

      <div class="form-group">
        <label for="edit-supplier-contact" class="form-label">ข้อมูลติดต่อ</label>
        <input
          id="edit-supplier-contact"
          type="text"
          class="input-field"
          placeholder="เช่น contact@email.com หรือ 02-123-4567"
          bind:value={editSupplier.contact_info}
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
  title="ยืนยันการแก้ไขผู้จัดจำหน่าย"
  message={`ต้องการบันทึกการแก้ไขผู้จัดจำหน่าย "${editSupplier.name.trim()}" ใช่หรือไม่?`}
  confirmText="บันทึกการแก้ไข"
  cancelText="ยกเลิก"
  variant="primary"
  onConfirm={confirmSaveEditSupplier}
  onCancel={cancelEditConfirm}
/>

<ConfirmModal
  open={showDeleteConfirm}
  title="ยืนยันการลบผู้จัดจำหน่าย"
  message={`ต้องการลบผู้จัดจำหน่าย "${deleteTargetName}" ใช่หรือไม่? การดำเนินการนี้ไม่สามารถย้อนกลับได้`}
  confirmText="ลบ"
  cancelText="ยกเลิก"
  variant="danger"
  onConfirm={confirmDeleteSupplier}
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
    width: 260px;
    min-width: 200px;
  }

  .col-contact {
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

  .supplier-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr;
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
</style>