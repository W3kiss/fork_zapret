<script lang="ts">
  import type { StrategyInfo } from '$lib/api';
  import Icon from './Icon.svelte';

  interface Props {
    items: StrategyInfo[];
    value: string;
    disabled?: boolean;
    testing?: string | null;
    onchange?: (file: string) => void;
  }
  let { items, value, disabled = false, testing = null, onchange }: Props = $props();

  let open = $state(false);
  let root: HTMLDivElement;
  const current = $derived(items.find((i) => i.file === value));

  $effect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => {
      if (!root.contains(e.target as Node)) open = false;
    };
    const esc = (e: KeyboardEvent) => e.key === 'Escape' && (open = false);
    window.addEventListener('mousedown', close);
    window.addEventListener('keydown', esc);
    return () => {
      window.removeEventListener('mousedown', close);
      window.removeEventListener('keydown', esc);
    };
  });

  function pick(file: string) {
    open = false;
    if (file !== value) onchange?.(file);
  }
</script>

<div class="select" bind:this={root}>
  <button class="trigger" class:open {disabled} onclick={() => (open = !open)}>
    <Icon name="zap" size={16} />
    <span class="v">{testing ?? current?.name ?? value}</span>
    <span class="file">{current?.file ?? ''}</span>
    <Icon name="chevron-down" size={16} class="chev" />
  </button>
  {#if open}
    <div class="menu fade-in" role="listbox">
      {#each items as it (it.file)}
        <button role="option" aria-selected={it.file === value} class:sel={it.file === value} onclick={() => pick(it.file)}>
          <span class="n">{it.name}</span>
          <span class="f">{it.file}</span>
          {#if it.file === value}<Icon name="check" size={15} />{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .select {
    position: relative;
    flex: 1;
    min-width: 0;
  }
  .trigger {
    width: 100%;
    height: 46px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 14px;
    border-radius: 14px;
    border: 1px solid var(--border-strong);
    background: var(--field);
    cursor: pointer;
    text-align: left;
  }
  .trigger:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .trigger.open {
    border-color: var(--text-3);
  }
  .v {
    font-weight: 700;
    font-size: 14px;
  }
  .file {
    flex: 1;
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .trigger :global(.chev) {
    color: var(--text-2);
    transition: transform 0.25s var(--ease);
  }
  .trigger.open :global(.chev) {
    transform: rotate(180deg);
  }
  .menu {
    position: absolute;
    z-index: 30;
    left: 0;
    right: 0;
    bottom: calc(100% + 8px);
    max-height: 340px;
    overflow-y: auto;
    padding: 6px;
    border-radius: 16px;
    border: 1px solid var(--border-strong);
    background: var(--panel);
    backdrop-filter: blur(20px);
    box-shadow: var(--shadow);
  }
  .menu button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 12px;
    border: 0;
    border-radius: 10px;
    background: transparent;
    cursor: pointer;
    text-align: left;
  }
  .menu button:hover {
    background: var(--field);
  }
  .menu .sel {
    background: var(--accent-soft);
  }
  .n {
    font-weight: 650;
    min-width: 120px;
  }
  .f {
    flex: 1;
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-3);
  }
</style>
