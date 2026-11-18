// English dictionary: exactly the same keys as fr.ts (checked by TypeScript).

import type fr from './fr';

type Shape<T> = { [K in keyof T]: T[K] extends string ? string : Shape<T[K]> };

const en: Shape<typeof fr> = {
  app: {
    tagline: 'Toolpaths for the ProtoMAX',
    units: 'Units',
    inches: 'inches',
    language: 'Language',
    theme: 'Theme',
    themes: { system: 'System theme', light: 'Light theme', dark: 'Dark theme' },
    help: 'Help',
  },
  common: {
    close: 'Close',
    cancel: 'Cancel',
    apply: 'Apply',
  },
  tools: {
    label: 'Tools',
    soon: 'Coming soon',
    select: { name: 'Select', hint: 'Pick an element of the drawing.' },
    hand: { name: 'Hand', hint: 'Drag to move the view. Space + drag does the same with any tool.' },
    import: { name: 'Import', hint: 'Open an SVG or DXF drawing.' },
    move: { name: 'Move', hint: 'Move the drawing on the bed.' },
    rotate: { name: 'Rotate', hint: 'Rotate the drawing.' },
    scale: { name: 'Scale', hint: 'Enlarge or shrink the drawing.' },
    clean: { name: 'Clean up', hint: 'Close contours, remove duplicates, join segments.' },
  },
  bed: {
    title: 'Bed',
    machine: 'Machine',
    width: 'Width (X)',
    height: 'Depth (Y)',
    origin: 'Origin',
    originValue: 'bottom left',
    empty: 'Import a drawing to get started.',
  },
  workspace: {
    zoom: 'Zoom {v}',
  },
  status: {
    ready: 'Engine ready',
    loading: 'Loading engine…',
    failed: 'Engine unavailable',
    engine: 'Engine {v}',
  },
  alerts: {
    info: '{n} note',
    infos: '{n} notes',
  },
  help: {
    title: 'Help',
    shortcuts: 'Shortcuts',
    keys: {
      help: 'Open help',
      select: 'Select tool',
      hand: 'Hand tool',
      pan: 'Hold to move the view',
      wheel: 'Zoom around the pointer',
      fit: 'Fit the bed',
    },
    wheel: 'Wheel',
    space: 'Space',
  },
};

export default en;
