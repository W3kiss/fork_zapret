<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { api, isTauri, LINKS } from '$lib/api';
  import Icon from '$lib/components/Icon.svelte';

  const AVATAR_KEY = 'fz-avatar';
  let avatar = $state<string | null>(null);
  let avatarFailed = $state(false);

  // Cached copy first (instant), then a cache-busted refresh from GitHub so a changed
  // avatar shows up automatically. Falls back to the initials badge offline.
  $effect(() => {
    try {
      avatar = localStorage.getItem(AVATAR_KEY);
    } catch {
      /* storage unavailable */
    }
    if (!isTauri()) {
      avatar = `${LINKS.developer}.png?size=256&t=${Date.now()}`;
      return;
    }
    let alive = true;
    (async () => {
      try {
        const cached = await api.avatar(false);
        if (alive && cached) avatar = cached;
        const fresh = await api.avatar(true);
        if (alive && fresh) {
          avatar = fresh;
          try {
            localStorage.setItem(AVATAR_KEY, fresh);
          } catch {
            /* quota */
          }
        }
      } catch {
        /* offline */
      }
    })();
    return () => (alive = false);
  });

  const open = (url: string) => (isTauri() ? api.openUrl(url).catch(app.fail) : window.open(url, '_blank'));

  const features = [
    { icon: 'terminal', title: 'about.f1', desc: 'about.f1d' },
    { icon: 'wand', title: 'about.f2', desc: 'about.f2d' },
    { icon: 'history', title: 'about.f3', desc: 'about.f3d' }
  ];
  const tech = ['Tauri 2', 'Rust', 'Tokio', 'Svelte 5', 'SvelteKit', 'TypeScript', 'WinDivert', 'zapret'];
</script>

<section class="about fade-in">
  <div class="hero card">
    <div class="orbits">
      <span class="o o1"></span><span class="o o2"></span><span class="o o3"></span>
      <div class="avatar">
        {#if avatar && !avatarFailed}
          <img src={avatar} alt="W3kiss" onerror={() => (avatarFailed = true)} />
        {:else}
          <span class="initials">W3</span>
        {/if}
      </div>
    </div>
    <div class="role"><span></span>{t('about.role')}<span></span></div>
    <h1>W3kiss</h1>
    <p class="desc">{t('about.desc')}</p>
    <div class="links">
      <button class="btn primary" onclick={() => open(LINKS.developer)}>
        <Icon name="github" size={15} />{t('about.github')}<Icon name="external" size={13} />
      </button>
      <button class="btn" onclick={() => open(LINKS.zapret)}>
        <Icon name="shield" size={15} />{t('about.zapret')}<Icon name="external" size={13} />
      </button>
      <button class="btn" onclick={() => open(LINKS.zapretReleases)}>
        <Icon name="download" size={15} />{t('about.releases')}<Icon name="external" size={13} />
      </button>
    </div>
  </div>

  <div class="stats">
    <div class="card stat"><b>{app.strategies.length}</b><span>{t('about.statStrategies')}</span></div>
    <div class="card stat"><b>3</b><span>{t('about.statIpset')}</span></div>
    <div class="card stat"><b>{app.zapretVersion}</b><span>{t('about.statZapret')}</span></div>
    <div class="card stat"><b>{app.appVersion}</b><span>{t('about.statApp')}</span></div>
  </div>

  <div class="features">
    {#each features as f (f.title)}
      <div class="card feat">
        <span class="fi"><Icon name={f.icon} size={18} /></span>
        <h3>{t(f.title)}</h3>
        <p>{t(f.desc)}</p>
      </div>
    {/each}
  </div>

  <div class="tech">
    <div class="tlabel">{t('about.tech')}</div>
    <div class="chips">
      {#each tech as x (x)}<span class="chip">{x}</span>{/each}
    </div>
  </div>

  <div class="foot">
    <p>{t('about.core')}</p>
    <p class="faint">{t('about.disclaimer')}</p>
    <p>{t('about.thanks')}</p>
  </div>
</section>

<style>
  .about {
    max-width: 880px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .hero {
    position: relative;
    padding: 34px 30px 30px;
    text-align: center;
    overflow: hidden;
  }
  .orbits {
    position: relative;
    width: 150px;
    height: 150px;
    margin: 0 auto 10px;
    display: grid;
    place-items: center;
  }
  .o {
    position: absolute;
    border-radius: 50%;
    border: 1px solid var(--border);
  }
  .o1 {
    inset: 18px;
  }
  .o2 {
    inset: -20px;
  }
  .o3 {
    inset: -70px;
    border-color: color-mix(in srgb, var(--border) 60%, transparent);
  }
  .avatar {
    position: relative;
    width: 104px;
    height: 104px;
    border-radius: 50%;
    overflow: hidden;
    border: 1px solid var(--border-strong);
    background: var(--card-hover);
    display: grid;
    place-items: center;
  }
  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .initials {
    font-size: 30px;
    font-weight: 800;
    color: var(--text-2);
  }
  .role {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--text-2);
  }
  .role span {
    width: 26px;
    height: 1px;
    background: var(--border-strong);
  }
  h1 {
    margin: 8px 0 10px;
    font-size: 58px;
    font-weight: 800;
    letter-spacing: -0.04em;
    line-height: 1;
  }
  .desc {
    max-width: 520px;
    margin: 0 auto 22px;
    color: var(--text-2);
    line-height: 1.6;
  }
  .links {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: 8px;
  }
  .links .btn {
    border-radius: 999px;
    height: 40px;
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
  }
  .stat {
    padding: 22px 10px;
    text-align: center;
  }
  .stat b {
    display: block;
    font-size: 30px;
    font-weight: 800;
    letter-spacing: -0.02em;
  }
  .stat span {
    font-size: 12px;
    color: var(--text-2);
  }
  .features {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }
  .feat {
    padding: 20px;
  }
  .fi {
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: 12px;
    border: 1px solid var(--border-strong);
  }
  .feat h3 {
    margin: 16px 0 6px;
    font-size: 15px;
  }
  .feat p {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-2);
    line-height: 1.55;
  }
  .tech {
    text-align: center;
    margin-top: 6px;
  }
  .tlabel {
    font-size: 11px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--text-2);
    margin-bottom: 12px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
  }
  .foot {
    text-align: center;
    font-size: 12px;
    color: var(--text-2);
    line-height: 1.4;
  }
  .foot p {
    margin: 4px 0;
  }
</style>
