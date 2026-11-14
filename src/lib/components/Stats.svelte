<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { i18n, t } from '$lib/i18n/index.svelte';
  import { api, isTauri } from '$lib/api';
  import Icon from './Icon.svelte';

  const s = $derived(app.stats);

  function dur(sec: number): string {
    if (sec < 60) return `${sec} ${t('unit.sec')}`;
    const m = Math.floor(sec / 60);
    if (m < 60) return `${m} ${t('unit.min')}`;
    const h = Math.floor(m / 60);
    const rm = m % 60;
    if (h < 24) return `${h} ${t('unit.hour')} ${rm} ${t('unit.min')}`;
    const d = Math.floor(h / 24);
    return `${d} ${t('unit.day')} ${h % 24} ${t('unit.hour')}`;
  }

  const topStrategies = $derived.by(() => {
    const entries = Object.entries(s.perStrategy).sort((a, b) => b[1] - a[1]).slice(0, 5);
    const max = entries.length ? entries[0][1] : 1;
    return entries.map(([file, sec]) => ({ name: app.strategyName(file), sec, pct: max ? (sec / max) * 100 : 0 }));
  });

  const recent = $derived.by(() => {
    const r = s.recent.slice(-14);
    const max = r.reduce((m, x) => Math.max(m, x.seconds), 1);
    return r.map((x) => ({ ...x, h: Math.max(6, (x.seconds / max) * 100) }));
  });

  const lastConnected = $derived(s.lastConnected ? new Date(s.lastConnected).toLocaleString(i18n.lang) : '—');

  async function reset() {
    if (!isTauri()) return;
    try {
      await api.resetStats();
      await app.refresh();
    } catch (e) {
      app.fail(e);
    }
  }
</script>

<div class="card box full">
  <div class="section-title">
    <span class="ico"><Icon name="chart" /></span>
    <div><h3>{t('stats.title')}</h3><p>{t('stats.sub')}</p></div>
    <span class="push"><button class="btn sm" onclick={reset}><Icon name="trash" size={13} />{t('stats.reset')}</button></span>
  </div>

  <div class="tiles">
    <div class="tile"><b>{s.sessions}</b><span>{t('stats.sessions')}</span></div>
    <div class="tile"><b>{dur(s.totalSeconds)}</b><span>{t('stats.uptime')}</span></div>
    <div class="tile"><b class:bad={s.crashes > 0}>{s.crashes}</b><span>{t('stats.crashes')}</span></div>
    <div class="tile"><b class="small">{lastConnected}</b><span>{t('stats.lastConnected')}</span></div>
  </div>

  {#if topStrategies.length}
    <div class="block">
      <div class="lbl">{t('stats.byStrategy')}</div>
      {#each topStrategies as row (row.name)}
        <div class="bar-row">
          <span class="bname">{row.name}</span>
          <div class="track"><span style:width="{row.pct}%"></span></div>
          <span class="bval">{dur(row.sec)}</span>
        </div>
      {/each}
    </div>
  {/if}

  {#if recent.length}
    <div class="block">
      <div class="lbl">{t('stats.recent')}</div>
      <div class="spark">
        {#each recent as r (r.start)}
          <span class="col" class:crash={r.crashed} style:height="{r.h}%" title="{app.strategyName(r.strategy)} · {dur(r.seconds)}"></span>
        {/each}
      </div>
    </div>
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
  .tiles {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 10px;
  }
  .tile {
    padding: 14px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: var(--field);
    text-align: center;
  }
  .tile b {
    display: block;
    font-size: 20px;
    font-weight: 750;
  }
  .tile b.small {
    font-size: 12px;
    font-weight: 600;
  }
  .tile b.bad {
    color: var(--danger);
  }
  .tile span {
    font-size: 10.5px;
    color: var(--text-3);
  }
  .block {
    margin-top: 18px;
  }
  .lbl {
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-3);
    margin-bottom: 10px;
  }
  .bar-row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 6px;
  }
  .bname {
    width: 110px;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .track {
    flex: 1;
    height: 8px;
    border-radius: 99px;
    background: var(--border);
    overflow: hidden;
  }
  .track span {
    display: block;
    height: 100%;
    background: var(--accent);
    border-radius: 99px;
  }
  .bval {
    width: 100px;
    text-align: right;
    font-size: 11px;
    color: var(--text-3);
  }
  .spark {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    height: 70px;
    padding: 6px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .col {
    flex: 1;
    background: var(--accent);
    border-radius: 3px 3px 0 0;
    min-height: 6px;
    opacity: 0.85;
  }
  .col.crash {
    background: var(--danger);
  }
</style>
