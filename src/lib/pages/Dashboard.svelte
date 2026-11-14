<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { api, isTauri } from '$lib/api';
  import Icon from '$lib/components/Icon.svelte';
  import ConnectButton from '$lib/components/ConnectButton.svelte';
  import StrategySelect from '$lib/components/StrategySelect.svelte';

  let now = $state(Date.now());
  let showArgs = $state(false);
  let args = $state<string[]>([]);

  $effect(() => {
    const id = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(id);
  });

  const uptime = $derived.by(() => {
    if (!app.connected || !app.status.since) return '00:00:00';
    const s = Math.max(0, Math.floor((now - app.status.since) / 1000));
    const p = (n: number) => String(n).padStart(2, '0');
    return `${p(Math.floor(s / 3600))}:${p(Math.floor((s % 3600) / 60))}:${p(s % 60)}`;
  });

  const ap = $derived(app.autopick);
  const testingName = $derived(app.autopickRunning && ap?.name ? ap.name : null);
  const progress = $derived(ap && ap.total ? Math.round(((ap.phase === 'result' ? ap.index : ap.index - 1) / ap.total) * 100) : 0);

  const stateLabel = $derived(app.autopickRunning ? t('status.testing') : t(`status.${app.status.state}`));

  const gfLabel = $derived(t(`gf.${app.gameFilter.mode}`));

  function toggleArgs() {
    showArgs = !showArgs;
  }

  $effect(() => {
    // refresh preview when the strategy changes while open
    const file = app.config.strategy;
    const gf = app.gameFilter.mode;
    if (showArgs && isTauri()) api.previewArgs(file).then((a) => (args = a)).catch((e) => app.fail(e));
    void gf;
  });

  function copyArgs() {
    navigator.clipboard?.writeText(`winws.exe ${args.map((a) => (a.includes(' ') ? `"${a}"` : a)).join(' ')}`);
    app.toast(t('common.copied'), 'ok');
  }
</script>

