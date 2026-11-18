// Web Worker du moteur : charge le WebAssembly de `fabomx-wasm`, annonce sa
// version et la machine cible, puis répond aux demandes du fil principal.
// Le calcul ne bloque jamais l'interface.

import init, { clean_apply, clean_preview, engine_version, inspect, load, machine, overlaps, place, sides } from '../wasm/fabomx.js';

type Request =
  | { kind: 'inspect'; id: number; text: string; unit: string; clean: string }
  | { kind: 'load'; id: number; geo: number; text: string; unit: string; clean: string }
  | { kind: 'place'; id: number; geo: number; matrix: number[] }
  | { kind: 'preview'; id: number; geo: number; opts: string; k: number }
  | { kind: 'clean'; id: number; geo: number; opts: string; k: number; next: number }
  | { kind: 'sides'; id: number; geo: number }
  | { kind: 'overlaps'; id: number; placed: string };

/** `postMessage` du worker (les types DOM ne connaissent que celui de `window`). */
const post = (msg: unknown) => (self as unknown as Worker).postMessage(msg);

const ready = init().then(() => post({ kind: 'ready', version: engine_version(), machine: JSON.parse(machine()) }));
ready.catch((e) => post({ kind: 'failed', error: String(e) }));

onmessage = async (ev: MessageEvent<Request>) => {
  await ready;
  const req = ev.data;
  try {
    let value: string;
    if (req.kind === 'inspect') value = inspect(req.text, req.unit, req.clean);
    else if (req.kind === 'load') value = load(req.geo, req.text, req.unit, req.clean);
    else if (req.kind === 'place') value = place(req.geo, new Float64Array(req.matrix));
    else if (req.kind === 'preview') value = clean_preview(req.geo, req.opts, req.k);
    else if (req.kind === 'clean') value = clean_apply(req.geo, req.opts, req.k, req.next);
    else if (req.kind === 'sides') value = sides(req.geo);
    else value = overlaps(req.placed);
    post({ kind: 'result', id: req.id, ok: true, value: JSON.parse(value) });
  } catch (e) {
    post({ kind: 'result', id: req.id, ok: false, error: String(e) });
  }
};
