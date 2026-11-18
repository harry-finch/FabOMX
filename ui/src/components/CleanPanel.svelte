<script lang="ts">
  // Section « Nettoyer » (outil Nettoyer) : cinq opérations à cocher, leurs
  // seuils (mm du plateau), le nombre de corrections de chacune et le bilan,
  // en direct, avant d'appliquer ; ce qui restera ouvert et pourquoi ; la
  // liste des défauts (un clic centre la vue dessus). « Nettoyer » corrige
  // la géométrie d'origine : la transformation est gardée, une étape.
  import { CircleCheck, Sparkles } from '@lucide/svelte';
  import Section from './Section.svelte';
  import NumField from './NumField.svelte';
  import { cleaning } from '../lib/cleaning.svelte';
  import { doc, type Drawing } from '../lib/document.svelte';
  import { engine } from '../lib/engine.svelte';
  import { factorOf } from '../lib/affine';
  import { ui } from '../lib/ui.svelte';
  import { fmtLen, num } from '../lib/format';
  import { t, type Key } from '../lib/i18n.svelte';
  import type { CleanCounts, Defect } from '../lib/types';

  let { drawing: d }: { drawing: Drawing | null } = $props();

  // L'aperçu suit le dessin (géométrie, échelle) et les réglages.
  $effect(() => {
    void [d?.geo, d?.matrix, cleaning.options];
    cleaning.refresh(d);
  });

  const pv = $derived(d && cleaning.preview?.geo === d.geo ? cleaning.preview.preview : null);
  let busy = $state(false);

  type Op = 'duplicates' | 'overlaps' | 'tiny' | 'join' | 'simplify' | 'arcs';
  const OPS: { op: Op; count: keyof CleanCounts }[] = [
    { op: 'duplicates', count: 'duplicates' },
    { op: 'overlaps', count: 'overlaps' },
    { op: 'tiny', count: 'tiny' },
    { op: 'join', count: 'joins' },
    { op: 'simplify', count: 'simplified' },
    { op: 'arcs', count: 'arcs' },
  ];
  const KINDS: Defect['kind'][] = ['gap', 'duplicate', 'overlap', 'tiny', 'open'];
  const MAX_SHOWN = 30;

  const groups = $derived(
    pv
      ? KINDS.map((kind) => ({ kind, items: pv.defects.map((x, i) => ({ x, i })).filter(({ x }) => x.kind === kind) })).filter((g) => g.items.length)
      : [],
  );

  function label(x: Defect, n: number): string {
    if (x.kind !== 'open') return `${t(`clean.defects.${x.kind}` as Key)} ${n}`;
    if (x.gap == null) return t('clean.openIsolated', { n });
    return t('clean.openGap', { n, gap: fmtLen(x.gap, ui.units, 3) });
  }

  async function apply() {
    if (!d || busy) return;
    busy = true;
    const next = doc.reserve();
    const res = await engine.cleanApply(d.geo, $state.snapshot(cleaning.options), factorOf(d.matrix), next).catch(() => null);
    busy = false;
    if (!res || 'error' in res) return;
    doc.replaceGeometry(d.id, next, res.report, res.snaps);
    const c = res.counts;
    ui.toast(t('clean.done', { n: c.duplicates + c.overlaps + c.tiny + c.joins + c.simplified + c.arcs }));
  }
</script>