<section class="page fade-in">
  <header class="head">
    <div>
      <span class="eyebrow">{t('dash.eyebrow')}</span>
      <h1 class="hero-title">{t('dash.title1')}<br />{t('dash.title2a')} <span class="dim">{t('dash.title2b')}</span></h1>
      <p class="hero-desc">{t('dash.desc')}</p>
    </div>
    <div class="deco card">
      <div class="deco-grid"></div>
      <div class="deco-ring"></div>
      <Icon name={app.connected ? 'shield-check' : 'shield'} size={40} stroke={1.6} />
    </div>
  </header>

  <div class="stats">
    <div class="stat card">
      <span class="si"><Icon name="grid" size={16} /></span>
      <div><strong>{app.strategies.length}</strong><span>{t('dash.strategies')}</span></div>
    </div>
    <div class="stat card">
      <span class="si"><Icon name="clock" size={16} /></span>
      <div><strong class="mono">{uptime}</strong><span>{t('dash.uptime')}</span></div>
    </div>
    <div class="stat card">
      <span class="si"><Icon name="gamepad" size={16} /></span>
      <div><strong>{gfLabel}</strong><span>{t('dash.gameFilter')}</span></div>
    </div>
    <div class="stat card">
      <span class="si"><Icon name="network" size={16} /></span>
      <div><strong>{app.ipsetMode}</strong><span>{t('dash.ipset')}</span></div>
    </div>
    <div class="stat card wide">
      <span class="line"></span>
      <span class="faint">
        {#if app.config.smartMode}{t('dash.smartOn')}{:else if app.config.hotkey}{t('dash.hotkeyHint', { key: app.config.hotkey })}{:else}zapret {app.zapretVersion}{/if}
      </span>
    </div>
  </div>

  <div class="main card">
    <div class="power">
      <ConnectButton
        state={app.status.state}
        testing={app.autopickRunning}
        disabled={app.autopickRunning || app.status.state === 'connecting'}
        onclick={() => app.toggle()}
      />
      <div class="state" class:ok={app.connected}>
        <span class="sd"></span>{stateLabel}
      </div>
      <div class="hint">
        {#if app.connected}
          {app.strategyName(app.status.strategy)} · {t('dash.tapDisconnect')}
        {:else if !app.autopickRunning}
          {t('dash.tapConnect')}
        {/if}
      </div>
      {#if app.status.error && !app.connected}
        <div class="err"><Icon name="alert" size={14} /> <span>{app.status.error}</span></div>
      {/if}
    </div>

    <div class="controls">
      <div class="lbl">{t('dash.strategy')}</div>
      <div class="row">
        <StrategySelect
          items={app.strategies}
          value={app.config.strategy}
          disabled={app.autopickRunning}
          testing={testingName}
          results={app.pickResults}
          best={app.bestStrategy}
          onchange={(f) => app.selectStrategy(f)}
        />
        {#if app.autopickRunning}
          <button class="btn danger pick" onclick={() => app.cancelAutopick()}>
            <Icon name="x" size={16} />{t('dash.autopickStop')}
          </button>
        {:else}
          <button class="btn primary pick" onclick={() => app.startAutopick()} disabled={app.status.state === 'connecting'}>
            <Icon name="wand" size={16} />{t('dash.autopick')}
          </button>
        {/if}
      </div>

      {#if ap && (app.autopickRunning || ap.phase === 'done' || ap.phase === 'failed' || ap.phase === 'cancelled')}
        <div class="ap fade-in">
          <div class="ap-top">
            {#if app.autopickRunning}
              <Icon name="loader" size={15} class="spin" />
              <span>{t('dash.autopickTesting', { name: ap.name ?? '', i: ap.index, n: ap.total })}</span>
            {:else if ap.phase === 'done'}
              <Icon name="sparkles" size={15} />
              <span>{t('dash.autopickFound', { name: ap.name ?? '' })}</span>
            {:else if ap.phase === 'failed'}
              <Icon name="alert" size={15} />
              <span>{t('dash.autopickFailed')}</span>
            {:else}
              <Icon name="info" size={15} />
              <span>{t('dash.autopickCancelled')}</span>
            {/if}
            {#if ap.phase === 'result'}
              <span class="faint res"
                >{t('dash.autopickResult', { name: ap.name ?? '', passed: ap.passed, checks: ap.checks })}{#if ap.latencyMs != null}
                  · {ap.latencyMs} {t('unit.ms')}{/if}</span
              >
            {/if}
          </div>
          <div class="bar"><span style:width="{ap.phase === 'done' ? 100 : progress}%" class:done={ap.phase === 'done'}></span></div>
        </div>
      {/if}

      {#if !app.autopickRunning}
        <div class="pick-hint faint">{t('dash.autopickHint')}</div>
      {/if}

      <button class="args-toggle" onclick={toggleArgs}>
        <Icon name="terminal" size={14} />{showArgs ? t('dash.hideArgs') : t('dash.showArgs')}
      </button>
      {#if showArgs}
        <div class="args fade-in">
          <button class="btn sm copy" onclick={copyArgs}><Icon name="copy" size={13} />{t('common.copy')}</button>
          {#each args as a, i (i)}
            <code class:newblk={a === '--new'}>{a}</code>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</section>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .head {
    display: flex;
    justify-content: space-between;
    gap: 24px;
  }
  .deco {
    position: relative;
    width: 210px;
    height: 120px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    overflow: hidden;
    align-self: center;
    color: var(--text-2);
  }
  .deco-grid {
    position: absolute;
    inset: 0;
    background-image:
      linear-gradient(var(--border) 1px, transparent 1px),
      linear-gradient(90deg, var(--border) 1px, transparent 1px);
    background-size: 14px 14px;
    mask-image: radial-gradient(circle at 60% 50%, #000, transparent 75%);
  }
  .deco-ring {
    position: absolute;
    width: 110px;
    height: 110px;
    border-radius: 50%;
    border: 1px solid var(--border-strong);
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(4, minmax(130px, auto)) 1fr;
    gap: 10px;
  }
  .stat {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border-radius: 18px;
  }
  .si {
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    border: 1px solid var(--border);
    color: var(--text-2);
  }
  .stat strong {
    display: block;
    font-size: 17px;
    font-weight: 750;
    text-transform: capitalize;
  }
  .stat strong.mono {
    font-family: var(--mono);
    font-size: 15px;
    text-transform: none;
  }
  .stat span {
    font-size: 10.5px;
    color: var(--text-3);
  }
  .wide {
    justify-content: flex-end;
    font-size: 11px;
  }
  .wide .line {
    width: 26px;
    height: 1px;
    background: var(--border-strong);
  }
  .main {
    display: grid;
    grid-template-columns: 340px 1fr;
    gap: 28px;
    padding: 26px 30px;
    align-items: center;
  }
  .power {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .state {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 20px;
    font-weight: 750;
    letter-spacing: -0.01em;
  }
  .sd {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-3);
  }
  .state.ok .sd {
    background: var(--ok);
    box-shadow: 0 0 10px var(--ok);
  }
  .hint {
    font-size: 12px;
    color: var(--text-3);
    min-height: 16px;
  }
  .err {
    max-width: 320px;
    display: flex;
    gap: 6px;
    margin-top: 6px;
    padding: 8px 10px;
    border-radius: 10px;
    background: var(--danger-soft);
    color: var(--danger);
    font-size: 11.5px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .controls {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }
  .lbl {
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--text-3);
  }
  .row {
    display: flex;
    gap: 10px;
  }
  .pick {
    height: 46px;
    border-radius: 14px;
    padding: 0 18px;
  }
  .ap {
    padding: 14px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .ap-top {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
  }
  .res {
    margin-left: auto;
    font-weight: 500;
    font-size: 11.5px;
  }
  .bar {
    margin-top: 10px;
    height: 5px;
    border-radius: 99px;
    background: var(--border);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--accent);
    border-radius: 99px;
    transition: width 0.5s var(--ease);
  }
  .bar span.done {
    background: var(--ok);
  }
  .pick-hint {
    font-size: 11.5px;
    margin-top: -4px;
  }
  .args-toggle {
    align-self: flex-start;
    display: inline-flex;
    gap: 6px;
    align-items: center;
    border: 0;
    background: none;
    color: var(--text-3);
    font-size: 12px;
    cursor: pointer;
    padding: 0;
  }
  .args-toggle:hover {
    color: var(--text);
  }
  .args {
    position: relative;
    max-height: 210px;
    overflow: auto;
    padding: 12px;
    padding-right: 90px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--field);
    display: flex;
    flex-wrap: wrap;
    gap: 4px 6px;
    user-select: text;
  }
  .args code {
    font-family: var(--mono);
    font-size: 10.5px;
    color: var(--text-2);
    word-break: break-all;
  }
  .args code.newblk {
    color: var(--accent);
    flex-basis: 100%;
  }
  .copy {
    position: absolute;
    right: 8px;
    top: 8px;
  }
  @media (max-width: 1180px) {
    .stats {
      grid-template-columns: repeat(4, 1fr);
    }
    .wide {
      display: none;
    }
    .main {
      grid-template-columns: 1fr;
      justify-items: stretch;
    }
  }
</style>
