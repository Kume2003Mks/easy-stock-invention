<script lang="ts">
  export interface ActionMenuItem {
    id?: string;
    label: string;
    icon?: 'edit' | 'delete' | 'view' | 'history' | 'copy' | 'adjust';
    variant?: 'default' | 'danger' | 'primary';
    disabled?: boolean;
    onclick?: () => void;
  }

  let {
    items = [],
    align = 'right',
    ariaLabel = 'เมนูการจัดการ',
  }: {
    items: ActionMenuItem[];
    align?: 'left' | 'right';
    ariaLabel?: string;
  } = $props();

  let isOpen = $state(false);
  let openUpward = $state(false);
  let containerRef = $state<HTMLDivElement | null>(null);
  let dropdownRef = $state<HTMLDivElement | null>(null);
  let highlightedIndex = $state(-1);
  let menuRef = $state<HTMLUListElement | null>(null);

  const instanceId = `action-menu-${Math.random().toString(36).substring(2, 9)}`;

  function checkFlipDirection() {
    if (!containerRef) return;
    const rect = containerRef.getBoundingClientRect();
    
    // Estimate required menu height based on items count
    const requiredHeight = Math.max(120, items.length * 38 + 14);

    // Space relative to viewport
    const viewportSpaceBelow = window.innerHeight - rect.bottom;
    const viewportSpaceAbove = rect.top;

    // Space relative to nearest bounding container (e.g. .table-wrapper, .card, .content-area)
    const container =
      containerRef.closest('.table-wrapper') ||
      containerRef.closest('.card') ||
      containerRef.closest('.content-area');
    
    let containerSpaceBelow = viewportSpaceBelow;
    let containerSpaceAbove = viewportSpaceAbove;

    if (container) {
      const containerRect = container.getBoundingClientRect();
      containerSpaceBelow = containerRect.bottom - rect.bottom;
      containerSpaceAbove = rect.top - containerRect.top;
    }

    // Rule 1: If space above inside container or viewport is too small, NEVER flip upward (must open downward)
    if (containerSpaceAbove < requiredHeight + 10 || viewportSpaceAbove < requiredHeight + 10) {
      openUpward = false;
      return;
    }

    // Rule 2: If space below inside container & viewport is sufficient, stay downward
    if (containerSpaceBelow >= requiredHeight + 16 && viewportSpaceBelow >= requiredHeight + 16) {
      openUpward = false;
      return;
    }

    // Rule 3: Space below is constrained AND space above is ample -> flip upward
    openUpward = containerSpaceAbove > containerSpaceBelow && containerSpaceAbove >= requiredHeight;
  }

  function toggleMenu(e: MouseEvent) {
    e.stopPropagation();
    if (!isOpen) {
      checkFlipDirection();
      isOpen = true;
      highlightedIndex = 0;
    } else {
      closeMenu();
    }
  }

  function closeMenu() {
    isOpen = false;
    openUpward = false;
    highlightedIndex = -1;
  }

  // Also verify and refine after menu DOM is mounted
  $effect(() => {
    if (isOpen && dropdownRef) {
      checkFlipDirection();
    }
  });

  function handleItemClick(item: ActionMenuItem, e: MouseEvent) {
    e.stopPropagation();
    if (item.disabled) return;
    closeMenu();
    if (item.onclick) {
      item.onclick();
    }
  }

  // Ensure parent td/tr are elevated in z-index when open so subsequent sticky cells do not overlap
  $effect(() => {
    if (!containerRef) return;
    const td = containerRef.closest('td');
    const tr = containerRef.closest('tr');
    if (isOpen) {
      if (td) td.style.zIndex = '50';
      if (tr) tr.style.zIndex = '50';
    } else {
      if (td) td.style.zIndex = '';
      if (tr) tr.style.zIndex = '';
    }
  });

  // Handle click outside and Escape key
  $effect(() => {
    if (!isOpen) return;

    function handlePointerDown(e: MouseEvent | TouchEvent) {
      if (containerRef && !containerRef.contains(e.target as Node)) {
        closeMenu();
      }
    }

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault();
        closeMenu();
        const btn = containerRef?.querySelector<HTMLButtonElement>('.action-menu-trigger');
        btn?.focus();
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        highlightedIndex = (highlightedIndex + 1) % items.length;
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        highlightedIndex = (highlightedIndex - 1 + items.length) % items.length;
      } else if (e.key === 'Enter' || e.key === ' ') {
        if (highlightedIndex >= 0 && items[highlightedIndex]) {
          e.preventDefault();
          const item = items[highlightedIndex];
          if (!item.disabled && item.onclick) {
            closeMenu();
            item.onclick();
          }
        }
      } else if (e.key === 'Tab') {
        closeMenu();
      }
    }

    window.addEventListener('pointerdown', handlePointerDown);
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
      window.removeEventListener('keydown', handleKeyDown);
    };
  });
</script>

