<script lang="ts">
  // Section « Découpe » (outil Découpe) : la légende des côtés avec le nombre
  // de contours de chacun, le côté des contours sélectionnés (sur le
  // plateau, Maj + clic pour en ajouter) et le retour au côté détecté, puis
  // les avertissements : contours qui se croisent (un clic centre la vue) et
  // dessins qui se chevauchent. Un changement de côté est une étape.
  import { CircleCheck, RotateCcw, TriangleAlert } from '@lucide/svelte';
  import Section from './Section.svelte';
  import Segmented from './Segmented.svelte';
  import { cutting, allowed } from '../lib/cutting.svelte';
  import { doc, type Drawing } from '../lib/document.svelte';
  import { num } from '../lib/format';
  import { tip } from '../lib/tip';
  import { t } from '../lib/i18n.svelte';
  import type { Side } from '../lib/types';

  let { drawing: d }: { drawing: Drawing | null } = $props();

  const SIDES: Side[] = ['outside', 'inside', 'line', 'skip'];

  // Les contours de tous les dessins (pour les couleurs du plateau), et les
  // chevauchements, recalculés un peu après le dernier changement.
  $effect(() => {
    cutting.ensure(doc.drawings);
  });
  $effect(() => {
    const list = doc.drawings;
    const timer = setTimeout(() => cutting.refreshOverlaps(list), 150);
    return () => clearTimeout(timer);
  });

  const s = $derived(d ? cutting.sides(d) : null);
  const counts = $derived(
    d && s ? SIDES.map((side) => ({ side, n: s.contours.filter((_, i) => cutting.sideOf(d, i) === side).length })) : [],
  );
  const picked = $derived(d && cutting.selection?.id === d.id ? cutting.selection.contours : []);
  const options = $derived(allowed(!s || picked.every((i) => s.contours[i]?.closed)));
  const common = $derived.by(() => {
    if (!d || !picked.length) return null;
    const first = cutting.sideOf(d, picked[0]);
    return picked.every((i) => cutting.sideOf(d, i) === first) ? first : null;
  });
  const manual = $derived(!!d && picked.some((i) => d.sides[i] != null));
  const crossings = $derived(s?.warnings ?? []);
  const overlaps = $derived(
    d
      ? cutting.overlaps
          .filter(([a, b]) => a === d.id || b === d.id)
          .map(([a, b]) => doc.drawings.find((x) => x.id === (a === d.id ? b : a)))
          .filter((x): x is Drawing => !!x)
      : [],
  );
</script>

<Section id="cut" title={t('cut.title')}>
  {#if !d}
    <p class="muted">{t('cut.select')}</p>
  {:else}
    <p class="sub">{t('cut.legend')}</p>
    <ul class="legend">
      {#each counts as { side, n } (side)}
        <li use:tip={t(`cut.sideHints.${side}`)}>
          <span class="swatch {side}" aria-hidden="true"></span>
          <span class="name">{t(`cut.sides.${side}`)}</span>
          <span class="count tabular">{num(n)}</span>
        </li>
      {/each}
    </ul>

    {#if picked.length}
      <p class="sub">{t('cut.selected', { n: picked.length })}</p>
      <Segmented
        label={t('cut.side')}
        columns={2}
        value={common ?? ('' as Side)}
        onchange={(side) => doc.setSides(d.id, picked, side)}
        options={options.map((id) => ({ id, label: t(`cut.sides.${id}`), hint: t(`cut.sideHints.${id}`) }))}
      />
      {#if options.length < SIDES.length}
        <p class="hint">{t('cut.openOnly')}</p>
      {/if}
      <button class="btn" type="button" disabled={!manual} use:tip={t('cut.autoHint')} onclick={() => doc.setSides(d.id, picked, null)}
        ><RotateCcw size={14} /> {t('cut.auto')}</button
      >
    {:else}
      <p class="hint">{t('cut.pickHint')}</p>
    {/if}

    {#if d.sidesReset}
      <p class="notice"><TriangleAlert size={14} /> {t('cut.reset')}</p>
    {/if}
    <p class="hint">{t('cut.direction')}</p>

    <p class="sub">{t('cut.warnings')}</p>
    {#if crossings.length || overlaps.length}
      <ul class="warnings">
        {#each crossings as w, n (n)}
          <li>
            <button type="button" onclick={() => cutting.focus(d.id, w.x, w.y)}>{t('cut.crossing')} {n + 1}</button>
          </li>
        {/each}
        {#each overlaps as other (other.id)}
          <li><span>{t('cut.overlap', { name: other.name })}</span></li>
        {/each}
      </ul>
    {:else if s}
      <p class="ok"><CircleCheck size={14} /> {t('cut.noWarning')}</p>
    {/if}
  {/if}
</Section>

<style>
  .muted {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .sub {
    margin: var(--s1) 0 0;
    color: var(--text-secondary);
    font-size: var(--fs-sm);
  }

  .legend {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .legend li {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--s2);
    min-height: 22px;
  }

  .swatch {
    height: 0;
    border-top: 2px solid var(--cut-line);
  }

  .swatch.outside {
    border-top-color: var(--cut-outside);
  }

  .swatch.inside {
    border-top-color: var(--cut-inside);
  }

  .swatch.skip {
    border-top: 2px dashed var(--cut-skip);
  }

  .count {
    color: var(--text-secondary);
  }

  .hint {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-xs);
  }

  .notice {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    margin: 0;
    padding: var(--s2);
    border-radius: var(--radius);
    background: var(--warning-tint);
    color: var(--warning);
    font-size: var(--fs-sm);
  }

  .notice :global(svg) {
    flex: none;
    margin-top: 1px;
  }

  .ok {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    color: var(--success);
    font-size: var(--fs-sm);
  }

  .btn {
    align-self: flex-start;
  }

  .warnings {
    display: flex;
    flex-direction: column;
    gap: 1px;
    max-height: 200px;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .warnings button,
  .warnings span {
    display: block;
    width: 100%;
    min-height: 24px;
    padding: 4px var(--s1);
    background: none;
    border: 0;
    border-radius: var(--radius);
    color: var(--warning);
    font-size: var(--fs-sm);
    text-align: left;
    overflow-wrap: anywhere;
  }

  .warnings button:hover {
    background: var(--bg-hover);
  }
</style>
