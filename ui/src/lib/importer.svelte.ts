// Import d'un dessin : choix du fichier (bouton Importer, ⌘O, dépôt), puis
// le dialogue d'import montre l'état du dessin ; rien n'est posé avant
// « Importer ». Changer l'unité relit le fichier dans cette unité.

import { engine } from './engine.svelte';
import { doc } from './document.svelte';
import { ui } from './ui.svelte';
import { t } from './i18n.svelte';
import type { ImportFailure, Report, Unit } from './types';

export interface Pending {
  name: string;
  text: string;
  /** Unité choisie ; absente : celle du fichier. */
  unit?: Unit;
  result: Report | ImportFailure | null;
  busy: boolean;
}

const isReport = (r: Report | ImportFailure | null): r is Report => !!r && !('error' in r);

class Importer {
  pending = $state<Pending | null>(null);

  /** Ouvre le choix du fichier. */
  pick() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.svg,.dxf,image/svg+xml';
    input.onchange = () => {
      const file = input.files?.[0];
      if (file) this.open(file);
    };
    input.click();
  }

  async open(file: File) {
    let text: string;
    try {
      text = await file.text();
    } catch {
      this.pending = { name: file.name, text: '', result: { error: 'import.unreadable' }, busy: false };
      return;
    }
    this.pending = { name: file.name, text, result: null, busy: true };
    await this.#read();
  }

  async setUnit(unit: Unit) {
    if (!this.pending) return;
    this.pending.unit = unit;
    await this.#read();
  }

  async #read() {
    const p = this.pending;
    if (!p) return;
    p.busy = true;
    try {
      const result = await engine.inspect(p.text, p.unit);
      if (this.pending === p) p.result = result;
    } catch {
      if (this.pending === p) p.result = { error: 'import.unreadable' };
    } finally {
      p.busy = false;
    }
  }

  get report(): Report | null {
    const r = this.pending?.result ?? null;
    return isReport(r) ? r : null;
  }

  async confirm() {
    const p = this.pending;
    const report = this.report;
    if (!p || !report || p.busy || !engine.machine) return;
    p.busy = true;
    const id = doc.reserve();
    const loaded = await engine.load(id, p.text, report.unit).catch(() => null);
    if (!loaded || 'error' in loaded) {
      p.result = { error: 'import.unreadable' };
      p.busy = false;
      return;
    }
    doc.add(id, p.name, report, loaded.snaps, engine.machine);
    ui.toast(t('import.added', { name: p.name }));
    this.pending = null;
  }

  cancel() {
    this.pending = null;
  }
}

export const importer = new Importer();
