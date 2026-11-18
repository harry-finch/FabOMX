// Parcours (outil Parcours) : le plan du moteur pour tous les dessins (point
// de départ, ordre, mouvements, bilan, remarques), recalculé quand le travail
// change, le dernier demandé gagnant ; le contour choisi dans la liste ;
// « Choisir sur le plateau » pour le point de départ ; la lecture pas à pas
// (avancement de 0 à 1 le long du parcours, 20 s au plus).

import { engine } from './engine.svelte';
import { cutting } from './cutting.svelte';
import { doc } from './document.svelte';
import { ui } from './ui.svelte';
import { t } from './i18n.svelte';
import type { Move, OrderRefused, Plan } from './types';

const MAX_PLAY = 20;
const MIN_PLAY = 4;
/** Vitesse de lecture visée (mm/s), bornée par la durée. */
const PLAY_SPEED = 60;

/** Longueur d'un mouvement (mm). */
export function moveLength(m: Move): number {
  let l = 0;
  for (let i = 2; i + 1 < m.pts.length; i += 2) l += Math.hypot(m.pts[i] - m.pts[i - 2], m.pts[i + 1] - m.pts[i - 1]);
  return l;
}

class Pathing {
  plan = $state.raw<Plan | null>(null);
  /** L'ordre choisi à la main n'est plus permis : le plan affiché est l'automatique. */
  orderLost = $state(false);
  /** Rang choisi dans la liste (mis en évidence sur le plateau). */
  picked = $state<number | null>(null);
  pickingStart = $state(false);
  progress = $state(1);
  playing = $state(false);
  /** Point à montrer (plateau), et un compteur pour recentrer la vue. */
  focused = $state<{ x: number; y: number; seq: number } | null>(null);
  #seq = 0;
  #asked = 0;
  #frame = 0;

  /** Longueurs cumulées des mouvements du plan, et la longueur totale. */
  get lengths(): { ends: number[]; total: number } {
    const ends: number[] = [];
    let total = 0;
    for (const m of this.plan?.moves ?? []) ends.push((total += moveLength(m)));
    return { ends, total };
  }

  #input(order: [number, number][] | null) {
    return {
      start: doc.job.start,
      order,
      drawings: doc.drawings
        .filter((d) => cutting.sides(d))
        .map((d) => {
          const n = cutting.sides(d)!.contours.length;
          return {
            key: d.id,
            geo: d.geo,
            m: d.matrix,
            specs: Array.from({ length: n }, (_, i) => cutting.spec(d, i)),
            qualities: Array.from({ length: n }, (_, i) => cutting.qualityOf(d, i)),
          };
        }),
    };
  }

  /** Recalcule le plan ; un ordre à la main devenu interdit laisse place à l'automatique. */
  async refresh() {
    const ask = ++this.#asked;
    let res = await engine.plan(this.#input(doc.job.order)).catch(() => null);
    let lost = false;
    if (res && 'error' in res && res.error === 'path.order') {
      lost = true;
      res = await engine.plan(this.#input(null)).catch(() => null);
    }
    if (ask !== this.#asked || !res || 'error' in res) return;
    this.plan = res;
    this.orderLost = lost;
    if (this.picked != null && this.picked >= res.order.length) this.picked = null;
  }

  /** Nom d'un contour pour les listes et les messages : « Trou 2 · plaque.svg ». */
  label(key: number, contour: number): string {
    const d = doc.drawings.find((x) => x.id === key);
    if (!d) return '';
    return `${t(`cut.sides.${cutting.sideOf(d, contour)}`)} ${contour + 1} · ${d.name}`;
  }

  /** Déplace le contour de rang `i` de `delta` rangs ; refusé (et dit) s'il serait coupé avant un contour qu'il contient. */
  async shift(i: number, delta: number) {
    const p = this.plan;
    const j = i + delta;
    if (!p || j < 0 || j >= p.order.length) return;
    const order = p.order.map((s) => [s.key, s.contour] as [number, number]);
    [order[i], order[j]] = [order[j], order[i]];
    const res = await engine.plan(this.#input(order)).catch(() => null);
    if (!res) return;
    if ('error' in res) {
      if (res.error === 'path.order') {
        const r = res as OrderRefused;
        ui.toast(t('path.orderRefused', { contour: this.label(...r.contour), inner: this.label(...r.inner) }));
      }
      return;
    }
    doc.setOrder(order);
    this.picked = j;
  }

  focus(x: number, y: number) {
    this.focused = { x, y, seq: ++this.#seq };
  }

  play() {
    if (this.playing) return this.pause();
    const { total } = this.lengths;
    if (!total) return;
    if (this.progress >= 1) this.progress = 0;
    const duration = Math.min(MAX_PLAY, Math.max(MIN_PLAY, total / PLAY_SPEED)) * 1000;
    this.playing = true;
    let last = performance.now();
    const tick = (now: number) => {
      if (!this.playing) return;
      this.progress = Math.min(1, this.progress + (now - last) / duration);
      last = now;
      if (this.progress >= 1) this.playing = false;
      else this.#frame = requestAnimationFrame(tick);
    };
    this.#frame = requestAnimationFrame(tick);
  }

  pause() {
    this.playing = false;
    cancelAnimationFrame(this.#frame);
  }

  seek(v: number) {
    this.pause();
    this.progress = Math.max(0, Math.min(1, v));
  }
}

export const pathing = new Pathing();
