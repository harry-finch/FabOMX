# Consignes pour les agents

> Ce fichier est repris dans CLAUDE.md.

FabOMX : Parcours d’outils pour la ProtoMAX.

## Skills à utiliser

Copiées dans `.agents/skills/` (liens dans `.claude/skills/`) depuis la bibliothèque de skills.

- `agent-learnings-manager` : Use this skill to read, apply, and update the ../fabtrack-dev/SKILL.md file. Trigger when starting a new session, resolving a complex bug, or when the user asks to "remember this" or "update…
- `cam-geometry` : Implement or review 2D/2.5D CAM geometry, cutter compensation, pockets, contours, tabs, leads, ramps, or related toolpath algorithms.
- `cam-ux-ui` : Design, implement, or review UX/UI for CAM workflows, geometry selection, operation setup, simulation, warnings, or machine/job configuration.
- `cam-validation` : Design or run CAM correctness tests, regression fixtures, golden tests, invariants, simulation checks, or physical validation protocols.
- `chef-orchestre` : Orchestrer des projets de développement logiciel dans Codex en qualifiant la demande, décomposant les tâches, déléguant sélectivement à des sous-agents selon la complexité et le coût, contrôlant…
- `fab-cleanup` : Nettoyer un projet — retirer ce qui ne sert plus (dossiers reconstructibles, résidus non suivis, fichiers suivis devenus inutiles) sans rien perdre d'irrécupérable. Utiliser quand l'utilisateur lance…
- `fab-design-system` : Thème, style graphique et philosophie UX communs aux applications Fab (FabSpeed, FabVents et les suivantes) pour makers et fablabs. Utiliser pour créer une nouvelle application de la famille, ajouter…
- `fab-roadmap` : Feuille de route d'un projet au format de FabBoard (dossier roadmap/) — spécifier une idée avec l'utilisateur et la transformer en spécification détaillée, tenir les statuts à jour pendant le…
- `protomax-waterjet` : Analyze or implement future OMAX ProtoMAX/waterjet CAM semantics, OMX output, piercing, kerf, quality, leads, or waterjet integration.

## Feuille de route

La feuille de route est dans `roadmap/`, versionnée avec le projet. `ROADMAP.md` en est la vue générée : ne pas le modifier à la main.

- `roadmap/items/` : un fichier par élément (`R-001-titre.md`). Statuts : `idee`, `specifiee`, `en-cours`, `a-tester`, `livree`, `abandonnee`. À partir de `specifiee`, le corps suit la spécification (But, Inclus, Hors périmètre, Dépendances, Critères d'acceptation, Vérifications de l'utilisateur).
- `roadmap/ordre.txt` : l'ordre de priorité des éléments restants.
- `roadmap/actions/` : ce que l'utilisateur doit faire (valider, tester, fournir, décider).

Quand tu t'arrêtes pour attendre l'utilisateur, crée une action, puis à la reprise lis les réponses des actions faites :

```
fabboard action add --type valider "Valider la spécification" --element R-001
fabboard roadmap lint
```

`fabboard` est la ligne de commande de FabBoard (`fabboard roadmap`, `fabboard action`, `fabboard doctor`). Le format complet est décrit dans le dépôt de FabBoard (`docs/FORMAT.md`).

## Commandes

À compléter : installation, tests, lancement.
