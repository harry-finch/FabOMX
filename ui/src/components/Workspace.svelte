<script lang="ts">
  // Espace de travail : le plateau de la machine à l'échelle, sa grille en mm
  // (10 mm, et 50 mm plus marquée) et l'origine. Repère du moteur : mm,
  // origine en bas à gauche, Y vers le haut ; l'affichage retourne l'axe Y.
  // Vue : cadrée sur le plateau tant qu'on n'y a pas touché ; zoom à la
  // molette autour du pointeur ; déplacement avec l'outil Main, espace +
  // glisser ou le bouton du milieu ; `fit()` recadre (touche F).
  // Les dessins importés sont des objets de l'utilisateur : trait clair sur
  // liseré sombre, contours ouverts en attention, sélection en accent.
  // Un clic choisit le dessin dont un tracé passe sous le pointeur, sinon
  // celui dont la boîte le contient ; un clic dans le vide désélectionne.
  // Outils : Sélection et Déplacer (glisser le dessin, au-delà de 3 px pour
  // qu'un simple clic ne fasse que sélectionner), Tourner (glisser autour du point
  // de référence, Maj par 15°), Mettre à l'échelle (poignées de la boîte ;
  // côtés seulement cadenas ouvert). Un geste = une étape d'annulation.
  // « Choisir sur le plateau » : le clic suivant place le point de référence,
  // accroché aux points du dessin à HIT px près. Un dessin qui déborde du
  // plateau a sa boîte en attention.
  // Outil Découpe : chaque contour à la couleur de son côté (repères en
  // pointillés), une flèche pour son sens de parcours, les croisements
  // marqués ; un clic choisit un contour, Maj + clic en ajoute ou en retire
  // un ; un dessin qui en chevauche un autre a sa boîte en attention.
  import { untrack } from 'svelte';
  import { theme } from '../lib/theme.svelte';
  import { tools } from '../lib/tools.svelte';
  import type { Machine } from '../lib/engine.svelte';
  import { doc, refPoint, type Drawing } from '../lib/document.svelte';
  import { cleaning } from '../lib/cleaning.svelte';
  import { cutting, reversed } from '../lib/cutting.svelte';
  import type { Side } from '../lib/types';
  import { apply, invert, rotateAbout, scaleAbout, then, translate, type M, type Pt } from '../lib/affine';

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
  const TOKENS = ['canvas-bg', 'bed-fill', 'bed-edge', 'bed-grid', 'bed-grid-major', 'bed-origin', 'ctrl', 'ctrl-edge', 'ctrl-active', 'ctrl-active-fill', 'drawing-open', 'cut-outside', 'cut-inside', 'cut-line', 'cut-skip'];
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
  /** Geste en cours sur un dessin : départ (mm), matrice et point de référence au départ. */
  type Gesture = { kind: 'move' | 'rotate' | 'scale'; id: number; p0: Pt; m0: M; ref: Pt; hx: number; hy: number; s0: Pt; moved: boolean };
  let gesture = $state<Gesture | null>(null);
  /** Seuil (px) avant qu'un clic sur un dessin devienne un déplacement. */
  const DRAG = 3;
  const drawings = $derived(doc.drawings);
  const selected = $derived(doc.selected);
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

  const sx = (x: number) => view.ox + x * view.s;
  const sy = (y: number) => view.oy - y * view.s;

  function tracePath(ctx: CanvasRenderingContext2D, m: M, pts: number[]) {
    for (let i = 0; i < pts.length; i += 2) {
      const [x, y] = apply(m, pts[i], pts[i + 1]);
      if (i === 0) ctx.moveTo(sx(x), sy(y));
      else ctx.lineTo(sx(x), sy(y));
    }
  }

  const overflows = (d: Drawing) =>
    d.placement.warnings.some((w) => w.code === 'place.overflow') || (tools.active === 'cut' && cutting.overlaps.some(([a, b]) => a === d.id || b === d.id));

  /** Poignées d'échelle : coins, et côtés quand le cadenas est ouvert. `(hx, hy)` de 0 à 2 depuis le bas gauche. */
  function handles(d: Drawing): { hx: number; hy: number; x: number; y: number }[] {
    const [x0, y0, x1, y1] = d.placement.bbox;
    const out = [];
    for (const hx of [0, 1, 2])
      for (const hy of [0, 1, 2]) {
        const corner = hx !== 1 && hy !== 1;
        const side = (hx === 1) !== (hy === 1);
        if (corner || (side && !doc.lock)) out.push({ hx, hy, x: x0 + (hx * (x1 - x0)) / 2, y: y0 + (hy * (y1 - y0)) / 2 });
      }
    return out;
  }

  function drawDrawings(ctx: CanvasRenderingContext2D) {
    ctx.lineJoin = 'round';
    ctx.lineCap = 'round';
    for (const d of drawings) {
      const sel = d.id === selected;
      const over = overflows(d);
      if (sel || over) {
        const [x0, y0, x1, y1] = d.placement.bbox;
        const r = [sx(x0), sy(y1), (x1 - x0) * view.s, (y1 - y0) * view.s];
        if (sel) {
          ctx.fillStyle = colors['ctrl-active-fill'];
          ctx.fillRect(r[0], r[1], r[2], r[3]);
        }
        ctx.setLineDash([4, 4]);
        ctx.strokeStyle = over ? colors['drawing-open'] : colors['ctrl-active'];
        ctx.lineWidth = 1;
        ctx.strokeRect(Math.round(r[0]) - 3.5, Math.round(r[1]) - 3.5, Math.round(r[2]) + 7, Math.round(r[3]) + 7);
        ctx.setLineDash([]);
      }
      if (tools.active === 'cut' && cutting.sides(d)) {
        drawSides(ctx, d);
        continue;
      }
      // Liseré sous tous les tracés, puis le trait.
      ctx.beginPath();
      for (const tr of d.report.traces) tracePath(ctx, d.matrix, tr.pts);
      ctx.strokeStyle = colors['ctrl-edge'];
      ctx.lineWidth = 4;
      ctx.stroke();
      for (const open of [false, true]) {
        ctx.beginPath();
        for (const tr of d.report.traces) if (tr.open === open) tracePath(ctx, d.matrix, tr.pts);
        ctx.strokeStyle = open ? colors['drawing-open'] : sel ? colors['ctrl-active'] : colors['ctrl'];
        ctx.lineWidth = open ? 2 : 1.5;
        ctx.stroke();
      }
    }
    const d = doc.selection;
    if (!d) return;
    if (tools.active === 'clean') return drawDefects(ctx, d);
    if (tools.active === 'cut') return drawCrossings(ctx, d);
    // Point de référence : un cercle et sa croix.
    const [rx, ry] = refPoint(d);
    const [px, py] = [sx(rx), sy(ry)];
    ctx.strokeStyle = colors['ctrl-edge'];
    ctx.lineWidth = 4;
    ctx.beginPath();
    ctx.arc(px, py, 5, 0, Math.PI * 2);
    ctx.stroke();
    ctx.strokeStyle = colors['ctrl-active'];
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.arc(px, py, 5, 0, Math.PI * 2);
    ctx.moveTo(px - 9, py);
    ctx.lineTo(px + 9, py);
    ctx.moveTo(px, py - 9);
    ctx.lineTo(px, py + 9);
    ctx.stroke();
    if (tools.active === 'scale') {
      for (const h of handles(d)) {
        ctx.fillStyle = colors['ctrl'];
        ctx.strokeStyle = colors['ctrl-active'];
        ctx.lineWidth = 1.5;
        ctx.fillRect(Math.round(sx(h.x)) - 4, Math.round(sy(h.y)) - 4, 8, 8);
        ctx.strokeRect(Math.round(sx(h.x)) - 4.5, Math.round(sy(h.y)) - 4.5, 9, 9);
      }
    }
  }

  function contourPath(ctx: CanvasRenderingContext2D, m: M, pts: number[], closed: boolean) {
    tracePath(ctx, m, pts);
    if (closed) ctx.closePath();
  }

  /** Outil Découpe : les contours d'un dessin à la couleur de leur côté, la sélection en accent dessous. */
  function drawSides(ctx: CanvasRenderingContext2D, d: Drawing) {
    const s = cutting.sides(d)!;
    const picked = cutting.selection?.id === d.id ? cutting.selection.contours : [];
    ctx.beginPath();
    for (const c of s.contours) contourPath(ctx, d.matrix, c.pts, c.closed);
    ctx.strokeStyle = colors['ctrl-edge'];
    ctx.lineWidth = 4;
    ctx.stroke();
    if (picked.length) {
      ctx.beginPath();
      for (const i of picked) if (s.contours[i]) contourPath(ctx, d.matrix, s.contours[i].pts, s.contours[i].closed);
      ctx.strokeStyle = colors['ctrl-active'];
      ctx.lineWidth = 6;
      ctx.stroke();
    }
    const sides: Side[] = ['outside', 'inside', 'line', 'skip'];
    for (const side of sides) {
      ctx.beginPath();
      s.contours.forEach((c, i) => {
        if (cutting.sideOf(d, i) === side) contourPath(ctx, d.matrix, c.pts, c.closed);
      });
      ctx.strokeStyle = colors[`cut-${side}`];
      ctx.lineWidth = 2;
      ctx.setLineDash(side === 'skip' ? [6, 4] : []);
      ctx.stroke();
    }
    ctx.setLineDash([]);
    s.contours.forEach((c, i) => {
      const side = cutting.sideOf(d, i);
      if (side !== 'skip') arrow(ctx, d.matrix, c.pts, c.closed, reversed(side, c.area, d.matrix), colors[`cut-${side}`]);
    });
  }

  /** Flèche du sens de parcours, au milieu de l'arête la plus longue du contour (à l'écran). */
  function arrow(ctx: CanvasRenderingContext2D, m: M, pts: number[], closed: boolean, back: boolean, color: string) {
    const scr: Pt[] = [];
    for (let i = 0; i + 1 < pts.length; i += 2) {
      const [x, y] = apply(m, pts[i], pts[i + 1]);
      scr.push([sx(x), sy(y)]);
    }
    if (closed && scr.length) scr.push(scr[0]);
    let total = 0;
    let best = -1;
    let len = 0;
    for (let i = 1; i < scr.length; i++) {
      const l = Math.hypot(scr[i][0] - scr[i - 1][0], scr[i][1] - scr[i - 1][1]);
      total += l;
      if (l > len) [best, len] = [i, l];
    }
    if (total < 40 || best < 0) return;
    const [ax, ay] = scr[best - 1];
    const [bx, by] = scr[best];
    const [px, py] = [(ax + bx) / 2, (ay + by) / 2];
    let [ux, uy] = [(bx - ax) / len, (by - ay) / len];
    if (back) [ux, uy] = [-ux, -uy];
    // Pointe en avant du milieu, ailes en arrière.
    const size = 7;
    const tip: Pt = [px + (size / 2) * ux, py + (size / 2) * uy];
    const wing = (sgn: number): Pt => [tip[0] - size * ux - sgn * size * 0.6 * uy, tip[1] - size * uy + sgn * size * 0.6 * ux];
    ctx.beginPath();
    ctx.moveTo(...wing(1));
    ctx.lineTo(...tip);
    ctx.lineTo(...wing(-1));
    ctx.strokeStyle = colors['ctrl-edge'];
    ctx.lineWidth = 4;
    ctx.stroke();
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.stroke();
  }

  /** Outil Découpe : les croisements du dessin sélectionné, celui montré en accent. */
  function drawCrossings(ctx: CanvasRenderingContext2D, d: Drawing) {
    for (const w of cutting.sides(d)?.warnings ?? []) {
      const [wx, wy] = apply(d.matrix, w.x, w.y);
      const f = cutting.focused;
      const on = f?.id === d.id && f.x === w.x && f.y === w.y;
      marker(ctx, sx(wx), sy(wy), on);
    }
  }

  function marker(ctx: CanvasRenderingContext2D, x: number, y: number, on: boolean) {
    const r = on ? 9 : 5;
    ctx.lineWidth = 4;
    ctx.strokeStyle = colors['ctrl-edge'];
    ctx.beginPath();
    ctx.arc(x, y, r, 0, Math.PI * 2);
    ctx.stroke();
    ctx.lineWidth = on ? 2 : 1.5;
    ctx.strokeStyle = on ? colors['ctrl-active'] : colors['drawing-open'];
    ctx.beginPath();
    ctx.arc(x, y, r, 0, Math.PI * 2);
    ctx.stroke();
  }

  /** Outil Nettoyer : les défauts de l'aperçu, le défaut choisi en accent. */
  function drawDefects(ctx: CanvasRenderingContext2D, d: Drawing) {
    const pv = cleaning.preview;
    if (!pv || pv.geo !== d.geo) return;
    pv.preview.defects.forEach((x, i) => {
      const [wx, wy] = apply(d.matrix, x.x, x.y);
      marker(ctx, sx(wx), sy(wy), cleaning.focused?.index === i);
    });
  }

  // Un défaut choisi dans la liste : la vue se centre dessus, assez près pour le voir.
  $effect(() => {
    const f = cleaning.focused;
    const d = doc.selection;
    if (!f || !d) return;
    const [wx, wy] = apply(d.matrix, f.defect.x, f.defect.y);
    untrack(() => {
      const s = Math.max(view.s, 20 * PX_PER_MM);
      view = { s, ox: width / 2 - wx * s, oy: height / 2 + wy * s };
      auto = false;
    });
  });

  // Un croisement choisi dans la liste : la vue se centre dessus.
  $effect(() => {
    const f = cutting.focused;
    const d = f && doc.drawings.find((x) => x.id === f.id);
    if (!f || !d) return;
    const [wx, wy] = apply(d.matrix, f.x, f.y);
    untrack(() => {
      const s = Math.max(view.s, 20 * PX_PER_MM);
      view = { s, ox: width / 2 - wx * s, oy: height / 2 + wy * s };
      auto = false;
    });
  });

  /** Distance (px) du point d'écran (px, py) au tracé `p` du dessin de matrice `m`. */
  function distance(m: M, p: number[], closed: boolean, px: number, py: number): number {
    let best = Infinity;
    const n = p.length / 2;
    const edges = closed ? n : n - 1;
    let [ax, ay] = apply(m, p[0], p[1]);
    for (let k = 1; k <= edges; k++) {
      const j = (k % n) * 2;
      const [bx, by] = apply(m, p[j], p[j + 1]);
      const [x0, y0, x1, y1] = [sx(ax), sy(ay), sx(bx), sy(by)];
      const vx = x1 - x0;
      const vy = y1 - y0;
      const l2 = vx * vx + vy * vy;
      const t = l2 > 0 ? Math.max(0, Math.min(1, ((px - x0) * vx + (py - y0) * vy) / l2)) : 0;
      best = Math.min(best, Math.hypot(x0 + vx * t - px, y0 + vy * t - py));
      [ax, ay] = [bx, by];
    }
    return best;
  }

  /** Le contour (outil Découpe) le plus proche sous le point d'écran, le dessin le plus récent d'abord. */
  function hitContour(px: number, py: number): { d: Drawing; i: number } | null {
    for (const d of [...drawings].reverse()) {
      const s = cutting.sides(d);
      if (!s) continue;
      let best: { i: number; k: number } | null = null;
      s.contours.forEach((c, i) => {
        const k = distance(d.matrix, c.pts, c.closed, px, py);
        if (k <= HIT && (!best || k < best.k)) best = { i, k };
      });
      if (best) return { d, i: (best as { i: number }).i };
    }
    return null;
  }

  /** Le dessin sous le point d'écran (px, py), le plus récent d'abord. */
  function hit(px: number, py: number): number | null {
    const near = (d: Drawing) => d.report.traces.some(({ pts }) => distance(d.matrix, pts, false, px, py) <= HIT);
    const list = [...drawings].reverse();
    const onTrace = list.find(near);
    if (onTrace) return onTrace.id;
    const [wx, wy] = toWorld(px, py);
    const inside = list
      .filter((d) => {
        const [x0, y0, x1, y1] = d.placement.bbox;
        return wx >= x0 && wx <= x1 && wy >= y0 && wy <= y1;
      })
      .sort((a, b) => a.placement.width * a.placement.height - b.placement.width * b.placement.height);
    return inside[0]?.id ?? null;
  }

  /** Le point de référence choisi par un clic : accroché au point du dessin le plus proche. */
  function pickRef(d: Drawing, px: number, py: number) {
    let best: [number, number] | null = null;
    let dist = HIT;
    for (let i = 0; i + 1 < d.snaps.length; i += 2) {
      const [x, y] = apply(d.matrix, d.snaps[i], d.snaps[i + 1]);
      const k = Math.hypot(sx(x) - px, sy(y) - py);
      if (k <= dist) {
        dist = k;
        best = [d.snaps[i], d.snaps[i + 1]];
      }
    }
    const [x, y] = best ?? apply(invert(d.matrix), ...toWorld(px, py));
    doc.update(d.id, { ref: { kind: 'point', x, y } });
    doc.picking = false;
  }

  $effect(() => {
    void theme.applied;
    readColors();
    request();
  });

  $effect(() => {
    void [view, width, height, machine, drawings, selected, tools.active, doc.lock, cleaning.preview, cleaning.focused, cutting.byGeo, cutting.selection, cutting.overlaps, cutting.focused];
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
    if (e.button === 0 && !panning) {
      const [px, py] = local(e);
      const sel = doc.selection;
      if (doc.picking && sel) return pickRef(sel, px, py);
      const tool = tools.active;
      if (tool === 'scale' && sel) {
        const h = handles(sel).find((h) => Math.abs(sx(h.x) - px) <= HIT && Math.abs(sy(h.y) - py) <= HIT);
        if (h) return start(e, 'scale', sel, h.hx, h.hy);
      }
      if (tool === 'cut') {
        const c = hitContour(px, py);
        if (!c) {
          cutting.clear();
          doc.select(hit(px, py));
          return;
        }
        if (c.d.id !== doc.selected) doc.select(c.d.id);
        cutting.pick(c.d, c.i, e.shiftKey);
        return;
      }
      const id = hit(px, py);
      if ((tool === 'move' || tool === 'select') && id != null) {
        doc.select(id);
        return start(e, 'move', doc.selection!);
      }
      if (tool === 'rotate' && sel && (id == null || id === sel.id)) return start(e, 'rotate', sel);
      doc.select(id);
      return;
    }
    if (!(e.button === 1 || (e.button === 0 && panning))) return;
    e.preventDefault();
    canvas.setPointerCapture(e.pointerId);
    drag = { x: e.clientX, y: e.clientY, ox: view.ox, oy: view.oy };
  }

  function start(e: PointerEvent, kind: 'move' | 'rotate' | 'scale', d: Drawing, hx = 0, hy = 0) {
    e.preventDefault();
    canvas.setPointerCapture(e.pointerId);
    doc.begin();
    const s0 = local(e);
    gesture = { kind, id: d.id, p0: toWorld(...s0), m0: d.matrix, ref: refPoint(d), hx, hy, s0, moved: kind !== 'move' };
  }

  function follow(e: PointerEvent, p: Pt) {
    const g = gesture!;
    if (!g.moved) {
      const [px, py] = local(e);
      if (Math.hypot(px - g.s0[0], py - g.s0[1]) < DRAG) return;
      g.moved = true;
    }
    const [dx, dy] = [p[0] - g.p0[0], p[1] - g.p0[1]];
    let m: M = g.m0;
    if (g.kind === 'move') m = then(translate(dx, dy), g.m0);
    else if (g.kind === 'rotate') {
      const a = (Math.atan2(p[1] - g.ref[1], p[0] - g.ref[0]) - Math.atan2(g.p0[1] - g.ref[1], g.p0[0] - g.ref[0])) * (180 / Math.PI);
      const deg = e.shiftKey ? Math.round(a / 15) * 15 : a;
      m = then(rotateAbout(g.ref, deg), g.m0);
    } else {
      // Échelle : rapport des distances au point de référence, par axe.
      const ratio = (now: number, then0: number) => (Math.abs(then0) > 1e-9 ? now / then0 : 1);
      const v0 = [g.p0[0] - g.ref[0], g.p0[1] - g.ref[1]];
      const v = [p[0] - g.ref[0], p[1] - g.ref[1]];
      let kx = g.hx === 1 ? 1 : ratio(v[0], v0[0]);
      let ky = g.hy === 1 ? 1 : ratio(v[1], v0[1]);
      if (doc.lock) {
        const l2 = v0[0] * v0[0] + v0[1] * v0[1];
        kx = ky = l2 > 1e-12 ? (v[0] * v0[0] + v[1] * v0[1]) / l2 : 1;
      }
      const floor = (k: number) => (Math.abs(k) < 0.001 ? (k < 0 ? -0.001 : 0.001) : k);
      m = then(scaleAbout(g.ref, floor(kx), floor(ky)), g.m0);
    }
    doc.update(g.id, { matrix: m });
  }

  function pointermove(e: PointerEvent) {
    const [px, py] = local(e);
    pointer = toWorld(px, py);
    if (gesture) return follow(e, pointer);
    if (!drag) return;
    view = { s: view.s, ox: drag.ox + e.clientX - drag.x, oy: drag.oy + e.clientY - drag.y };
    auto = false;
  }

  function pointerup(e: PointerEvent) {
    if (drag || gesture) canvas.releasePointerCapture(e.pointerId);
    if (gesture) doc.end();
    drag = null;
    gesture = null;
  }

  const cursor = $derived(
    doc.picking ? 'crosshair' : tools.active === 'move' || gesture?.kind === 'move' ? 'move' : tools.active === 'rotate' && selected != null ? 'alias' : undefined,
  );
</script>

<div class="host" bind:this={host}>
  <canvas
    bind:this={canvas}
    class:grab={panning}
    class:grabbing={drag}
    style:cursor={panning ? undefined : cursor}
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
