<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import type { AppConfig } from '$lib/api';
  import { t } from '$lib/i18n/index.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import ThemePicker from '$lib/components/ThemePicker.svelte';
  import LanguagePicker from '$lib/components/LanguagePicker.svelte';

  type BoolKey = 'autostart' | 'autoConnect' | 'smartMode' | 'minimizeToTray' | 'sounds';
  const OPTIONS: { k: BoolKey; icon: string; title: string; sub: string }[] = [
    { k: 'autostart', icon: 'rocket', title: 'set.autostart', sub: 'set.autostartSub' },
    { k: 'autoConnect', icon: 'power', title: 'set.autoConnect', sub: 'set.autoConnectSub' },
    { k: 'smartMode', icon: 'radar', title: 'set.smart', sub: 'set.smartSub' },
    { k: 'minimizeToTray', icon: 'monitor', title: 'set.tray', sub: 'set.traySub' },
    { k: 'sounds', icon: 'volume', title: 'set.sounds', sub: 'set.soundsSub' }
  ];

  let step = $state(0);
  const steps = ['onb.step1', 'onb.step2', 'onb.step3'];

  async function finish() {
    await app.patchConfig({ onboarded: true });
    app.showOnboarding = false;
    app.page = 'dashboard';
  }
</script>

<div class="overlay fade-in">
  <div class="panel card">
    <div class="top">
      <Logo size={56} glow />
      <h1>{t('onb.welcome')}</h1>
      <p>{t('onb.sub')}</p>
      <div class="steps">
        {#each steps as s, i (s)}
          <button class="stp" class:active={i === step} class:done={i < step} onclick={() => (step = i)}>
            <span class="n">{#if i < step}<Icon name="check" size={12} stroke={3} />{:else}{i + 1}{/if}</span>
            {t(s)}
          </button>
          {#if i < steps.length - 1}<span class="sep"></span>{/if}
        {/each}
      </div>
    </div>

    <div class="body">
      {#if step === 0}
        <div class="fade-in"><LanguagePicker value={app.config.language} onchange={(c) => app.patchConfig({ language: c })} /></div>
      {:else if step === 1}
        <div class="fade-in"><ThemePicker value={app.config.theme} onchange={(id) => app.patchConfig({ theme: id })} /></div>
      {:else}
        <div class="opts fade-in">
          {#each OPTIONS as o (o.k)}
            <div class="opt">
              <span class="oi"><Icon name={o.icon} size={16} /></span>
              <div class="ot"><b>{t(o.title)}</b><span>{t(o.sub)}</span></div>
              <Toggle checked={app.config[o.k]} onchange={(v) => app.patchConfig({ [o.k]: v } as Partial<AppConfig>)} />
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="nav">
      <button class="btn" onclick={() => (step = Math.max(0, step - 1))} disabled={step === 0}>
        <Icon name="arrow-left" size={15} />{t('common.back')}
      </button>
      {#if step < steps.length - 1}
        <button class="btn primary" onclick={() => step++}>{t('common.next')}<Icon name="arrow-right" size={15} /></button>
      {:else}
        <button class="btn primary" onclick={finish}><Icon name="zap" size={15} />{t('onb.start')}</button>
      {/if}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    padding: 30px;
    background: color-mix(in srgb, var(--bg) 70%, transparent);
    backdrop-filter: blur(6px);
  }
  .panel {
    width: min(920px, 100%);
    max-height: calc(100vh - 60px);
    display: flex;
    flex-direction: column;
    padding: 30px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }
  .top {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }
  h1 {
    margin: 16px 0 6px;
    font-size: 36px;
    letter-spacing: -0.03em;
  }
  .top p {
    margin: 0;
    color: var(--text-2);
  }
  .steps {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 22px 0 6px;
  }
  .stp {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 0;
    background: none;
    color: var(--text-3);
    font-weight: 600;
    cursor: pointer;
  }
  .stp.active,
  .stp.done {
    color: var(--text);
  }
  .n {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    border: 1px solid var(--border-strong);
    font-size: 12px;
  }
  .stp.active .n {
    background: var(--accent);
    color: var(--accent-text);
    border-color: transparent;
  }
  .sep {
    width: 36px;
    height: 1px;
    background: var(--border-strong);
  }
  .body {
    flex: 1;
    overflow-y: auto;
    padding: 20px 2px;
    min-height: 220px;
  }
  .opts {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .opt {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .oi {
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    border: 1px solid var(--border);
  }
  .ot {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .ot b {
    font-size: 13.5px;
  }
  .ot span {
    font-size: 11.5px;
    color: var(--text-3);
  }
  .nav {
    display: flex;
    justify-content: space-between;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }
</style>
