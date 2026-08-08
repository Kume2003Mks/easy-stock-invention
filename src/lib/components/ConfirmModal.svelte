<script lang="ts">
  import Modal from './Modal.svelte';

  let {
    open = false,
    title = 'ยืนยันการดำเนินการ',
    message = '',
    confirmText = 'ยืนยัน',
    cancelText = 'ยกเลิก',
    variant = 'danger',
    closeOnOutsideClick = true,
    outsideClick = true,
    onConfirm = () => {},
    onCancel = () => {},
  }: {
    open?: boolean;
    title?: string;
    message?: string;
    confirmText?: string;
    cancelText?: string;
    variant?: 'danger' | 'primary' | 'default';
    closeOnOutsideClick?: boolean;
    outsideClick?: boolean;
    onConfirm?: () => void;
    onCancel?: () => void;
  } = $props();

  function handleConfirm() {
    onConfirm();
  }
</script>

<Modal {open} {title} onClose={onCancel} maxWidth="420px" closeOnOutsideClick={closeOnOutsideClick || outsideClick}>
  <div class="confirm-content">
    <p class="confirm-message">{message}</p>
    <div class="confirm-actions">
      <button type="button" class="btn-outline" onclick={onCancel}>
        {cancelText}
      </button>
      <button
        type="button"
        class="confirm-btn {variant === 'danger'
          ? 'btn-danger'
          : variant === 'primary'
            ? 'btn-primary'
            : 'btn-outline'}"
        onclick={handleConfirm}
      >
        {confirmText}
      </button>
    </div>
  </div>
</Modal>

<style>
  .confirm-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .confirm-message {
    font-size: 15px;
    line-height: 1.6;
    color: var(--color-text-primary);
    margin: 0;
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
  }

  .confirm-btn {
    padding: 10px 22px;
    border-radius: var(--radius-md);
    font-weight: 500;
    font-size: 14px;
    transition: opacity 0.2s;
  }

  .btn-danger {
    background-color: var(--color-danger);
    color: var(--color-surface);
  }

  .btn-danger:hover {
    opacity: 0.9;
  }
</style>