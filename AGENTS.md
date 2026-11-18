# Consignes pour les agents

> Ce fichier est repris dans CLAUDE.md.

FabOMX : parcours d’outils pour la découpe jet d’eau sur la ProtoMAX (OMAX), dans la famille des applications Fab. Pas encore de code ni de pile technique choisie.
Lire [README.md](README.md) et [STATUS.md](STATUS.md) avant de modifier.

## Skills à utiliser (`.agents/skills/`)

Liens dans `.claude/skills/`, installées depuis la bibliothèque de skills par FabBoard (`hairline-*` par `npx skills add`, voir `skills-lock.json`).

- `fab-roadmap` : spécifier une idée, tenir les statuts, créer une action quand on attend l’utilisateur.
- `project-documentation` : écrire ou mettre à jour README, AGENTS.md, STATUS.md.
- `protomax-waterjet` : tout ce qui touche au jet d’eau, à OMAX, à la ProtoMAX ou au format OMX.
- `cam-geometry` : géométrie 2D (import, nettoyage, contours, compensation, amorces).
- `cam-validation` : tests de correction, cas de référence, invariants de la géométrie et des parcours.
- `cam-ux-ui` : interface de sélection de géométrie, de réglage des opérations, d’aperçu.
- `fab-design-system` : toute modification de l’interface (thème, composants, textes).
- `fab-illustration-system`, `hairline-create`, `hairline-adopt` : figures au trait (carte FabBoard, état vide, À propos).
- `fab-cleanup` : faire le ménage dans le projet.
- `chef-orchestre` : orchestrer un travail multi-fichiers avec des sous-agents.
- `agent-learnings-manager` : relire et mettre à jour les leçons d’agent (`../fabtrack-dev/SKILL.md`).

## Feuille de route

Elle est dans `roadmap/` (un fichier par élément, un par action de l’utilisateur), lue par FabBoard ; `ROADMAP.md` est généré, ne jamais le modifier à la main. Skill `fab-roadmap` : spécifier, tenir les statuts, créer une action quand on attend l’utilisateur, `fabboard roadmap lint` avant chaque commit.

## Règles

- Mettre à jour la documentation dans le même changement que le comportement.

## Commandes

Aucune pour l’instant : à compléter avec le code (installation, tests, lancement).
