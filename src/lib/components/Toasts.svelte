<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import Icon from './Icon.svelte';
</script>

<div class="toasts" aria-live="polite">
  {#each app.toasts as toast (toast.id)}
    <div class="toast {toast.kind}">
      <Icon name={toast.kind === 'ok' ? 'check' : toast.kind === 'error' ? 'alert' : 'info'} size={16} />
      <span>{toast.text}</span>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 22px;
    bottom: 22px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 420px;
  }
  .toast {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: 14px;
    border: 1px solid var(--border-strong);
    background: var(--panel);
    backdrop-filter: blur(16px);
    box-shadow: var(--shadow);
    font-size: 13px;
    line-height: 1.4;
    white-space: pre-wrap;
    word-break: break-word;
    animation: slide 0.35s var(--ease) both;
  }
  .ok :global(.icon) {
    color: var(--ok);
  }
  .error :global(.icon) {
    color: var(--danger);
  }
  @keyframes slide {
    from {
      opacity: 0;
      transform: translateX(20px);
    }
  }
</style>
