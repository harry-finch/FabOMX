// Réglages du nettoyage (cases et seuils, en mm du plateau) et « Nettoyer à
// l'import », mémorisés sur le poste ; l'aperçu du dessin sélectionné, que
// partagent la section Nettoyer et l'espace de travail ; le défaut à montrer.

import { storageKey } from './app';
import { engine } from './engine.svelte';
import { factorOf } from './affine';
import type { Drawing } from './document.svelte';
import type { CleanOptions, CleanPreview, Defect } from './types';

const KEY = storageKey('clean.v1');

export const DEFAULTS: CleanOptions = { duplicates: true, overlaps: true, tiny: true, tiny_size: 0.1, join: true, join_gap: 0.01, simplify: true, arcs: true, arc_tol: 0.01 };

function read(): { options: CleanOptions; atImport: boolean } {
  try {
    const v = JSON.parse(localStorage.getItem(KEY) ?? 'null');
    if (v && typeof v === 'object') return { options: { ...DEFAULTS, ...v.options }, atImport: !!v.atImport };
  } catch {
    /* stockage indisponible ou illisible */
  }
  return { options: { ...DEFAULTS }, atImport: false };
}

class Cleaning {
  #saved = read();
  options = $state<CleanOptions>(this.#saved.options);
  atImport = $state(this.#saved.atImport);
  /** Aperçu du dessin sélectionné, pour sa géométrie et son échelle actuelles. */
  preview = $state<{ geo: number; preview: CleanPreview } | null>(null);
  /** Défaut mis en évidence (index dans `preview.defects`), et un compteur pour recentrer la vue. */
  focused = $state<{ index: number; defect: Defect; seq: number } | null>(null);
  #seq = 0;
  #asked = 0;

  save() {
    try {
      localStorage.setItem(KEY, JSON.stringify({ options: this.options, atImport: this.atImport }));
    } catch {
      /* ignoré */
    }
  }

  set(patch: Partial<CleanOptions>) {
    this.options = { ...this.options, ...patch };
    this.save();
  }

  setAtImport(on: boolean) {
    this.atImport = on;
    this.save();
  }

  /** Recalcule l'aperçu de `d` ; le dernier demandé gagne. */
  async refresh(d: Drawing | null) {
    const ask = ++this.#asked;
    if (!d) {
      this.preview = null;
      return;
    }
    const p = await engine.cleanPreview(d.geo, $state.snapshot(this.options), factorOf(d.matrix)).catch(() => null);
    if (ask !== this.#asked) return;
    this.preview = p && !('error' in p) ? { geo: d.geo, preview: p } : null;
    this.focused = null;
  }

  focus(index: number) {
    const defect = this.preview?.preview.defects[index];
    if (defect) this.focused = { index, defect, seq: ++this.#seq };
  }
}

export const cleaning = new Cleaning();
