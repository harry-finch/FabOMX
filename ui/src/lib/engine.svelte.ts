// Client du moteur (Web Worker) : état de chargement, version et machine
// cible, lus par la barre d'état, l'espace de travail et l'inspecteur.

/** Zone de coupe d'une machine, en mm, origine en bas à gauche. */
export interface Machine {
  id: string;
  name: string;
  width: number;
  height: number;
}

class Engine {
  state = $state<'loading' | 'ready' | 'failed'>('loading');
  version = $state<string | undefined>();
  machine = $state<Machine | null>(null);

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
      }
    };
    worker.onerror = (ev) => {
      console.error(ev.message);
      this.state = 'failed';
    };
  }
}

export const engine = new Engine();
