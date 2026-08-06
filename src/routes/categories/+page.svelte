<script lang="ts">
  let categories = $state([
    { category_id: 'C01', name: 'เครื่องดื่ม' },
    { category_id: 'C02', name: 'สินค้าทั่วไป' },
    { category_id: 'C03', name: 'ขนมขบเคี้ยว' }
  ]);
</script>

<header class="topbar">
  <h1>หมวดหมู่</h1>
  <button class="btn-primary">+ เพิ่มหมวดหมู่</button>
</header>

<div class="content-area">
  <div class="card">
    <table class="data-table">
      <thead>
        <tr>
          <th>รหัสหมวดหมู่</th>
          <th>ชื่อหมวดหมู่</th>
          <th>จัดการ</th>
        </tr>
      </thead>
      <tbody>
        {#each categories as c}
          <tr>
            <td>{c.category_id}</td>
            <td class="font-medium">{c.name}</td>
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