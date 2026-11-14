<script lang="ts">
  import type { StrategyInfo, StrategyResult } from '$lib/api';
  import { t } from '$lib/i18n/index.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    items: StrategyInfo[];
    value: string;
    disabled?: boolean;
    testing?: string | null;
    results?: Record<string, StrategyResult>;
    best?: string | null;
    onchange?: (file: string) => void;
  }
  let { items, value, disabled = false, testing = null, results = {}, best = null, onchange }: Props = $props();

  let open = $state(false);
  let root: HTMLDivElement;
  const current = $derived(items.find((i) => i.file === value));
  const currentResult = $derived(results[value]);

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

  /** good = all targets reachable, mid = at least half, bad = less. */
  function quality(r: StrategyResult | undefined): 'good' | 'mid' | 'bad' | 'none' {
    if (!r) return 'none';
    if (r.passed === r.checks && r.checks > 0) return 'good';
    if (r.passed * 2 >= r.checks && r.passed > 0) return 'mid';
    return 'bad';
  }

  function ping(r: StrategyResult | undefined): string {
    if (!r) return '—';
    if (r.latencyMs == null) return t('dash.noPing');
    return `${r.latencyMs} ${t('unit.ms')}`;
  }
</script>

<div class="select" bind:this={root}>
  <button class="trigger" class:open {disabled} onclick={() => (open = !open)}>
    <Icon name="zap" size={16} />
    <span class="v">{testing ?? current?.name ?? value}</span>
    {#if !testing && currentResult}
      <span class="badge {quality(currentResult)}">{ping(currentResult)}</span>
    {/if}
    <span class="file">{current?.file ?? ''}</span>
    <Icon name="chevron-down" size={16} class="chev" />
  </button>
  {#if open}
    <div class="menu fade-in" role="listbox">
      {#each items as it (it.file)}
        {@const r = results[it.file]}
        <button role="option" aria-selected={it.file === value} class:sel={it.file === value} onclick={() => pick(it.file)}>
          <span class="n">
            {it.name}
            {#if best === it.file}<span class="best"><Icon name="star" size={11} />{t('dash.best')}</span>{/if}
          </span>
          {#if r}
            <span class="score {quality(r)}">{r.passed}/{r.checks}</span>
            <span class="badge {quality(r)}">{ping(r)}</span>
          {:else}
            <span class="untested">{t('dash.notTested')}</span>
          {/if}
          <span class="mark">{#if it.file === value}<Icon name="check" size={15} />{/if}</span>
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
    max-height: 380px;
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
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 650;
    min-width: 0;
  }
  .best {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px 7px;
    border-radius: 999px;
    background: var(--ok-soft);
    color: var(--ok);
    font-size: 10px;
    font-weight: 700;
  }
  .score {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-3);
    min-width: 36px;
    text-align: right;
  }
  .badge {
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 999px;
    min-width: 64px;
    text-align: center;
    border: 1px solid var(--border);
  }
  .good {
    color: var(--ok);
  }
  .badge.good {
    background: var(--ok-soft);
    border-color: transparent;
  }
  .mid {
    color: var(--warn);
  }
  .badge.mid {
    background: color-mix(in srgb, var(--warn) 14%, transparent);
    border-color: transparent;
  }
  .bad {
    color: var(--danger);
  }
  .badge.bad {
    background: var(--danger-soft);
    border-color: transparent;
  }
  .untested {
    font-size: 11px;
    color: var(--text-3);
  }
  .mark {
    width: 15px;
    display: grid;
    place-items: center;
  }
</style>
