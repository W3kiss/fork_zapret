export interface ThemeDef {
  id: string;
  /** Preview colours: background, panel, accent */
  bg: string;
  panel: string;
  accent: string;
  stars: boolean;
  light?: boolean;
}

export const THEMES: ThemeDef[] = [
  { id: 'star', bg: '#000000', panel: '#0d0d10', accent: '#ffffff', stars: true },
  { id: 'mono', bg: '#0b0b0d', panel: '#141417', accent: '#f4f4f5', stars: false },
  { id: 'cyan', bg: '#061013', panel: '#0b1a1f', accent: '#22d3ee', stars: false },
  { id: 'violet', bg: '#0c0a14', panel: '#151224', accent: '#a78bfa', stars: false },
  { id: 'neon', bg: '#0b0410', panel: '#16091f', accent: '#ff2bd6', stars: false },
  { id: 'emerald', bg: '#06100c', panel: '#0b1a14', accent: '#34d399', stars: false },
  { id: 'amoled', bg: '#000000', panel: '#050505', accent: '#d4d4d8', stars: false },
  { id: 'light', bg: '#f3f4f7', panel: '#ffffff', accent: '#2563eb', stars: false, light: true }
];

export const themeById = (id: string) => THEMES.find((t) => t.id === id) ?? THEMES[0];
