<script lang="ts">
  import "../app.css";
  import { page } from "$app/state";
  let { children } = $props();

  const navItems = [
    { href: "/products", name: "สินค้าคงคลัง", icon: "box" },
    { href: "/categories", name: "หมวดหมู่", icon: "shapes" },
    { href: "/suppliers", name: "ผู้จัดจำหน่าย", icon: "truck" },
    // { href: '/transactions', name: 'รายงาน', icon: 'chart' },
    { href: "/settings", name: "ตั้งค่า", icon: "settings" },
  ];

  function isActive(href: string): boolean {
    return (
      page.url.pathname === href || page.url.pathname.startsWith(href + "/")
    );
  }
</script>

<div class="layout">
  <aside class="sidebar">
    <div class="logo">
      <h2>Easy Stock</h2>
      <!-- <p>POS System</p> -->
    </div>

    <nav>
      <ul>
        {#each navItems as item}
          <li>
            <a
              href={item.href}
              class="nav-btn {isActive(item.href) ? 'active' : ''}"
            >
              <span class="icon-wrapper">
                {#if item.icon === "box"}
                  <!-- Inventory / Box icon -->
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path
                      d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"
                    />
                    <path d="m3.3 7 8.7 5 8.7-5" />
                    <path d="M12 22V12" />
                  </svg>
                {:else if item.icon === "shapes"}
                  <!-- Categories / Shapes icon (Triangle, Square, Circle) -->
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="M12 3l4.5 7.5h-9z" />
                    <rect x="4" y="14" width="6.5" height="6.5" rx="1" />
                    <circle cx="17" cy="17.25" r="3.5" />
                  </svg>
                {:else if item.icon === "truck"}
                  <!-- Suppliers / Delivery Truck icon -->
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <rect x="2" y="5" width="13" height="10" rx="2" />
                    <polygon points="15 8 19 8 22 11 22 15 15 15 15 8" />
                    <circle cx="6" cy="18" r="2" />
                    <circle cx="18" cy="18" r="2" />
                  </svg>
                {:else if item.icon === "chart"}
                  <!-- Reports / Transactions / Bar Chart icon -->
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <rect x="3" y="3" width="18" height="18" rx="3" />
                    <path d="M7 16v-4" />
                    <path d="M12 16V8" />
                    <path d="M17 16v-6" />
                  </svg>
                {:else if item.icon === "settings"}
                  <!-- Settings / Gear icon -->
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path
                      d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
                    />
                    <circle cx="12" cy="12" r="3" />
                  </svg>
                {/if}
              </span>
              <span class="tab-label">{item.name}</span>
            </a>
          </li>
        {/each}
      </ul>
    </nav>
  </aside>

  <main class="main-content">
    {@render children()}
  </main>
</div>

<style>
  .layout {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .sidebar {
    width: 280px;
    background-color: var(--color-surface);
    border-right: var(--border-subtle);
    padding: var(--space-xl) var(--space-md);
    display: flex;
    flex-direction: column;
  }

  .logo {
    margin-bottom: var(--space-xl);
    padding-left: var(--space-sm);
  }

  .logo h2 {
    margin-bottom: 4px;
    color: var(--color-primary);
  }

  nav ul {
    list-style: none;
  }

  nav li {
    margin-bottom: 8px;
  }

  .nav-btn {
    width: 100%;
    display: flex;
    align-items: center;
    padding: 14px 18px;
    border-radius: 12px; /* Rounded shape */
    color: #4c566a;
    font-size: 17px;
    font-weight: 500;
    position: relative;
    background-color: transparent;
    text-decoration: none;
    transition: all 0.2s ease-in-out;
  }

  .icon-wrapper {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    margin-right: 16px;
    color: currentColor;
  }

  .tab-label {
    flex: 1;
    text-align: left;
  }

  .nav-btn:hover {
    background-color: #f4f7fa;
    color: var(--color-text-primary);
  }

  .nav-btn.active {
    background-color: #ebf1f7; /* Light tinted background */
    color: var(--color-primary);
    border-right: 4px solid var(--color-primary); /* Left edge indicator */
    font-weight: 600;
  }

  /* Active Indicator Bar on the Right Edge (matching image reference) */
  /* .nav-btn.active::after {
    content: '';
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 4px;
    height: 60%;
    background-color: var(--color-primary);
    border-radius: 4px;
  } */

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--color-background);
    overflow: hidden;
  }
</style>
