// Outils de la palette (panneau gauche). Tous les outils du jalon v0.1 y
// figurent ; ceux qui ne sont pas encore développés sont grisés (« à venir »).
// L'outil actif est mémorisé sur le poste.

import type { Component } from 'svelte';
import { FileUp, Hand, MousePointer2, Move, RotateCw, Scaling, WandSparkles } from '@lucide/svelte';
import { storageKey } from './app';

export type ToolId = 'select' | 'hand' | 'import' | 'move' | 'rotate' | 'scale' | 'clean';

export interface Tool {
  id: ToolId;
  icon: Component<{ size?: number }>;
  /** Touche du raccourci, en minuscule. */
  key: string;
  enabled: boolean;
}

/** Les outils par groupes, dans l'ordre de la palette. */
export const TOOL_GROUPS: Tool[][] = [
  [
    { id: 'select', icon: MousePointer2, key: 'v', enabled: true },
    { id: 'hand', icon: Hand, key: 'h', enabled: true },
  ],
  [{ id: 'import', icon: FileUp, key: 'i', enabled: false }],
  [
    { id: 'move', icon: Move, key: 'm', enabled: false },
    { id: 'rotate', icon: RotateCw, key: 'r', enabled: false },
    { id: 'scale', icon: Scaling, key: 's', enabled: false },
  ],
  [{ id: 'clean', icon: WandSparkles, key: 'c', enabled: false }],
];

const TOOLS = TOOL_GROUPS.flat();
const KEY = storageKey('tool.v1');

function read(): ToolId {
  try {
    const v = localStorage.getItem(KEY);
    const tool = TOOLS.find((x) => x.id === v);
    if (tool?.enabled) return tool.id;
  } catch {
    /* stockage indisponible */
  }
  return 'select';
}

class Tools {
  active = $state<ToolId>(read());

  /** Active un outil disponible ; un outil grisé ne réagit pas. */
  select(id: ToolId) {
    if (!TOOLS.find((x) => x.id === id)?.enabled) return;
    this.active = id;
    try {
      localStorage.setItem(KEY, id);
    } catch {
      /* ignoré */
    }
  }

  /** L'outil d'une touche de raccourci, s'il y en a un. */
  byKey(key: string): Tool | undefined {
    return TOOLS.find((x) => x.key === key.toLowerCase());
  }
}

export const tools = new Tools();
