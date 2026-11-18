<script lang="ts">
  // Les informations d'un dessin lu : unité (modifiable dans le dialogue
  // d'import), dimensions, éléments par type, contours, ce qu'il faudra
  // nettoyer et les avertissements. Partagé par le dialogue d'import et le
  // panneau droit (en lecture seule).
  import { CircleCheck } from '@lucide/svelte';
  import Segmented from './Segmented.svelte';
  import Alerts, { type Alert } from './Alerts.svelte';
  import type { Kind, Report, Unit } from '../lib/types';
  import { ui } from '../lib/ui.svelte';
  import { engine } from '../lib/engine.svelte';
  import { fmtLen, num } from '../lib/format';
  import { t, labelled, type Key } from '../lib/i18n.svelte';

  let { report, onunit }: { report: Report; onunit?: (u: Unit) => void } = $props();

  const KINDS: Kind[] = ['line', 'arc', 'circle', 'ellipse', 'polyline', 'rect', 'spline', 'path'];
  const BASE_UNITS: Unit[] = ['mm', 'cm', 'in', 'px'];
  const units = $derived(BASE_UNITS.includes(report.detected) ? BASE_UNITS : [...BASE_UNITS, report.detected]);
  const KNOWN_IGNORED = ['text', 'mtext', 'insert', 'hatch', 'dimension', 'image', 'use', 'foreignobject'];

  const cleanup = $derived(
    (
      [
        ['open', report.cleanup.open],
        ['duplicates', report.cleanup.duplicates],
        ['gaps', report.cleanup.gaps],
        ['tiny', report.cleanup.tiny],
      ] as const
    ).filter(([, n]) => n > 0),
  );

  function ignoredList(): string {
    return Object.entries(report.ignored)
      .map(([name, n]) => {
        const k = name.toLowerCase();
        const label = KNOWN_IGNORED.includes(k) ? t(`ignored.${k}` as Key) : name;
        return `${label} ${num(n)}`;
      })
      .join(' · ');
  }

  const alerts = $derived<Alert[]>(
    report.warnings.map((w) => {
      if (w.code === 'import.larger_than_bed')
        return { code: w.code, severity: 'warning', message: t('import.warnings.larger', { w: fmtLen(w.width, ui.units, 1), h: fmtLen(w.height, ui.units, 1), bed: fmtLen(engine.machine?.width, ui.units, 1) }) };
      if (w.code === 'import.ignored') return { code: w.code, severity: 'info', message: labelled(t('import.warnings.ignored', { n: w.count }), ignoredList()) };
      return { code: w.code, severity: 'warning', message: t('import.warnings.noClosed') };
    }),
  );
</script>

<div class="info">
  {#if onunit}
    <p class="sub">{t('import.unit')} <span class="muted">· {t(report.source === 'file' ? 'import.sourceFile' : 'import.sourceAssumed', { u: t(`units.${report.detected}`) })}</span></p>
    <Segmented label={t('import.unit')} value={report.unit} onchange={onunit} options={units.map((u) => ({ id: u, label: t(`units.${u}`) }))} />
  {/if}
  <dl class="props">
    {#if !onunit}
      <dt>{t('import.unit')}</dt>
      <dd>{t(`units.${report.unit}`)}{report.source === 'assumed' ? ` (${t('import.assumed')})` : ''}</dd>
    {/if}
    <dt>{t('import.size')}</dt>
    <dd class="tabular strong">{fmtLen(report.width, ui.units, 2)} × {fmtLen(report.height, ui.units, 2)}</dd>
    {#each KINDS.filter((k) => report.counts[k]) as k (k)}
      <dt>{t(`kinds.${k}`)}</dt>
      <dd class="tabular">{num(report.counts[k])}</dd>
    {/each}
    <dt>{t('import.closed')}</dt>
    <dd class="tabular">{num(report.closed)}</dd>
    <dt>{t('import.open')}</dt>
    <dd class="tabular" class:warn={report.open > 0}>{num(report.open)}</dd>
  </dl>

  <p class="sub">{t('import.cleanup')}</p>
  {#if cleanup.length}
    <ul class="cleanup">
      {#each cleanup as [k, n] (k)}
        <li><span>{t(`import.clean.${k}`)}</span><span class="tabular">{num(n)}</span></li>
      {/each}
    </ul>
    <p class="hint">{t('import.cleanHint')}</p>
  {:else}
    <p class="ok"><CircleCheck size={14} /> {t('import.nothingToClean')}</p>
  {/if}

  {#if alerts.length}
    <Alerts {alerts} />
  {/if}
</div>

<style>
  .info {
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }

  .sub {
    margin: var(--s1) 0 0;
    color: var(--text-secondary);
    font-size: var(--fs-sm);
  }

  .muted {
    color: var(--text-muted);
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

  .strong {
    font-weight: 600;
  }

  .warn {
    color: var(--warning);
  }

  .cleanup {
    display: grid;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .cleanup li {
    display: flex;
    justify-content: space-between;
    gap: var(--s2);
    color: var(--warning);
    font-size: var(--fs-sm);
  }

  .hint {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-xs);
  }

  .ok {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    color: var(--success);
    font-size: var(--fs-sm);
  }
</style>
