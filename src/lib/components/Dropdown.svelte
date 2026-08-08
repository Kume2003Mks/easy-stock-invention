<script lang="ts">
  export interface DropdownOption {
    value: string | number;
    label: string;
    disabled?: boolean;
  }

  let {
    id = '',
    label = '',
    options = [],
    value = $bindable(''),
    onchange = () => {},
    minWidth = 'auto',
    placeholder = '',
    disabled = false,
    name = '',
    searchable,
    hasError = false,
  }: {
    id?: string;
    label?: string;
    options: DropdownOption[];
    value?: string | number;
    onchange?: (val: string | number) => void;
    minWidth?: string;
    placeholder?: string;
    disabled?: boolean;
    name?: string;
    searchable?: boolean;
    hasError?: boolean;
  } = $props();

  let isOpen = $state(false);
  let openUpward = $state(false);
  let alignRight = $state(false);
  let highlightedIndex = $state(-1);
  let searchQuery = $state('');
  let containerRef = $state<HTMLDivElement | null>(null);
  let menuContainerRef = $state<HTMLDivElement | null>(null);
  let searchInputRef = $state<HTMLInputElement | null>(null);
  let menuRef = $state<HTMLUListElement | null>(null);

  // Unique ID for ARIA
  const autoId = `dropdown-${Math.random().toString(36).substring(2, 9)}`;
  const instanceId = $derived(id || autoId);
  const listboxId = $derived(`${instanceId}-listbox`);

  // Find currently selected option
  const selectedOption = $derived(
    options.find((opt) => String(opt.value) === String(value))
  );

  // Display text in the trigger button
  const displayText = $derived(
    selectedOption ? selectedOption.label : placeholder || 'เลือกรายการ'
  );

  const isPlaceholder = $derived(!selectedOption && !!placeholder);

  // Auto-enable search if options count > 7 unless explicitly disabled
  const showSearch = $derived(
    searchable !== undefined ? searchable : options.length > 7
  );

  // Filtered options based on search query
  const filteredOptions = $derived.by(() => {
    if (!searchQuery.trim()) return options;
    const q = searchQuery.toLowerCase().trim();
    return options.filter(
      (opt) =>
        opt.label.toLowerCase().includes(q) ||
        String(opt.value).toLowerCase().includes(q)
    );
  });

  // Check if width is full
  const isFullWidth = $derived(
    minWidth === '100%' || minWidth === '100vw'
  );

  function checkFlipDirection() {
    if (!containerRef) return;
    const rect = containerRef.getBoundingClientRect();
    
    // Calculate required menu height (search bar + items + padding)
    const searchHeight = showSearch ? 46 : 0;
    const requiredHeight = Math.min(260, options.length * 38 + 16 + searchHeight);

    // Space relative to viewport
    const viewportSpaceBelow = window.innerHeight - rect.bottom;
    const viewportSpaceAbove = rect.top;

    // Space relative to nearest container (.card, .content-area)
    const container =
      containerRef.closest('.card') ||
      containerRef.closest('.content-area') ||
      containerRef.closest('.table-wrapper');
    
    let containerSpaceBelow = viewportSpaceBelow;
    let containerSpaceAbove = viewportSpaceAbove;

    if (container) {
      const containerRect = container.getBoundingClientRect();
      containerSpaceBelow = containerRect.bottom - rect.bottom;
      containerSpaceAbove = rect.top - containerRect.top;
    }

    // Rule 1: If space above inside container or viewport is too small, NEVER flip upward
    if (containerSpaceAbove < requiredHeight + 10 || viewportSpaceAbove < requiredHeight + 10) {
      openUpward = false;
    } else if (containerSpaceBelow >= requiredHeight + 16 && viewportSpaceBelow >= requiredHeight + 16) {
      // Rule 2: If space below is ample, stay downward
      openUpward = false;
    } else {
      // Rule 3: Space below is tight AND space above is ample -> flip upward
      openUpward = containerSpaceAbove > containerSpaceBelow && containerSpaceAbove >= requiredHeight;
    }

    // Horizontal check: if near the right edge of viewport/card, align right edge
    const spaceRight = window.innerWidth - rect.left;
    alignRight = spaceRight < 240 && rect.right > 200;
  }

  function toggleOpen() {
    if (disabled) return;
    if (isOpen) {
      closeMenu();
    } else {
      openMenu();
    }
  }

  function openMenu() {
    if (disabled) return;
    checkFlipDirection();
    isOpen = true;
    searchQuery = '';
    
    // Set initial highlighted index to the currently selected option
    const idx = filteredOptions.findIndex(
      (opt) => String(opt.value) === String(value)
    );
    highlightedIndex = idx >= 0 ? idx : 0;

    // Focus search input if searchable
    if (showSearch) {
      setTimeout(() => {
        searchInputRef?.focus();
      }, 30);
    }
  }

  function closeMenu(refocus = false) {
    isOpen = false;
    openUpward = false;
    alignRight = false;
    searchQuery = '';
    highlightedIndex = -1;
    if (refocus && containerRef) {
      const triggerBtn = containerRef.querySelector<HTMLButtonElement>('.dropdown-trigger');
      triggerBtn?.focus();
    }
  }

  // Verify and refine after menu DOM is mounted
  $effect(() => {
    if (isOpen && menuContainerRef) {
      checkFlipDirection();
    }
  });

  function selectOption(opt: DropdownOption) {
    if (opt.disabled) return;
    value = opt.value;
    onchange(opt.value);
    closeMenu(true);
  }

  // Handle outside click & Escape
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
        closeMenu(true);
      }
    }

    window.addEventListener('pointerdown', handlePointerDown);
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
      window.removeEventListener('keydown', handleKeyDown);
    };
  });

  // Auto-scroll highlighted item into view
  $effect(() => {
    if (isOpen && highlightedIndex >= 0 && menuRef) {
      const items = menuRef.querySelectorAll<HTMLElement>('.dropdown-item');
      const target = items[highlightedIndex];
      if (target) {
        target.scrollIntoView({ block: 'nearest' });
      }
    }
  });

  // Keyboard navigation on trigger / dropdown
  function handleTriggerKeyDown(e: KeyboardEvent) {
    if (disabled) return;

    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (!isOpen) {
        openMenu();
      } else {
        moveHighlight(e.key === 'ArrowDown' ? 1 : -1);
      }
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      if (!isOpen) {
        openMenu();
      } else if (highlightedIndex >= 0 && filteredOptions[highlightedIndex]) {
        selectOption(filteredOptions[highlightedIndex]);
      }
    } else if (e.key === 'Tab' && isOpen) {
      closeMenu();
    }
  }

  function handleSearchKeyDown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      moveHighlight(1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      moveHighlight(-1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (highlightedIndex >= 0 && filteredOptions[highlightedIndex]) {
        selectOption(filteredOptions[highlightedIndex]);
      }
    } else if (e.key === 'Tab') {
      closeMenu();
    }
  }

  function moveHighlight(direction: number) {
    if (filteredOptions.length === 0) return;
    let next = highlightedIndex + direction;
    if (next < 0) next = filteredOptions.length - 1;
    if (next >= filteredOptions.length) next = 0;

    // Skip disabled
    const start = next;
    while (filteredOptions[next]?.disabled) {
      next += direction;
      if (next < 0) next = filteredOptions.length - 1;
      if (next >= filteredOptions.length) next = 0;
      if (next === start) break;
    }

    highlightedIndex = next;
  }
