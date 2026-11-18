// Côtés et qualités de coupe (outil Découpe) : les contours de chaque géométrie et leurs
// côtés détectés, demandés au moteur une fois par géométrie ; les contours
// sélectionnés (d'un seul dessin) ; les dessins qui se chevauchent ;
// l'avertissement à montrer. Le côté retenu d'un contour est celui choisi à
// la main (`Drawing.sides`), sinon celui détecté. La qualité retenue est
// celle du contour (`Drawing.qualities`), sinon la qualité par défaut,
// mémorisée sur le poste ; un contour Ne pas couper n'en a pas. Amorces :
// forme et longueurs propres au contour (`Drawing.leads`) sinon celles par
// défaut (poste), point d'entrée choisi (`Drawing.entries`) sinon
// automatique ; le moteur les place, le dernier calcul demandé gagnant.

import { storageKey } from './app';
import { engine } from './engine.svelte';
import { det, type M } from './affine';
import type { Drawing } from './document.svelte';
import type { LeadShape, LeadSpec, Leads, Side, Sides } from './types';

/** Faut-il parcourir le contour à l'envers de son tracé, pour que la pièce
 * reste à droite du jet (comme sur la ProtoMAX) ? Mêmes règles que `cut::reversed` du moteur. */
export function reversed(side: Side, area: number, m: M): boolean {
  const ccw = area * Math.sign(det(m)) > 0;
  if (side === 'outside') return ccw;
  if (side === 'inside') return !ccw;
  return false;
}

/** Côtés qu'un contour peut prendre : un tracé ouvert n'a ni extérieur ni trou. */
export function allowed(closed: boolean): Side[] {
  return closed ? ['outside', 'inside', 'line', 'skip'] : ['line', 'skip'];
}

/** Qualités de coupe, comme dans LAYOUT (`cut::QUALITIES` du moteur). */
export const QUALITIES = [1, 2, 3, 4, 5];
const KEY = storageKey('cut.v1');

/** Amorces par défaut : celles des exemples LAYOUT (`lead::LEAD_IN`, `LEAD_OUT`). Bornes de `lead::LEAD_RANGE`. */
export const LEAD_DEFAULTS = { shape: 'line' as LeadShape, lead_in: 3.8, lead_out: 1.3 };
export const LEAD_MIN = 0.5;
export const LEAD_MAX = 20;

type Saved = { quality: number; shape: LeadShape; lead_in: number; lead_out: number };

function read(): Saved {
  const out: Saved = { quality: 3, ...LEAD_DEFAULTS };
  try {
    const v = JSON.parse(localStorage.getItem(KEY) ?? 'null') ?? {};
    if (QUALITIES.includes(v.quality)) out.quality = v.quality;
    if (v.shape === 'line' || v.shape === 'arc') out.shape = v.shape;
    for (const k of ['lead_in', 'lead_out'] as const) if (typeof v[k] === 'number' && v[k] >= LEAD_MIN && v[k] <= LEAD_MAX) out[k] = v[k];
  } catch {
    /* stockage indisponible ou illisible */
  }
  return out;
}

const saved = read();

class Cutting {
  /** Qualité par défaut des contours coupés sans qualité propre. */
  quality = $state(saved.quality);
  /** Amorces par défaut (longueurs en mm du plateau). */
  lead = $state({ shape: saved.shape, lead_in: saved.lead_in, lead_out: saved.lead_out });
  /** Amorces calculées, par dessin, avec la clé de ce qui les a produites. */
  leadsOf = $state.raw<Record<number, { key: string; leads: Leads }>>({});
  /** « Choisir le point d'entrée » : le prochain clic sur le contour sélectionné le place. */
  pickingEntry = $state(false);
  #leadAsked: Record<number, number> = {};
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

  #save() {
    try {
      localStorage.setItem(KEY, JSON.stringify({ quality: this.quality, ...this.lead }));
    } catch {
      /* ignoré */
    }
  }

  setDefault(q: number) {
    if (!QUALITIES.includes(q)) return;
    this.quality = q;
    this.#save();
  }

  setLeadDefaults(patch: Partial<typeof LEAD_DEFAULTS>) {
    this.lead = { ...this.lead, ...patch };
    this.#save();
  }

  /** Réglage d'amorce retenu du contour `i` de `d`, pour le moteur. */
  spec(d: Drawing, i: number): LeadSpec {
    const own = d.leads[i] ?? {};
    return {
      side: this.sideOf(d, i),
      shape: own.shape ?? this.lead.shape,
      lead_in: own.lead_in ?? this.lead.lead_in,
      lead_out: own.lead_out ?? this.lead.lead_out,
      entry: d.entries[i] ?? null,
    };
  }

  /** Recalcule les amorces de `d` si ce qui les produit a changé ; le dernier demandé gagne. */
  async refreshLeads(d: Drawing) {
    const s = this.byGeo[d.geo];
    if (!s) return;
    const specs = s.contours.map((_, i) => this.spec(d, i));
    const key = JSON.stringify([d.geo, d.matrix, specs]);
    if (this.leadsOf[d.id]?.key === key) return;
    const ask = (this.#leadAsked[d.id] = (this.#leadAsked[d.id] ?? 0) + 1);
    const res = await engine.leads(d.geo, d.matrix, specs).catch(() => null);
    if (ask !== this.#leadAsked[d.id] || !res || 'error' in res) return;
    this.leadsOf = { ...this.leadsOf, [d.id]: { key, leads: res } };
  }

  /** Amorces calculées de `d`, si elles sont celles de sa géométrie actuelle. */
  leads(d: Drawing): Leads | null {
    const l = this.leadsOf[d.id];
    return l && l.key.startsWith(`[${d.geo},`) ? l.leads : null;
  }

  /** Qualité retenue du contour `i` de `d`, `null` s'il n'est pas coupé. Mêmes règles que `cut::quality`. */
  qualityOf(d: Drawing, i: number): number | null {
    if (this.sideOf(d, i) === 'skip') return null;
    return d.qualities[i] ?? this.quality;
  }

  /** Sélectionne tous les contours de `d`. */
  selectAll(d: Drawing) {
    const s = this.byGeo[d.geo];
    if (s?.contours.length) this.selection = { id: d.id, contours: s.contours.map((_, i) => i) };
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
    this.pickingEntry = false;
  }

  focus(id: number, x: number, y: number) {
    this.focused = { id, x, y, seq: ++this.#seq };
  }
}

export const cutting = new Cutting();
