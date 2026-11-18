<script lang="ts">
  // Dialogue d'import : nom et format du fichier, aperçu, puis les
  // informations du dessin (unité modifiable). « Importer » pose le dessin,
  // centré sur le plateau ; un fichier refusé dit pourquoi, et Importer
  // reste grisé.
  import { CircleX, FileText } from '@lucide/svelte';
  import Dialog from './Dialog.svelte';
  import DrawingPreview from './DrawingPreview.svelte';
  import DrawingInfo from './DrawingInfo.svelte';
  import { importer } from '../lib/importer.svelte';
  import { t, type Key } from '../lib/i18n.svelte';
  import { cleaning } from '../lib/cleaning.svelte';
  import { tip } from '../lib/tip';
  import { num } from '../lib/format';
  import type { CleanCounts } from '../lib/types';

  const p = $derived(importer.pending!);
  const report = $derived(importer.report);
  const failure = $derived(p.result && 'error' in p.result ? p.result.error : null);

  function cleanedList(c: CleanCounts): string {
    const parts = (['duplicates', 'overlaps', 'tiny', 'joins', 'simplified', 'arcs'] as const).filter((k) => c[k] > 0).map((k) => t(`clean.counts.${k}`, { n: num(c[k]) }));
    return parts.length ? t('clean.cleaned', { list: parts.join(', ') }) : t('clean.cleanedNothing');
  }
</script>

<Dialog title={t('import.title')} width={600} onclose={() => importer.cancel()}>
  <p class="file">
    <FileText size={14} />
    <span class="name">{p.name}</span>
    {#if report}<span class="format">{report.format.toUpperCase()}</span>{/if}
  </p>
  {#if failure}
    <p class="error" role="alert"><CircleX size={14} /> {t(`import.errors.${failure.split('.')[1]}` as Key)}</p>
  {:else if report}
    <div class="body">
      <label class="atimport" use:tip={t('clean.atImportHint')}>
        <input type="checkbox" checked={cleaning.atImport} disabled={p.busy} onchange={(e) => importer.setClean((e.currentTarget as HTMLInputElement).checked)} />
        {t('clean.atImport')}
        {#if report.cleaned}<span class="cleaned">· {cleanedList(report.cleaned)}</span>{/if}
      </label>
      <DrawingPreview {report} />
      <DrawingInfo {report} onunit={(u) => importer.setUnit(u)} />
    </div>
  {:else}
    <p class="loading">{t('import.reading')}</p>
  {/if}
  {#snippet footer({ close })}
    <button class="btn" type="button" onclick={close}>{t('common.cancel')}</button>
    <button class="btn primary" type="button" disabled={!report || p.busy} onclick={() => importer.confirm()}>{t('import.confirm')}</button>
  {/snippet}
</Dialog>

<style>
  .file {
    display: flex;
    align-items: center;
    gap: var(--s2);
    margin: 0 0 var(--s3);
    color: var(--text-secondary);
  }

  .name {
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .format {
    padding: 1px 5px;
    border-radius: 3px;
    background: var(--neutral-tint);
    font-size: var(--fs-xs);
    font-weight: 600;
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }

  .atimport {
    display: flex;
    align-items: center;
    gap: var(--s2);
    font-size: var(--fs-sm);
    color: var(--text-secondary);
  }

  .atimport input {
    margin: 0;
    accent-color: var(--accent);
  }

  .cleaned {
    color: var(--success);
  }

  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    color: var(--danger);
  }

  .loading {
    margin: 0;
    color: var(--text-muted);
  }
</style>
