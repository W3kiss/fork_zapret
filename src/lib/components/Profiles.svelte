<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { api, isTauri, type Profile } from '$lib/api';
  import Icon from './Icon.svelte';

  let name = $state('');

  function captureCurrent(): Profile {
    return {
      name: name.trim(),
      strategy: app.config.strategy,
      gameFilterMode: app.gameFilter.mode,
      gameFilterTcp: app.gameFilter.tcp,
      gameFilterUdp: app.gameFilter.udp,
      ipsetMode: app.ipsetMode
    };
  }

  async function save() {
    const n = name.trim();
    if (!n) return;
    const profile = captureCurrent();
    const rest = app.config.profiles.filter((p) => p.name.toLowerCase() !== n.toLowerCase());
    await app.patchConfig({ profiles: [...rest, profile] });
    name = '';
    app.toast(t('toast.saved'), 'ok');
  }

  async function apply(p: Profile) {
    if (!isTauri()) return;
    try {
      await api.applyProfile(p.name);
      await app.refresh();
      app.toast(t('prof.applied', { name: p.name }), 'ok');
    } catch (e) {
      app.fail(e);
    }
  }

  async function remove(p: Profile) {
    await app.patchConfig({ profiles: app.config.profiles.filter((x) => x.name !== p.name) });
  }

  const gfLabel = (p: Profile) => t(`gf.${p.gameFilterMode}`);
</script>

<div class="card box full">
  <div class="section-title">
    <span class="ico"><Icon name="layers" /></span>
    <div><h3>{t('prof.title')}</h3><p>{t('prof.sub')}</p></div>
  </div>

  <form class="save" onsubmit={(e) => (e.preventDefault(), save())}>
    <input class="field" bind:value={name} placeholder={t('prof.namePlaceholder')} maxlength="24" spellcheck="false" />
    <button class="btn primary" type="submit" disabled={!name.trim()}><Icon name="plus" size={15} />{t('prof.saveCurrent')}</button>
  </form>

  {#if app.config.profiles.length}
    <div class="list">
      {#each app.config.profiles as p (p.name)}
        <div class="row">
          <span class="pi"><Icon name="zap" size={15} /></span>
          <div class="info">
            <b>{p.name}</b>
            <span>{app.strategyName(p.strategy)} · {t('dash.gameFilter')}: {gfLabel(p)} · IPSet: {p.ipsetMode}</span>
          </div>
          <button class="btn sm primary" onclick={() => apply(p)}>{t('prof.apply')}</button>
          <button class="btn sm icon-only" onclick={() => remove(p)} aria-label="delete"><Icon name="trash" size={13} /></button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty faint">{t('prof.empty')}</p>
  {/if}
</div>

<style>
  .full {
    grid-column: 1 / -1;
  }
  .box {
    padding: 20px;
  }
  .save {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }
  .save .field {
    flex: 1;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
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
  .pi {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    border: 1px solid var(--border);
    color: var(--text-2);
  }
  .info {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .info b {
    font-size: 13.5px;
  }
  .info span {
    font-size: 11px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .empty {
    padding: 8px 0;
    font-size: 12.5px;
  }
</style>
