import ru, { type Dict } from './ru';
import en from './en';
import uk from './uk';
import kk from './kk';
import de from './de';

export type Key = keyof Dict;

export const LANGUAGES = [
  { code: 'ru', short: 'RU', label: 'Русский' },
  { code: 'en', short: 'EN', label: 'English' },
  { code: 'uk', short: 'UK', label: 'Українська' },
  { code: 'kk', short: 'KZ', label: 'Қазақша' },
  { code: 'de', short: 'DE', label: 'Deutsch' }
] as const;

const dicts: Record<string, Partial<Dict>> = { ru, en, uk, kk, de };

class I18n {
  lang = $state('ru');

  set(code: string) {
    this.lang = dicts[code] ? code : 'en';
    if (typeof document !== 'undefined') document.documentElement.lang = this.lang;
  }

  t = (key: Key | string, params?: Record<string, string | number>): string => {
    const d = dicts[this.lang] ?? en;
    let s = (d as Record<string, string>)[key] ?? (en as Record<string, string>)[key] ?? (ru as Record<string, string>)[key] ?? key;
    if (params) {
      for (const [k, v] of Object.entries(params)) s = s.replaceAll(`{${k}}`, String(v));
    }
    return s;
  };

  /** Translates backend error codes (e.g. "NO_IPSET_BACKUP") when known. */
  err = (e: unknown): string => {
    const msg = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e);
    const key = `err.${msg}`;
    const tr = this.t(key);
    return tr === key ? msg : tr;
  };
}

export const i18n = new I18n();
export const t = i18n.t;
