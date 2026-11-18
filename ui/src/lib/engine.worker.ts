// Web Worker du moteur : charge le WebAssembly de `fabomx-wasm`, annonce sa
// version et la machine cible, puis répond aux demandes du fil principal.
// Le calcul ne bloque jamais l'interface.

import init, { engine_version, inspect, load, machine, place } from '../wasm/fabomx.js';

type Request =
  | { kind: 'inspect'; id: number; text: string; unit: string }
  | { kind: 'load'; id: number; drawing: number; text: string; unit: string }
  | { kind: 'place'; id: number; drawing: number; matrix: number[] };

/** `postMessage` du worker (les types DOM ne connaissent que celui de `window`). */
const post = (msg: unknown) => (self as unknown as Worker).postMessage(msg);

const ready = init().then(() => post({ kind: 'ready', version: engine_version(), machine: JSON.parse(machine()) }));
ready.catch((e) => post({ kind: 'failed', error: String(e) }));

onmessage = async (ev: MessageEvent<Request>) => {
  await ready;
  const req = ev.data;
  try {
    let value: string;
    if (req.kind === 'inspect') value = inspect(req.text, req.unit);
    else if (req.kind === 'load') value = load(req.drawing, req.text, req.unit);
    else value = place(req.drawing, new Float64Array(req.matrix));
    post({ kind: 'result', id: req.id, ok: true, value: JSON.parse(value) });
  } catch (e) {
    post({ kind: 'result', id: req.id, ok: false, error: String(e) });
  }
};
