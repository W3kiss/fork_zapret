<script lang="ts">
  import { THEMES } from '$lib/themes';
  import { t } from '$lib/i18n/index.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    value: string;
    onchange?: (id: string) => void;
  }
  let { value, onchange }: Props = $props();
</script>

<div class="grid">
  {#each THEMES as th (th.id)}
    <button class="theme" class:active={value === th.id} onclick={() => onchange?.(th.id)}>
      <div class="preview" style:background={th.bg} style:border-color={th.light ? '#e2e8f0' : 'rgba(255,255,255,.08)'}>
        {#if th.stars}
          <span class="st" style="left:62%;top:18%"></span>
          <span class="st" style="left:44%;top:72%"></span>
          <span class="st" style="left:88%;top:58%"></span>
          <span class="st" style="left:30%;top:30%"></span>
        {/if}
        <span class="side" style:background={th.panel}></span>
        <span class="top" style:background={th.panel}></span>
        <span class="box" style:background={th.panel}></span>
        <span class="pill" style:background={th.accent}></span>
        {#if value === th.id}
          <span class="tick"><Icon name="check" size={12} stroke={3} /></span>
        {/if}
      </div>
      <div class="name">{t(`theme.${th.id}`)}</div>
      <div class="sub">{t(`theme.${th.id}Sub`)}</div>
    </button>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(168px, 1fr));
    gap: 10px;
  }
  .theme {
    padding: 8px 8px 12px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: var(--field);
    cursor: pointer;
    text-align: left;
    transition:
      border-color 0.2s,
      transform 0.2s var(--ease);
  }
  .theme:hover {
    transform: translateY(-2px);
    border-color: var(--border-strong);
  }
  .theme.active {
    border-color: var(--text);
    box-shadow: 0 0 24px -8px color-mix(in srgb, var(--text) 40%, transparent);
  }
  .preview {
    position: relative;
    height: 82px;
    border-radius: 11px;
    border: 1px solid;
    overflow: hidden;
  }
  .preview span {
    position: absolute;
    border-radius: 6px;
    border: 1px solid rgba(127, 127, 127, 0.18);
  }
  .side {
    left: 8px;
    top: 8px;
    bottom: 8px;
    width: 22%;
  }
  .top {
    left: 32%;
    right: 8px;
    top: 8px;
    height: 26%;
  }
  .box {
    left: 32%;
    width: 26%;
    bottom: 8px;
    height: 40%;
  }
  .pill {
    right: 10px;
    top: 48%;
    width: 30%;
    height: 7px;
    border-radius: 99px !important;
    border: 0 !important;
  }
  .st {
    width: 2px;
    height: 2px;
    border-radius: 50% !important;
    border: 0 !important;
    background: #fff;
    z-index: 2;
  }
  .tick {
    right: 6px !important;
    top: 6px !important;
    left: auto;
    width: 20px;
    height: 20px;
    display: grid;
    place-items: center;
    border-radius: 50% !important;
    background: #fff;
    color: #000;
    border: 0 !important;
    z-index: 3;
  }
  .name {
    margin: 10px 4px 0;
    font-weight: 700;
    font-size: 13px;
  }
  .sub {
    margin: 2px 4px 0;
    font-size: 11px;
    color: var(--text-3);
  }
</style>
