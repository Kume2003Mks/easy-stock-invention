<script lang="ts">
  let {
    value = $bindable(''),
    min = '',
    max = '',
    placeholder = 'วว/ดด/ปปปป',
    hasError = false
  }: {
    value: string;
    min?: string;
    max?: string;
    placeholder?: string;
    hasError?: boolean;
  } = $props();

  let hiddenInput = $state<HTMLInputElement | null>(null);

  // แปลงจาก "YYYY-MM-DD" เป็น "วัน/เดือน/ปี" (DD/MM/YYYY)
  function formatToDayMonthYear(val: string): string {
    if (!val || typeof val !== 'string') return placeholder;
    const parts = val.trim().split('-');
    if (parts.length === 3) {
      const [y, m, d] = parts;
      if (y && m && d) {
        return `${d.padStart(2, '0')}/${m.padStart(2, '0')}/${y}`;
      }
    }
    return val;
  }

  function openPicker() {
    if (hiddenInput) {
      try {
        if ('showPicker' in hiddenInput && typeof hiddenInput.showPicker === 'function') {
          hiddenInput.showPicker();
        } else {
          hiddenInput.focus();
        }
      } catch {
        hiddenInput.focus();
      }
    }
  }
</script>

<div
  class="date-picker-box {hasError ? 'has-error' : ''}"
  role="button"
  tabindex="0"
  onclick={openPicker}
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      openPicker();
    }
  }}
>
  <span class="display-text {!value ? 'placeholder' : ''}">
    {formatToDayMonthYear(value)}
  </span>

  <svg
    class="calendar-icon"
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
    <line x1="16" y1="2" x2="16" y2="6" />
    <line x1="8" y1="2" x2="8" y2="6" />
    <line x1="3" y1="10" x2="21" y2="10" />
  </svg>

  <!-- Native date input คลุมทับทั้งกล่องเพื่อให้คลิกตรงไหนก็เปิดปฏิทินทันที -->
  <input
    type="date"
    bind:this={hiddenInput}
    bind:value={value}
    {min}
    {max}
    class="native-date-overlay"
    tabindex="-1"
    aria-hidden="true"
  />
</div>

<style>
  .date-picker-box {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    width: 170px;
    height: 38px;
    padding: 0 12px;
    border: var(--border-subtle);
    border-radius: var(--radius-md);
    background-color: var(--color-surface);
    cursor: pointer;
    user-select: none;
    transition: border-color 0.2s, box-shadow 0.2s;
    box-sizing: border-box;
  }

  .date-picker-box:hover {
    border-color: var(--color-primary);
  }

  .date-picker-box:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px rgba(94, 129, 172, 0.2);
  }

  .date-picker-box.has-error {
    border-color: var(--color-danger, #BF616A);
    background-color: #FFF5F5;
  }

  .date-picker-box.has-error:hover {
    border-color: var(--color-danger, #BF616A);
  }

  .date-picker-box.has-error:focus {
    border-color: var(--color-danger, #BF616A);
    box-shadow: 0 0 0 2px rgba(191, 97, 106, 0.2);
  }

  .date-picker-box.has-error .calendar-icon {
    color: var(--color-danger, #BF616A);
  }

  .display-text {
    font-family: var(--font-body);
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    letter-spacing: 0.5px;
    pointer-events: none;
  }

  .display-text.placeholder {
    color: #98A2B3;
    font-weight: 400;
  }

  .calendar-icon {
    color: var(--color-primary);
    flex-shrink: 0;
    pointer-events: none;
  }

  /* คลุมพื้นที่ทั้งกล่องแบบโปร่งใส ให้คลิกตรงไหนก็กระตุ้น Calendar Picker */
  .native-date-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
    margin: 0;
    padding: 0;
    border: none;
    z-index: 2;
  }

  .native-date-overlay::-webkit-calendar-picker-indicator {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
  }
</style>
