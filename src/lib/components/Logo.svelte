<script lang="ts">
  // Fork Zapret mark: shield with a lightning bolt and a small star (matches the app icon).
  interface Props {
    size?: number;
    glow?: boolean;
  }
  let { size = 44, glow = false }: Props = $props();
</script>

<span class="logo" class:glow style:width="{size}px" style:height="{size}px">
  <svg viewBox="0 0 64 64" width={size * 0.62} height={size * 0.62} aria-hidden="true">
    <path
      d="M32 5.5c5.8 4.4 12.6 6.8 20 7.2 1.1.1 2 1 2 2.1v14.6c0 13.5-8.8 23.6-21.2 28.9-.5.2-1.1.2-1.6 0C18.8 53 10 42.9 10 29.4V14.8c0-1.1.9-2 2-2.1 7.4-.4 14.2-2.8 20-7.2z"
      fill="none"
      stroke="currentColor"
      stroke-width="3.6"
      stroke-linejoin="round"
    />
    <path d="M35.5 15 22.5 34.5h9.5l-3 15L42 29.5h-9.8z" fill="currentColor" />
    <path d="M47 8.5l1.1 2.6 2.6 1.1-2.6 1.1L47 15.9l-1.1-2.6-2.6-1.1 2.6-1.1z" fill="currentColor" opacity=".85" />
  </svg>
</span>

<style>
  .logo {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    border-radius: 30%;
    background: var(--text);
    color: var(--bg);
    box-shadow: 0 0 0 1px var(--border-strong);
    transition: box-shadow 0.4s;
  }
  :global([data-theme='light']) .logo {
    background: #0f172a;
    color: #fff;
  }
  .glow {
    box-shadow:
      0 0 0 1px var(--border-strong),
      0 0 40px -6px var(--accent);
  }
</style>
