<script lang="ts">
  // Section « Parcours » (outil Parcours) : le point de départ (saisi, choisi
  // sur le plateau, ou l'origine), l'ordre de coupe (un clic met un contour en
  // évidence ; Plus tôt / Plus tard, refusés quand un contour passerait avant
  // l'un de ceux qu'il contient ; retour à l'ordre automatique), le bilan, les
  // remarques (un clic centre la vue) et la lecture pas à pas. Le plan est
  // recalculé un peu après chaque changement du travail. « Exporter l'OMX »
  // liste d'abord les remarques qui restent, à confirmer.
  import { Crosshair, House, ArrowUp, ArrowDown, RotateCcw, Play, Pause, CircleCheck, TriangleAlert, Download } from '@lucide/svelte';
  import Dialog from './Dialog.svelte';
  import Section from './Section.svelte';
  import NumField from './NumField.svelte';
  import { doc } from '../lib/document.svelte';
  import { cutting } from '../lib/cutting.svelte';
  import { pathing } from '../lib/pathing.svelte';
  import { ui } from '../lib/ui.svelte';
  import { fmtLen, num } from '../lib/format';
  import { tip } from '../lib/tip';
  import { t } from '../lib/i18n.svelte';

  // Contours et amorces de tous les dessins, puis le plan.
  $effect(() => {
    cutting.ensure(doc.drawings);
  });
  $effect(() => {
    const list = doc.drawings;
    const timer = setTimeout(() => cutting.refreshOverlaps(list), 150);
    return () => clearTimeout(timer);
  });
  $effect(() => {
    void [doc.drawings, doc.job, cutting.byGeo, cutting.quality, cutting.lead.shape, cutting.lead.lead_in, cutting.lead.lead_out];
    const timer = setTimeout(() => pathing.refresh(), 80);
    return () => clearTimeout(timer);
  });

  const plan = $derived(pathing.plan);
  const start = $derived(doc.job.start);
  const qualities = $derived(plan ? Object.entries(plan.stats.cut).sort(([a], [b]) => Number(a) - Number(b)) : []);
  const picked = $derived(pathing.picked);
  let confirm = $state<string[] | null>(null);
  let busy = $state(false);

  async function exportOmx() {
    busy = true;
    await pathing.exportOmx();
    busy = false;
  }

  function ask() {
    const remarks = pathing.remarks();
    if (remarks.length) confirm = remarks;
    else exportOmx();
  }
</script>

