<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { i18n, t } from '$lib/i18n/index.svelte';
  import { api, isTauri, type BackupInfo, type GameMode, type IpsetMode, type UpdateInfo } from '$lib/api';
  import Icon from '$lib/components/Icon.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import ThemePicker from '$lib/components/ThemePicker.svelte';
  import LanguagePicker from '$lib/components/LanguagePicker.svelte';

  // ---------------- smart mode processes
  let newProc = $state('');
  function addProc() {
    const p = newProc.trim();
    if (!p) return;
    const name = /\.exe$/i.test(p) ? p : `${p}.exe`;
    if (!app.config.smartProcesses.some((x) => x.toLowerCase() === name.toLowerCase())) {
      app.patchConfig({ smartProcesses: [...app.config.smartProcesses, name] });
    }
    newProc = '';
  }
  function removeProc(p: string) {
    app.patchConfig({ smartProcesses: app.config.smartProcesses.filter((x) => x !== p) });
  }

  // ---------------- game filter & ipset
  let tcp = $state(app.gameFilter.tcp);
  let udp = $state(app.gameFilter.udp);
  let ipsetBusy = $state(false);

  async function setGameMode(mode: GameMode, tcpRange = app.gameFilter.tcp, udpRange = app.gameFilter.udp) {
    if (!isTauri()) {
      app.gameFilter = { mode, tcp: tcpRange, udp: udpRange };
      return;
    }
    try {
      app.gameFilter = await api.setGameFilter({ mode, tcp: tcpRange, udp: udpRange });
      tcp = app.gameFilter.tcp;
      udp = app.gameFilter.udp;
      app.toast(t('set.applied'), 'ok');
      await app.reconnectIfRunning();
    } catch (e) {
      app.fail(e);
    }
  }

  async function setIpset(mode: string) {
    if (!isTauri()) {
      app.ipsetMode = mode as IpsetMode;
      return;
    }
    ipsetBusy = true;
    try {
      const r = await api.setIpsetMode(mode as IpsetMode);
      app.ipsetMode = r.mode;
      app.ipsetHasBackup = r.hasBackup;
      app.toast(t('set.applied'), 'ok');
      await app.reconnectIfRunning();
    } catch (e) {
      app.fail(e);
    } finally {
      ipsetBusy = false;
    }
  }

  async function updateIpset() {
    if (!isTauri()) return;
    ipsetBusy = true;
    try {
      const r = await api.updateIpsetList();
      app.ipsetMode = r.mode;
      app.ipsetHasBackup = r.hasBackup;
      app.toast(t('set.applied'), 'ok');
      await app.reconnectIfRunning();
    } catch (e) {
      app.fail(e);
    } finally {
      ipsetBusy = false;
    }
  }

  // ---------------- hotkey recorder
  let recording = $state(false);
  function keyName(code: string): string | null {
    if (/^Key[A-Z]$/.test(code)) return code.slice(3);
    if (/^Digit\d$/.test(code)) return code.slice(5);
    if (/^F\d{1,2}$/.test(code)) return code;
    if (/^Numpad\d$/.test(code)) return code;
    const map: Record<string, string> = {
      Space: 'Space',
      Backquote: '`',
      Minus: '-',
      Equal: '=',
      BracketLeft: '[',
      BracketRight: ']',
      Backslash: '\\',
      Semicolon: ';',
      Quote: "'",
      Comma: ',',
      Period: '.',
      Slash: '/',
      Insert: 'Insert',
      Delete: 'Delete',
      Home: 'Home',
      End: 'End',
      PageUp: 'PageUp',
      PageDown: 'PageDown',
      ArrowUp: 'Up',
      ArrowDown: 'Down',
      ArrowLeft: 'Left',
      ArrowRight: 'Right',
      Pause: 'Pause'
    };
    return map[code] ?? null;
  }
  function onHotkey(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === 'Escape') {
      recording = false;
      return;
    }
    const key = keyName(e.code);
    if (!key) return; // modifier only
    const mods = [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift', e.metaKey && 'Super'].filter(Boolean) as string[];
    if (!mods.length && !/^F\d/.test(key)) return;
    recording = false;
    app.patchConfig({ hotkey: [...mods, key].join('+') }).then((ok) => ok && app.toast(t('toast.saved'), 'ok'));
  }

  // ---------------- updates & backups
  let info = $state<UpdateInfo | null>(null);
  let checking = $state(false);
  let installing = $state(false);
  let backups = $state<BackupInfo[]>([]);
  let confirmId = $state<string | null>(null);

  const upd = $derived(app.update);
  const updBusy = $derived(installing || (upd != null && !['done', 'error'].includes(upd.phase)));

  async function loadBackups() {
    if (isTauri()) backups = await api.listBackups().catch(() => []);
  }
  $effect(() => {
    loadBackups();
  });

  async function check() {
    if (!isTauri()) return;
    checking = true;
    try {
      info = await api.checkUpdate();
    } catch (e) {
      app.fail(e);
    } finally {
      checking = false;
    }
  }

  async function install() {
    installing = true;
    try {
      const v = await api.installUpdate();
      app.toast(t('upd.done', { v }), 'ok');
      info = info ? { ...info, local: v, hasUpdate: false } : info;
      await app.refresh();
      await loadBackups();
    } catch (e) {
      app.fail(e);
    } finally {
      installing = false;
    }
  }

  async function createBackup() {
    try {
      await api.createBackup();
      await loadBackups();
      app.toast(t('toast.saved'), 'ok');
    } catch (e) {
      app.fail(e);
    }
  }

  async function restore(id: string) {
    if (confirmId !== id) {
      confirmId = id;
      setTimeout(() => confirmId === id && (confirmId = null), 3500);
      return;
    }
    confirmId = null;
    try {
      const v = await api.restoreBackup(id);
      app.toast(t('set.restored', { v }), 'ok');
      await app.refresh();
    } catch (e) {
      app.fail(e);
    }
  }

  async function deleteBackup(id: string) {
    try {
      await api.deleteBackup(id);
      await loadBackups();
    } catch (e) {
      app.fail(e);
    }
  }

  // ---------------- reset
  let confirmReset = $state(false);
  async function reset() {
    if (!confirmReset) {
      confirmReset = true;
      setTimeout(() => (confirmReset = false), 3500);
      return;
    }
    confirmReset = false;
    if (!isTauri()) return;
    try {
      app.config = await api.resetSettings();
      i18n.set(app.config.language);
      app.toast(t('set.resetDone'), 'ok');
      app.showOnboarding = true;
    } catch (e) {
      app.fail(e);
    }
  }

  const fmtSize = (b: number) => (b > 1048576 ? `${(b / 1048576).toFixed(1)} MB` : `${Math.round(b / 1024)} KB`);
  const fmtDate = (ms: number) => new Date(ms).toLocaleString(i18n.lang);

  const updLabel = $derived.by(() => {
    if (!upd) return '';
    if (upd.phase === 'download') return t('upd.download', { p: Math.round(upd.progress * 100) });
    if (upd.phase === 'done') return t('upd.done', { v: upd.message ?? '' });
    if (upd.phase === 'error') return `${t('upd.error')}: ${upd.message ?? ''}`;
    return t(`upd.${upd.phase}`);
  });
