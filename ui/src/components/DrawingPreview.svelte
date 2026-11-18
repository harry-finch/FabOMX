<script lang="ts">
  // Aperçu d'un dessin dans le dialogue d'import : les tracés à l'échelle,
  // les contours ouverts en attention, et un repère sur chaque défaut à
  // nettoyer. Y vers le haut, comme le plateau.
  import type { Report } from '../lib/types';

  let { report, height = 200 }: { report: Report; height?: number } = $props();

  let width = $state(0);
  const PAD = 12;

  const view = $derived.by(() => {
    const [x0, y0, x1, y1] = report.bbox;
    const w = Math.max(x1 - x0, 1e-9);
    const h = Math.max(y1 - y0, 1e-9);
    const s = Math.min((width - 2 * PAD) / w, (height - 2 * PAD) / h);
    const ox = (width - w * s) / 2 - x0 * s;
    const oy = (height + h * s) / 2 + y0 * s;
    return { s, ox, oy };
  });

  function path(pts: number[]): string {
    const { s, ox, oy } = view;
    let d = '';
    for (let i = 0; i < pts.length; i += 2) d += `${i ? 'L' : 'M'}${(ox + pts[i] * s).toFixed(1)} ${(oy - pts[i + 1] * s).toFixed(1)}`;
    return d;
  }
</script>

<div class="preview" bind:clientWidth={width} style:height="{height}px">
  {#if width > 0}
    <svg {width} {height} aria-hidden="true">
      {#each report.traces as tr, i (i)}
        <path class:open={tr.open} d={path(tr.pts)} />
      {/each}
      {#each report.markers as m, i (i)}
        <circle class="marker" cx={view.ox + m.x * view.s} cy={view.oy - m.y * view.s} r="5" />
      {/each}
    </svg>
  {/if}
</div>

<style>
  .preview {
    background: var(--canvas-bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  svg {
    display: block;
  }

  path {
    fill: none;
    stroke: var(--text-primary);
    stroke-width: 1;
    stroke-linejoin: round;
    stroke-linecap: round;
  }

  path.open {
    stroke: var(--drawing-open);
    stroke-width: 1.5;
  }

  .marker {
    fill: none;
    stroke: var(--drawing-open);
    stroke-width: 1.5;
  }
</style>
