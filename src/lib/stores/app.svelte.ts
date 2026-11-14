import {
  api,
  on,
  EVENTS,
  isTauri,
  type AppConfig,
  type AutopickProgress,
  type GameFilter,
  type IpsetMode,
  type Snapshot,
  type Stats,
  type StrategyResult,
  type Status,
  type StrategyInfo,
  type UpdateProgress
} from '$lib/api';
import { i18n, t } from '$lib/i18n/index.svelte';
import { sounds } from '$lib/sound';

export type Page = 'dashboard' | 'lists' | 'settings' | 'about';

export interface Toast {
  id: number;
  kind: 'ok' | 'error' | 'info';
  text: string;
}

const DEFAULT_CONFIG: AppConfig = {
  language: 'ru',
  theme: 'star',
  onboarded: true,
  strategy: 'general.bat',
  autostart: false,
  autoConnect: false,
  minimizeToTray: true,
  smartMode: false,
  smartProcesses: ['Discord.exe', 'chrome.exe', 'firefox.exe', 'msedge.exe', 'opera.exe', 'brave.exe', 'browser.exe', 'yandex.exe'],
  hotkey: 'Ctrl+Shift+Z',
  sounds: true,
  zapretPath: null,
  profiles: [],
  schedules: []
};

const EMPTY_STATS: Stats = { sessions: 0, totalSeconds: 0, crashes: 0, perStrategy: {}, lastConnected: null, recent: [] };

class AppStore {
  ready = $state(false);
  page = $state<Page>('dashboard');
  showOnboarding = $state(false);

  config = $state<AppConfig>({ ...DEFAULT_CONFIG });
  status = $state<Status>({ state: 'disconnected', strategy: null, since: null, error: null, origin: null });
  strategies = $state<StrategyInfo[]>([]);
  zapretVersion = $state('—');
  zapretDir = $state('');
  appVersion = $state('1.0.0');
  isAdmin = $state(true);
  gameFilter = $state<GameFilter>({ mode: 'disabled', tcp: '1024-65535', udp: '1024-65535' });
  ipsetMode = $state<IpsetMode>('loaded');
  ipsetHasBackup = $state(false);

  stats = $state<Stats>({ ...EMPTY_STATS });
  /** Last auto-pick measurement per strategy file. */
  pickResults = $state<Record<string, StrategyResult>>({});

  /** Best strategy by the same rule as the backend: most reachable, then lowest ping. */
  bestStrategy = $derived.by(() => {
    const list = Object.values(this.pickResults).filter((r) => r.passed > 0 && r.passed * 2 >= r.checks);
    list.sort((a, b) => b.passed - a.passed || (a.latencyMs ?? Infinity) - (b.latencyMs ?? Infinity));
    return list[0]?.file ?? null;
  });

  autopick = $state<AutopickProgress | null>(null);
  autopickRunning = $state(false);
  update = $state<UpdateProgress | null>(null);

  toasts = $state<Toast[]>([]);
  private toastId = 0;

  connected = $derived(this.status.state === 'connected');
  busy = $derived(this.status.state === 'connecting' || this.autopickRunning);
  currentStrategy = $derived(this.strategies.find((s) => s.file === this.config.strategy) ?? null);

  toast(text: string, kind: Toast['kind'] = 'info') {
    const id = ++this.toastId;
    this.toasts = [...this.toasts, { id, kind, text }];
    setTimeout(() => (this.toasts = this.toasts.filter((x) => x.id !== id)), 3800);
  }

  fail = (e: unknown) => {
    this.toast(i18n.err(e), 'error');
    if (this.config.sounds) sounds.error();
  };

  strategyName(file: string | null | undefined): string {
    if (!file) return '—';
    return this.strategies.find((s) => s.file === file)?.name ?? file.replace(/\.bat$/i, '');
  }

  applySnapshot(s: Snapshot) {
    this.config = s.config;
    this.status = s.status;
    this.strategies = s.strategies;
    this.zapretVersion = s.zapretVersion;
    this.zapretDir = s.zapretDir;
    this.appVersion = s.appVersion;
    this.isAdmin = s.isAdmin;
    this.gameFilter = s.gameFilter;
    this.ipsetMode = s.ipsetMode;
    this.ipsetHasBackup = s.ipsetHasBackup;
    this.autopickRunning = s.autopickRunning;
    this.stats = s.stats ?? { ...EMPTY_STATS };
    this.pickResults = Object.fromEntries((s.pickResults ?? []).map((r) => [r.file, r]));
    i18n.set(s.config.language);
  }

