<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { api, isTauri } from '$lib/api';
  import Icon from '$lib/components/Icon.svelte';

  const FILES = [
    { name: 'list-general-user.txt', icon: 'user' },
    { name: 'list-general.txt', icon: 'globe' },
    { name: 'list-google.txt', icon: 'zap' },
    { name: 'list-exclude-user.txt', icon: 'shield-off' },
    { name: 'ipset-all.txt', icon: 'network' },
    { name: 'ipset-exclude-user.txt', icon: 'filter' }
  ];
  const PAGE = 300;

  let file = $state(FILES[0].name);
  let entries = $state<string[]>([]);
  let loading = $state(false);
  let query = $state('');
  let input = $state('');
  let limit = $state(PAGE);
  let dirty = $state(false);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const list = q ? entries.filter((e) => e.toLowerCase().includes(q)) : entries;
    return list;
  });
  const visible = $derived(filtered.slice(0, limit));

  $effect(() => {
    const f = file;
    load(f);
  });

  async function load(f: string) {
    limit = PAGE;
    if (!isTauri()) {
      entries = ['discord.com', 'discord.gg', 'youtube.com', 'googlevideo.com', 'ytimg.com'];
      return;
    }
    loading = true;
    try {
      entries = await api.listRead(f);
    } catch (e) {
      app.fail(e);
      entries = [];
    } finally {
      loading = false;
    }
  }

  async function add() {
    const parts = input.split(/[\s,;]+/).map((s) => s.trim()).filter(Boolean);
    if (!parts.length || !isTauri()) return;
    try {
      const before = entries.length;
      entries = await api.listAdd(file, parts);
      app.toast(t('lists.added', { n: Math.max(entries.length - before, parts.length) }), 'ok');
      input = '';
      dirty = true;
    } catch (e) {
      app.fail(e);
    }
  }

  async function remove(entry: string) {
    if (!isTauri()) return;
    try {
      entries = await api.listRemove(file, entry);
      app.toast(t('lists.removed', { entry }), 'info');
      dirty = true;
    } catch (e) {
      app.fail(e);
    }
  }

  async function reconnect() {
    await app.reconnectIfRunning();
    dirty = false;
  }
</script>

<section class="page fade-in">
  <header>
    <span class="eyebrow">{t('lists.eyebrow')}</span>
    <h1 class="hero-title">{t('lists.title1')}<br />{t('lists.title2a')} <span class="dim">{t('lists.title2b')}</span></h1>
    <p class="hero-desc">{t('lists.desc')}</p>
  </header>

  <div class="files">
    {#each FILES as f (f.name)}
      <button class="file card" class:active={file === f.name} onclick={() => (file = f.name)}>
        <span class="fi"><Icon name={f.icon} size={16} /></span>
        <span class="ft">
          <span class="fn">{f.name}</span>
          <span class="fd">{t(`lists.file.${f.name}`)}</span>
        </span>
      </button>
    {/each}
  </div>

  <div class="editor card">
    <form class="add" onsubmit={(e) => (e.preventDefault(), add())}>
      <Icon name="plus" size={16} />
      <input class="bare" bind:value={input} placeholder={t('lists.placeholder')} spellcheck="false" />
      <button class="btn primary" type="submit" disabled={!input.trim()}>{t('common.add')}</button>
    </form>

    <div class="toolbar">
      <div class="search">
        <Icon name="search" size={15} />
        <input class="bare" bind:value={query} placeholder={t('lists.search')} spellcheck="false" />
        {#if query}<button class="clear" onclick={() => (query = '')}><Icon name="x" size={14} /></button>{/if}
      </div>
      <span class="count">{t('lists.count', { n: filtered.length })}</span>
    </div>

    {#if dirty && app.connected}
      <div class="apply fade-in">
        <Icon name="info" size={15} />
        <span>{t('lists.applyHint')}</span>
        <button class="btn sm" onclick={reconnect}><Icon name="refresh" size={13} />{t('lists.reconnect')}</button>
      </div>
    {/if}

    <div class="entries">
      {#if loading}
        <div class="empty"><Icon name="loader" size={18} class="spin" /> {t('common.loading')}</div>
      {:else if !filtered.length}
        <div class="empty">{query ? t('lists.nothingFound') : t('lists.empty')}</div>
      {:else}
        {#each visible as e (e)}
          <div class="entry">
            <span class="ev">{e}</span>
            <button class="del" title={t('common.remove')} onclick={() => remove(e)}><Icon name="trash" size={14} /></button>
          </div>
        {/each}
        {#if filtered.length > limit}
          <button class="btn more" onclick={() => (limit += PAGE)}>{t('lists.showMore')} ({filtered.length - limit})</button>
        {/if}
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
  .files {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
  }
  .file {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 16px;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.2s;
  }
  .file:hover {
    border-color: var(--border-strong);
  }
  .file.active {
    border-color: var(--text);
    background: var(--card-hover);
  }
  .fi {
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    border: 1px solid var(--border);
    color: var(--text-2);
  }
  .ft {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .fn {
    font-family: var(--mono);
    font-size: 12px;
    font-weight: 600;
  }
  .fd {
    font-size: 11px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .editor {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .add,
  .search {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 6px 0 14px;
    height: 50px;
    border-radius: 14px;
    border: 1px solid var(--border-strong);
    background: var(--field);
    color: var(--text-2);
  }
  .search {
    flex: 1;
    height: 40px;
    border-color: var(--border);
  }
  .bare {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 13.5px;
  }
  .bare::placeholder {
    color: var(--text-3);
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .count {
    font-size: 12px;
    color: var(--text-3);
    white-space: nowrap;
  }
  .clear {
    border: 0;
    background: none;
    color: var(--text-3);
    cursor: pointer;
    display: grid;
    place-items: center;
    padding: 6px;
  }
  .apply {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 12px;
    background: var(--accent-soft);
    font-size: 12.5px;
  }
  .apply .btn {
    margin-left: auto;
  }
  .entries {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 6px;
    max-height: calc(100vh - 470px);
    min-height: 160px;
    overflow-y: auto;
    padding-right: 4px;
  }
  .entry {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 6px 0 12px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--field);
  }
  .ev {
    flex: 1;
    font-family: var(--mono);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    user-select: text;
  }
  .del {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    border: 0;
    background: transparent;
    color: var(--text-3);
    cursor: pointer;
    opacity: 0;
    transition:
      opacity 0.15s,
      color 0.15s,
      background 0.15s;
  }
  .entry:hover .del {
    opacity: 1;
  }
  .del:hover {
    color: var(--danger);
    background: var(--danger-soft);
  }
  .empty {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 50px 0;
    color: var(--text-3);
  }
  .more {
    grid-column: 1 / -1;
  }
</style>
