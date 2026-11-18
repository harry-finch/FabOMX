<script lang="ts">
  // Palette d'outils (panneau gauche, 44 px) : un bouton d'icône par outil,
  // groupes séparés par un trait. Infobulle sur deux lignes : « Nom (touche) »
  // puis l'usage, ou « À venir » pour un outil pas encore développé.
  import { TOOL_GROUPS, tools, type ToolId } from '../lib/tools.svelte';
  import { tip } from '../lib/tip';
  import { t } from '../lib/i18n.svelte';

  let { onaction }: { onaction: (id: ToolId) => void } = $props();
</script>

<div class="palette" role="toolbar" aria-label={t('tools.label')} aria-orientation="vertical">
  {#each TOOL_GROUPS as group, g (g)}
    {#if g > 0}<span class="sep" aria-hidden="true"></span>{/if}
    {#each group as tool (tool.id)}
      {@const Icon = tool.icon}
      <button
        type="button"
        role={tool.action ? undefined : 'radio'}
        class="tool"
        class:on={tools.active === tool.id}
        aria-checked={tool.action ? undefined : tools.active === tool.id}
        aria-label={t(`tools.${tool.id}.name`)}
        aria-disabled={!tool.enabled}
        use:tip={`${t(`tools.${tool.id}.name`)} (${tool.key.toUpperCase()})\n${tool.enabled ? t(`tools.${tool.id}.hint`) : t('tools.soon')}`}
        onclick={() => (tool.action ? tool.enabled && onaction(tool.id) : tools.select(tool.id))}
      >
        <Icon size={16} />
      </button>
    {/each}
  {/each}
</div>

<style>
  .palette {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: var(--s2) 0;
  }

  .tool {
    display: grid;
    place-items: center;
    width: 30px;
    height: 28px;
    padding: 0;
    background: none;
    border: 0;
    border-radius: var(--radius);
    color: var(--text-secondary);
  }

  .tool:hover:not([aria-disabled='true']) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tool.on {
    background: var(--bg-selected);
    color: var(--accent-text);
  }

  .tool[aria-disabled='true'] {
    opacity: 0.35;
    cursor: default;
  }

  .sep {
    width: 18px;
    height: 1px;
    margin: var(--s1) 0;
    background: var(--border-strong);
  }
</style>
