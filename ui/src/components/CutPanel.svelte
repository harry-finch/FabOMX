<script lang="ts">
  // Section « Découpe » (outil Découpe) : la qualité par défaut (un réglage
  // du poste), la légende des côtés et les qualités avec le nombre de
  // contours de chacun, le côté et la qualité des contours sélectionnés (sur
  // le plateau, Maj + clic pour en ajouter, ⌘A pour tous) et le retour au
  // côté détecté ou à la qualité par défaut, puis les avertissements :
  // contours qui se croisent (un clic centre la vue) et dessins qui se
  // chevauchent. Amorces : forme et longueurs des contours sélectionnés, et
  // le point d'entrée d'un contour seul, choisi sur le plateau. En bas, les
  // réglages par défaut du poste (qualité, amorces). Un changement sur des
  // contours est une étape ; un réglage par défaut n'en est pas une.
  import { CircleCheck, Crosshair, ListChecks, RotateCcw, TriangleAlert } from '@lucide/svelte';
  import NumField from './NumField.svelte';
  import Section from './Section.svelte';
  import Segmented from './Segmented.svelte';
  import { cutting, allowed, QUALITIES, LEAD_MIN, LEAD_MAX } from '../lib/cutting.svelte';
  import { doc, type Drawing } from '../lib/document.svelte';
  import { fmtLen, num } from '../lib/format';
  import { ui } from '../lib/ui.svelte';
  import { tip } from '../lib/tip';
  import { t, type Key } from '../lib/i18n.svelte';
  import type { LeadShape, Side } from '../lib/types';

  let { drawing: d }: { drawing: Drawing | null } = $props();

  const SIDES: Side[] = ['outside', 'inside', 'line', 'skip'];

  // Les contours de tous les dessins (pour les couleurs du plateau), et les
  // chevauchements, recalculés un peu après le dernier changement ; les
  // amorces de chaque dessin quand ce qui les produit change.
  $effect(() => {
    cutting.ensure(doc.drawings);
  });
  $effect(() => {
    const list = doc.drawings;
    void [cutting.byGeo, cutting.lead.shape, cutting.lead.lead_in, cutting.lead.lead_out];
    const timer = setTimeout(() => list.forEach((x) => cutting.refreshLeads(x)), 60);
    return () => clearTimeout(timer);
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
  /** Contours coupés de la sélection : ceux qui ont une qualité. */
  const cutPicked = $derived(d ? picked.filter((i) => cutting.qualityOf(d, i) != null) : []);
  const commonQuality = $derived.by(() => {
    if (!d || !cutPicked.length) return null;
    const first = cutting.qualityOf(d, cutPicked[0]);
    return cutPicked.every((i) => cutting.qualityOf(d, i) === first) ? first : null;
  });
  const ownQuality = $derived(!!d && cutPicked.some((i) => d.qualities[i] != null));
  const qualityCounts = $derived(
    d && s
      ? QUALITIES.map((q) => ({ q, n: s.contours.filter((_, i) => cutting.qualityOf(d, i) === q).length })).filter((x) => x.n > 0)
      : [],
  );
  const qualityOptions = QUALITIES.map((q) => ({ id: String(q), label: String(q), hint: t(`cut.qualityHints.${q}` as Key) }));
  /** Contours sélectionnés qui ont des amorces : fermés, extérieur ou trou. */
  const leadPicked = $derived(
    d && s ? picked.filter((i) => s.contours[i]?.closed && ['outside', 'inside'].includes(cutting.sideOf(d, i))) : [],
  );
  const specs = $derived(d ? leadPicked.map((i) => cutting.spec(d, i)) : []);
  const same = <K extends 'shape' | 'lead_in' | 'lead_out'>(k: K) => (specs.length && specs.every((x) => x[k] === specs[0][k]) ? specs[0][k] : null);
  const commonShape = $derived(same('shape'));
  const commonIn = $derived(same('lead_in'));
  const commonOut = $derived(same('lead_out'));
  const own = (k: 'shape' | 'lead_in' | 'lead_out') => !!d && leadPicked.some((i) => d.leads[i]?.[k] != null);
  const ownLeads = $derived(!!d && leadPicked.some((i) => d.leads[i] != null));
  const shapeOptions = (['line', 'arc'] as LeadShape[]).map((id) => ({ id, label: t(`cut.shapes.${id}`), hint: t(`cut.shapeHints.${id}`) }));
  const single = $derived(d && picked.length === 1 && cutting.sideOf(d, picked[0]) !== 'skip' ? picked[0] : null);
  const leadWarnings = $derived(d ? (cutting.leads(d)?.warnings ?? []) : []);
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
    {#if qualityCounts.length}
      <p class="sub">{t('cut.qualities')}</p>
      <ul class="legend">
        {#each qualityCounts as { q, n } (q)}
          <li use:tip={t(`cut.qualityHints.${q}` as Key)}>
            <span class="badge tabular" aria-hidden="true">{q}</span>
            <span class="name">{t('cut.qualityCount', { q })}</span>
            <span class="count tabular">{num(n)}</span>
          </li>
        {/each}
      </ul>
    {/if}

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
      <p class="sub">{t('cut.ownQuality')}</p>
      {#if cutPicked.length}
        <Segmented
          label={t('cut.quality')}
          value={commonQuality == null ? '' : String(commonQuality)}
          onchange={(q) => doc.setQualities(d.id, cutPicked, Number(q))}
          options={qualityOptions}
        />
        {#if cutPicked.length < picked.length}
          <p class="hint">{t('cut.noQuality')}</p>
        {/if}
      {:else}
        <p class="hint">{t('cut.noQuality')}</p>
      {/if}

      <p class="sub">{t('cut.leads')}</p>
      {#if leadPicked.length}
        <Segmented label={t('cut.leadShape')} value={commonShape ?? ('' as LeadShape)} onchange={(shape) => doc.setLeads(d.id, leadPicked, { shape })} options={shapeOptions} />
        <NumField
          label={t('cut.leadIn')}
          value={commonIn ?? cutting.lead.lead_in}
          min={LEAD_MIN}
          max={LEAD_MAX}
          decimals={2}
          step={0.1}
          hint={t('cut.leadInHint')}
          modified={own('lead_in')}
          onreset={() => doc.setLeads(d.id, leadPicked, { lead_in: undefined })}
          resetHint={t('cut.defaultHint', { v: fmtLen(cutting.lead.lead_in, ui.units, 2) })}
          onchange={(v) => doc.setLeads(d.id, leadPicked, { lead_in: v })}
        />
        <NumField
          label={t('cut.leadOut')}
          value={commonOut ?? cutting.lead.lead_out}
          min={LEAD_MIN}
          max={LEAD_MAX}
          decimals={2}
          step={0.1}
          hint={t('cut.leadOutHint')}
          modified={own('lead_out')}
          onreset={() => doc.setLeads(d.id, leadPicked, { lead_out: undefined })}
          resetHint={t('cut.defaultHint', { v: fmtLen(cutting.lead.lead_out, ui.units, 2) })}
          onchange={(v) => doc.setLeads(d.id, leadPicked, { lead_out: v })}
        />
      {/if}
      {#if leadPicked.length < picked.length}
        <p class="hint">{t('cut.noLead')}</p>
      {/if}
      <p class="sub">{t('cut.entry')}</p>
      {#if single != null}
        <div class="row">
          <button class="btn" class:on={cutting.pickingEntry} type="button" aria-pressed={cutting.pickingEntry} use:tip={t('cut.pickEntryHint')} onclick={() => (cutting.pickingEntry = !cutting.pickingEntry)}
            ><Crosshair size={14} /> {t(cutting.pickingEntry ? 'cut.pickingEntry' : 'cut.pickEntry')}</button
          >
          <button class="btn" type="button" disabled={d.entries[single] == null} use:tip={t('cut.autoEntryHint')} onclick={() => doc.setEntry(d.id, single, null)}
            ><RotateCcw size={14} /> {t('cut.autoEntry')}</button
          >
        </div>
      {:else}
        <p class="hint">{t('cut.oneForEntry')}</p>
      {/if}
      <div class="row">
        <button class="btn" type="button" disabled={!manual} use:tip={t('cut.autoHint')} onclick={() => doc.setSides(d.id, picked, null)}
          ><RotateCcw size={14} /> {t('cut.auto')}</button
        >
        <button class="btn" type="button" disabled={!ownQuality} use:tip={t('cut.useDefaultHint')} onclick={() => doc.setQualities(d.id, cutPicked, null)}
          ><RotateCcw size={14} /> {t('cut.useDefault')}</button
        >
        <button class="btn" type="button" disabled={!ownLeads} use:tip={t('cut.useDefaultLeadsHint')} onclick={() => doc.setLeads(d.id, leadPicked, null)}
          ><RotateCcw size={14} /> {t('cut.useDefaultLeads')}</button
        >
      </div>
    {:else}
      <p class="hint">{t('cut.pickHint')}</p>
    {/if}
    {#if s?.contours.length && picked.length < s.contours.length}
      <button class="btn" type="button" use:tip={t('cut.selectAllHint')} onclick={() => cutting.selectAll(d)}><ListChecks size={14} /> {t('cut.selectAll')}</button>
    {/if}

    {#if d.sidesReset}
      <p class="notice"><TriangleAlert size={14} /> {t('cut.reset')}</p>
    {/if}
    <p class="hint">{t('cut.direction')}</p>

    <p class="sub">{t('cut.warnings')}</p>
    {#if crossings.length || overlaps.length || leadWarnings.length}
      <ul class="warnings">
        {#each leadWarnings as w, n (n)}
          <li>
            <button type="button" onclick={() => cutting.focus(d.id, w.x, w.y)}>{t('cut.tooShort')} {n + 1}</button>
          </li>
        {/each}
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
  <div class="defaults">
    <p class="title">{t('cut.defaults')}</p>
    <p class="sub">{t('cut.defaultQuality')}</p>
    <Segmented label={t('cut.defaultQuality')} value={String(cutting.quality)} onchange={(q) => cutting.setDefault(Number(q))} options={qualityOptions} />
    <p class="sub">{t('cut.leadShape')}</p>
    <Segmented label={t('cut.leadShape')} value={cutting.lead.shape} onchange={(shape) => cutting.setLeadDefaults({ shape })} options={shapeOptions} />
    <NumField label={t('cut.leadIn')} value={cutting.lead.lead_in} min={LEAD_MIN} max={LEAD_MAX} decimals={2} step={0.1} hint={t('cut.leadInHint')} onchange={(v) => cutting.setLeadDefaults({ lead_in: v })} />
    <NumField label={t('cut.leadOut')} value={cutting.lead.lead_out} min={LEAD_MIN} max={LEAD_MAX} decimals={2} step={0.1} hint={t('cut.leadOutHint')} onchange={(v) => cutting.setLeadDefaults({ lead_out: v })} />
    <p class="hint">{t('cut.defaultsHint')}</p>
  </div>
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

  .badge {
    justify-self: center;
    color: var(--text-primary);
    font-size: var(--fs-xs);
    font-weight: 600;
  }

  .defaults {
    display: flex;
    flex-direction: column;
    gap: var(--s2);
    margin-top: var(--s1);
    padding-top: var(--s3);
    border-top: 1px solid var(--divider);
  }

  .title {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--fs-sm);
    font-weight: 600;
  }

  .btn.on {
    border-color: var(--accent);
    color: var(--accent-text);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s2);
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
