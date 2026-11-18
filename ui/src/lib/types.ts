// Types des données du moteur (JSON de `fabomx-wasm`).

export type Unit = 'mm' | 'cm' | 'm' | 'in' | 'ft' | 'px' | 'pt' | 'pc';
export type Kind = 'line' | 'arc' | 'circle' | 'ellipse' | 'polyline' | 'rect' | 'spline' | 'path';

export type ImportWarning =
  | { code: 'import.no_closed_contour' }
  | { code: 'import.larger_than_bed'; width: number; height: number }
  | { code: 'import.ignored'; count: number };

/** L'état d'un dessin lu dans un fichier (mm, Y vers le haut). */
export interface Report {
  format: 'svg' | 'dxf';
  unit: Unit;
  detected: Unit;
  source: 'file' | 'assumed';
  width: number;
  height: number;
  bbox: [number, number, number, number];
  counts: Partial<Record<Kind, number>>;
  closed: number;
  open: number;
  cleanup: { open: number; duplicates: number; gaps: number; tiny: number };
  markers: { kind: 'gap' | 'duplicate' | 'tiny'; x: number; y: number }[];
  warnings: ImportWarning[];
  ignored: Record<string, number>;
  traces: { open: boolean; pts: number[] }[];
  /** Pièces proposées « Ne pas couper » d'après le fichier (pointillés, calques de construction). */
  construction: number;
  /** Ce que « Nettoyer à l'import » a corrigé. */
  cleaned?: CleanCounts;
}

/** Refus d'un fichier, par code stable du moteur. */
export interface ImportFailure {
  error: 'import.unknown_format' | 'import.unreadable' | 'import.binary_dxf' | 'import.empty';
}

export type PlaceWarning = { code: 'place.overflow'; x: number; y: number } | { code: 'place.distorted' };

/** Un dessin transformé, sur le plateau (mm). */
export interface Placement {
  bbox: [number, number, number, number];
  width: number;
  height: number;
  warnings: PlaceWarning[];
}

/** Réglages du nettoyage ; seuils en mm du plateau. */
export interface CleanOptions {
  duplicates: boolean;
  overlaps: boolean;
  tiny: boolean;
  tiny_size: number;
  join: boolean;
  join_gap: number;
  simplify: boolean;
  arcs: boolean;
  arc_tol: number;
}

export interface CleanCounts {
  duplicates: number;
  overlaps: number;
  tiny: number;
  joins: number;
  simplified: number;
  arcs: number;
}

/** Un défaut situé dans la géométrie d'origine ; `gap` : écart d'une extrémité qui restera libre. */
export interface Defect {
  kind: 'duplicate' | 'overlap' | 'tiny' | 'gap' | 'open';
  x: number;
  y: number;
  gap?: number;
}

export interface CleanPreview {
  counts: CleanCounts;
  closed: number;
  open: number;
  defects: Defect[];
}

/** Côté de coupe d'un contour : la pièce dedans, autour, fente, ou repère. */
export type Side = 'outside' | 'inside' | 'line' | 'skip';

/** Un contour mis en segments, dans le repère d'origine du dessin. */
export interface Contour {
  closed: boolean;
  /** Repère d'après le fichier. */
  construction: boolean;
  /** Côté détecté. */
  side: Side;
  depth: number;
  /** Aire signée (positive dans le sens trigonométrique) ; 0 si ouvert. */
  area: number;
  crossing: [number, number] | null;
  /** `[x, y, …]` dans le sens du tracé ; un contour fermé ne répète pas le premier point. */
  pts: number[];
}

export type CutWarning = { code: 'cut.crossing'; contour: number; x: number; y: number };

export interface Sides {
  contours: Contour[];
  warnings: CutWarning[];
}

export type LeadShape = 'line' | 'arc';

/** Onglets demandés : aucun, un nombre réparti (le premier est l'onglet de fin), ou placés (repère d'origine). */
export type TabSpec = { kind: 'none' } | { kind: 'count'; n: number } | { kind: 'at'; end: boolean; points: [number, number][] };

/** Réglages d'amorce d'un contour pour le moteur (longueurs en mm du plateau). */
export interface LeadSpec {
  side: Side;
  shape: LeadShape;
  lead_in: number;
  lead_out: number;
  entry: [number, number] | null;
  tabs: TabSpec;
  tab_width: number;
  /** Tracé ouvert : côté de la pièce par rapport au sens dessiné ; `null` : sur la ligne. */
  part: Part | null;
  /** Tracé ouvert : amorces dans le prolongement. */
  extend: boolean;
}

export type Part = 'left' | 'right';

/** Un onglet placé (repère d'origine) : centre, onglet de fin ou non, sortie et reprise `[x, y, …]`. */
export interface Tab {
  at: [number, number];
  end: boolean;
  start: number;
  stop: number;
  lead_out: number[];
  lead_in: number[];
}

/** Les amorces d'un contour, en repère d'origine ; `lead_in` et `lead_out` : `[x, y, …]`. */
export interface Lead {
  pierce: [number, number];
  entry: [number, number];
  lead_in: number[];
  lead_out: number[];
  reversed: boolean;
  /** MAKE décale la saignée (pièce à droite du jet). */
  offset: boolean;
  tabs: Tab[];
}

export type LeadWarning = { code: 'lead.too_short' | 'tab.too_close'; contour: number; x: number; y: number };

export interface Leads {
  leads: (Lead | null)[];
  warnings: LeadWarning[];
}

export type MoveKind = 'traverse' | 'lead_in' | 'cut' | 'lead_out';

/** Un mouvement de la buse sur le plateau ; `step` : rang du contour (aucun pour le retour). */
export interface Move {
  kind: MoveKind;
  step: number | null;
  side: Side | null;
  quality: number | null;
  offset: boolean;
  pts: number[];
}

/** Un contour dans l'ordre de coupe (`key` : identifiant du dessin). */
export interface Step {
  key: number;
  contour: number;
  side: Side;
  pierce: [number, number];
}

export interface PathStats {
  /** Longueur coupée par qualité (mm), amorces à part. */
  cut: Record<string, number>;
  leads: number;
  traverse: number;
  pierces: number;
}

export type PathWarning =
  | { code: 'path.over_cut'; step: number; x: number; y: number }
  | { code: 'lead.too_short' | 'tab.too_close'; key: number; contour: number; x: number; y: number };

export interface Plan {
  start: [number, number];
  order: Step[];
  moves: Move[];
  stats: PathStats;
  warnings: PathWarning[];
}

/** Ordre refusé : `contour` serait coupé avant `inner`, qu'il contient. */
export interface OrderRefused {
  error: 'path.order';
  contour: [number, number];
  inner: [number, number];
}
