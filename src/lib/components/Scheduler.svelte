<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import type { Schedule } from '$lib/api';
  import Icon from './Icon.svelte';
  import Toggle from './Toggle.svelte';

  // Store weekdays as 0=Sun..6=Sat; display Mon-first.
  const DAYS = [
    { v: 1, k: 'day.mon' },
    { v: 2, k: 'day.tue' },
    { v: 3, k: 'day.wed' },
    { v: 4, k: 'day.thu' },
    { v: 5, k: 'day.fri' },
    { v: 6, k: 'day.sat' },
    { v: 0, k: 'day.sun' }
  ];

  let time = $state('08:00');
  let action = $state<'connect' | 'disconnect'>('connect');
  let days = $state<number[]>([]);

  function toggleDay(v: number) {
    days = days.includes(v) ? days.filter((d) => d !== v) : [...days, v];
  }

  async function add() {
    if (!/^\d{2}:\d{2}$/.test(time)) return;
    const rule: Schedule = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
      enabled: true,
      action,
      time,
      days: [...days].sort((a, b) => a - b)
    };
    await app.patchConfig({ schedules: [...app.config.schedules, rule] });
    days = [];
  }

  async function toggle(id: string, enabled: boolean) {
    await app.patchConfig({ schedules: app.config.schedules.map((s) => (s.id === id ? { ...s, enabled } : s)) });
  }

  async function remove(id: string) {
    await app.patchConfig({ schedules: app.config.schedules.filter((s) => s.id !== id) });
  }

  function daysLabel(s: Schedule): string {
    if (!s.days.length) return t('sched.everyDay');
    return DAYS.filter((d) => s.days.includes(d.v)).map((d) => t(d.k)).join(', ');
  }
</script>

<div class="card box full">
  <div class="section-title">
    <span class="ico"><Icon name="calendar" /></span>
    <div><h3>{t('sched.title')}</h3><p>{t('sched.sub')}</p></div>
  </div>

  <div class="builder">
    <input class="field time" type="time" bind:value={time} />
    <div class="acts">
      <button class:active={action === 'connect'} onclick={() => (action = 'connect')}>{t('sched.connect')}</button>
      <button class:active={action === 'disconnect'} onclick={() => (action = 'disconnect')}>{t('sched.disconnect')}</button>
    </div>
    <div class="days">
      {#each DAYS as d (d.v)}
        <button class="day" class:on={days.includes(d.v)} onclick={() => toggleDay(d.v)}>{t(d.k)}</button>
      {/each}
    </div>
    <button class="btn primary" onclick={add}><Icon name="plus" size={15} />{t('common.add')}</button>
  </div>
  <p class="hint faint">{t('sched.hint')}</p>

  {#if app.config.schedules.length}
    <div class="list">
      {#each app.config.schedules as s (s.id)}
        <div class="row" class:off={!s.enabled}>
          <span class="time">{s.time}</span>
          <span class="badge {s.action}">{t(`sched.${s.action}`)}</span>
          <span class="days-lbl">{daysLabel(s)}</span>
          <span class="push"></span>
          <Toggle checked={s.enabled} onchange={(v) => toggle(s.id, v)} />
          <button class="btn sm icon-only" onclick={() => remove(s.id)} aria-label="delete"><Icon name="trash" size={13} /></button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty faint">{t('sched.empty')}</p>
  {/if}
</div>

<style>
  .full {
    grid-column: 1 / -1;
  }
  .box {
    padding: 20px;
  }
  .builder {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }
  .time {
    width: 110px;
    font-family: var(--mono);
  }
  .acts {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .acts button {
    padding: 6px 12px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-size: 12.5px;
    font-weight: 600;
  }
  .acts button.active {
    background: var(--accent);
    color: var(--accent-text);
  }
  .days {
    display: flex;
    gap: 4px;
  }
  .day {
    width: 34px;
    height: 34px;
    border-radius: 9px;
    border: 1px solid var(--border);
    background: var(--field);
    color: var(--text-2);
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
  }
  .day.on {
    background: var(--accent);
    color: var(--accent-text);
    border-color: transparent;
  }
  .hint {
    margin: 10px 0 0;
    font-size: 11.5px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 12px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 8px 8px 12px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .row.off {
    opacity: 0.5;
  }
  .time {
    font-family: var(--mono);
    font-weight: 700;
    font-size: 14px;
  }
  .badge {
    font-size: 11px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
  }
  .badge.connect {
    color: var(--ok);
  }
  .badge.disconnect {
    color: var(--text-2);
  }
  .days-lbl {
    font-size: 11.5px;
    color: var(--text-3);
  }
  .push {
    flex: 1;
  }
  .empty {
    padding: 8px 0;
    font-size: 12.5px;
  }
</style>
