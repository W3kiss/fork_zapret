<script lang="ts">
  interface Props {
    checked: boolean;
    disabled?: boolean;
    onchange?: (v: boolean) => void;
    label?: string;
  }
  let { checked, disabled = false, onchange, label }: Props = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  class="toggle"
  class:on={checked}
  {disabled}
  onclick={() => onchange?.(!checked)}
>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    position: relative;
    width: 44px;
    height: 26px;
    flex-shrink: 0;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    background: var(--field);
    cursor: pointer;
    transition:
      background 0.25s,
      border-color 0.25s;
    padding: 0;
  }
  .toggle:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--text-2);
    transition:
      transform 0.3s var(--ease),
      background 0.25s;
  }
  .on {
    background: var(--accent);
    border-color: transparent;
  }
  .on .knob {
    transform: translateX(18px);
    background: var(--accent-text);
  }
</style>
