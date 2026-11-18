<script lang="ts" generics="T extends string">
  // Groupe de boutons exclusifs (radio), compact. Navigation au clavier par flèches.
  // `columns` : boutons en grille, pour des libellés trop longs pour une ligne.
  import { tip } from '../lib/tip';

  let {
    options,
    value,
    onchange,
    label,
    columns,
  }: {
    options: { id: T; label: string; hint?: string }[];
    value: T;
    onchange: (v: T) => void;
    label: string;
    columns?: number;
  } = $props();

  function key(e: KeyboardEvent, i: number) {
    const d = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    const next = options[(i + d + options.length) % options.length];
    onchange(next.id);
    const group = (e.currentTarget as HTMLElement).parentElement;
    (group?.children[(i + d + options.length) % options.length] as HTMLElement | undefined)?.focus();
  }
</script>

<div class="seg" class:grid={columns} style:grid-template-columns={columns ? `repeat(${columns}, minmax(0, 1fr))` : undefined} role="radiogroup" aria-label={label}>
  {#each options as o, i (o.id)}
    <button
      type="button"
      role="radio"
      aria-checked={o.id === value}
      tabindex={o.id === value ? 0 : -1}
      class:on={o.id === value}
      use:tip={o.hint}
      onclick={() => onchange(o.id)}
      onkeydown={(e) => key(e, i)}>{o.label}</button
    >
  {/each}
</div>

<style>
  .seg {
    display: flex;
    width: 100%;
    height: var(--control-h);
    padding: 2px;
    gap: 2px;
    background: var(--segment-track);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .seg.grid {
    display: grid;
    height: auto;
  }

  .seg.grid button {
    min-height: calc(var(--control-h) - 6px);
    padding: 2px var(--s1);
  }

  button {
    flex: 1;
    border: 0;
    border-radius: 3px;
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--fs-sm);
    font-weight: 500;
  }

  button:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  button.on {
    background: var(--bg-elevated);
    color: var(--text-primary);
    box-shadow: inset 0 0 0 1px var(--border-strong);
  }
</style>