<Section id="path" title={t('path.title')}>
  {#if !doc.drawings.length}
    <p class="muted">{t('path.empty')}</p>
  {:else}
    <p class="sub" use:tip={t('path.startHint')}>{t('path.start')}</p>
    <NumField label="X" value={start[0]} onchange={(v) => doc.setStart([v, start[1]])} />
    <NumField label="Y" value={start[1]} onchange={(v) => doc.setStart([start[0], v])} />
    <div class="row">
      <button class="btn" class:on={pathing.pickingStart} type="button" aria-pressed={pathing.pickingStart} use:tip={t('path.pickStartHint')} onclick={() => (pathing.pickingStart = !pathing.pickingStart)}
        ><Crosshair size={14} /> {t(pathing.pickingStart ? 'path.pickingStart' : 'path.pickStart')}</button
      >
      <button class="btn" type="button" disabled={start[0] === 0 && start[1] === 0} use:tip={t('path.originHint')} onclick={() => doc.setStart([0, 0])}><House size={14} /> {t('path.origin')}</button>
    </div>
    <p class="hint">{t('path.nozzle')}</p>

    {#if plan && !plan.order.length}
      <p class="muted">{t('path.nothing')}</p>
    {:else if plan}
      <p class="sub">{t('path.order')}</p>
      {#if pathing.orderLost}
        <p class="notice"><TriangleAlert size={14} /> {t('path.orderLost')}</p>
      {/if}
      <ol class="order">
        {#each plan.order as s, i (`${s.key}:${s.contour}`)}
          <li>
            <button type="button" class:on={picked === i} onclick={() => (pathing.picked = picked === i ? null : i)}>
              <span class="rank tabular">{i + 1}</span>
              <span class="swatch {s.side}" aria-hidden="true"></span>
              <span class="name">{pathing.label(s.key, s.contour)}</span>
            </button>
          </li>
        {/each}
      </ol>
      <div class="row">
        <button class="btn" type="button" disabled={picked == null || picked === 0} onclick={() => picked != null && pathing.shift(picked, -1)}><ArrowUp size={14} /> {t('path.earlier')}</button>
        <button class="btn" type="button" disabled={picked == null || picked >= plan.order.length - 1} onclick={() => picked != null && pathing.shift(picked, 1)}><ArrowDown size={14} /> {t('path.later')}</button>
        <button class="btn" type="button" disabled={doc.job.order == null} use:tip={t('path.autoOrderHint')} onclick={() => doc.setOrder(null)}><RotateCcw size={14} /> {t('path.autoOrder')}</button>
      </div>

      <p class="sub">{t('path.stats')}</p>
      <dl class="props">
        {#each qualities as [q, l] (q)}
          <dt>{t('path.cutQuality', { q })}</dt>
          <dd class="tabular">{fmtLen(l, ui.units, 1)}</dd>
        {/each}
        <dt>{t('path.leads')}</dt>
        <dd class="tabular">{fmtLen(plan.stats.leads, ui.units, 1)}</dd>
        <dt>{t('path.traverse')}</dt>
        <dd class="tabular">{fmtLen(plan.stats.traverse, ui.units, 1)}</dd>
        <dt>{t('path.pierces')}</dt>
        <dd class="tabular">{num(plan.stats.pierces)}</dd>
      </dl>

      <p class="sub">{t('path.warnings')}</p>
      {#if plan.warnings.length}
        <ul class="warnings">
          {#each plan.warnings as w, n (n)}
            <li>
              <button type="button" onclick={() => pathing.focus(w.x, w.y)}
                >{t(w.code === 'lead.too_short' ? 'path.tooShort' : w.step >= plan.order.length ? 'path.overCutBack' : 'path.overCut')} {n + 1}</button
              >
            </li>
          {/each}
        </ul>
      {:else}
        <p class="ok"><CircleCheck size={14} /> {t('path.noWarning')}</p>
      {/if}

      <p class="sub">{t('path.playback')}</p>
      <div class="play">
        <button class="btn icon" type="button" aria-label={t(pathing.playing ? 'path.pause' : 'path.play')} use:tip={t(pathing.playing ? 'path.pause' : 'path.play')} onclick={() => pathing.play()}>
          {#if pathing.playing}<Pause size={14} />{:else}<Play size={14} />{/if}
        </button>
        <input type="range" min="0" max="1000" step="1" aria-label={t('path.progress')} value={Math.round(pathing.progress * 1000)} oninput={(e) => pathing.seek(Number((e.currentTarget as HTMLInputElement).value) / 1000)} />
      </div>

      <button class="btn primary export" type="button" disabled={busy || !plan.order.length} use:tip={t('path.exportHint')} onclick={ask}><Download size={14} /> {t('path.export')}</button>
    {/if}
  {/if}
</Section>

{#if confirm}
  <Dialog title={t('path.confirmTitle')} width={460} onclose={() => (confirm = null)}>
    <p class="intro">{t('path.confirmIntro')}</p>
    <ul class="remarks">
      {#each confirm as r (r)}
        <li><TriangleAlert size={14} /> {r}</li>
      {/each}
    </ul>
    {#snippet footer({ close })}
      <button class="btn" type="button" onclick={close}>{t('common.cancel')}</button>
      <button
        class="btn primary"
        type="button"
        onclick={() => {
          close();
          exportOmx();
        }}>{t('path.exportAnyway')}</button
      >
    {/snippet}
  </Dialog>
{/if}

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

  .hint {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-xs);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s2);
  }

  .btn.on {
    border-color: var(--accent);
    color: var(--accent-text);
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

  .order {
    display: flex;
    flex-direction: column;
    gap: 1px;
    max-height: 240px;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .order button {
    display: grid;
    grid-template-columns: 22px 14px minmax(0, 1fr);
    align-items: center;
    gap: var(--s2);
    width: 100%;
    min-height: 24px;
    padding: 2px var(--s1);
    background: none;
    border: 0;
    border-radius: var(--radius);
    color: var(--text-secondary);
    font-size: var(--fs-sm);
    text-align: left;
  }

  .order button:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .order button.on {
    background: var(--bg-selected);
    color: var(--accent-text);
  }

  .rank {
    color: var(--text-muted);
    text-align: right;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .ok {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    color: var(--success);
    font-size: var(--fs-sm);
  }

  .warnings {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .warnings button {
    width: 100%;
    min-height: 24px;
    padding: 4px var(--s1);
    background: none;
    border: 0;
    border-radius: var(--radius);
    color: var(--warning);
    font-size: var(--fs-sm);
    text-align: left;
  }

  .warnings button:hover {
    background: var(--bg-hover);
  }

  .export {
    align-self: flex-start;
    margin-top: var(--s2);
  }

  .intro {
    margin: 0 0 var(--s2);
    color: var(--text-secondary);
  }

  .remarks {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .remarks li {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    color: var(--warning);
    font-size: var(--fs-sm);
  }

  .remarks :global(svg) {
    flex: none;
    margin-top: 2px;
  }

  .play {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: var(--s2);
  }

  .play input {
    width: 100%;
    accent-color: var(--accent);
  }
</style>
