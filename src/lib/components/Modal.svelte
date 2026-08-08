<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    open = false,
    title = '',
    onClose = () => {},
    maxWidth = '560px',
    closeOnOutsideClick = false,
    outsideClick = false,
    children,
  }: {
    open?: boolean;
    title?: string;
    onClose?: () => void;
    maxWidth?: string;
    closeOnOutsideClick?: boolean;
    outsideClick?: boolean;
    children?: Snippet;
  } = $props();

  let modalRef = $state<HTMLDivElement | null>(null);

  const allowOutsideClose = $derived(closeOnOutsideClick || outsideClick);

  // Handle Escape key to close
  $effect(() => {
    if (!open) return;

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });

  // Lock body scroll when modal is open
  $effect(() => {
    if (open) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  });

  // Focus the modal when opened
  $effect(() => {
    if (open && modalRef) {
      modalRef.focus();
    }
  });
</script>

{#if open}
  <div
    class="modal-overlay"
    role="presentation"
    onclick={(e) => {
      if (allowOutsideClose && e.target === e.currentTarget) {
        onClose();
      }
    }}
  >
    <div
      bind:this={modalRef}
      class="modal-panel"
      style={`--modal-max-width: ${maxWidth}`}
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex="-1"
    >
      <header class="modal-header">
        <h2 class="modal-title">{title}</h2>
        <button
          type="button"
          class="modal-close"
          aria-label="ปิด"
          onclick={onClose}
        >
          <svg
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
            <path d="M18 6 6 18" />
            <path d="m6 6 12 12" />
          </svg>
        </button>
      </header>
      <div class="modal-body">
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 2000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-lg);
    background-color: rgba(46, 52, 64, 0.5);
    backdrop-filter: blur(2px);
    animation: overlayFadeIn 0.18s ease forwards;
  }

  .modal-panel {
    width: 100%;
    max-width: var(--modal-max-width);
    max-height: calc(100vh - 64px);
    display: flex;
    flex-direction: column;
    background-color: var(--color-surface);
    border-radius: var(--radius-lg);
    box-shadow: 0 24px 64px rgba(46, 52, 64, 0.2),
                0 8px 24px rgba(46, 52, 64, 0.1);
    outline: none;
    animation: modalSlideIn 0.2s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    padding: var(--space-lg) var(--space-xl);
    border-bottom: var(--border-subtle);
    flex-shrink: 0;
  }

  .modal-title {
    font-size: 20px;
    font-weight: 600;
    margin: 0;
  }

  .modal-close {
    width: 36px;
    height: 36px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    background-color: transparent;
    color: var(--color-text-primary);
    opacity: 0.6;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
    flex-shrink: 0;
  }

  .modal-close:hover {
    background-color: var(--color-background);
    color: var(--color-text-primary);
    opacity: 1;
    border-color: var(--color-muted);
  }

  .modal-body {
    padding: var(--space-xl);
    overflow-y: auto;
    flex: 1;
  }

  @keyframes overlayFadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes modalSlideIn {
    from {
      opacity: 0;
      transform: translateY(16px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>