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

  const p = $derived(importer.pending!);
  const report = $derived(importer.report);
  const failure = $derived(p.result && 'error' in p.result ? p.result.error : null);
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
