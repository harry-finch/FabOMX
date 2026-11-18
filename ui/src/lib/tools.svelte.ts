// Outils de la palette (panneau gauche). Les outils prévus y figurent ;
// ceux qui ne sont pas encore développés sont grisés (« à venir »).
// Une action (Importer) agit au clic sans devenir l'outil actif. L'outil
// actif est mémorisé sur le poste.

import type { Component } from 'svelte';
import { FileUp, Hand, MousePointer2, Move, RotateCw, Scaling, Scissors, WandSparkles } from '@lucide/svelte';
import { storageKey } from './app';

export type ToolId = 'select' | 'hand' | 'import' | 'move' | 'rotate' | 'scale' | 'clean' | 'cut';

export interface Tool {
  id: ToolId;
  icon: Component<{ size?: number }>;
  /** Touche du raccourci, en minuscule. */
  key: string;
  enabled: boolean;
  /** Action immédiate, pas un mode. */
  action?: boolean;
}

/** Les outils par groupes, dans l'ordre de la palette. */
export const TOOL_GROUPS: Tool[][] = [
  [
    { id: 'select', icon: MousePointer2, key: 'v', enabled: true },
    { id: 'hand', icon: Hand, key: 'h', enabled: true },
  ],
  [{ id: 'import', icon: FileUp, key: 'i', enabled: true, action: true }],
  [
    { id: 'move', icon: Move, key: 'm', enabled: true },
    { id: 'rotate', icon: RotateCw, key: 'r', enabled: true },
    { id: 'scale', icon: Scaling, key: 's', enabled: true },
  ],
  [{ id: 'clean', icon: WandSparkles, key: 'c', enabled: true }],
  [{ id: 'cut', icon: Scissors, key: 'd', enabled: true }],
];

const TOOLS = TOOL_GROUPS.flat();
const KEY = storageKey('tool.v1');

function read(): ToolId {
  try {
    const v = localStorage.getItem(KEY);
    const tool = TOOLS.find((x) => x.id === v);
    if (tool?.enabled && !tool.action) return tool.id;
  } catch {
    /* stockage indisponible */
  }
  return 'select';
}

class Tools {
  active = $state<ToolId>(read());

  /** Active un outil disponible ; un outil grisé ou une action ne change rien. */
  select(id: ToolId) {
    const tool = TOOLS.find((x) => x.id === id);
    if (!tool?.enabled || tool.action) return;
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
