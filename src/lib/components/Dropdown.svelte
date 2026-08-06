<script lang="ts">
  interface DropdownOption {
    value: string;
    label: string;
  }

  let {
    id,
    label,
    options,
    value = $bindable(''),
    onchange = () => {},
    minWidth = 'auto',
    placeholder = '',
  }: {
    id: string;
    label?: string;
    options: DropdownOption[];
    value?: string | number;
    onchange?: () => void;
    minWidth?: string;
    placeholder?: string;
  } = $props();
</script>

<div class="dropdown-wrapper" style={`--dropdown-min-width: ${minWidth}`}>
  {#if label}
    <label for={id} class="dropdown-label">{label}</label>
  {/if}
  <div class="dropdown-box">
    <select
      {id}
      class="dropdown-select"
      bind:value
      onchange={onchange}
    >
      {#if placeholder}
        <option value="" disabled>{placeholder}</option>
      {/if}
      {#each options as opt}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
    <!-- Chevron icon -->
    <svg
      class="dropdown-chevron"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path d="m6 9 6 6 6-6" />
    </svg>
  </div>
</div>

<style>
  .dropdown-wrapper {
    display: inline-flex;
    flex-direction: row;
    align-items: center;
    gap: 6px;
  }

  .dropdown-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    opacity: 0.8;
  }

  .dropdown-box {
    position: relative;
    display: inline-flex;
    align-items: center;
  }

  .dropdown-select {
    width: 100%;
    min-width: var(--dropdown-min-width);
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    padding: 10px 36px 10px 14px;
    border: var(--border-subtle);
    border-radius: 12px; /* Rounded shape */
    font-family: var(--font-body);
    font-size: 14px;
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    outline: none;
    cursor: pointer;
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  .dropdown-select:hover {
    border-color: var(--color-primary);
  }

  .dropdown-select:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(94, 129, 172, 0.15);
  }

  .dropdown-chevron {
    position: absolute;
    right: 14px;
    pointer-events: none;
    color: var(--color-text-primary);
    opacity: 0.6;
  }
</style>