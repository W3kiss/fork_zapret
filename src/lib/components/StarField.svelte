<script lang="ts">
  // Animated twinkling star field with cursor parallax, constellation lines near the
  // pointer and occasional shooting stars. Rendered on a single canvas.
  interface Props {
    density?: number;
    interactive?: boolean;
  }
  let { density = 1, interactive = true }: Props = $props();

  let canvas: HTMLCanvasElement;

  interface Star {
    x: number;
    y: number;
    z: number; // depth 0.2..1 — parallax & size
    r: number;
    phase: number;
    speed: number;
  }

  $effect(() => {
    const ctx = canvas.getContext('2d')!;
    let w = 0,
      h = 0,
      dpr = 1;
    let stars: Star[] = [];
    let raf = 0;
    let mx = 0.5,
      my = 0.5,
      tx = 0.5,
      ty = 0.5;
    let px = -9999,
      py = -9999;
    let shooting: { x: number; y: number; vx: number; vy: number; life: number } | null = null;
    const rgb = getComputedStyle(document.documentElement).getPropertyValue('--star').trim() || '255,255,255';

    const resize = () => {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      w = canvas.clientWidth;
      h = canvas.clientHeight;
      canvas.width = w * dpr;
      canvas.height = h * dpr;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      const count = Math.round(((w * h) / 5200) * density);
      stars = Array.from({ length: count }, () => {
        const z = 0.2 + Math.random() * 0.8;
        return {
          x: Math.random() * w,
          y: Math.random() * h,
          z,
          r: 0.35 + z * 1.05 * Math.random() + 0.15,
          phase: Math.random() * Math.PI * 2,
          speed: 0.6 + Math.random() * 1.8
        };
      });
    };

    const onMove = (e: PointerEvent) => {
      tx = e.clientX / Math.max(w, 1);
      ty = e.clientY / Math.max(h, 1);
      px = e.clientX;
      py = e.clientY;
    };
    const onLeave = () => {
      px = py = -9999;
    };

    const draw = (time: number) => {
      const t = time / 1000;
      mx += (tx - mx) * 0.04;
      my += (ty - my) * 0.04;
      const ox = (mx - 0.5) * 38;
      const oy = (my - 0.5) * 38;
      ctx.clearRect(0, 0, w, h);

      const near: { x: number; y: number }[] = [];
      for (const s of stars) {
        const x = (((s.x + ox * s.z) % w) + w) % w;
        const y = (((s.y + oy * s.z) % h) + h) % h;
        const tw = 0.55 + 0.45 * Math.sin(t * s.speed + s.phase);
        const a = (0.25 + 0.75 * s.z) * tw;
        ctx.beginPath();
        ctx.arc(x, y, s.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${rgb},${a.toFixed(3)})`;
        ctx.fill();
        if (s.r > 1.05 && tw > 0.92) {
          ctx.fillStyle = `rgba(${rgb},${(a * 0.18).toFixed(3)})`;
          ctx.fillRect(x - s.r * 3, y - 0.3, s.r * 6, 0.6);
          ctx.fillRect(x - 0.3, y - s.r * 3, 0.6, s.r * 6);
        }
        if (interactive) {
          const dx = x - px,
            dy = y - py;
          if (dx * dx + dy * dy < 150 * 150 && near.length < 14) near.push({ x, y });
        }
      }

      // Constellation lines around the cursor.
      if (near.length > 1) {
        ctx.lineWidth = 0.6;
        for (let i = 0; i < near.length; i++) {
          for (let j = i + 1; j < near.length; j++) {
            const dx = near[i].x - near[j].x,
              dy = near[i].y - near[j].y;
            const d = Math.sqrt(dx * dx + dy * dy);
            if (d < 95) {
              ctx.strokeStyle = `rgba(${rgb},${(0.16 * (1 - d / 95)).toFixed(3)})`;
              ctx.beginPath();
              ctx.moveTo(near[i].x, near[i].y);
              ctx.lineTo(near[j].x, near[j].y);
              ctx.stroke();
            }
          }
        }
      }

      // Shooting star.
      if (!shooting && Math.random() < 0.0025) {
        shooting = { x: Math.random() * w * 0.8, y: Math.random() * h * 0.4, vx: 7 + Math.random() * 4, vy: 2.5 + Math.random() * 2, life: 1 };
      }
      if (shooting) {
        const s = shooting;
        const g = ctx.createLinearGradient(s.x, s.y, s.x - s.vx * 12, s.y - s.vy * 12);
        g.addColorStop(0, `rgba(${rgb},${(0.8 * s.life).toFixed(3)})`);
        g.addColorStop(1, `rgba(${rgb},0)`);
        ctx.strokeStyle = g;
        ctx.lineWidth = 1.2;
        ctx.beginPath();
        ctx.moveTo(s.x, s.y);
        ctx.lineTo(s.x - s.vx * 12, s.y - s.vy * 12);
        ctx.stroke();
        s.x += s.vx;
        s.y += s.vy;
        s.life -= 0.018;
        if (s.life <= 0 || s.x > w + 100) shooting = null;
      }

      raf = requestAnimationFrame(draw);
    };

    resize();
    raf = requestAnimationFrame(draw);
    window.addEventListener('resize', resize);
    window.addEventListener('pointermove', onMove);
    document.addEventListener('pointerleave', onLeave);
    const onVis = () => {
      cancelAnimationFrame(raf);
      if (!document.hidden) raf = requestAnimationFrame(draw);
    };
    document.addEventListener('visibilitychange', onVis);

    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener('resize', resize);
      window.removeEventListener('pointermove', onMove);
      document.removeEventListener('pointerleave', onLeave);
      document.removeEventListener('visibilitychange', onVis);
    };
  });
</script>

<canvas bind:this={canvas} class="stars" aria-hidden="true"></canvas>

<style>
  .stars {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100vh;
    pointer-events: none;
    z-index: 0;
  }
</style>