</script>

<div
  bind:this={containerRef}
  class="dropdown-wrapper"
  class:is-open={isOpen}
  class:full-width={isFullWidth}
  style={`--dropdown-min-width: ${minWidth}`}
>
  {#if label}
    <label for={instanceId} class="dropdown-label">{label}</label>
  {/if}

  <div class="dropdown-box" class:is-open={isOpen} class:full-width={isFullWidth}>
    <!-- Hidden native input for forms -->
    {#if name}
      <input type="hidden" {name} {value} />
    {/if}

    <!-- Custom Trigger Button -->
    <button
      type="button"
      id={instanceId}
      class="dropdown-trigger"
      class:is-open={isOpen}
      class:is-placeholder={isPlaceholder}
      class:is-disabled={disabled}
      class:has-error={hasError}
      aria-haspopup="listbox"
      aria-expanded={isOpen}
      aria-controls={listboxId}
      {disabled}
      onclick={toggleOpen}
      onkeydown={handleTriggerKeyDown}
    >
      <span class="dropdown-value-text">{displayText}</span>
      
      <!-- Chevron icon with smooth rotation -->
      <svg
        class="dropdown-chevron"
        class:rotate={isOpen}
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="m6 9 6 6 6-6" />
      </svg>
    </button>

    <!-- Custom Popover Menu -->
    {#if isOpen}
      <div
        bind:this={menuContainerRef}
        class="dropdown-menu-container"
        class:open-upward={openUpward}
        class:align-right={alignRight}
        class:full-width={isFullWidth}
      >
        <!-- Search bar inside menu when options list is long -->
        {#if showSearch}
          <div class="dropdown-search-wrapper">
            <svg
              class="dropdown-search-icon"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <input
              bind:this={searchInputRef}
              type="text"
              class="dropdown-search-input"
              placeholder="ค้นหาตัวเลือก..."
              bind:value={searchQuery}
              onkeydown={handleSearchKeyDown}
            />
            {#if searchQuery}
              <button
                type="button"
                class="dropdown-search-clear"
                onclick={() => {
                  searchQuery = '';
                  searchInputRef?.focus();
                }}
                aria-label="ล้างการค้นหา"
              >
                ✕
              </button>
            {/if}
          </div>
        {/if}

        <!-- Option List -->
        <ul
          bind:this={menuRef}
          id={listboxId}
          class="dropdown-menu"
          role="listbox"
          tabindex="-1"
        >
          {#if filteredOptions.length === 0}
            <li class="dropdown-empty" role="presentation">
              ไม่พบตัวเลือก
            </li>
          {:else}
            {#each filteredOptions as opt, index (opt.value)}
              {@const isSelected = String(opt.value) === String(value)}
              {@const isHighlighted = highlightedIndex === index}
              <li
                id={`${listboxId}-option-${index}`}
                role="option"
                aria-selected={isSelected}
                aria-disabled={opt.disabled}
                class="dropdown-item"
                class:selected={isSelected}
                class:highlighted={isHighlighted}
                class:disabled={opt.disabled}
                onmouseenter={() => {
                  if (!opt.disabled) highlightedIndex = index;
                }}
                onclick={() => selectOption(opt)}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    selectOption(opt);
                  }
                }}
              >
                <span class="dropdown-item-label">{opt.label}</span>
                {#if isSelected}
                  <svg
                    class="dropdown-item-check"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                {/if}
              </li>
            {/each}
          {/if}
        </ul>
      </div>
    {/if}
  </div>
</div>

<style>
  .dropdown-wrapper {
    display: inline-flex;
    flex-direction: row;
    align-items: center;
    gap: 8px;
    position: relative;
    user-select: none;
  }

  .dropdown-wrapper.full-width {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    width: 100%;
    gap: 6px;
  }

  .dropdown-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    opacity: 0.85;
    white-space: nowrap;
  }

  .dropdown-box {
    position: relative;
    display: inline-flex;
    min-width: var(--dropdown-min-width);
  }

  .dropdown-box.full-width {
    display: flex;
    width: 100%;
    min-width: 100%;
  }

  /* Trigger Button */
  .dropdown-trigger {
    width: 100%;
    min-width: var(--dropdown-min-width);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 9px 14px;
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    font-family: var(--font-body);
    font-size: 14px;
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    outline: none;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.2s cubic-bezier(0.4, 0, 0.2, 1),
                box-shadow 0.2s cubic-bezier(0.4, 0, 0.2, 1),
                background-color 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .dropdown-trigger:hover:not(:disabled) {
    border-color: var(--color-primary);
  }

  .dropdown-trigger:focus-visible,
  .dropdown-trigger.is-open {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(94, 129, 172, 0.18);
  }

  .dropdown-trigger.is-placeholder .dropdown-value-text {
    color: var(--color-text-primary);
    opacity: 0.55;
  }

  .dropdown-trigger.is-disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background-color: var(--color-background);
  }

  .dropdown-trigger.has-error {
    border-color: var(--color-danger);
  }

  .dropdown-trigger.has-error:focus-visible,
  .dropdown-trigger.has-error.is-open {
    border-color: var(--color-danger);
    box-shadow: 0 0 0 3px rgba(191, 97, 106, 0.15);
  }

  .dropdown-value-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.4;
  }

  .dropdown-chevron {
    flex-shrink: 0;
    color: var(--color-text-primary);
    opacity: 0.6;
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1),
                opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1),
                color 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .dropdown-trigger:hover .dropdown-chevron,
  .dropdown-trigger.is-open .dropdown-chevron {
    opacity: 0.9;
    color: var(--color-primary);
  }

  .dropdown-chevron.rotate {
    transform: rotate(180deg);
  }

  .dropdown-wrapper.is-open,
  .dropdown-box.is-open {
    z-index: 100;
  }

  /* Popover Menu Panel */
  .dropdown-menu-container {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    min-width: 100%;
    width: max-content;
    max-width: 320px;
    z-index: 1000;
    background-color: var(--color-surface);
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 32px rgba(46, 52, 64, 0.12),
                0 4px 12px rgba(46, 52, 64, 0.05);
    padding: 6px;
    animation: dropdownSlideIn 0.16s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .dropdown-menu-container.align-right {
    left: auto;
    right: 0;
  }

  .dropdown-menu-container.open-upward {
    top: auto;
    bottom: calc(100% + 6px);
    animation: dropdownSlideUp 0.16s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .dropdown-menu-container.full-width {
    width: 100%;
    max-width: 100%;
  }

  @keyframes dropdownSlideIn {
    from {
      opacity: 0;
      transform: translateY(-6px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes dropdownSlideUp {
    from {
      opacity: 0;
      transform: translateY(6px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  /* Search Input in Menu */
  .dropdown-search-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    padding: 4px 6px 8px 6px;
    border-bottom: 1px solid var(--color-muted);
    margin-bottom: 4px;
  }

  .dropdown-search-icon {
    position: absolute;
    left: 14px;
    color: var(--color-text-primary);
    opacity: 0.5;
    pointer-events: none;
  }

  .dropdown-search-input {
    width: 100%;
    padding: 6px 26px 6px 30px;
    border: 1px solid transparent;
    border-radius: 8px;
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--color-text-primary);
    background-color: var(--color-background);
    outline: none;
    transition: border-color 0.15s, background-color 0.15s;
  }

  .dropdown-search-input:focus {
    border-color: var(--color-primary);
    background-color: var(--color-surface);
  }

  .dropdown-search-clear {
    position: absolute;
    right: 12px;
    background: none;
    border: none;
    padding: 2px 4px;
    font-size: 11px;
    color: var(--color-text-primary);
    opacity: 0.5;
    cursor: pointer;
    border-radius: 4px;
  }

  .dropdown-search-clear:hover {
    opacity: 0.9;
    color: var(--color-danger);
  }

  /* Option List */
  .dropdown-menu {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 240px;
    overflow-y: auto;
    overscroll-behavior: contain;
  }

  /* Sleek Scrollbar */
  .dropdown-menu::-webkit-scrollbar {
    width: 6px;
  }

  .dropdown-menu::-webkit-scrollbar-track {
    background: transparent;
  }

  .dropdown-menu::-webkit-scrollbar-thumb {
    background-color: var(--color-muted);
    border-radius: 999px;
  }

  .dropdown-menu::-webkit-scrollbar-thumb:hover {
    background-color: var(--color-primary);
  }

  /* Item Styles */
  .dropdown-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 12px;
    border-radius: 8px;
    margin-bottom: 2px;
    font-family: var(--font-body);
    font-size: 14px;
    color: var(--color-text-primary);
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  .dropdown-item:last-child {
    margin-bottom: 0;
  }

  .dropdown-item:hover,
  .dropdown-item.highlighted {
    background-color: var(--color-background);
    color: var(--color-text-primary);
  }

  .dropdown-item.selected {
    background-color: rgba(94, 129, 172, 0.12);
    color: var(--color-primary);
    font-weight: 600;
  }

  .dropdown-item.selected:hover,
  .dropdown-item.selected.highlighted {
    background-color: rgba(94, 129, 172, 0.18);
  }

  .dropdown-item.disabled {
    opacity: 0.4;
    cursor: not-allowed;
    pointer-events: none;
  }

  .dropdown-item-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dropdown-item-check {
    flex-shrink: 0;
    color: var(--color-primary);
  }

  .dropdown-empty {
    padding: 16px 12px;
    text-align: center;
    font-size: 13px;
    color: var(--color-text-primary);
    opacity: 0.55;
  }
</style>