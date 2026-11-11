<script lang="ts">
  import type { ConnState } from '$lib/api';
  import Icon from './Icon.svelte';

  interface Props {
    state: ConnState;
    testing?: boolean;
    disabled?: boolean;
    onclick?: () => void;
  }
  let { state, testing = false, disabled = false, onclick }: Props = $props();
  const busy = $derived(state === 'connecting' || testing);
</script>

<div class="wrap" class:connected={state === 'connected'} class:busy>
  <span class="ring r3"></span>
  <span class="ring r2"></span>
  <span class="ring r1"></span>
  {#if busy}<span class="orbit"></span>{/if}
  <button class="core" {disabled} {onclick} aria-label="power">
    <span class="inner">
      <Icon name="power" size={54} stroke={2.2} />
    </span>
  </button>
</div>

<style>
  .wrap {
    --c: var(--text-3);
    position: relative;
    width: 300px;
    height: 300px;
    display: grid;
    place-items: center;
  }
  .connected {
    --c: var(--ok);
  }
  .busy {
    --c: var(--warn);
  }
  .ring {
    position: absolute;
    border-radius: 50%;
    border: 1px solid var(--border);
    transition:
      border-color 0.5s,
      transform 0.6s var(--ease);
  }
  .r1 {
    inset: 46px;
  }
  .r2 {
    inset: 22px;
  }
  .r3 {
    inset: 0;
  }
  .connected .ring {
    border-color: color-mix(in srgb, var(--c) 28%, transparent);
    animation: pulse 3.2s ease-out infinite;
  }
  .connected .r2 {
    animation-delay: 0.5s;
  }
  .connected .r3 {
    animation-delay: 1s;
  }
  @keyframes pulse {
    0% {
      transform: scale(0.96);
      opacity: 1;
    }
    70% {
      transform: scale(1.04);
      opacity: 0.35;
    }
    100% {
      transform: scale(0.96);
      opacity: 1;
    }
  }
  .orbit {
    position: absolute;
    inset: 34px;
    border-radius: 50%;
    border: 2px solid transparent;
    border-top-color: var(--c);
    border-right-color: color-mix(in srgb, var(--c) 40%, transparent);
    animation: spin 1.1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .core {
    position: relative;
    width: 168px;
    height: 168px;
    border-radius: 50%;
    padding: 0;
    cursor: pointer;
    border: 1px solid var(--border-strong);
    background:
      radial-gradient(circle at 50% 30%, color-mix(in srgb, var(--text) 10%, transparent), transparent 60%),
      var(--card);
    color: var(--text-2);
    box-shadow:
      inset 0 1px 0 color-mix(in srgb, var(--text) 10%, transparent),
      0 30px 80px -30px rgba(0, 0, 0, 0.9);
    transition:
      transform 0.25s var(--ease),
      box-shadow 0.5s,
      color 0.4s,
      border-color 0.4s;
  }
  .core:hover:not(:disabled) {
    transform: scale(1.03);
    color: var(--text);
  }
  .core:active:not(:disabled) {
    transform: scale(0.97);
  }
  .core:disabled {
    cursor: progress;
  }
  .inner {
    display: grid;
    place-items: center;
    width: 100%;
    height: 100%;
  }
  .connected .core {
    color: var(--ok);
    border-color: color-mix(in srgb, var(--ok) 50%, transparent);
    box-shadow:
      inset 0 1px 0 color-mix(in srgb, var(--text) 10%, transparent),
      0 0 0 8px color-mix(in srgb, var(--ok) 8%, transparent),
      0 0 90px -10px color-mix(in srgb, var(--ok) 55%, transparent);
  }
  .busy .core {
    color: var(--warn);
  }
  .busy .inner {
    animation: breathe 1.4s ease-in-out infinite;
  }
  @keyframes breathe {
    50% {
      opacity: 0.45;
      transform: scale(0.94);
    }
  }
</style>