</script>

<svelte:window onkeydown={onHotkey} />

<section class="page fade-in">
  <header>
    <span class="eyebrow">{t('set.eyebrow')}</span>
    <h1 class="hero-title">{t('set.title1')}<br />{t('set.title2a')} <span class="dim">{t('set.title2b')}</span></h1>
    <p class="hero-desc">{t('set.desc')}</p>
  </header>

  <div class="grid">
    <!-- General -->
    <div class="card box">
      <div class="section-title">
        <span class="ico"><Icon name="settings" /></span>
        <div><h3>{t('set.general')}</h3></div>
      </div>
      <div class="rows">
        <div class="srow">
          <span class="si"><Icon name="rocket" size={16} /></span>
          <div class="st"><b>{t('set.autostart')}</b><span>{t('set.autostartSub')}</span></div>
          <Toggle checked={app.config.autostart} onchange={(v) => app.patchConfig({ autostart: v })} />
        </div>
        <div class="srow">
          <span class="si"><Icon name="power" size={16} /></span>
          <div class="st"><b>{t('set.autoConnect')}</b><span>{t('set.autoConnectSub')}</span></div>
          <Toggle checked={app.config.autoConnect} onchange={(v) => app.patchConfig({ autoConnect: v })} />
        </div>
        <div class="srow">
          <span class="si"><Icon name="monitor" size={16} /></span>
          <div class="st"><b>{t('set.tray')}</b><span>{t('set.traySub')}</span></div>
          <Toggle checked={app.config.minimizeToTray} onchange={(v) => app.patchConfig({ minimizeToTray: v })} />
        </div>
        <div class="srow">
          <span class="si"><Icon name="volume" size={16} /></span>
          <div class="st"><b>{t('set.sounds')}</b><span>{t('set.soundsSub')}</span></div>
          <Toggle checked={app.config.sounds} onchange={(v) => app.patchConfig({ sounds: v })} />
        </div>
      </div>
    </div>

    <!-- Smart mode + hotkey -->
    <div class="card box">
      <div class="section-title">
        <span class="ico"><Icon name="radar" /></span>
        <div><h3>{t('set.smart')}</h3><p>{t('set.smartSub')}</p></div>
        <span class="push"><Toggle checked={app.config.smartMode} onchange={(v) => app.patchConfig({ smartMode: v })} /></span>
      </div>
      <div class="lbl">{t('set.smartProcesses')}</div>
      <div class="procs" class:off={!app.config.smartMode}>
        {#each app.config.smartProcesses as p (p)}
          <span class="chip">{p}<button onclick={() => removeProc(p)} aria-label="remove"><Icon name="x" size={12} /></button></span>
        {/each}
        <form onsubmit={(e) => (e.preventDefault(), addProc())} class="procadd">
          <input class="field sm" bind:value={newProc} placeholder={t('set.addProcess')} spellcheck="false" />
          <button class="btn sm icon-only" type="submit" aria-label="add"><Icon name="plus" size={14} /></button>
        </form>
      </div>

      <div class="divider"></div>

      <div class="section-title">
        <span class="ico"><Icon name="keyboard" /></span>
        <div><h3>{t('set.hotkey')}</h3><p>{t('set.hotkeySub')}</p></div>
      </div>
      <div class="hk">
        <button class="hk-btn" class:rec={recording} onclick={() => (recording = !recording)}>
          {#if recording}
            <span class="recdot"></span>{t('set.hotkeyRecord')}
          {:else if app.config.hotkey}
            {#each app.config.hotkey.split('+') as k, i (i)}<kbd>{k}</kbd>{/each}
          {:else}
            <span class="faint">—</span>
          {/if}
        </button>
        <button class="btn sm" onclick={() => app.patchConfig({ hotkey: '' })} disabled={!app.config.hotkey}>{t('set.hotkeyClear')}</button>
      </div>
    </div>

    <!-- Filters -->
    <div class="card box">
      <div class="section-title">
        <span class="ico"><Icon name="gamepad" /></span>
        <div><h3>{t('set.gameFilter')}</h3><p>{t('set.gameFilterSub')}</p></div>
      </div>
      <Segmented
        value={app.gameFilter.mode}
        options={[
          { value: 'disabled', label: t('gf.disabled') },
          { value: 'all', label: t('gf.all') },
          { value: 'tcp', label: t('gf.tcp') },
          { value: 'udp', label: t('gf.udp') }
        ]}
        onchange={(v) => setGameMode(v as GameMode)}
      />
      <div class="ranges">
        <label>
          <span>{t('set.tcpRange')}</span>
          <input class="field" bind:value={tcp} spellcheck="false" />
        </label>
        <label>
          <span>{t('set.udpRange')}</span>
          <input class="field" bind:value={udp} spellcheck="false" />
        </label>
        <button
          class="btn"
          disabled={tcp === app.gameFilter.tcp && udp === app.gameFilter.udp}
          onclick={() => setGameMode(app.gameFilter.mode, tcp, udp)}>{t('common.save')}</button
        >
      </div>

      <div class="divider"></div>

      <div class="section-title">
        <span class="ico"><Icon name="network" /></span>
        <div><h3>{t('set.ipset')}</h3><p>{t('set.ipsetSub')}</p></div>
      </div>
      <Segmented
        value={app.ipsetMode}
        disabled={ipsetBusy}
        options={[
          { value: 'loaded', label: 'loaded', sub: t('ipset.loadedSub') },
          { value: 'none', label: 'none', sub: t('ipset.noneSub') },
          { value: 'any', label: 'any', sub: t('ipset.anySub') }
        ]}
        onchange={setIpset}
      />
      <button class="btn sm mt" onclick={updateIpset} disabled={ipsetBusy}>
        <Icon name={ipsetBusy ? 'loader' : 'download'} size={14} class={ipsetBusy ? 'spin' : ''} />{t('set.ipsetUpdate')}
      </button>
    </div>

    <!-- Updates -->
    <div class="card box">
      <div class="section-title">
        <span class="ico"><Icon name="download" /></span>
        <div><h3>{t('set.updates')}</h3><p>{t('set.updatesSub')}</p></div>
      </div>
      <div class="vers">
        <div class="ver"><span>{t('set.localVer')}</span><b>{info?.local ?? app.zapretVersion}</b></div>
        <Icon name="arrow-right" size={16} class="faint" />
        <div class="ver"><span>{t('set.latestVer')}</span><b>{info?.latest ?? '—'}</b></div>
        <div class="vbtns">
          <button class="btn sm" onclick={check} disabled={checking || updBusy}>
            <Icon name="refresh" size={14} class={checking ? 'spin' : ''} />{t('set.check')}
          </button>
          {#if info?.hasUpdate}
            <button class="btn sm primary" onclick={install} disabled={updBusy}><Icon name="download" size={14} />{t('set.install')}</button>
          {/if}
        </div>
      </div>
      {#if info && !upd}
        <div class="note" class:ok={!info.hasUpdate}>
          {info.hasUpdate ? t('set.updateAvail', { v: info.latest }) : t('set.upToDate')}
        </div>
      {/if}
      {#if upd}
        <div class="progress">
          <div class="pl" class:err={upd.phase === 'error'}>{updLabel}</div>
          <div class="bar">
            <span
              class:ok={upd.phase === 'done'}
              style:width="{upd.phase === 'download' ? upd.progress * 100 : upd.phase === 'done' ? 100 : upd.phase === 'apply' ? 92 : upd.phase === 'extract' ? 80 : 4}%"
            ></span>
          </div>
        </div>
      {/if}

      <div class="divider"></div>

      <div class="section-title">
        <span class="ico"><Icon name="history" /></span>
        <div><h3>{t('set.backups')}</h3><p>{t('set.backupsSub')}</p></div>
        <span class="push"><button class="btn sm" onclick={createBackup} disabled={updBusy}><Icon name="archive" size={14} />{t('set.createBackup')}</button></span>
      </div>
      <div class="backups">
        {#each backups as b (b.id)}
          <div class="bk">
            <Icon name="archive" size={15} class="faint" />
            <div class="bt"><b>zapret {b.version}</b><span>{fmtDate(b.created)} · {fmtSize(b.size)}</span></div>
            <button class="btn sm" class:danger={confirmId === b.id} onclick={() => restore(b.id)} disabled={updBusy}>
              <Icon name="rollback" size={13} />{confirmId === b.id ? t('common.confirm') : t('set.restore')}
            </button>
            <button class="btn sm icon-only" onclick={() => deleteBackup(b.id)} aria-label="delete"><Icon name="trash" size={13} /></button>
          </div>
        {:else}
          <div class="faint empty">{t('set.noBackups')}</div>
        {/each}
      </div>
    </div>

    <!-- Appearance -->
    <div class="card box full">
      <div class="section-title">
        <span class="ico"><Icon name="languages" /></span>
        <div><h3>{t('set.language')}</h3><p>{t('set.languageSub')}</p></div>
      </div>
      <LanguagePicker value={app.config.language} onchange={(code) => app.patchConfig({ language: code })} />

      <div class="section-title mt2">
        <span class="ico"><Icon name="palette" /></span>
        <div><h3>{t('set.theme')}</h3><p>{t('set.themeSub')}</p></div>
      </div>
      <ThemePicker value={app.config.theme} onchange={(id) => app.patchConfig({ theme: id })} />
    </div>

    <!-- Misc / reset -->
    <div class="card box full misc">
      <div class="section-title">
        <span class="ico"><Icon name="rollback" /></span>
        <div><h3>{t('set.reset')}</h3><p>{t('set.resetSub')}</p></div>
      </div>
      <div class="mbtns">
        <button class="btn sm" onclick={() => (app.showOnboarding = true)}><Icon name="sparkles" size={14} />{t('set.onboarding')}</button>
        <button class="btn sm" onclick={() => isTauri() && api.openZapretFolder().catch(app.fail)}><Icon name="folder" size={14} />{t('set.folder')}</button>
        <button class="btn sm danger" onclick={reset}><Icon name="trash" size={14} />{confirmReset ? t('common.confirm') : t('set.resetBtn')}</button>
      </div>
      <div class="path faint">{app.zapretDir}</div>
    </div>
  </div>
</section>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
    align-items: start;
  }
  .box {
    padding: 20px;
  }
  .full {
    grid-column: 1 / -1;
  }
  .push {
    margin-left: auto;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .srow {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 8px;
    border-radius: 12px;
  }
  .srow:hover {
    background: var(--field);
  }
  .si {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    border: 1px solid var(--border);
    color: var(--text-2);
  }
  .st {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .st b {
    font-size: 13.5px;
    font-weight: 650;
  }
  .st span {
    font-size: 11.5px;
    color: var(--text-3);
  }
  .lbl {
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-3);
    margin-bottom: 8px;
  }
  .procs {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    transition: opacity 0.2s;
  }
  .procs.off {
    opacity: 0.5;
  }
  .chip button {
    display: grid;
    place-items: center;
    border: 0;
    background: none;
    padding: 0;
    color: var(--text-3);
    cursor: pointer;
  }
  .chip button:hover {
    color: var(--danger);
  }
  .procadd {
    display: flex;
    gap: 6px;
  }
  .field.sm {
    height: 30px;
    width: 130px;
    font-size: 12px;
    border-radius: 999px;
  }
  .divider {
    height: 1px;
    margin: 20px 0;
    background: var(--border);
  }
  .hk {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .hk-btn {
    flex: 1;
    height: 46px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-radius: 14px;
    border: 1px dashed var(--border-strong);
    background: var(--field);
    cursor: pointer;
    font-weight: 600;
  }
  .hk-btn.rec {
    border-style: solid;
    border-color: var(--accent);
    color: var(--accent);
  }
  .recdot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--danger);
    animation: blink 1s infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0.2;
    }
  }
  kbd {
    font-family: var(--mono);
    font-size: 12px;
    padding: 4px 9px;
    border-radius: 8px;
    border: 1px solid var(--border-strong);
    background: var(--card-hover);
    box-shadow: 0 2px 0 var(--border-strong);
  }
  .ranges {
    display: grid;
    grid-template-columns: 1fr 1fr auto;
    gap: 10px;
    align-items: end;
    margin-top: 12px;
  }
  .ranges label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 11.5px;
    color: var(--text-3);
  }
  .ranges .field {
    font-family: var(--mono);
    font-size: 12.5px;
  }
  .mt {
    margin-top: 12px;
  }
  .mt2 {
    margin-top: 26px;
  }
  .vers {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px;
    border-radius: 14px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .ver {
    display: flex;
    flex-direction: column;
  }
  .ver span {
    font-size: 10.5px;
    color: var(--text-3);
  }
  .ver b {
    font-size: 20px;
    font-family: var(--mono);
  }
  .vbtns {
    margin-left: auto;
    display: flex;
    gap: 6px;
  }
  .note {
    margin-top: 10px;
    font-size: 12.5px;
    color: var(--warn);
  }
  .note.ok {
    color: var(--ok);
  }
  .progress {
    margin-top: 12px;
  }
  .pl {
    font-size: 12.5px;
    margin-bottom: 8px;
  }
  .pl.err {
    color: var(--danger);
  }
  .bar {
    height: 6px;
    border-radius: 99px;
    background: var(--border);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--accent);
    transition: width 0.4s var(--ease);
  }
  .bar span.ok {
    background: var(--ok);
  }
  .backups {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 260px;
    overflow-y: auto;
  }
  .bk {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 8px 8px 12px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .bt {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .bt b {
    font-size: 13px;
  }
  .bt span {
    font-size: 11px;
    color: var(--text-3);
  }
  .empty {
    padding: 14px 0;
    font-size: 12.5px;
  }
  .mbtns {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .path {
    margin-top: 12px;
    font-family: var(--mono);
    font-size: 11px;
    user-select: text;
  }
  @media (max-width: 1180px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
