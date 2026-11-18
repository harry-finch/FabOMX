// Client du moteur (Web Worker) : état de chargement, version et machine
// cible, lus par la barre d'état, l'espace de travail et l'inspecteur ; les
// lectures de fichiers passent par des promesses.

import type { ImportFailure, Report, Unit } from './types';

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

  /** L'état d'un dessin SVG ou DXF ; `unit` absente : celle du fichier. */
  inspect(text: string, unit?: Unit): Promise<Report | ImportFailure> {
    const id = ++this.#seq;
    return new Promise((resolve, reject) => {
      if (!this.#worker) return reject(new Error('engine.unavailable'));
      this.#pending.set(id, { resolve, reject });
      this.#worker.postMessage({ kind: 'inspect', id, text, unit: unit ?? '' });
    });
  }
}

export const engine = new Engine();
