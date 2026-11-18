<script lang="ts">
  // Panneau droit : paramètres et formulaires. Sans sélection ni dessin, il
  // montre le plateau de la machine (en lecture seule) et dit quoi faire.
  import Section from './Section.svelte';
  import type { Machine } from '../lib/engine.svelte';
  import { ui } from '../lib/ui.svelte';
  import { fmtLen } from '../lib/format';
  import { t } from '../lib/i18n.svelte';

  let { machine }: { machine: Machine | null } = $props();
</script>

<p class="empty">{t('bed.empty')}</p>
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
