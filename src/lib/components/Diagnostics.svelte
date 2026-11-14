<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { api, isTauri, type Check } from '$lib/api';
  import Icon from './Icon.svelte';

  let checks = $state<Check[]>([]);
  let running = $state(false);
  let done = $state(false);

  async function run() {
    if (!isTauri()) return;
    running = true;
    try {
      checks = await api.runDiagnostics();
      done = true;
    } catch (e) {
      app.fail(e);
    } finally {
      running = false;
    }
  }

  const icon = (s: Check['status']) => (s === 'ok' ? 'check' : s === 'warn' ? 'alert' : 'x');
  const fails = $derived(checks.filter((c) => c.status === 'fail').length);
  const warns = $derived(checks.filter((c) => c.status === 'warn').length);
</script>

<div class="card box full">
  <div class="section-title">
    <span class="ico"><Icon name="activity" /></span>
    <div><h3>{t('diag.title')}</h3><p>{t('diag.sub')}</p></div>
    <span class="push">
      <button class="btn sm" onclick={run} disabled={running}>
        <Icon name={running ? 'loader' : 'activity'} size={14} class={running ? 'spin' : ''} />{t('diag.run')}
      </button>
    </span>
  </div>

  {#if done}
    <div class="summary" class:ok={fails === 0 && warns === 0} class:warn={fails === 0 && warns > 0} class:fail={fails > 0}>
      {#if fails > 0}{t('diag.hasFails', { n: fails })}{:else if warns > 0}{t('diag.hasWarns', { n: warns })}{:else}{t('diag.allOk')}{/if}
    </div>
    <div class="list">
      {#each checks as c (c.name)}
        <div class="row {c.status}">
          <span class="dot"><Icon name={icon(c.status)} size={13} stroke={3} /></span>
          <span class="detail">{c.detail}</span>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty faint">{t('diag.idle')}</p>
  {/if}
</div>

<style>
  .full {
    grid-column: 1 / -1;
  }
  .box {
    padding: 20px;
  }
  .push {
    margin-left: auto;
  }
  .summary {
    padding: 10px 12px;
    border-radius: 12px;
    font-size: 12.5px;
    font-weight: 600;
    margin-bottom: 10px;
  }
  .summary.ok {
    background: var(--ok-soft);
    color: var(--ok);
  }
  .summary.warn {
    background: color-mix(in srgb, var(--warn) 14%, transparent);
    color: var(--warn);
  }
  .summary.fail {
    background: var(--danger-soft);
    color: var(--danger);
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 12px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .dot {
    width: 22px;
    height: 22px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .row.ok .dot {
    background: var(--ok-soft);
    color: var(--ok);
  }
  .row.warn .dot {
    background: color-mix(in srgb, var(--warn) 16%, transparent);
    color: var(--warn);
  }
  .row.fail .dot {
    background: var(--danger-soft);
    color: var(--danger);
  }
  .detail {
    font-size: 12.5px;
  }
  .empty {
    padding: 8px 0;
    font-size: 12.5px;
  }
</style>
