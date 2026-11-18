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
}

/** Refus d'un fichier, par code stable du moteur. */
export interface ImportFailure {
  error: 'import.unknown_format' | 'import.unreadable' | 'import.binary_dxf' | 'import.empty';
}