<Section id="clean" title={t('clean.title')}>
  {#if !d}
    <p class="muted">{t('clean.select')}</p>
  {:else}
    <div class="ops">
      {#each OPS as { op, count } (op)}
        {@const on = cleaning.options[op]}
        {@const n = pv?.counts[count] ?? 0}
        <label class="op">
          <input type="checkbox" checked={on} onchange={(e) => cleaning.set({ [op]: (e.currentTarget as HTMLInputElement).checked })} />
          <span class="name">{t(`clean.ops.${op}`)}</span>
          <span class="count tabular" class:some={n > 0} class:off={!on}>{pv ? num(n) : '…'}</span>
        </label>
        {#if op === 'tiny'}
          <div class="indent">
            <NumField label={t('clean.tinySize')} value={cleaning.options.tiny_size} min={0.001} max={1} decimals={3} step={0.01} disabled={!on} onchange={(v) => cleaning.set({ tiny_size: v })} hint={t('clean.tinyHint')} />
          </div>
        {:else if op === 'join'}
          <div class="indent">
            <NumField label={t('clean.joinGap')} value={cleaning.options.join_gap} min={0.001} max={1} decimals={3} step={0.01} disabled={!on} onchange={(v) => cleaning.set({ join_gap: v })} hint={t('clean.joinHint')} />
          </div>
        {:else if op === 'arcs'}
          <div class="indent">
            <NumField label={t('clean.arcTol')} value={cleaning.options.arc_tol} min={0.001} max={0.1} decimals={3} step={0.005} disabled={!on} onchange={(v) => cleaning.set({ arc_tol: v })} hint={t('clean.arcHint')} />
          </div>
        {/if}
      {/each}
    </div>

    {#if pv}
      <p class="sub">{t('clean.after')}</p>
      <dl class="props">
        <dt>{t('import.closed')}</dt>
        <dd class="tabular">{num(pv.closed)}</dd>
        <dt>{t('import.open')}</dt>
        <dd class="tabular" class:warn={pv.open > 0}>{num(pv.open)}</dd>
      </dl>
      {#if pv.open === 0}
        <p class="ok"><CircleCheck size={14} /> {t('clean.allClosed')}</p>
      {:else}
        <p class="hint">{t('clean.openHint')}</p>
      {/if}

      {#if groups.length}
        <p class="sub">{t('clean.list')}</p>
        <ul class="defects">
          {#each groups as g (g.kind)}
            {#each g.items.slice(0, MAX_SHOWN) as { x, i }, n (i)}
              <li>
                <button type="button" class:on={cleaning.focused?.index === i} class:open={x.kind === 'open'} onclick={() => cleaning.focus(i)}>{label(x, n + 1)}</button>
              </li>
            {/each}
            {#if g.items.length > MAX_SHOWN}
              <li class="more">{t('clean.more', { n: g.items.length - MAX_SHOWN })}</li>
            {/if}
          {/each}
        </ul>
      {/if}
    {/if}

    <button class="btn primary" type="button" disabled={busy || !pv} onclick={apply}><Sparkles size={14} /> {t('clean.apply')}</button>
  {/if}
</Section>

<style>
  .muted {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .ops {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .op {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--s2);
    min-height: 24px;
    cursor: pointer;
  }

  .op input {
    margin: 0;
    accent-color: var(--accent);
  }

  .count {
    color: var(--text-muted);
  }

  .count.some {
    color: var(--warning);
    font-weight: 600;
  }

  .count.off {
    opacity: 0.6;
  }

  .indent {
    padding-left: 22px;
  }

  .sub {
    margin: var(--s2) 0 0;
    color: var(--text-secondary);
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

  .warn {
    color: var(--warning);
  }

  .ok {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    color: var(--success);
    font-size: var(--fs-sm);
  }

  .hint {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-xs);
  }

  .defects {
    display: flex;
    flex-direction: column;
    gap: 1px;
    max-height: 220px;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .defects button {
    width: 100%;
    height: 24px;
    padding: 0 var(--s1);
    background: none;
    border: 0;
    border-radius: var(--radius);
    color: var(--text-secondary);
    font-size: var(--fs-sm);
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .defects button:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .defects button.open {
    color: var(--warning);
  }

  .defects button.on {
    background: var(--bg-selected);
    color: var(--accent-text);
  }

  .more {
    padding: 0 var(--s1);
    color: var(--text-muted);
    font-size: var(--fs-xs);
  }

  .btn.primary {
    align-self: flex-start;
  }
</style>
