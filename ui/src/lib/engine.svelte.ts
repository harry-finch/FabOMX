// Client du moteur (Web Worker) : état de chargement, version et machine
// cible, lus par la barre d'état, l'espace de travail et l'inspecteur ; les
// lectures de fichiers passent par des promesses.

import type { CleanCounts, CleanOptions, CleanPreview, ImportFailure, LeadSpec, Leads, Placement, Report, Sides, Unit } from './types';
import type { M } from './affine';

/** Zone de coupe d'une machine, en mm, origine en bas à gauche. */
export interface Machine {
  id: string;
  name: string;
  width: number;
  height: number;
}

type Pending = { resolve: (v: any) => void; reject: (e: Error) => void };

class Engine {
  state = $state<'loading' | 'ready' | 'failed'>('loading');
  version = $state<string | undefined>();
  machine = $state<Machine | null>(null);
  #worker: Worker | null = null;
  #seq = 0;
  #pending = new Map<number, Pending>();

  constructor() {
    if (typeof Worker === 'undefined') return;
    const worker = new Worker(new URL('./engine.worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = (ev) => {
      const msg = ev.data;
      if (msg.kind === 'ready') {
        this.version = msg.version;
        this.machine = msg.machine;
        this.state = 'ready';
      } else if (msg.kind === 'failed') {
        console.error(msg.error);
        this.state = 'failed';
      } else if (msg.kind === 'result') {
        const p = this.#pending.get(msg.id);
        this.#pending.delete(msg.id);
        if (msg.ok) p?.resolve(msg.value);
        else p?.reject(new Error(msg.error));
      }
    };
    worker.onerror = (ev) => {
      console.error(ev.message);
      this.state = 'failed';
    };
    this.#worker = worker;
  }

  #call<T>(msg: Record<string, unknown>): Promise<T> {
    const id = ++this.#seq;
    return new Promise((resolve, reject) => {
      if (!this.#worker) return reject(new Error('engine.unavailable'));
      this.#pending.set(id, { resolve, reject });
      this.#worker.postMessage({ ...msg, id });
    });
  }

  /** L'état d'un dessin SVG ou DXF ; `unit` absente : celle du fichier ; `clean` : nettoyé à l'import. */
  inspect(text: string, unit?: Unit, clean?: CleanOptions): Promise<Report | ImportFailure> {
    return this.#call({ kind: 'inspect', text, unit: unit ?? '', clean: clean ? JSON.stringify(clean) : '' });
  }

  /** Garde la géométrie d'origine sous `geo` ; rend ses points d'accrochage. */
  load(geo: number, text: string, unit: Unit, clean?: CleanOptions): Promise<{ snaps: number[] } | ImportFailure> {
    return this.#call({ kind: 'load', geo, text, unit, clean: clean ? JSON.stringify(clean) : '' });
  }

  /** La géométrie `geo` transformée par `matrix`, depuis l'origine. */
  place(geo: number, matrix: M): Promise<Placement | { error: string }> {
    return this.#call({ kind: 'place', geo, matrix });
  }

  /** Ce que ferait le nettoyage de `geo` à l'échelle `k` (mm du plateau par mm d'origine). */
  cleanPreview(geo: number, opts: CleanOptions, k: number): Promise<CleanPreview | { error: string }> {
    return this.#call({ kind: 'preview', geo, opts: JSON.stringify(opts), k });
  }

  /** Nettoie `geo` et range le résultat sous `next` (l'ancienne géométrie reste, pour annuler). */
  cleanApply(geo: number, opts: CleanOptions, k: number, next: number): Promise<{ report: Report; snaps: number[]; counts: CleanCounts } | { error: string }> {
    return this.#call({ kind: 'clean', geo, opts: JSON.stringify(opts), k, next });
  }

  /** Les contours de `geo` et leurs côtés détectés. */
  sides(geo: number): Promise<Sides | { error: string }> {
    return this.#call({ kind: 'sides', geo });
  }

  /** Les paires (indices dans `placed`) de dessins placés qui se chevauchent. */
  overlaps(placed: { geo: number; m: M }[]): Promise<[number, number][] | { error: string }> {
    return this.#call({ kind: 'overlaps', placed: JSON.stringify(placed) });
  }

  /** Amorces et perçages des contours de `geo` placée par `matrix`, un réglage par contour. */
  leads(geo: number, matrix: M, specs: LeadSpec[]): Promise<Leads | { error: string }> {
    return this.#call({ kind: 'leads', geo, matrix, specs: JSON.stringify(specs) });
  }
}

export const engine = new Engine();
