// Le document : les dessins posés sur le plateau, la sélection, et
// l'historique pour annuler / rétablir. Chaque modification remplace la
// liste (les dessins sont immuables) et empile l'état précédent.

import type { Report } from './types';

export interface Drawing {
  id: number;
  /** Nom du fichier importé. */
  name: string;
  report: Report;
  /** Décalage (mm) qui pose le dessin sur le plateau. */
  offset: [number, number];
}

const HISTORY = 100;

class Doc {
  drawings = $state<Drawing[]>([]);
  selected = $state<number | null>(null);
  #undo: Drawing[][] = [];
  #redo: Drawing[][] = [];
  #next = 1;
  /** Taille des piles, pour l'état des commandes. */
  depth = $state({ undo: 0, redo: 0 });

  get selection(): Drawing | null {
    return this.drawings.find((d) => d.id === this.selected) ?? null;
  }

  #commit(next: Drawing[]) {
    this.#undo.push(this.drawings);
    if (this.#undo.length > HISTORY) this.#undo.shift();
    this.#redo = [];
    this.drawings = next;
    this.#sync();
  }

  #sync() {
    if (this.selected != null && !this.drawings.some((d) => d.id === this.selected)) this.selected = null;
    this.depth = { undo: this.#undo.length, redo: this.#redo.length };
  }

  /** Ajoute un dessin centré sur un plateau de `bed` (mm) et le sélectionne. */
  add(name: string, report: Report, bed: { width: number; height: number }) {
    const [x0, y0, x1, y1] = report.bbox;
    const drawing: Drawing = { id: this.#next++, name, report, offset: [bed.width / 2 - (x0 + x1) / 2, bed.height / 2 - (y0 + y1) / 2] };
    this.#commit([...this.drawings, drawing]);
    this.selected = drawing.id;
  }

  remove(id: number) {
    if (!this.drawings.some((d) => d.id === id)) return;
    this.#commit(this.drawings.filter((d) => d.id !== id));
  }

  undo() {
    const prev = this.#undo.pop();
    if (!prev) return;
    this.#redo.push(this.drawings);
    this.drawings = prev;
    this.#sync();
  }

  redo() {
    const next = this.#redo.pop();
    if (!next) return;
    this.#undo.push(this.drawings);
    this.drawings = next;
    this.#sync();
  }

  select(id: number | null) {
    this.selected = id;
  }
}

export const doc = new Doc();
