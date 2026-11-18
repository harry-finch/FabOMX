<script lang="ts">
  // Panneau droit : paramètres et formulaires. Avec un dessin sélectionné,
  // ses informations (celles du dialogue d'import, en lecture seule) et le
  // bouton Retirer ; sinon, le plateau de la machine et ce qu'il faut faire.
  import { Trash2 } from '@lucide/svelte';
  import Section from './Section.svelte';
  import DrawingInfo from './DrawingInfo.svelte';
  import Transform from './Transform.svelte';
  import CleanPanel from './CleanPanel.svelte';
  import CutPanel from './CutPanel.svelte';
  import PathPanel from './PathPanel.svelte';
  import { tools } from '../lib/tools.svelte';
  import type { Machine } from '../lib/engine.svelte';
  import type { Drawing } from '../lib/document.svelte';
  import { ui } from '../lib/ui.svelte';
  import { fmtLen } from '../lib/format';
  import { tip } from '../lib/tip';
  import { t } from '../lib/i18n.svelte';

  let {
    machine,
    drawing,
    count,
    onremove,
  }: { machine: Machine | null; drawing: Drawing | null; count: number; onremove: (id: number) => void } = $props();
</script>

{#if tools.active === 'clean'}
  <CleanPanel {drawing} />
{:else if tools.active === 'cut'}
  <CutPanel {drawing} />
{:else if tools.active === 'path'}
  <PathPanel />
{/if}
{#if drawing}
  {#if !['clean', 'cut', 'path'].includes(tools.active)}<Transform {drawing} />{/if}
  <Section id="drawing" title={t('drawing.title')}>
    {#snippet actions()}
      <button class="btn quiet icon" type="button" use:tip={t('drawing.removeHint')} aria-label={t('drawing.remove')} onclick={() => onremove(drawing.id)}><Trash2 size={14} /></button>
    {/snippet}
    <p class="name">{drawing.name} <span class="format">{drawing.report.format.toUpperCase()}</span></p>
    <DrawingInfo report={drawing.report} placement={drawing.placement} />
    <button class="btn" type="button" onclick={() => onremove(drawing.id)}><Trash2 size={14} /> {t('drawing.remove')}</button>
  </Section>
{:else if !['clean', 'cut', 'path'].includes(tools.active)}
  <p class="empty">{t(count ? 'bed.select' : 'bed.empty')}</p>
{/if}
{#if machine}
  <Section id="bed" title={t('bed.title')}>
    <dl class="props">
      <dt>{t('bed.machine')}</dt>
      <dd>{machine.name}</dd>
      <dt>{t('bed.width')}</dt>
      <dd class="tabular">{fmtLen(machine.width, ui.units, 1)}</dd>
      <dt>{t('bed.height')}</dt>
      <dd class="tabular">{fmtLen(machine.height, ui.units, 1)}</dd>
      <dt>{t('bed.origin')}</dt>
      <dd>{t('bed.originValue')}</dd>
    </dl>
  </Section>
{/if}

<style>
  .empty {
    margin: 0;
    padding: var(--s3);
    border-bottom: 1px solid var(--divider);
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .name {
    margin: 0;
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .format {
    padding: 1px 5px;
    border-radius: 3px;
    background: var(--neutral-tint);
    color: var(--text-secondary);
    font-size: var(--fs-xs);
  }

  .btn {
    align-self: flex-start;
  }

  .props {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 6px var(--s2);
    margin: 0;
  }

  .props dt {
    color: var(--text-secondary);
    font-size: var(--fs-sm);
  }

  .props dd {
    margin: 0;
    text-align: right;
  }
</style>
