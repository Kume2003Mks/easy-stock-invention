<script lang="ts">
  let transactions = $state([
    { transaction_id: 'TX001', product_id: 'P001', transaction_type: 'IN', quantity: 50, reference_no: 'PO-2026-08-01', transaction_date: '2026-08-01 10:30' },
    { transaction_id: 'TX002', product_id: 'P002', transaction_type: 'OUT', quantity: 2, reference_no: 'INV-001', transaction_date: '2026-08-02 14:15' },
    { transaction_id: 'TX003', product_id: 'P003', transaction_type: 'ADJUST', quantity: -1, reference_no: 'Damaged goods', transaction_date: '2026-08-03 09:00' },
  ]);

  let products = $state([
    { product_id: 'P001', name: 'Premium Coffee Beans 500g' },
    { product_id: 'P002', name: 'Oat Milk Barista Edition' },
    { product_id: 'P003', name: 'Ceramic Mug Set' }
  ]);

  function getProductName(product_id: string): string {
    return products.find(p => p.product_id === product_id)?.name ?? product_id;
  }
</script>

<header class="topbar">
  <h1>รายงาน</h1>
  <button class="btn-outline">ส่งออก</button>
</header>

<div class="content-area">
  <div class="card">
    <table class="data-table">
      <thead>
        <tr>
          <th>วันที่</th>
          <th>สินค้า</th>
          <th>ประเภท</th>
          <th>จำนวน</th>
          <th>อ้างอิง</th>
        </tr>
      </thead>
      <tbody>
        {#each transactions as tx}
          <tr>
            <td>{tx.transaction_date}</td>
            <td class="font-medium">{getProductName(tx.product_id)}</td>
            <td>
              <span class="badge {tx.transaction_type === 'IN' ? 'badge-success' : tx.transaction_type === 'OUT' ? 'badge-primary' : 'badge-neutral'}">
                {tx.transaction_type === 'IN' ? 'รับเข้า' : tx.transaction_type === 'OUT' ? 'จ่ายออก' : 'ปรับปรุง'}
              </span>
            </td>
            <td>{tx.quantity > 0 ? '+' + tx.quantity : tx.quantity}</td>
            <td>{tx.reference_no}</td>
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

  .badge {
    padding: 4px 10px;
    border-radius: 100px;
    font-size: 13px;
    font-weight: 600;
  }

  .badge-success {
    background-color: #E8F2E2;
    color: var(--color-accent-success);
  }

  .badge-primary {
    background-color: #EBF1F7;
    color: var(--color-primary);
  }

  .badge-neutral {
    background-color: var(--color-muted);
    color: var(--color-text-primary);
  }
</style>