// Application de bureau (Tauri) : dialogues Ouvrir et Enregistrer natifs, fichiers récents,
// événements du menu macOS. Dans un navigateur, `isDesktop` est faux et
// l'import passe par le sélecteur du navigateur.

type Invoke = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
type Listen = (event: string, handler: (e: { payload: unknown }) => void) => Promise<() => void>;

const tauri = (globalThis as { __TAURI__?: { core: { invoke: Invoke }; event: { listen: Listen } } }).__TAURI__;

export const isDesktop = !!tauri;

export interface Opened {
  path: string;
  name: string;
  text: string;
}

/** Dialogue Ouvrir natif (SVG, DXF) ; `null` si annulé. */
export const openFile = () => tauri!.core.invoke<Opened | null>('open_file');

/** Dialogue Enregistrer natif (OMX) puis écriture ; le chemin, ou `null` si annulé. Rejette `unwritable`. */
export const saveFile = (name: string, text: string) => tauri!.core.invoke<string | null>('save_file', { name, text });

/** Relit un fichier ; rejette `missing` s'il n'existe plus (il sort alors des récents). */
export const openPath = (path: string) => tauri!.core.invoke<Opened>('open_path', { path });

/** Un dessin importé passe en tête des récents. */
export const remember = (path: string) => tauri!.core.invoke<void>('remember', { path });

export const recentFiles = () => (isDesktop ? tauri!.core.invoke<string[]>('recent_files') : Promise.resolve([]));

export const clearRecent = () => tauri!.core.invoke<void>('clear_recent');

/** Le menu natif suit la langue de l'interface. */
export const setMenuLocale = (value: string) => (isDesktop ? tauri!.core.invoke<void>('set_locale', { value }) : Promise.resolve());

/** Choix du menu natif (`import`, `undo`, `recent:2`…). */
export function onMenu(handler: (id: string) => void): () => void {
  if (!tauri) return () => {};
  let stop = () => {};
  tauri.event.listen('menu', (e) => handler(String(e.payload))).then((f) => (stop = f));
  return () => stop();
}
