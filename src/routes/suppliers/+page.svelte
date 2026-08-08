<script lang="ts">
  import ActionMenu from '$lib/components/ActionMenu.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';

  let suppliers = $state([
    { supplier_id: 'S01', name: 'บริษัท โกลบอล เทรด', contact_info: 'contact@globaltrade.com' },
    { supplier_id: 'S02', name: 'ช่างฝีมือท้องถิ่น', contact_info: '02-123-4567' }
  ]);

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

  function confirmSaveSupplier() {
    const nextId = `S${String(suppliers.length + 1).padStart(2, '0')}`;
    suppliers = [
      ...suppliers,
      {
        supplier_id: nextId,
        name: newSupplier.name.trim(),
        contact_info: newSupplier.contact_info.trim()
      }
    ];

    cancelSave();
    closeAddModal();
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

  function confirmDeleteSupplier() {
    if (deleteTargetId) {
      suppliers = suppliers.filter((s) => s.supplier_id !== deleteTargetId);
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
    <div class="table-wrapper">
      <table class="data-table">
        <thead>
          <tr>
            <th>รหัสผู้จัดจำหน่าย</th>
            <th>ชื่อ</th>
            <th>ข้อมูลติดต่อ</th>
            <th class="col-actions">จัดการ</th>
          </tr>
        </thead>
        <tbody>
          {#each suppliers as s}
            <tr>
              <td>{s.supplier_id}</td>
              <td class="font-medium">{s.name}</td>
              <td>{s.contact_info}</td>
              <td class="col-actions">
                <ActionMenu
                  align="right"
                  items={[
                    {
                      label: 'แก้ไขข้อมูล',
                      icon: 'edit',
                      onclick: () => console.log('Edit supplier', s.supplier_id),
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
        </tbody>
      </table>
    </div>
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