// Web Worker du moteur : charge le WebAssembly de `fabomx-wasm` et annonce
// sa version et la machine cible. Le calcul ne bloque jamais l'interface.

import init, { engine_version, machine } from '../wasm/fabomx.js';

/** `postMessage` du worker (les types DOM ne connaissent que celui de `window`). */
const post = (msg: unknown) => (self as unknown as Worker).postMessage(msg);

init()
  .then(() => post({ kind: 'ready', version: engine_version(), machine: JSON.parse(machine()) }))
  .catch((e) => post({ kind: 'failed', error: String(e) }));
