<script lang="ts">
  // Espace de travail : le plateau de la machine à l'échelle, sa grille en mm
  // (10 mm, et 50 mm plus marquée) et l'origine. Repère du moteur : mm,
  // origine en bas à gauche, Y vers le haut ; l'affichage retourne l'axe Y.
  // Vue : cadrée sur le plateau tant qu'on n'y a pas touché ; zoom à la
  // molette autour du pointeur ; déplacement avec l'outil Main, espace +
  // glisser ou le bouton du milieu ; `fit()` recadre (touche F).
  // Les dessins importés sont des objets de l'utilisateur : trait clair sur
  // liseré sombre, contours ouverts en attention, sélection en accent.
  // L'outil Sélection choisit le dessin dont un tracé passe sous le pointeur,
  // sinon celui dont la boîte le contient ; un clic dans le vide désélectionne.
  import { theme } from '../lib/theme.svelte';
  import { tools } from '../lib/tools.svelte';
  import type { Machine } from '../lib/engine.svelte';
  import type { Drawing } from '../lib/document.svelte';

  let {
    machine,
    drawings,
    selected,
    onselect,
    pointer = $bindable(null),
    zoom = $bindable(1),
  }: {
    machine: Machine | null;
    drawings: Drawing[];
    selected: number | null;
    onselect: (id: number | null) => void;
    /** Position du pointeur sur le plateau, en mm (null hors de la vue). */
    pointer?: [number, number] | null;
    /** Zoom : 1 = taille réelle à l'écran (96 px CSS par pouce). */
    zoom?: number;
  } = $props();

  const PX_PER_MM = 96 / 25.4;
  const MARGIN = 32;
  const MIN_SCALE = 0.05 * PX_PER_MM;
  const MAX_SCALE = 40 * PX_PER_MM;
  const TOKENS = ['canvas-bg', 'bed-fill', 'bed-edge', 'bed-grid', 'bed-grid-major', 'bed-origin', 'ctrl', 'ctrl-edge', 'ctrl-active', 'ctrl-active-fill', 'drawing-open'];
  /** Distance de prise d'un tracé, en px. */
  const HIT = 8;

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

    drawDrawings(ctx);
  }

  function tracePath(ctx: CanvasRenderingContext2D, d: Drawing, pts: number[]) {
    const { s, ox, oy } = view;
    const [dx, dy] = d.offset;
    for (let i = 0; i < pts.length; i += 2) {
      const x = ox + (pts[i] + dx) * s;
      const y = oy - (pts[i + 1] + dy) * s;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
  }

  function drawDrawings(ctx: CanvasRenderingContext2D) {
    ctx.lineJoin = 'round';
    ctx.lineCap = 'round';
    for (const d of drawings) {
      const sel = d.id === selected;
      if (sel) {
        const [x0, y0, x1, y1] = d.report.bbox;
        const [dx, dy] = d.offset;
        const r = [view.ox + (x0 + dx) * view.s, view.oy - (y1 + dy) * view.s, (x1 - x0) * view.s, (y1 - y0) * view.s];
        ctx.fillStyle = colors['ctrl-active-fill'];
        ctx.fillRect(r[0], r[1], r[2], r[3]);
        ctx.setLineDash([4, 4]);
        ctx.strokeStyle = colors['ctrl-active'];
        ctx.lineWidth = 1;
        ctx.strokeRect(Math.round(r[0]) - 3.5, Math.round(r[1]) - 3.5, Math.round(r[2]) + 7, Math.round(r[3]) + 7);
        ctx.setLineDash([]);
      }
      // Liseré sombre sous tous les tracés, puis le trait.
      ctx.beginPath();
      for (const tr of d.report.traces) tracePath(ctx, d, tr.pts);
      ctx.strokeStyle = colors['ctrl-edge'];
      ctx.lineWidth = 4;
      ctx.stroke();
      for (const open of [false, true]) {
        ctx.beginPath();
        for (const tr of d.report.traces) if (tr.open === open) tracePath(ctx, d, tr.pts);
        ctx.strokeStyle = open ? colors['drawing-open'] : sel ? colors['ctrl-active'] : colors['ctrl'];
        ctx.lineWidth = open ? 2 : 1.5;
        ctx.stroke();
      }
    }
  }

  /** Le dessin sous le point d'écran (sx, sy), le plus récent d'abord. */
  function hit(sx: number, sy: number): number | null {
    const { s, ox, oy } = view;
    const near = (d: Drawing) => {
      const [dx, dy] = d.offset;
      const X = (x: number) => ox + (x + dx) * s;
      const Y = (y: number) => oy - (y + dy) * s;
      for (const { pts: p } of d.report.traces) {
        for (let i = 0; i + 3 < p.length; i += 2) {
          const ax = X(p[i]);
          const ay = Y(p[i + 1]);
          const vx = X(p[i + 2]) - ax;
          const vy = Y(p[i + 3]) - ay;
          const l2 = vx * vx + vy * vy;
          const t = l2 > 0 ? Math.max(0, Math.min(1, ((sx - ax) * vx + (sy - ay) * vy) / l2)) : 0;
          if (Math.hypot(ax + vx * t - sx, ay + vy * t - sy) <= HIT) return true;
        }
      }
      return false;
    };
    const list = [...drawings].reverse();
    const onTrace = list.find(near);
    if (onTrace) return onTrace.id;
    const [wx, wy] = toWorld(sx, sy);
    const inside = list
      .filter((d) => {
        const [x0, y0, x1, y1] = d.report.bbox;
        const [dx, dy] = d.offset;
        return wx >= x0 + dx && wx <= x1 + dx && wy >= y0 + dy && wy <= y1 + dy;
      })
      .sort((a, b) => a.report.width * a.report.height - b.report.width * b.report.height);
    return inside[0]?.id ?? null;
  }

  $effect(() => {
    void theme.applied;
    readColors();
    request();
  });

  $effect(() => {
    void [view, width, height, machine, drawings, selected];
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
    if (e.button === 0 && !panning && tools.active === 'select') {
      const [sx, sy] = local(e);
      onselect(hit(sx, sy));
      return;
    }
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
