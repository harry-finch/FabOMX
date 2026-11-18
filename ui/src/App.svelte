<script lang="ts">
  // Fenêtre de FabOMX : barre d'application, palette d'outils (gauche),
  // espace de travail avec le plateau (centre), paramètres (droite), barre
  // d'état. Raccourcis : touches des outils, F (recadrer), ? (aide).
  import AppBar from './components/AppBar.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import Dialog from './components/Dialog.svelte';
  import ToolPalette from './components/ToolPalette.svelte';
  import Workspace from './components/Workspace.svelte';
  import Inspector from './components/Inspector.svelte';
  import { engine } from './lib/engine.svelte';
  import { tools } from './lib/tools.svelte';
  import { ui } from './lib/ui.svelte';
  import { fmtLength, lengthUnit, pct } from './lib/format';
  import { t } from './lib/i18n.svelte';

  let workspace: Workspace;
  let pointer = $state<[number, number] | null>(null);
  let zoom = $state(1);

  function keydown(e: KeyboardEvent) {
    const el = e.target as HTMLElement | null;
    if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable)) return;
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.key === '?') ui.help = true;
    else if (e.key === 'f' || e.key === 'F') workspace.fit();
    else {
      const tool = tools.byKey(e.key);
      if (tool) tools.select(tool.id);
    }
  }
</script>

<svelte:window onkeydown={keydown} />

<div class="app">
  <AppBar />
  <div class="main">
    <aside class="panel left">
      <ToolPalette />
    </aside>
    <Workspace bind:this={workspace} machine={engine.machine} bind:pointer bind:zoom />
    <aside class="panel right">
      <Inspector machine={engine.machine} />
    </aside>
  </div>
  <StatusBar state={engine.state} version={engine.version}>
    {#snippet right()}
      {#if pointer}
        <span class="tabular">X {fmtLength(pointer[0], ui.units, 1)} · Y {fmtLength(pointer[1], ui.units, 1)} {lengthUnit(ui.units)}</span>
        <span class="sep"></span>
      {/if}
      <span class="tabular">{t('workspace.zoom', { v: pct(zoom * 100) })}</span>
    {/snippet}
  </StatusBar>
</div>

{#if ui.help}
  <Dialog title={t('help.title')} onclose={() => (ui.help = false)}>
    <h3>{t('help.shortcuts')}</h3>
    <dl class="keys">
      <dt><kbd>V</kbd></dt>
      <dd>{t('help.keys.select')}</dd>
      <dt><kbd>H</kbd></dt>
      <dd>{t('help.keys.hand')}</dd>
      <dt><kbd>{t('help.space')}</kbd></dt>
      <dd>{t('help.keys.pan')}</dd>
      <dt><kbd>{t('help.wheel')}</kbd></dt>
      <dd>{t('help.keys.wheel')}</dd>
      <dt><kbd>F</kbd></dt>
      <dd>{t('help.keys.fit')}</dd>
      <dt><kbd>?</kbd></dt>
      <dd>{t('help.keys.help')}</dd>
    </dl>
  </Dialog>
{/if}

<div class="toasts" aria-live="polite">
  {#each ui.toasts as toast (toast.id)}
    <div class="toast">{toast.text}</div>
  {/each}
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100vh;
    min-width: 0;
  }

  /* Disposition propre à FabOMX (écart voulu au kit, qui met les réglages à
     gauche) : palette d'outils étroite à gauche, paramètres à droite. */
  .main {
    display: grid;
    grid-template-columns: 44px minmax(0, 1fr) 296px;
    min-height: 0;
  }

  @media (max-width: 1100px) {
    .main {
      grid-template-columns: 44px minmax(0, 1fr) 264px;
    }
  }

  .panel {
    background: var(--bg-panel);
    overflow-y: auto;
    min-height: 0;
  }

  .left {
    border-right: 1px solid var(--border);
  }

  .right {
    border-left: 1px solid var(--border);
  }

  .keys {
    display: grid;
    grid-template-columns: 120px 1fr;
    gap: 6px var(--s3);
    margin: 0;
  }

  .keys dd {
    margin: 0;
  }

  kbd {
    display: inline-block;
    padding: 1px 6px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    background: var(--bg-control);
    font: 500 var(--fs-sm) var(--font);
    color: var(--text-primary);
  }

  .toasts {
    position: fixed;
    bottom: 36px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 6px;
    z-index: 900;
    pointer-events: none;
  }

  .toast {
    padding: 8px 14px;
    border-radius: var(--radius-lg);
    background: var(--tooltip-bg);
    color: var(--tooltip-text);
    font-size: var(--fs-sm);
    box-shadow: var(--shadow-pop);
    animation: rise 160ms ease-out;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
