# Consignes pour les agents

> Ce fichier est repris dans CLAUDE.md.

FabOMX : parcours d’outils pour la découpe jet d’eau sur la ProtoMAX (OMAX), dans la famille des applications Fab. À partir d’un dessin SVG ou DXF, produire un fichier OMX. Workspace Rust (`fabomx-core` pur, `fabomx-wasm` pour l’interface) + interface Svelte 5 dans `ui/`, même pile que FabVents et FabWeave.
Lire [README.md](README.md), [STATUS.md](STATUS.md) et la feuille de route ([ROADMAP.md](ROADMAP.md), générée depuis `roadmap/`) avant de modifier.

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

- `fabomx-core` reste pur : aucune IO, aucun réseau, aucune dépendance d’interface ; déterministe.
- Millimètres et degrés dans le moteur et dans les fichiers ; les pouces ne sont qu’un affichage.
- Repère du moteur : origine en bas à gauche du plateau, Y vers le haut. L’espace de travail retourne l’axe.
- Les valeurs ne sont jamais corrigées en silence : une valeur hors domaine produit une erreur ou un avertissement à code stable.
- Disposition propre à FabOMX, écart voulu à `fab-design-system` : palette d’outils étroite (44 px) à gauche, paramètres et formulaires à droite.
- Un outil de la palette qui n’est pas encore développé reste grisé (« À venir »), il ne réagit ni au clic ni à son raccourci.
- Interface : aucune couleur en dur dans les composants ni dans le dessin de l’espace de travail, tout passe par les jetons de `ui/src/app.css` ; textes par `t('clé')`, `fr.ts` est la référence et `en.ts` a les mêmes clés.
- Mettre à jour la documentation dans le même changement que le comportement.

## Commandes

- Installation : `cd ui && npm install` (Rust 1.88 par `rust-toolchain.toml`, `wasm-pack`, Node.js).
- Tests : `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check`, `cd ui && npm run check`.
- Moteur pour l’interface : `cd ui && npm run wasm` (wasm-pack, sortie dans `ui/src/wasm/`, hors Git).
- Développement : `cd ui && npm run dev` → http://localhost:5177 (relancer `npm run wasm` après une modification du moteur).
- Build : `cd ui && npm run build` (moteur compris), puis `npm run preview`.
