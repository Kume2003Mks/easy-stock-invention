<script lang="ts">
  import Modal from './Modal.svelte';

  let {
    open = false,
    title = 'เกิดข้อผิดพลาด',
    message = '',
    details = '',
    buttonText = 'เข้าใจแล้ว',
    onClose = () => {},
  }: {
    open?: boolean;
    title?: string;
    message?: string;
    details?: string;
    buttonText?: string;
    onClose?: () => void;
  } = $props();

  let showDetails = $state(false);

  function handleClose() {
    showDetails = false;
    onClose();
  }
</script>

<Modal {open} {title} onClose={handleClose} maxWidth="460px">
  <div class="error-content">
    <div class="error-icon-wrapper">
      <div class="error-icon-circle">
        <svg
          width="28"
          height="28"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
      </div>
    </div>

    <div class="error-text-container">
      <p class="error-message">{message || 'เกิดข้อผิดพลาดในการดำเนินการ'}</p>

      {#if details && details !== message}
        <button
          type="button"
          class="details-toggle"
          onclick={() => (showDetails = !showDetails)}
        >
          <span>{showDetails ? 'ซ่อนรายละเอียดทางเทคนิค' : 'แสดงรายละเอียดทางเทคนิค'}</span>
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="toggle-icon {showDetails ? 'is-rotated' : ''}"
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>

        {#if showDetails}
          <div class="details-box">
            <code>{details}</code>
          </div>
        {/if}
      {/if}
    </div>

    <div class="error-actions">
      <button type="button" class="btn-danger-action" onclick={handleClose}>
        {buttonText}
      </button>
    </div>
  </div>
</Modal>

<style>
  .error-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-lg);
    text-align: center;
  }

  .error-icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .error-icon-circle {
    width: 60px;
    height: 60px;
    border-radius: 50%;
    background-color: #fde8e8;
    color: var(--color-danger);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 0 0 8px #fef2f2;
    animation: iconPop 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  }

  @keyframes iconPop {
    0% {
      transform: scale(0.5);
      opacity: 0;
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }

  .error-text-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
    width: 100%;
  }

  .error-message {
    font-size: 15px;
    line-height: 1.6;
    color: var(--color-text-primary);
    margin: 0;
    font-weight: 500;
  }

  .details-toggle {
    font-size: 12px;
    color: var(--color-text-primary);
    opacity: 0.6;
    background: transparent;
    border: none;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 4px;
    margin-top: 4px;
    transition: all 0.15s ease;
  }

  .details-toggle:hover {
    opacity: 0.9;
    background-color: var(--color-background);
  }

  .toggle-icon {
    transition: transform 0.2s ease;
  }

  .toggle-icon.is-rotated {
    transform: rotate(180deg);
  }

  .details-box {
    width: 100%;
    background-color: var(--color-background);
    border: 1px solid var(--color-muted);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 12px;
    color: var(--color-danger);
    text-align: left;
    word-break: break-all;
    max-height: 120px;
    overflow-y: auto;
  }

  .error-actions {
    display: flex;
    justify-content: center;
    width: 100%;
    margin-top: var(--space-sm);
  }

  .btn-danger-action {
    width: 100%;
    padding: 12px 24px;
    border-radius: var(--radius-md);
    background-color: var(--color-danger);
    color: var(--color-surface);
    font-weight: 500;
    font-size: 15px;
    transition: all 0.2s ease;
    cursor: pointer;
    border: none;
  }

  .btn-danger-action:hover {
    opacity: 0.9;
  }
</style>
