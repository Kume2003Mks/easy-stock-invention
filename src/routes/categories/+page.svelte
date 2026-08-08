<script lang="ts">
  import ActionMenu from '$lib/components/ActionMenu.svelte';

  let categories = $state([
    { category_id: 'C01', name: 'เครื่องดื่ม' },
    { category_id: 'C02', name: 'สินค้าทั่วไป' },
    { category_id: 'C03', name: 'ขนมขบเคี้ยว' }
  ]);

  function deleteCategory(id: string) {
    categories = categories.filter((c) => c.category_id !== id);
  }
</script>

<header class="topbar">
  <h1>หมวดหมู่</h1>
  <button class="btn-primary">+ เพิ่มหมวดหมู่</button>
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
                      onclick: () => deleteCategory(c.category_id),
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