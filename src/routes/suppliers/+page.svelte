<script lang="ts">
  let suppliers = $state([
    { supplier_id: 'S01', name: 'บริษัท โกลบอล เทรด', contact_info: 'contact@globaltrade.com' },
    { supplier_id: 'S02', name: 'ช่างฝีมือท้องถิ่น', contact_info: '02-123-4567' }
  ]);
</script>

<header class="topbar">
  <h1>ผู้จัดจำหน่าย</h1>
  <button class="btn-primary">+ เพิ่มผู้จัดจำหน่าย</button>
</header>

<div class="content-area">
  <div class="card">
    <table class="data-table">
      <thead>
        <tr>
          <th>รหัสผู้จัดจำหน่าย</th>
          <th>ชื่อ</th>
          <th>ข้อมูลติดต่อ</th>
          <th>จัดการ</th>
        </tr>
      </thead>
      <tbody>
        {#each suppliers as s}
          <tr>
            <td>{s.supplier_id}</td>
            <td class="font-medium">{s.name}</td>
            <td>{s.contact_info}</td>
            <td><button class="action-btn">แก้ไข</button></td>
          </tr>
        {/each}
      </tbody>
    </table>
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

  .action-btn {
    color: var(--color-primary);
    font-weight: 500;
    font-size: 14px;
  }

  .action-btn:hover {
    text-decoration: underline;
  }
</style>