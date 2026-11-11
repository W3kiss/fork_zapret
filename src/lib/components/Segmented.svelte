<script lang="ts">
  interface Option {
    value: string;
    label: string;
    sub?: string;
  }
  interface Props {
    options: Option[];
    value: string;
    disabled?: boolean;
    onchange?: (v: string) => void;
  }
  let { options, value, disabled = false, onchange }: Props = $props();
</script>

<div class="seg" class:disabled role="radiogroup">
  {#each options as o (o.value)}
    <button
      type="button"
      role="radio"
      aria-checked={o.value === value}
      class:active={o.value === value}
      {disabled}
      onclick={() => o.value !== value && onchange?.(o.value)}
    >
      <span class="l">{o.label}</span>
      {#if o.sub}<span class="s">{o.sub}</span>{/if}
    </button>
  {/each}
</div>

<style>
  .seg {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .seg.disabled {
    opacity: 0.5;
  }
  button {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    padding: 8px 10px;
    border: 1px solid transparent;
    border-radius: 10px;
    background: transparent;
    cursor: pointer;
    transition:
      background 0.2s,
      color 0.2s;
    color: var(--text-2);
  }
  button:hover:not(.active):not(:disabled) {
    color: var(--text);
    background: var(--card-hover);
  }
  .active {
    background: var(--accent);
    color: var(--accent-text);
  }
  .l {
    font-weight: 650;
    font-size: 13px;
  }
  .s {
    font-size: 10.5px;
    opacity: 0.75;
  }
</style>
