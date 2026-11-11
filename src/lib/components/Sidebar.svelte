<script lang="ts">
  import { app, type Page } from '$lib/stores/app.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import Icon from './Icon.svelte';
  import Logo from './Logo.svelte';

  const items: { id: Page; icon: string; title: string; sub: string }[] = [
    { id: 'dashboard', icon: 'power', title: 'nav.dashboard', sub: 'nav.dashboardSub' },
    { id: 'lists', icon: 'list', title: 'nav.lists', sub: 'nav.listsSub' },
    { id: 'settings', icon: 'settings', title: 'nav.settings', sub: 'nav.settingsSub' },
    { id: 'about', icon: 'user', title: 'nav.about', sub: 'nav.aboutSub' }
  ];
</script>

<aside class="sidebar">
  <div class="brand">
    <Logo size={46} glow={app.connected} />
    <div class="brand-text">
      <strong>Fork Zapret</strong>
      <span>{t('brand.tag')}</span>
    </div>
    <span class="dot" class:on={app.connected} class:busy={app.busy} title={t(`status.${app.status.state}`)}></span>
  </div>

  <div class="label">{t('side.workspace')}</div>

  <nav>
    {#each items as it (it.id)}
      <button class="nav" class:active={app.page === it.id} onclick={() => (app.page = it.id)}>
        <span class="ico"><Icon name={it.icon} size={18} /></span>
        <span class="txt">
          <span class="t">{t(it.title)}</span>
          <span class="s">{t(it.sub)}</span>
        </span>
        {#if app.page === it.id}<Icon name="arrow-right" size={15} class="arrow" />{/if}
      </button>
    {/each}
  </nav>

  <div class="spacer"></div>

  <button class="status-card" onclick={() => app.toggle()} disabled={app.autopickRunning}>
    <span class="sdot" class:on={app.connected} class:warn={!app.isAdmin}></span>
    <span class="txt">
      {#if !app.isAdmin}
        <span class="t">{t('side.noAdmin')}</span>
        <span class="s">{t('side.noAdminSub')}</span>
      {:else}
        <span class="t">{t(app.autopickRunning ? 'status.testing' : `status.${app.status.state}`)}</span>
        <span class="s">{app.connected ? app.strategyName(app.status.strategy) : t('side.adminSub')}</span>
      {/if}
    </span>
    <span class="pw" class:on={app.connected}><Icon name="power" size={15} /></span>
  </button>

  <div class="ver">v{app.appVersion} · zapret {app.zapretVersion}</div>
</aside>

<style>
  .sidebar {
    position: relative;
    z-index: 2;
    width: 270px;
    flex-shrink: 0;
    margin: 16px 0 16px 16px;
    padding: 18px 14px 14px;
    display: flex;
    flex-direction: column;
    border-radius: 26px;
    border: 1px solid var(--border);
    background: var(--panel);
    backdrop-filter: blur(18px);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 6px 18px;
  }
  .brand-text {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  .brand-text strong {
    font-size: 17px;
    letter-spacing: -0.01em;
  }
  .brand-text span {
    font-size: 9.5px;
    letter-spacing: 0.16em;
    color: var(--text-3);
    font-weight: 600;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-3);
    transition: background 0.3s;
  }
  .dot.on {
    background: var(--ok);
    box-shadow: 0 0 10px var(--ok);
  }
  .dot.busy {
    background: var(--warn);
    animation: blink 1s infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0.3;
    }
  }
  .label {
    padding: 0 10px 10px;
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--text-3);
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .nav {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px;
    border-radius: 16px;
    border: 1px solid transparent;
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition:
      background 0.2s,
      border-color 0.2s;
  }
  .nav:hover {
    background: var(--field);
  }
  .nav.active {
    background: var(--card-hover);
    border-color: var(--border-strong);
  }
  .ico {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border-radius: 11px;
    border: 1px solid var(--border);
    background: var(--field);
    color: var(--text-2);
  }
  .nav.active .ico {
    color: var(--text);
  }
  .txt {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  .t {
    font-weight: 650;
    font-size: 13.5px;
  }
  .s {
    font-size: 11px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .nav :global(.arrow) {
    color: var(--text-2);
  }
  .spacer {
    flex: 1;
  }
  .status-card {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: var(--field);
    text-align: left;
    cursor: pointer;
  }
  .status-card:hover:not(:disabled) {
    border-color: var(--border-strong);
  }
  .sdot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-3);
  }
  .sdot.on {
    background: var(--ok);
    box-shadow: 0 0 8px var(--ok);
  }
  .sdot.warn {
    background: var(--warn);
  }
  .pw {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    border: 1px solid var(--border-strong);
    color: var(--text-2);
  }
  .pw.on {
    color: var(--ok);
    border-color: color-mix(in srgb, var(--ok) 45%, transparent);
  }
  .ver {
    padding-top: 12px;
    text-align: center;
    font-size: 10.5px;
    color: var(--text-3);
  }
</style>
