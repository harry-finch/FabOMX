// Le document : les dessins posés sur le plateau, le travail (point de
// départ et ordre de coupe du parcours), la sélection, et l'historique pour
// annuler / rétablir. Chaque modification remplace la
// liste (les dessins sont immuables) et empile l'état précédent ; un geste
// à la souris (`begin` … `end`) ne fait qu'une étape.
// Un dessin garde sa géométrie d'origine (dans le moteur) et une matrice ;
// sa boîte et ses avertissements (`placement`) sont recalculés par le
// moteur à chaque changement de matrice, le dernier demandé gagnant.

import { engine } from './engine.svelte';
import { apply, translate, type M, type Pt } from './affine';
import type { LeadShape, Placement, Report, Side, TabSpec } from './types';

/** Point de référence : un des 9 points de la boîte (0 à 2, depuis le bas
 * gauche), ou un point du dessin d'origine, qui suit ses transformations. */
export type Ref = { kind: 'box'; ix: number; iy: number } | { kind: 'point'; x: number; y: number };

export interface Drawing {
  id: number;
  /** Géométrie d'origine dans le moteur ; un nettoyage en range une nouvelle. */
  geo: number;
  /** Nom du fichier importé. */
  name: string;
  /** L'import : géométrie d'origine, unité, éléments, défauts. */
  report: Report;
  matrix: M;
  ref: Ref;
  /** Points d'accrochage du dessin d'origine `[x, y, …]`. */
  snaps: number[];
  placement: Placement;
  /** Côtés choisis à la main, par indice de contour ; les autres sont ceux détectés. */
  sides: Record<number, Side>;
  /** Qualités de coupe propres (1 à 5), par indice de contour ; les autres ont la qualité par défaut. */
  qualities: Record<number, number>;
  /** Forme et longueurs d'amorce propres, par indice de contour. */
  leads: Record<number, LeadOpts>;
  /** Points d'entrée choisis à la main (repère d'origine), par indice de contour. */
  entries: Record<number, [number, number]>;
  /** Onglets demandés, par indice de contour (extérieurs). */
  tabs: Record<number, TabSpec>;
  /** Un nettoyage a recalculé les côtés et fait perdre des choix faits à la main (côtés, qualités). */
  sidesReset: boolean;
}

/** Réglages d'amorce propres à un contour ; ce qui manque vient des réglages par défaut. */
export interface LeadOpts {
  shape?: LeadShape;
  lead_in?: number;
  lead_out?: number;
  tab_width?: number;
}

/** Le travail : point de départ du parcours (plateau) et ordre de coupe
 * choisi à la main (`[dessin, contour]`), ou `null` pour l'ordre automatique. */
export interface Job {
  start: [number, number];
  order: [number, number][] | null;
}

type Snapshot = { drawings: Drawing[]; job: Job };

const HISTORY = 100;

/** Boîte transformée calculée sur les tracés, en attendant le moteur. */
function roughPlacement(report: Report, m: M): Placement {
  let x0 = Infinity;
  let y0 = Infinity;
  let x1 = -Infinity;
  let y1 = -Infinity;
  for (const tr of report.traces) {
    for (let i = 0; i < tr.pts.length; i += 2) {
      const [x, y] = apply(m, tr.pts[i], tr.pts[i + 1]);
      x0 = Math.min(x0, x);
      y0 = Math.min(y0, y);
      x1 = Math.max(x1, x);
      y1 = Math.max(y1, y);
    }
  }
  return { bbox: [x0, y0, x1, y1], width: x1 - x0, height: y1 - y0, warnings: [] };
}

/** Le point de référence d'un dessin, sur le plateau. */
export function refPoint(d: Drawing): Pt {
  if (d.ref.kind === 'point') return apply(d.matrix, d.ref.x, d.ref.y);
  const [x0, y0, x1, y1] = d.placement.bbox;
  return [x0 + (d.ref.ix * (x1 - x0)) / 2, y0 + (d.ref.iy * (y1 - y0)) / 2];
}

