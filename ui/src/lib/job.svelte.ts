// Le travail ouvert : son nom, son fichier (application de bureau), et s'il
// a des modifications non enregistrées. Un fichier `.fabomx` est un JSON
// autonome : la géométrie de chaque dessin telle que le moteur la garde
// (nettoyages compris), sa matrice, son point de référence et ses réglages
// faits à la main, le point de départ et l'ordre de coupe (par rang de
// dessin), et les réglages par défaut du poste au moment de l'enregistrement.
// Avant de perdre des modifications (nouveau, ouvrir, fermer), on demande.

import { engine } from './engine.svelte';
import { doc, roughPlacement, type Drawing, type Job } from './document.svelte';
import { cutting, LEAD_DEFAULTS } from './cutting.svelte';
import { ui } from './ui.svelte';
import { t } from './i18n.svelte';
import { APP_NAME } from './app';
import { baseName, forceClose, isDesktop, jobState, remember, saveFile, writeFile } from './desktop';
import type { Report } from './types';

/** Version du format écrite (`job::VERSION` du moteur). */
const VERSION = 1;
const EXT = '.fabomx';

type Defaults = { quality: number } & typeof LEAD_DEFAULTS;
export type Choice = 'save' | 'discard' | 'cancel';

class Jobs {
  /** Nom du travail (sans extension), `null` tant qu'il n'est pas enregistré. */
  name = $state<string | null>(null);
  /** Fichier du travail, dans l'application de bureau. */
  path: string | null = null;
  /** Révision du document au dernier enregistrement (ou à l'ouverture). */
  saved = $state(0);
  /** Question en cours avant de perdre des modifications. */
  asking = $state<((c: Choice) => void) | null>(null);
  /** Réglages par défaut d'un travail ouvert qui diffèrent de ceux du poste. */
  otherDefaults = $state<Defaults | null>(null);

  get dirty(): boolean {
    return doc.revision !== this.saved && (doc.drawings.length > 0 || this.name != null);
  }

  get title(): string {
    return `${this.name ?? t('job.untitled')}${this.dirty ? ' •' : ''} — ${APP_NAME}`;
  }

