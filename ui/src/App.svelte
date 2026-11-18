<script lang="ts">
  // Fenêtre de FabOMX : barre d'application, palette d'outils (gauche),
  // espace de travail avec le plateau (centre), paramètres (droite), barre
  // d'état. Raccourcis : touches des outils, I ou ⌘O (importer), F
  // (recadrer), Suppr (retirer), Échap (désélectionner), ⌘Z / ⇧⌘Z, ? (aide).
  // Un fichier déposé n'importe où dans la fenêtre ouvre le dialogue d'import.
  import AppBar from './components/AppBar.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import Dialog from './components/Dialog.svelte';
  import ToolPalette from './components/ToolPalette.svelte';
  import Workspace from './components/Workspace.svelte';
  import Inspector from './components/Inspector.svelte';
  import ImportDialog from './components/ImportDialog.svelte';
  import { engine } from './lib/engine.svelte';
  import { doc } from './lib/document.svelte';
  import { importer } from './lib/importer.svelte';
  import { tools, type ToolId } from './lib/tools.svelte';
  import { ui } from './lib/ui.svelte';
  import { fmtLength, lengthUnit, pct } from './lib/format';
  import { t } from './lib/i18n.svelte';

  let workspace: Workspace;
  let pointer = $state<[number, number] | null>(null);
  let zoom = $state(1);

  let dropping = $state(false);

  function action(id: ToolId) {
    if (id === 'import') importer.pick();
  }

  function remove(id: number) {
    const d = doc.drawings.find((x) => x.id === id);
    doc.remove(id);
    if (d) ui.toast(t('drawing.removed', { name: d.name }));
  }

  function keydown(e: KeyboardEvent) {
    const el = e.target as HTMLElement | null;
    if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable)) return;
    if (importer.pending || ui.help) return;
    const mod = e.metaKey || e.ctrlKey;
    if (mod && !e.altKey) {
      const k = e.key.toLowerCase();
      if (k === 'o') importer.pick();
      else if (k === 'z' && e.shiftKey) doc.redo();
      else if (k === 'z') doc.undo();
      else if (k === 'y') doc.redo();
      else return;
      e.preventDefault();
      return;
    }
    if (mod || e.altKey) return;
    if (e.key === '?') ui.help = true;
    else if (e.key === 'f' || e.key === 'F') workspace.fit();
    else if ((e.key === 'Delete' || e.key === 'Backspace') && doc.selected != null) {
      e.preventDefault();
      remove(doc.selected);
    } else if (e.key === 'Escape') doc.select(null);
    else {
      const tool = tools.byKey(e.key);
      if (tool?.action) {
        if (tool.enabled) action(tool.id);
      } else if (tool) tools.select(tool.id);
    }
  }

  const hasFiles = (e: DragEvent) => !!e.dataTransfer?.types.includes('Files');

  function dragover(e: DragEvent) {
    if (!hasFiles(e) || importer.pending) return;
    e.preventDefault();
    dropping = true;
  }

  function drop(e: DragEvent) {
    if (!hasFiles(e)) return;
    e.preventDefault();
    dropping = false;
    const file = e.dataTransfer?.files[0];
    if (file && !importer.pending) importer.open(file);
  }
</script>

<svelte:window
  onkeydown={keydown}
  ondragover={dragover}
  ondragleave={(e) => {
    if (!e.relatedTarget) dropping = false;
  }}
  ondrop={drop}
/>

<div class="app">
  <AppBar />
  <div class="main">
    <aside class="panel left">
      <ToolPalette onaction={action} />
    </aside>
    <Workspace
      bind:this={workspace}
      machine={engine.machine}
      drawings={doc.drawings}
      selected={doc.selected}
      onselect={(id) => doc.select(id)}
      bind:pointer
      bind:zoom
    />
    <aside class="panel right">
      <Inspector machine={engine.machine} drawing={doc.selection} count={doc.drawings.length} onremove={remove} />
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
      <dt><kbd>I / ⌘O</kbd></dt>
      <dd>{t('help.keys.import')}</dd>
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
      <dt><kbd>{t('help.del')}</kbd></dt>
      <dd>{t('help.keys.remove')}</dd>
      <dt><kbd>{t('help.esc')}</kbd></dt>
      <dd>{t('help.keys.deselect')}</dd>
      <dt><kbd>⌘Z / ⇧⌘Z</kbd></dt>
      <dd>{t('help.keys.undo')}</dd>
      <dt><kbd>?</kbd></dt>
      <dd>{t('help.keys.help')}</dd>
    </dl>
  </Dialog>
{/if}

{#if importer.pending}
  <ImportDialog />
{/if}

{#if dropping}
  <div class="drop" aria-hidden="true">{t('import.drop')}</div>
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

  /* Voile de dépôt : plein écran, bord pointillé accent. */
  .drop {
    position: fixed;
    inset: 0;
    z-index: 800;
    display: grid;
    place-content: center;
    background: var(--backdrop);
    border: 2px dashed var(--accent);
    color: var(--text-primary);
    font-size: var(--fs-lg);
    pointer-events: none;
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
