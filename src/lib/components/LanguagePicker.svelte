<script lang="ts">
  import { LANGUAGES } from '$lib/i18n/index.svelte';

  interface Props {
    value: string;
    onchange?: (code: string) => void;
  }
  let { value, onchange }: Props = $props();
</script>

<div class="langs">
  {#each LANGUAGES as l (l.code)}
    <button class:active={value === l.code} onclick={() => onchange?.(l.code)}>
      <span class="code">{l.short}</span>
      <span class="label">{l.label}</span>
    </button>
  {/each}
</div>

<style>
  .langs {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 8px;
  }
  button {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 46px;
    padding: 0 12px;
    border-radius: 13px;
    border: 1px solid var(--border);
    background: var(--field);
    cursor: pointer;
    transition: border-color 0.2s;
  }
  button:hover {
    border-color: var(--border-strong);
  }
  .active {
    border-color: var(--text);
    background: var(--card-hover);
  }
  .code {
    font-size: 10px;
    font-weight: 800;
    padding: 3px 6px;
    border-radius: 6px;
    background: var(--card-hover);
    border: 1px solid var(--border-strong);
    color: var(--text-2);
  }
  .label {
    font-weight: 650;
    font-size: 14px;
  }
</style>