  #defaults(): Defaults {
    return { quality: cutting.quality, ...$state.snapshot(cutting.lead) };
  }

  /** Le texte du fichier de travail. */
  async #serialize(): Promise<string> {
    const drawings = [];
    for (const d of doc.drawings) {
      const geometry = await engine.geoExport(d.geo);
      if ('error' in geometry) throw new Error(String(geometry.error));
      drawings.push({
        name: d.name,
        geometry,
        matrix: d.matrix,
        ref: d.ref,
        sides: d.sides,
        qualities: d.qualities,
        leads: d.leads,
        entries: d.entries,
        tabs: d.tabs,
        open: d.open,
      });
    }
    const index = new Map(doc.drawings.map((d, i) => [d.id, i]));
    const order = doc.job.order?.flatMap(([id, c]) => (index.has(id) ? [[index.get(id)!, c]] : [])) ?? null;
    const file = { format: 'fabomx', version: VERSION, app: engine.version ?? '', defaults: this.#defaults(), job: { start: doc.job.start, order }, drawings };
    return JSON.stringify(file, null, 1);
  }

  /** Demande avant de perdre des modifications ; vrai si l'on peut continuer. */
  async #proceed(): Promise<boolean> {
    if (!this.dirty) return true;
    const choice = await new Promise<Choice>((resolve) => (this.asking = resolve));
    this.asking = null;
    if (choice === 'cancel') return false;
    if (choice === 'save') return this.save();
    return true;
  }

  answer(c: Choice) {
    this.asking?.(c);
  }

  async newJob() {
    if (!(await this.#proceed())) return;
    doc.replaceAll([], { start: [0, 0], order: null });
    this.name = null;
    this.path = null;
    this.saved = doc.revision;
  }

  /** Enregistre ; rend vrai si c'est fait. */
  async save(): Promise<boolean> {
    if (isDesktop && this.path) return this.#write(this.path);
    return this.saveAs();
  }

  async saveAs(): Promise<boolean> {
    let text: string;
    try {
      text = await this.#serialize();
    } catch {
      ui.toast(t('job.saveFailed'));
      return false;
    }
    const name = `${this.name ?? doc.drawings[0]?.name.replace(/\.[^.]+$/, '') ?? t('job.untitled')}${EXT}`;
    if (isDesktop) {
      try {
        const path = await saveFile(name, text, 'job');
        if (!path) return false;
        this.#done(path, baseName(path));
        remember(path).catch(() => {});
        return true;
      } catch {
        ui.toast(t('job.saveFailed'));
        return false;
      }
    }
    const url = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
    Object.assign(document.createElement('a'), { href: url, download: name }).click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    this.#done(null, name);
    return true;
  }

  async #write(path: string): Promise<boolean> {
    try {
      await writeFile(path, await this.#serialize());
      this.#done(path, baseName(path));
      return true;
    } catch {
      ui.toast(t('job.saveFailed'));
      return false;
    }
  }

  #done(path: string | null, file: string) {
    this.path = path;
    this.name = file.replace(/\.fabomx$/i, '');
    this.saved = doc.revision;
    ui.toast(t('job.saved', { name: file }));
  }

  /** Ouvre un fichier de travail (texte lu par l'importeur ou le dialogue natif). */
  async open(text: string, file: string, path?: string) {
    if (!(await this.#proceed())) return;
    const v = await engine.jobCheck(text).catch(() => ({ error: 'job.unreadable' }));
    if ('error' in v) {
      ui.toast(t(v.error === 'job.newer' ? 'job.newer' : 'job.unreadable', { name: file }));
      return;
    }
    const data = v as { defaults?: Defaults; job?: { start?: [number, number]; order?: [number, number][] | null }; drawings: Record<string, any>[] };
    const drawings: Drawing[] = [];
    for (const f of data.drawings) {
      const geo = doc.reserve();
      const r = await engine.geoImport(geo, f.geometry).catch(() => ({ error: 'job.unreadable' }));
      if ('error' in r) {
        ui.toast(t('job.unreadable', { name: file }));
        return;
      }
      const report: Report = r.report;
      drawings.push({
        id: doc.reserve(),
        geo,
        name: String(f.name ?? ''),
        report,
        matrix: f.matrix,
        ref: f.ref ?? { kind: 'box', ix: 1, iy: 1 },
        snaps: r.snaps,
        placement: roughPlacement(report, f.matrix),
        sides: f.sides ?? {},
        qualities: f.qualities ?? {},
        leads: f.leads ?? {},
        entries: f.entries ?? {},
        tabs: f.tabs ?? {},
        open: f.open ?? {},
        sidesReset: false,
      });
    }
    const order = data.job?.order?.flatMap(([i, c]) => (drawings[i] ? [[drawings[i].id, c] as [number, number]] : [])) ?? null;
    const job: Job = { start: data.job?.start ?? [0, 0], order };
    doc.replaceAll(drawings, job);
    this.path = path ?? null;
    this.name = file.replace(/\.fabomx$/i, '');
    this.saved = doc.revision;
    if (isDesktop && path) remember(path).catch(() => {});
    ui.toast(t('job.opened', { name: file }));
    // Réglages par défaut d'un autre poste : on le dit, on ne change rien.
    const d = data.defaults;
    const mine = this.#defaults();
    if (d && (Object.keys(mine) as (keyof Defaults)[]).some((k) => d[k] != null && d[k] !== mine[k])) this.otherDefaults = { ...mine, ...d };
  }

  /** Reprendre les réglages par défaut du travail ouvert. */
  adoptDefaults() {
    const d = this.otherDefaults;
    if (!d) return;
    cutting.setDefault(d.quality);
    cutting.setLeadDefaults({ shape: d.shape, lead_in: d.lead_in, lead_out: d.lead_out, tab_width: d.tab_width });
    this.otherDefaults = null;
  }

  /** Fermeture demandée (application de bureau) : on demande, puis on ferme. */
  async close() {
    if (await this.#proceed()) forceClose().catch(() => {});
  }

  /** Le titre et l'état « modifié », pour la fenêtre. */
  sync() {
    document.title = this.title;
    jobState(this.title, this.dirty).catch(() => {});
  }
}

export const jobs = new Jobs();