class Doc {
  drawings = $state.raw<Drawing[]>([]);
  job = $state.raw<Job>({ start: [0, 0], order: null });
  selected = $state<number | null>(null);
  /** « Choisir sur le plateau » : le prochain clic place le point de référence. */
  picking = $state(false);
  /** Cadenas de l'échelle : largeur et hauteur liées. */
  lock = $state(true);
  #undo: Snapshot[] = [];
  #redo: Snapshot[] = [];
  #gesture: Drawing[] | null = null;
  #next = 1;
  /** Taille des piles, pour l'état des commandes. */
  depth = $state({ undo: 0, redo: 0 });

  get selection(): Drawing | null {
    return this.drawings.find((d) => d.id === this.selected) ?? null;
  }

  /** Un identifiant neuf : dessin, ou géométrie que le moteur garde. */
  reserve(): number {
    return this.#next++;
  }

  #commit(next: Drawing[], job: Job = this.job) {
    if (this.#gesture) {
      this.drawings = next;
      return;
    }
    this.#push({ drawings: this.drawings, job: this.job });
    this.drawings = next;
    this.job = job;
    this.#sync();
  }

  #push(state: Snapshot) {
    this.#undo.push(state);
    if (this.#undo.length > HISTORY) this.#undo.shift();
    this.#redo = [];
  }

  #sync() {
    if (this.selected != null && !this.drawings.some((d) => d.id === this.selected)) this.selected = null;
    this.depth = { undo: this.#undo.length, redo: this.#redo.length };
  }

  /** Pose un dessin centré sur un plateau de `bed` (mm) et le sélectionne. */
  add(geo: number, name: string, report: Report, snaps: number[], bed: { width: number; height: number }) {
    const [x0, y0, x1, y1] = report.bbox;
    const matrix = translate(bed.width / 2 - (x0 + x1) / 2, bed.height / 2 - (y0 + y1) / 2);
    const id = this.reserve();
    const drawing: Drawing = { id, geo, name, report, matrix, ref: { kind: 'box', ix: 1, iy: 1 }, snaps, placement: roughPlacement(report, matrix), sides: {}, qualities: {}, leads: {}, entries: {}, tabs: {}, sidesReset: false };
    this.#commit([...this.drawings, drawing]);
    this.selected = id;
    this.#place(drawing);
  }

  remove(id: number) {
    if (!this.drawings.some((d) => d.id === id)) return;
    this.#commit(this.drawings.filter((d) => d.id !== id));
  }

  /** Change un dessin : une étape d'annulation, ou une partie du geste en cours. */
  update(id: number, patch: Partial<Pick<Drawing, 'matrix' | 'ref'>>) {
    const d = this.drawings.find((x) => x.id === id);
    if (!d) return;
    const next: Drawing = { ...d, ...patch };
    if (patch.matrix) next.placement = { ...roughPlacement(d.report, patch.matrix), warnings: d.placement.warnings };
    this.#commit(this.drawings.map((x) => (x.id === id ? next : x)));
    if (patch.matrix) this.#place(next);
  }

  /** Côté de coupe des contours `indices` de `id` ; `null` : revenir au côté détecté. Une étape. */
  setSides(id: number, indices: number[], side: Side | null) {
    const d = this.drawings.find((x) => x.id === id);
    if (!d || !indices.length) return;
    const sides = { ...d.sides };
    for (const i of indices) {
      if (side) sides[i] = side;
      else delete sides[i];
    }
    this.#commit(this.drawings.map((x) => (x.id === id ? { ...d, sides, sidesReset: false } : x)));
  }

  /** Qualité propre des contours `indices` de `id` ; `null` : revenir à la qualité par défaut. Une étape. */
  setQualities(id: number, indices: number[], q: number | null) {
    const d = this.drawings.find((x) => x.id === id);
    if (!d || !indices.length) return;
    const qualities = { ...d.qualities };
    for (const i of indices) {
      if (q) qualities[i] = q;
      else delete qualities[i];
    }
    this.#commit(this.drawings.map((x) => (x.id === id ? { ...d, qualities, sidesReset: false } : x)));
  }

  /** Réglages d'amorce propres des contours `indices` de `id`, fusionnés ; `null` : revenir aux réglages par défaut. Une étape. */
  setLeads(id: number, indices: number[], patch: LeadOpts | null) {
    const d = this.drawings.find((x) => x.id === id);
    if (!d || !indices.length) return;
    const leads = { ...d.leads };
    for (const i of indices) {
      const next = Object.fromEntries(Object.entries({ ...leads[i], ...patch }).filter(([, v]) => v != null)) as LeadOpts;
      if (patch && Object.keys(next).length) leads[i] = next;
      else delete leads[i];
    }
    this.#commit(this.drawings.map((x) => (x.id === id ? { ...d, leads, sidesReset: false } : x)));
  }

  /** Onglets des contours `indices` de `id` ; `null` : aucun. Une étape. */
  setTabs(id: number, indices: number[], spec: TabSpec | null) {
    const d = this.drawings.find((x) => x.id === id);
    if (!d || !indices.length) return;
    const tabs = { ...d.tabs };
    for (const i of indices) {
      if (spec && spec.kind !== 'none') tabs[i] = spec;
      else delete tabs[i];
    }
    this.#commit(this.drawings.map((x) => (x.id === id ? { ...d, tabs, sidesReset: false } : x)));
  }

  /** Point d'entrée du contour `i` de `id` (repère d'origine) ; `null` : automatique. Une étape. */
  setEntry(id: number, i: number, at: [number, number] | null) {
    const d = this.drawings.find((x) => x.id === id);
    if (!d) return;
    const entries = { ...d.entries };
    if (at) entries[i] = at;
    else delete entries[i];
    this.#commit(this.drawings.map((x) => (x.id === id ? { ...d, entries, sidesReset: false } : x)));
  }

  /** Remplace la géométrie d'un dessin nettoyé ; la transformation est gardée,
   * les côtés sont recalculés (côtés, qualités, amorces et points d'entrée choisis à la main sont perdus). Une étape. */
  replaceGeometry(id: number, geo: number, report: Report, snaps: number[]) {
    const d = this.drawings.find((x) => x.id === id);
    if (!d) return;
    const sidesReset = [d.sides, d.qualities, d.leads, d.entries, d.tabs].some((r) => Object.keys(r).length > 0);
    const next: Drawing = { ...d, geo, report, snaps, sides: {}, qualities: {}, leads: {}, entries: {}, tabs: {}, sidesReset, placement: { ...roughPlacement(report, d.matrix), warnings: d.placement.warnings } };
    this.#commit(this.drawings.map((x) => (x.id === id ? next : x)));
    this.#place(next);
  }

  /** Début d'un geste à la souris : tout jusqu'à `end` fait une seule étape. */
  begin() {
    this.#gesture = this.drawings;
  }

  end() {
    const before = this.#gesture;
    this.#gesture = null;
    if (before && before !== this.drawings) {
      this.#push({ drawings: before, job: this.job });
      this.#sync();
    }
  }

  /** Demande au moteur la boîte exacte et les avertissements. */
  async #place(d: Drawing) {
    const m = d.matrix;
    try {
      const p = await engine.place(d.geo, m);
      if ('error' in p) return;
      // Le dessin a pu changer entre-temps : seul le résultat de sa matrice actuelle compte.
      const apply = (list: Drawing[]) => list.map((x) => (x.id === d.id && x.matrix === m && x.geo === d.geo ? { ...x, placement: p } : x));
      this.drawings = apply(this.drawings);
      if (this.#gesture) this.#gesture = apply(this.#gesture);
    } catch {
      /* moteur indisponible : la boîte approchée reste */
    }
  }

  undo() {
    const prev = this.#undo.pop();
    if (!prev) return;
    this.#redo.push({ drawings: this.drawings, job: this.job });
    this.drawings = prev.drawings;
    this.job = prev.job;
    this.#sync();
  }

  redo() {
    const next = this.#redo.pop();
    if (!next) return;
    this.#undo.push({ drawings: this.drawings, job: this.job });
    this.drawings = next.drawings;
    this.job = next.job;
    this.#sync();
  }

  /** Point de départ du parcours (plateau). Une étape. */
  setStart(at: [number, number]) {
    this.#commit(this.drawings, { ...this.job, start: at });
  }

  /** Ordre de coupe choisi à la main ; `null` : automatique. Une étape. */
  setOrder(order: [number, number][] | null) {
    this.#commit(this.drawings, { ...this.job, order });
  }

  select(id: number | null) {
    this.selected = id;
    if (id == null) this.picking = false;
  }
}

export const doc = new Doc();
