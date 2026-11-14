import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type ConnState = 'disconnected' | 'connecting' | 'connected';
export type IpsetMode = 'loaded' | 'none' | 'any';
export type GameMode = 'disabled' | 'all' | 'tcp' | 'udp';

export interface Profile {
  name: string;
  strategy: string;
  gameFilterMode: GameMode;
  gameFilterTcp: string;
  gameFilterUdp: string;
  ipsetMode: IpsetMode;
}

export interface Schedule {
  id: string;
  enabled: boolean;
  action: 'connect' | 'disconnect';
  time: string;
  days: number[];
}

export interface AppConfig {
  language: string;
  theme: string;
  onboarded: boolean;
  strategy: string;
  autostart: boolean;
  autoConnect: boolean;
  minimizeToTray: boolean;
  smartMode: boolean;
  smartProcesses: string[];
  hotkey: string;
  sounds: boolean;
  zapretPath: string | null;
  profiles: Profile[];
  schedules: Schedule[];
}

export interface Check {
  name: string;
  status: 'ok' | 'warn' | 'fail';
  detail: string;
}

export interface StatsSession {
  strategy: string;
  start: number;
  seconds: number;
  crashed: boolean;
}

export interface Stats {
  sessions: number;
  totalSeconds: number;
  crashes: number;
  perStrategy: Record<string, number>;
  lastConnected: number | null;
  recent: StatsSession[];
}

export interface AppUpdateInfo {
  current: string;
  latest: string;
  hasUpdate: boolean;
  url: string;
  setup: string | null;
  size: number;
}

export interface Status {
  state: ConnState;
  strategy: string | null;
  since: number | null;
  error: string | null;
  origin: string | null;
}

export interface StrategyInfo {
  file: string;
  name: string;
}

export interface GameFilter {
  mode: GameMode;
  tcp: string;
  udp: string;
}

export interface Snapshot {
  config: AppConfig;
  status: Status;
  strategies: StrategyInfo[];
  zapretVersion: string;
  zapretDir: string;
  gameFilter: GameFilter;
  ipsetMode: IpsetMode;
  ipsetHasBackup: boolean;
  isAdmin: boolean;
  appVersion: string;
  autopickRunning: boolean;
  stats: Stats;
  pickResults: StrategyResult[];
}

export interface AutopickProgress {
  phase: 'testing' | 'result' | 'done' | 'cancelled' | 'failed';
  index: number;
  total: number;
  file: string | null;
  name: string | null;
  passed: number;
  checks: number;
  latencyMs: number | null;
  found: string | null;
}

export interface StrategyResult {
  file: string;
  passed: number;
  checks: number;
  latencyMs: number | null;
  testedAt: number;
}

export interface UpdateInfo {
  local: string;
  latest: string;
  hasUpdate: boolean;
  url: string;
  notes: string;
  asset: string | null;
  size: number;
}

export interface UpdateProgress {
  phase: 'backup' | 'download' | 'extract' | 'apply' | 'done' | 'error';
  progress: number;
  downloaded: number;
  total: number;
  message: string | null;
}

export interface BackupInfo {
  id: string;
  version: string;
  created: number;
  size: number;
}

export interface IpsetState {
  mode: IpsetMode;
  hasBackup: boolean;
}

export const isTauri = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const api = {
  snapshot: () => invoke<Snapshot>('get_snapshot'),
  saveConfig: (config: AppConfig) => invoke<AppConfig>('save_config', { config }),
  resetSettings: () => invoke<AppConfig>('reset_settings'),
  strategies: () => invoke<StrategyInfo[]>('list_strategies'),
  previewArgs: (file: string) => invoke<string[]>('preview_args', { file }),
  connect: (strategy?: string) => invoke<Status>('connect', { strategy: strategy ?? null }),
  disconnect: () => invoke<Status>('disconnect'),
  toggle: () => invoke<Status>('toggle'),
  reconnectIfRunning: () => invoke<Status>('reconnect_if_running'),
  setGameFilter: (filter: GameFilter) => invoke<GameFilter>('set_game_filter', { filter }),
  setIpsetMode: (mode: IpsetMode) => invoke<IpsetState>('set_ipset_mode', { mode }),
  updateIpsetList: () => invoke<IpsetState>('update_ipset_list'),
  listRead: (name: string) => invoke<string[]>('list_read', { name }),
  listAdd: (name: string, entries: string[]) => invoke<string[]>('list_add', { name, entries }),
  listRemove: (name: string, entry: string) => invoke<string[]>('list_remove', { name, entry }),
  autopickStart: () => invoke<string | null>('autopick_start'),
  autopickCancel: () => invoke<void>('autopick_cancel'),
  checkUpdate: () => invoke<UpdateInfo>('check_update'),
  installUpdate: () => invoke<string>('install_update'),
  listBackups: () => invoke<BackupInfo[]>('list_backups'),
  createBackup: () => invoke<BackupInfo>('create_backup'),
  deleteBackup: (id: string) => invoke<void>('delete_backup', { id }),
  restoreBackup: (id: string) => invoke<string>('restore_backup', { id }),
  avatar: (refresh: boolean) => invoke<string | null>('get_avatar', { refresh }),
  openUrl: (url: string) => invoke<void>('open_url', { url }),
  openZapretFolder: () => invoke<void>('open_zapret_folder'),
  setTrayLabels: (labels: Record<string, string>) => invoke<void>('set_tray_labels', { labels }),
  quit: () => invoke<void>('quit_app'),
  winwsLog: () => invoke<string>('winws_log'),
  fullStop: () => invoke<Status>('zapret_full_stop'),
  applyProfile: (name: string) => invoke<void>('apply_profile', { name }),
  runDiagnostics: () => invoke<Check[]>('run_diagnostics'),
  getStats: () => invoke<Stats>('get_stats'),
  getPickResults: () => invoke<StrategyResult[]>('get_pick_results'),
  resetStats: () => invoke<Stats>('reset_stats'),
  checkAppUpdate: () => invoke<AppUpdateInfo>('check_app_update'),
  installAppUpdate: () => invoke<void>('install_app_update')
};

export function on<T>(event: string, cb: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (e) => cb(e.payload));
}

export const EVENTS = {
  status: 'zapret://status',
  autopick: 'autopick://progress',
  update: 'update://progress'
} as const;

export const LINKS = {
  developer: 'https://github.com/W3kiss',
  zapret: 'https://github.com/Flowseal/zapret-discord-youtube',
  zapretReleases: 'https://github.com/Flowseal/zapret-discord-youtube/releases'
} as const;
