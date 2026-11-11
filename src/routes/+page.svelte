<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from '$lib/stores/app.svelte';
  import { i18n, t } from '$lib/i18n/index.svelte';
  import { api, isTauri } from '$lib/api';
  import { themeById } from '$lib/themes';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import StarField from '$lib/components/StarField.svelte';
  import Toasts from '$lib/components/Toasts.svelte';
  import Dashboard from '$lib/pages/Dashboard.svelte';
  import Lists from '$lib/pages/Lists.svelte';
  import Settings from '$lib/pages/Settings.svelte';
  import About from '$lib/pages/About.svelte';
  import Onboarding from '$lib/pages/Onboarding.svelte';

  onMount(() => {
    app.init();
  });

  const theme = $derived(themeById(app.config.theme));

  // Apply theme to <html>.
  $effect(() => {
    document.documentElement.dataset.theme = theme.id;
  });

  // Keep tray menu labels in sync with the interface language.
  $effect(() => {
    void i18n.lang;
    if (!app.ready || !isTauri()) return;
    api
      .setTrayLabels({
        open: t('tray.open'),
        connect: t('tray.connect'),
        disconnect: t('tray.disconnect'),
        quit: t('tray.quit'),
        tipOn: t('tray.tipOn'),
        tipOff: t('tray.tipOff')
      })
      .catch(() => {});
  });

  // Block the WebView context menu / reload shortcuts for a native feel.
  function onKey(e: KeyboardEvent) {
    if (e.key === 'F5' || (e.ctrlKey && e.key.toLowerCase() === 'r')) e.preventDefault();
  }
</script>

<svelte:window onkeydown={onKey} oncontextmenu={(e) => e.preventDefault()} />

{#key theme.id}
  {#if theme.stars}<StarField density={1} />{/if}
{/key}
{#if !theme.stars && !theme.light}<div class="ambient"></div>{/if}

<div class="shell">
  <Sidebar />
  <main class="content">
    <div class="inner">
      {#if app.page === 'dashboard'}
        <Dashboard />
      {:else if app.page === 'lists'}
        <Lists />
      {:else if app.page === 'settings'}
        <Settings />
      {:else}
        <About />
      {/if}
    </div>
  </main>
</div>

{#if app.ready && app.showOnboarding}
  <Onboarding />
{/if}

<Toasts />

<style>
  .shell {
    position: relative;
    z-index: 1;
    display: flex;
    height: 100vh;
  }
  .content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }
  .inner {
    max-width: 1320px;
    margin: 0 auto;
    padding: 34px 34px 40px;
  }
  .ambient {
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 0;
    background:
      radial-gradient(900px 500px at 85% -10%, var(--accent-soft), transparent 60%),
      radial-gradient(700px 500px at -10% 110%, var(--accent-soft), transparent 60%);
  }
</style>