  async init() {
    if (!isTauri()) {
      // Browser preview (npm run dev without Tauri): show the UI with demo data.
      this.strategies = [
        { file: 'general.bat', name: 'general' },
        ...Array.from({ length: 13 }, (_, i) => ({ file: `general (ALT${i ? i + 1 : ''}).bat`, name: `ALT${i ? i + 1 : ''}` }))
      ];
      this.zapretVersion = '1.10.4';
      this.ready = true;
      return;
    }

    await on<Status>(EVENTS.status, (s) => {
      const prev = this.status.state;
      this.status = s;
      if (s.origin === 'autopick') return;
      if (prev !== 'connected' && s.state === 'connected') {
        if (this.config.sounds) sounds.connect();
      } else if (prev === 'connected' && s.state === 'disconnected') {
        if (this.config.sounds) sounds.disconnect();
        if (s.error) this.toast(s.error, 'error');
      }
    });

    await on<AutopickProgress>(EVENTS.autopick, (p) => {
      this.autopick = p;
      if (p.phase === 'result' && p.file) {
        this.pickResults = {
          ...this.pickResults,
          [p.file]: { file: p.file, passed: p.passed, checks: p.checks, latencyMs: p.latencyMs, testedAt: Date.now() }
        };
      }
      if (p.phase === 'testing' || p.phase === 'result') {
        this.autopickRunning = true;
        return;
      }
      this.autopickRunning = false;
      if (p.phase === 'done' && p.found) {
        this.config = { ...this.config, strategy: p.found };
        const ms = p.latencyMs != null ? ` · ${p.latencyMs} ${t('unit.ms')}` : '';
        this.toast(t('dash.autopickFound', { name: this.strategyName(p.found) }) + ms, 'ok');
        if (this.config.sounds) sounds.found();
      } else if (p.phase === 'failed') {
        this.toast(t('dash.autopickFailed'), 'error');
        if (this.config.sounds) sounds.error();
      }
    });

    await on<UpdateProgress>(EVENTS.update, (p) => (this.update = p));

    try {
      this.applySnapshot(await api.snapshot());
    } catch (e) {
      this.fail(e);
    }
    this.showOnboarding = !this.config.onboarded;
    this.ready = true;
  }

  async refresh() {
    if (!isTauri()) return;
    try {
      this.applySnapshot(await api.snapshot());
    } catch (e) {
      this.fail(e);
    }
  }

  /** Updates config locally and persists it; reverts on error. */
  async patchConfig(patch: Partial<AppConfig>) {
    const prev = this.config;
    this.config = { ...this.config, ...patch };
    if (patch.language) i18n.set(patch.language);
    if (!isTauri()) return true;
    try {
      this.config = await api.saveConfig(this.config);
      return true;
    } catch (e) {
      this.config = prev;
      i18n.set(prev.language);
      this.fail(e);
      return false;
    }
  }

  async toggle() {
    if (!isTauri()) return;
    try {
      if (this.connected || this.status.state === 'connecting') {
        await api.disconnect();
      } else {
        const s = await api.connect(this.config.strategy);
        if (s.state === 'connected') this.toast(t('toast.connected', { name: this.strategyName(s.strategy) }), 'ok');
      }
    } catch (e) {
      this.fail(e);
    }
  }

  async selectStrategy(file: string) {
    await this.patchConfig({ strategy: file });
    if (this.connected && isTauri()) {
      try {
        await api.connect(file);
      } catch (e) {
        this.fail(e);
      }
    }
  }

  async startAutopick() {
    if (!isTauri()) return;
    this.autopickRunning = true;
    this.autopick = null;
    try {
      const found = await api.autopickStart();
      if (found) this.config = { ...this.config, strategy: found };
    } catch (e) {
      this.fail(e);
    } finally {
      this.autopickRunning = false;
    }
  }

  async cancelAutopick() {
    if (isTauri()) await api.autopickCancel();
  }

  async reconnectIfRunning() {
    if (!isTauri() || !this.connected) return;
    try {
      await api.reconnectIfRunning();
    } catch (e) {
      this.fail(e);
    }
  }
}

export const app = new AppStore();
