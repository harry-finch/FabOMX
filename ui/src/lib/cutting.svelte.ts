// Côtés de coupe (outil Découpe) : les contours de chaque géométrie et leurs
// côtés détectés, demandés au moteur une fois par géométrie ; les contours
// sélectionnés (d'un seul dessin) ; les dessins qui se chevauchent ;
// l'avertissement à montrer. Le côté retenu d'un contour est celui choisi à
// la main (`Drawing.sides`), sinon celui détecté.

import { engine } from './engine.svelte';
import { det, type M } from './affine';
import type { Drawing } from './document.svelte';
import type { Side, Sides } from './types';

/** Faut-il parcourir le contour à l'envers de son tracé, pour que la pièce
 * reste à gauche du jet ? Mêmes règles que `cut::reversed` du moteur. */
export function reversed(side: Side, area: number, m: M): boolean {
  const ccw = area * Math.sign(det(m)) > 0;
  if (side === 'outside') return !ccw;
  if (side === 'inside') return ccw;
  return false;
}

/** Côtés qu'un contour peut prendre : un tracé ouvert n'a ni extérieur ni trou. */
export function allowed(closed: boolean): Side[] {
  return closed ? ['outside', 'inside', 'line', 'skip'] : ['line', 'skip'];
}

class Cutting {
  /** Contours et côtés détectés, par géométrie. */
  byGeo = $state.raw<Record<number, Sides>>({});
  /** Contours sélectionnés, dans le dessin `id`. */
  selection = $state.raw<{ id: number; contours: number[] } | null>(null);
  /** Paires d'identifiants de dessins qui se chevauchent. */
  overlaps = $state.raw<[number, number][]>([]);
  /** Point à montrer (repère d'origine du dessin `id`), et un compteur pour recentrer la vue. */
  focused = $state<{ id: number; x: number; y: number; seq: number } | null>(null);
  #seq = 0;
  #asking = new Set<number>();
  #asked = 0;

  /** Demande au moteur les contours des géométries qui manquent. */
  ensure(drawings: Drawing[]) {
    for (const d of drawings) {
      if (this.byGeo[d.geo] || this.#asking.has(d.geo)) continue;
      this.#asking.add(d.geo);
      engine
        .sides(d.geo)
        .then((s) => {
          if (!('error' in s)) this.byGeo = { ...this.byGeo, [d.geo]: s };
        })
        .catch(() => {})
        .finally(() => this.#asking.delete(d.geo));
    }
  }

  /** Recalcule les chevauchements ; le dernier demandé gagne. */
  async refreshOverlaps(drawings: Drawing[]) {
    const ask = ++this.#asked;
    if (drawings.length < 2) {
      this.overlaps = [];
      return;
    }
    const res = await engine.overlaps(drawings.map((d) => ({ geo: d.geo, m: d.matrix }))).catch(() => null);
    if (ask !== this.#asked || !res || 'error' in res) return;
    this.overlaps = res.map(([i, j]) => [drawings[i].id, drawings[j].id]);
  }

  sides(d: Drawing): Sides | null {
    return this.byGeo[d.geo] ?? null;
  }

  /** Côté retenu du contour `i` de `d`. */
  sideOf(d: Drawing, i: number): Side {
    return d.sides[i] ?? this.byGeo[d.geo]?.contours[i]?.side ?? 'line';
  }

  /** Sélectionne le contour `i` de `d` ; avec `add`, l'ajoute ou le retire. */
  pick(d: Drawing, i: number, add: boolean) {
    const cur = this.selection?.id === d.id ? this.selection.contours : [];
    if (!add) this.selection = { id: d.id, contours: [i] };
    else if (cur.includes(i)) this.selection = cur.length > 1 ? { id: d.id, contours: cur.filter((x) => x !== i) } : null;
    else this.selection = { id: d.id, contours: [...cur, i] };
  }

  clear() {
    this.selection = null;
  }

  focus(id: number, x: number, y: number) {
    this.focused = { id, x, y, seq: ++this.#seq };
  }
}

export const cutting = new Cutting();
