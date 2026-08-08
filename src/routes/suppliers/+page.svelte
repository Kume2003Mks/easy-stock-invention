<script lang="ts">
  import ActionMenu from '$lib/components/ActionMenu.svelte';

  let suppliers = $state([
    { supplier_id: 'S01', name: 'บริษัท โกลบอล เทรด', contact_info: 'contact@globaltrade.com' },
    { supplier_id: 'S02', name: 'ช่างฝีมือท้องถิ่น', contact_info: '02-123-4567' }
  ]);

  function deleteSupplier(id: string) {
    suppliers = suppliers.filter((s) => s.supplier_id !== id);
  }
</script>

<header class="topbar">
  <h1>ผู้จัดจำหน่าย</h1>
  <button class="btn-primary">+ เพิ่มผู้จัดจำหน่าย</button>
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
                      label: 'ดูรายการสินค้าที่จัดส่ง',
                      icon: 'view',
                      onclick: () => console.log('View products for', s.supplier_id),
                    },
                    {
                      label: 'ลบผู้จัดจำหน่าย',
                      icon: 'delete',
                      variant: 'danger',
                      onclick: () => deleteSupplier(s.supplier_id),
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
</style>