<div bind:this={containerRef} class="action-menu-wrapper" class:is-open={isOpen}>
  <!-- 3-Dots SVG Button -->
  <button
    type="button"
    class="action-menu-trigger"
    class:is-active={isOpen}
    aria-label={ariaLabel}
    aria-haspopup="true"
    aria-expanded={isOpen}
    onclick={toggleMenu}
  >
    <svg
      class="dots-icon"
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="1.5" />
      <circle cx="12" cy="5" r="1.5" />
      <circle cx="12" cy="19" r="1.5" />
    </svg>
  </button>

  <!-- Popover Menu Panel -->
  {#if isOpen}
    <div
      bind:this={dropdownRef}
      class="action-menu-dropdown"
      class:open-upward={openUpward}
      class:align-right={align === 'right'}
      class:align-left={align === 'left'}
      role="menu"
      aria-labelledby={instanceId}
    >
      <ul bind:this={menuRef} class="action-menu-list" role="none">
        {#each items as item, index}
          {@const isHighlighted = highlightedIndex === index}
          <li role="none">
            <button
              type="button"
              role="menuitem"
              class="action-menu-item {item.variant === 'danger' ? 'variant-danger' : item.variant === 'primary' ? 'variant-primary' : ''}"
              class:is-highlighted={isHighlighted}
              class:is-disabled={item.disabled}
              disabled={item.disabled}
              onmouseenter={() => (highlightedIndex = index)}
              onclick={(e) => handleItemClick(item, e)}
            >
              <!-- Icon rendering -->
              {#if item.icon === 'edit'}
                <svg class="item-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                  <path d="m15 5 4 4" />
                </svg>
              {:else if item.icon === 'delete'}
                <svg class="item-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M3 6h18" />
                  <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
                  <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                  <line x1="10" y1="11" x2="10" y2="17" />
                  <line x1="14" y1="11" x2="14" y2="17" />
                </svg>
              {:else if item.icon === 'view'}
                <svg class="item-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                  <circle cx="12" cy="12" r="3" />
                </svg>
              {:else if item.icon === 'history'}
                <svg class="item-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                  <path d="M3 3v5h5" />
                  <path d="M12 7v5l4 2" />
                </svg>
              {:else if item.icon === 'adjust'}
                <svg class="item-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="m21 16-4 4-4-4" />
                  <path d="M17 20V4" />
                  <path d="m3 8 4-4 4 4" />
                  <path d="M7 4v16" />
                </svg>
              {:else if item.icon === 'copy'}
                <svg class="item-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
                  <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
                </svg>
              {/if}

              <span class="item-label">{item.label}</span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>

<style>
  .action-menu-wrapper {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .action-menu-wrapper.is-open {
    z-index: 50;
  }

  /* 3-Dots Trigger Button */
  .action-menu-trigger {
    width: 32px;
    height: 32px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    background-color: transparent;
    color: var(--color-text-primary);
    opacity: 0.65;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
    padding: 0;
  }

  .action-menu-trigger:hover,
  .action-menu-trigger.is-active {
    background-color: var(--color-background);
    color: var(--color-primary);
    opacity: 1;
    border-color: var(--color-muted);
  }

  .action-menu-trigger.is-active {
    background-color: rgba(94, 129, 172, 0.12);
    border-color: var(--color-primary);
  }

  .dots-icon {
    flex-shrink: 0;
  }

  /* Popover Dropdown Panel */
  .action-menu-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    z-index: 1000;
    min-width: 145px;
    background-color: var(--color-surface);
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    box-shadow: 0 10px 25px -5px rgba(46, 52, 64, 0.12),
                0 4px 10px rgba(46, 52, 64, 0.05);
    padding: 4px;
    animation: menuSlideIn 0.15s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .action-menu-dropdown.open-upward {
    top: auto;
    bottom: calc(100% + 4px);
    box-shadow: 0 -10px 25px -5px rgba(46, 52, 64, 0.12),
                0 -4px 10px rgba(46, 52, 64, 0.05);
    animation: menuSlideUp 0.15s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .action-menu-dropdown.align-right {
    right: 0;
  }

  .action-menu-dropdown.align-left {
    left: 0;
  }

  @keyframes menuSlideIn {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes menuSlideUp {
    from {
      opacity: 0;
      transform: translateY(4px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .action-menu-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  /* Menu Items */
  .action-menu-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    border-radius: 6px;
    border: none;
    background: transparent;
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
    cursor: pointer;
    text-align: left;
    transition: background-color 0.12s ease, color 0.12s ease;
    white-space: nowrap;
  }

  .action-menu-item:hover,
  .action-menu-item.is-highlighted {
    background-color: var(--color-background);
    color: var(--color-text-primary);
  }

  .action-menu-item.variant-primary {
    color: var(--color-primary);
  }

  .action-menu-item.variant-primary:hover,
  .action-menu-item.variant-primary.is-highlighted {
    background-color: rgba(94, 129, 172, 0.1);
    color: var(--color-primary);
  }

  .action-menu-item.variant-danger {
    color: var(--color-danger);
  }

  .action-menu-item.variant-danger:hover,
  .action-menu-item.variant-danger.is-highlighted {
    background-color: rgba(191, 97, 106, 0.08);
    color: var(--color-danger);
  }

  .action-menu-item.is-disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .item-icon {
    flex-shrink: 0;
    opacity: 0.85;
  }

  .item-label {
    flex: 1;
  }
</style>
