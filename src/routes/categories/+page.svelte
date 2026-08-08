<script lang="ts">
  import ActionMenu from '$lib/components/ActionMenu.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';

  let categories = $state([
    { category_id: 'C01', name: 'เครื่องดื่ม' },
    { category_id: 'C02', name: 'สินค้าทั่วไป' },
    { category_id: 'C03', name: 'ขนมขบเคี้ยว' }
  ]);

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

  function confirmSaveCategory() {
    const nextId = `C${String(categories.length + 1).padStart(2, '0')}`;
    categories = [
      ...categories,
      {
        category_id: nextId,
        name: newCategory.name.trim()
      }
    ];

    cancelSave();
    closeAddModal();
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

  function confirmDeleteCategory() {
    if (deleteTargetId) {
      categories = categories.filter((c) => c.category_id !== deleteTargetId);
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
    <div class="table-wrapper">
      <table class="data-table">
        <thead>
          <tr>
            <th>รหัสหมวดหมู่</th>
            <th>ชื่อหมวดหมู่</th>
            <th class="col-actions">จัดการ</th>
          </tr>
        </thead>
        <tbody>
          {#each categories as c}
            <tr>
              <td>{c.category_id}</td>
              <td class="font-medium">{c.name}</td>
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
        </tbody>
      </table>
    </div>
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

  .font-medium {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .empty-state {
    text-align: center;
    padding: var(--space-xl);
    color: var(--color-text-primary);
    opacity: 0.6;
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