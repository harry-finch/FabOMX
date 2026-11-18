<script lang="ts">
  // Espace de travail : le plateau de la machine à l'échelle, sa grille en mm
  // (10 mm, et 50 mm plus marquée) et l'origine. Repère du moteur : mm,
  // origine en bas à gauche, Y vers le haut ; l'affichage retourne l'axe Y.
  // Vue : cadrée sur le plateau tant qu'on n'y a pas touché ; zoom à la
  // molette autour du pointeur ; déplacement avec l'outil Main, espace +
  // glisser ou le bouton du milieu ; `fit()` recadre (touche F).
  import { theme } from '../lib/theme.svelte';
  import { tools } from '../lib/tools.svelte';
  import type { Machine } from '../lib/engine.svelte';

  let {
    machine,
    pointer = $bindable(null),
    zoom = $bindable(1),
  }: {
    machine: Machine | null;
    /** Position du pointeur sur le plateau, en mm (null hors de la vue). */
    pointer?: [number, number] | null;
    /** Zoom : 1 = taille réelle à l'écran (96 px CSS par pouce). */
    zoom?: number;
  } = $props();

  const PX_PER_MM = 96 / 25.4;
  const MARGIN = 32;
  const MIN_SCALE = 0.05 * PX_PER_MM;
  const MAX_SCALE = 40 * PX_PER_MM;
  const TOKENS = ['canvas-bg', 'bed-fill', 'bed-edge', 'bed-grid', 'bed-grid-major', 'bed-origin'];

  let host: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  let width = $state(0);
  let height = $state(0);
  // Vue : s px par mm ; (ox, oy) position à l'écran de l'origine du plateau.
  let view = $state({ s: 1, ox: 0, oy: 0 });
  let auto = $state(true);
  let space = $state(false);
  let drag: { x: number; y: number; ox: number; oy: number } | null = $state(null);
  let colors: Record<string, string> = {};
  let frame = 0;

  const toWorld = (sx: number, sy: number): [number, number] => [(sx - view.ox) / view.s, (view.oy - sy) / view.s];

  /** Recadre la vue sur le plateau et reprend le cadrage automatique. */
  export function fit() {
    auto = true;
    frameBed();
  }

  function frameBed() {
    if (!machine || !width || !height) return;
    const s = Math.max(MIN_SCALE, Math.min((width - 2 * MARGIN) / machine.width, (height - 2 * MARGIN) / machine.height));
    view = { s, ox: (width - machine.width * s) / 2, oy: (height + machine.height * s) / 2 };
  }

  function readColors() {
    const cs = getComputedStyle(document.documentElement);
    colors = Object.fromEntries(TOKENS.map((k) => [k, cs.getPropertyValue(`--${k}`).trim()]));
  }

  function request() {
    if (!frame) frame = requestAnimationFrame(draw);
  }

  function gridLines(ctx: CanvasRenderingContext2D, m: Machine, step: number) {
    const { s, ox, oy } = view;
    ctx.beginPath();
    for (let x = step; x < m.width; x += step) {
      const sx = Math.round(ox + x * s) + 0.5;
      ctx.moveTo(sx, oy);
      ctx.lineTo(sx, oy - m.height * s);
    }
    for (let y = step; y < m.height; y += step) {
      const sy = Math.round(oy - y * s) + 0.5;
      ctx.moveTo(ox, sy);
      ctx.lineTo(ox + m.width * s, sy);
    }
    ctx.stroke();
  }

  function draw() {
    frame = 0;
    const ctx = canvas?.getContext('2d');
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.fillStyle = colors['canvas-bg'];
    ctx.fillRect(0, 0, width, height);
    if (!machine) return;
    const { s, ox, oy } = view;
    const w = machine.width * s;
    const h = machine.height * s;

    ctx.fillStyle = colors['bed-fill'];
    ctx.fillRect(ox, oy - h, w, h);
    ctx.lineWidth = 1;
    if (10 * s >= 6) {
      ctx.strokeStyle = colors['bed-grid'];
      gridLines(ctx, machine, 10);
    }
    ctx.strokeStyle = colors['bed-grid-major'];
    gridLines(ctx, machine, 50);
    ctx.strokeStyle = colors['bed-edge'];
    ctx.strokeRect(Math.round(ox) + 0.5, Math.round(oy - h) + 0.5, Math.round(w), Math.round(h));

    // Origine : deux traits le long de +X et +Y, et un point.
    const x0 = Math.round(ox) + 0.5;
    const y0 = Math.round(oy) + 0.5;
    ctx.strokeStyle = colors['bed-origin'];
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(x0 + 18, y0);
    ctx.lineTo(x0, y0);
    ctx.lineTo(x0, y0 - 18);
    ctx.stroke();
    ctx.fillStyle = colors['bed-origin'];
    ctx.beginPath();
    ctx.arc(x0, y0, 3, 0, Math.PI * 2);
    ctx.fill();
  }

  $effect(() => {
    void theme.applied;
    readColors();
    request();
  });

  $effect(() => {
    void [view, width, height, machine];
    zoom = view.s / PX_PER_MM;
    request();
  });

  $effect(() => {
    void [machine, width, height];
    if (auto) frameBed();
  });

  $effect(() => {
    const ro = new ResizeObserver(([entry]) => {
      width = entry.contentRect.width;
      height = entry.contentRect.height;
    });
    ro.observe(host);
    return () => ro.disconnect();
  });

  $effect(() => {
    const down = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      if (e.code === 'Space' && !(el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable))) {
        space = true;
        if (el?.tagName !== 'BUTTON') e.preventDefault();
      }
    };
    const up = (e: KeyboardEvent) => {
      if (e.code === 'Space') space = false;
    };
    const blur = () => (space = false);
    window.addEventListener('keydown', down);
    window.addEventListener('keyup', up);
    window.addEventListener('blur', blur);
    return () => {
      window.removeEventListener('keydown', down);
      window.removeEventListener('keyup', up);
      window.removeEventListener('blur', blur);
    };
  });

  const panning = $derived(tools.active === 'hand' || space);

  function local(e: MouseEvent): [number, number] {
    const r = canvas.getBoundingClientRect();
    return [e.clientX - r.left, e.clientY - r.top];
  }

  function wheel(e: WheelEvent) {
    e.preventDefault();
    const [sx, sy] = local(e);
    const delta = e.deltaMode === 1 ? e.deltaY * 16 : e.deltaY;
    const s = Math.min(MAX_SCALE, Math.max(MIN_SCALE, view.s * Math.exp(-delta * 0.0015)));
    const k = s / view.s;
    view = { s, ox: sx - (sx - view.ox) * k, oy: sy - (sy - view.oy) * k };
    auto = false;
  }

  function pointerdown(e: PointerEvent) {
    if (!(e.button === 1 || (e.button === 0 && panning))) return;
    e.preventDefault();
    canvas.setPointerCapture(e.pointerId);
    drag = { x: e.clientX, y: e.clientY, ox: view.ox, oy: view.oy };
  }

  function pointermove(e: PointerEvent) {
    const [sx, sy] = local(e);
    pointer = toWorld(sx, sy);
    if (!drag) return;
    view = { s: view.s, ox: drag.ox + e.clientX - drag.x, oy: drag.oy + e.clientY - drag.y };
    auto = false;
  }

  function pointerup(e: PointerEvent) {
    if (drag) canvas.releasePointerCapture(e.pointerId);
    drag = null;
  }
</script>

<div class="host" bind:this={host}>
  <canvas
    bind:this={canvas}
    class:grab={panning}
    class:grabbing={drag}
    style:width="{width}px"
    style:height="{height}px"
    onwheel={wheel}
    onpointerdown={pointerdown}
    onpointermove={pointermove}
    onpointerup={pointerup}
    onpointercancel={pointerup}
    onpointerleave={() => (pointer = null)}
  ></canvas>
</div>

<style>
  .host {
    position: relative;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    background: var(--canvas-bg);
  }

  canvas {
    position: absolute;
    inset: 0;
    display: block;
    touch-action: none;
  }

  canvas.grab {
    cursor: grab;
  }

  canvas.grabbing {
    cursor: grabbing;
  }
</style>
