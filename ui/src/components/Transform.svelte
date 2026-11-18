<script lang="ts">
  // Section « Transformer » du dessin sélectionné : point de référence (9
  // points de la boîte, coordonnées, ou choisi sur le plateau), position du
  // coin bas gauche, angle, échelle (cadenas) et miroir. Chaque valeur
  // validée est une étape d'annulation ; tout se calcule depuis la géométrie
  // d'origine, autour du point de référence.
  import { Crosshair, FlipHorizontal2, FlipVertical2, Lock, LockOpen, RotateCcw, RotateCw } from '@lucide/svelte';
  import Section from './Section.svelte';
  import NumField from './NumField.svelte';
  import { doc, refPoint, type Drawing } from '../lib/document.svelte';
  import { engine } from '../lib/engine.svelte';
  import { angleOf, factorOf, invert, apply, mirrorAbout, rotateAbout, scaleAbout, then, translate, type M } from '../lib/affine';
  import { tip } from '../lib/tip';
  import { t } from '../lib/i18n.svelte';

  let { drawing: d }: { drawing: Drawing } = $props();

  const ref = $derived(refPoint(d));
  const angle = $derived(angleOf(d.matrix));
  const bbox = $derived(d.placement.bbox);

  const set = (m: M) => doc.update(d.id, { matrix: then(m, d.matrix) });

  function moveRef(x: number, y: number) {
    const [ox, oy] = apply(invert(d.matrix), x, y);
    doc.update(d.id, { ref: { kind: 'point', x: ox, y: oy } });
  }

  function scaleTo(axis: 'x' | 'y', size: number) {
    const cur = axis === 'x' ? d.placement.width : d.placement.height;
    if (cur <= 0 || size <= 0) return;
    const k = size / cur;
    if (doc.lock) set(scaleAbout(ref, k, k));
    else set(axis === 'x' ? scaleAbout(ref, k, 1) : scaleAbout(ref, 1, k));
  }

  function center() {
    const bed = engine.machine;
    if (!bed) return;
    set(translate(bed.width / 2 - (bbox[0] + bbox[2]) / 2, bed.height / 2 - (bbox[1] + bbox[3]) / 2));
  }
</script>

<Section id="transform" title={t('transform.title')}>
  <p class="sub" use:tip={t('transform.refHint')}>{t('transform.ref')}</p>
  <div class="refrow">
    <div class="grid" role="radiogroup" aria-label={t('transform.ref')}>
      {#each [2, 1, 0] as iy (iy)}
        {#each [0, 1, 2] as ix (ix)}
          {@const on = d.ref.kind === 'box' && d.ref.ix === ix && d.ref.iy === iy}
          <button
            type="button"
            role="radio"
            aria-checked={on}
            class:on
            aria-label={t(`transform.anchors.${iy}${ix}` as 'transform.anchors.11')}
            use:tip={t(`transform.anchors.${iy}${ix}` as 'transform.anchors.11')}
            onclick={() => doc.update(d.id, { ref: { kind: 'box', ix, iy } })}
          ></button>
        {/each}
      {/each}
    </div>
    <button class="btn" class:active={doc.picking} type="button" aria-pressed={doc.picking} use:tip={t('transform.pickHint')} onclick={() => (doc.picking = !doc.picking)}>
      <Crosshair size={14} />
      {t('transform.pick')}
    </button>
  </div>
  <div class="pair">
    <NumField label="X" value={ref[0]} onchange={(v) => moveRef(v, ref[1])} />
    <NumField label="Y" value={ref[1]} onchange={(v) => moveRef(ref[0], v)} />
  </div>

  <p class="sub" use:tip={t('transform.positionHint')}>{t('transform.position')}</p>
  <div class="pair">
    <NumField label="X" value={bbox[0]} onchange={(v) => set(translate(v - bbox[0], 0))} />
    <NumField label="Y" value={bbox[1]} onchange={(v) => set(translate(0, v - bbox[1]))} />
  </div>
  <div class="buttons">
    <button class="btn" type="button" onclick={() => set(translate(-bbox[0], -bbox[1]))}>{t('transform.toOrigin')}</button>
    <button class="btn" type="button" onclick={center}>{t('transform.center')}</button>
  </div>

  <NumField label={t('transform.angle')} kind="angle" decimals={2} value={angle} onchange={(v) => set(rotateAbout(ref, v - angle))} hint={t('transform.angleHint')} />
  <div class="buttons">
    <button class="btn" type="button" use:tip={t('transform.ccw')} onclick={() => set(rotateAbout(ref, 90))}><RotateCcw size={14} /> +90°</button>
    <button class="btn" type="button" use:tip={t('transform.cw')} onclick={() => set(rotateAbout(ref, -90))}><RotateCw size={14} /> −90°</button>
  </div>

  <div class="scale">
    <div class="sizes">
      <NumField label={t('transform.width')} value={d.placement.width} min={0.001} onchange={(v) => scaleTo('x', v)} />
      <NumField label={t('transform.height')} value={d.placement.height} min={0.001} onchange={(v) => scaleTo('y', v)} />
    </div>
    <button
      class="btn quiet icon lock"
      type="button"
      aria-pressed={doc.lock}
      aria-label={t(doc.lock ? 'transform.locked' : 'transform.unlocked')}
      use:tip={t(doc.lock ? 'transform.locked' : 'transform.unlocked')}
      onclick={() => (doc.lock = !doc.lock)}
    >
      {#if doc.lock}<Lock size={14} />{:else}<LockOpen size={14} />{/if}
    </button>
  </div>
  <NumField label={t('transform.scale')} kind="percent" value={factorOf(d.matrix)} min={0.00001} onchange={(v) => set(scaleAbout(ref, v / factorOf(d.matrix), v / factorOf(d.matrix)))} hint={t('transform.scaleHint')} />

  <div class="buttons">
    <button class="btn" type="button" onclick={() => set(mirrorAbout(ref, true))}><FlipHorizontal2 size={14} /> {t('transform.mirrorH')}</button>
    <button class="btn" type="button" onclick={() => set(mirrorAbout(ref, false))}><FlipVertical2 size={14} /> {t('transform.mirrorV')}</button>
  </div>
</Section>

<style>
  .sub {
    margin: var(--s1) 0 0;
    color: var(--text-secondary);
    font-size: var(--fs-sm);
  }

  .refrow {
    display: flex;
    align-items: center;
    gap: var(--s3);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(3, 14px);
    gap: 4px;
    padding: 4px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--segment-track);
  }

  .grid button {
    width: 14px;
    height: 14px;
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--bg-control);
  }

  .grid button:hover {
    border-color: var(--accent);
  }

  .grid button.on {
    background: var(--accent);
    border-color: var(--accent);
  }

  .btn.active {
    border-color: var(--accent);
    color: var(--accent-text);
  }

  .pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--s2);
  }

  div.pair :global(.field) {
    grid-template-columns: 14px minmax(0, 1fr) 24px;
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s2);
  }

  .scale {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--s1);
  }

  .sizes {
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
</style>
