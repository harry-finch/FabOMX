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
- Importer est une action, pas un outil : il ouvre le choix du fichier sans changer l’outil actif ; un dessin posé ramène à l’outil Sélection. Le dialogue d’import signale ce qu’il faudra nettoyer ; il ne répare que si « Nettoyer à l’import » est coché.
- Import : lignes, arcs et cercles restent exacts ; les autres courbes (Bézier, splines, ellipses, cercles déformés) deviennent des arcs tangents à 0,01 mm près (`CURVE_TOL`, `arcs::biarcs`), en segments seulement sous `arcs::MIN_RADIUS`. L’opération « Reconnaître des arcs » du nettoyage (`arcs::fit`) traite les polylignes : coins de plus de 30°, lignes droites et petits détails intacts. Seuils de détection dans `fabomx-core/src/import.rs` (`JOINED`, `GAP`, `TINY`).
- Un dessin posé garde sa géométrie d’origine (dans le moteur) et une seule matrice affine : toute transformation se compose à cette matrice et se recalcule depuis l’origine (`fabomx-core/src/place.rs`), jamais sur un résultat déjà transformé. `ui/src/lib/affine.ts` reprend les formules d’`Affine` du moteur : les garder identiques.
- Nettoyage (`fabomx-core/src/clean.rs`) : rien n’est inventé, seules les jonctions sous le seuil ferment un contour ; seuils en mm du plateau, ramenés à l’origine par l’échelle du dessin. Un nettoyage range une nouvelle géométrie sous un nouvel identifiant (`geo`) dans le moteur, l’ancienne reste pour annuler.
- Côtés de coupe (`fabomx-core/src/cut.rs`) : Extérieur, Trou, Sur la ligne, Ne pas couper. Détection par imbrication, dessin par dessin (le côté alterne à chaque niveau) ; tracé ouvert ou contour qui se croise : sur la ligne ; repères d’après le fichier (pointillés, calque Defpoints ou dont le nom contient « construction » ou « repère ») : ne pas couper, jamais chaînés avec les traits coupés. La pièce reste à droite du jet, comme sur la ProtoMAX (exemples LAYOUT) : `cut::reversed` et `reversed` de `ui/src/lib/cutting.svelte.ts` suivent les mêmes règles. Les choix faits à la main vivent dans le dessin (`Drawing.sides`, par indice de contour) et sont perdus au nettoyage, qui change les contours.
- Qualité de coupe : 1 à 5 (`cut::QUALITIES`), celle du contour (`Drawing.qualities`) sinon la qualité par défaut, réglage du poste (`fabomx.cut.v1`) qui n’est pas une étape d’annulation ; un contour Ne pas couper n’en a pas (`cut::quality`, repris par `qualityOf` de `ui/src/lib/cutting.svelte.ts`). Pas de gravure tant que son code OMX n’est pas connu (R-014).
- Amorces (`fabomx-core/src/lead.rs`) : longueurs en mm du plateau (`LEAD_IN` 3,8, `LEAD_OUT` 1,3, comme LAYOUT), calculées dans le repère d’origine ; chute dehors pour un Extérieur, dedans pour un Trou ; une amorce qui touche un autre contour coupé du dessin (coins compris) est raccourcie, signalée sous `MIN_LEAD`. Réglages par défaut dans `fabomx.cut.v1`, propres au contour dans `Drawing.leads` et `Drawing.entries`.
- Parcours (`fabomx-core/src/path.rs`) : sur le plateau, tous dessins confondus ; un contour toujours après ceux qu’il contient (même dessin), puis le perçage le plus proche ; un ordre à la main qui enfreint cette règle est refusé (`path.order`). Point de départ et ordre sont dans le document (`doc.job`), annulables comme les dessins. C’est la source de l’export OMX.
- Onglets (`lead.rs`, `TabSpec`) : sur les extérieurs seulement ; l’onglet de fin arrête la coupe avant le point d’entrée (sans perçage), un onglet le long du contour arrête le jet (sortie dans la chute) et repercé après lui (entrée) ; positions en longueur le long du trajet. Largeur en mm du plateau (1 mm par défaut) sur le trajet du jet : le pont réel est plus étroit de la saignée.
- Tracé ouvert (`LeadSpec.part`, `extend`) : sur la ligne par défaut ; pièce d’un côté (gauche ou droite du sens dessiné, repère d’origine) : la pièce est à droite du jet, ce qui impose le sens et le bout de perçage ; amorces dans le prolongement au choix. Réglages dans `Drawing.open`.
- OMX (`fabomx-core/src/omx.rs`) : format exact de LAYOUT (pouces, 10 décimales, CRLF), vérifié par la réécriture octet pour octet des exemples de `omx/`. Q 0 = déplacement, 9 = amorce, 1 à 5 = qualité ; côté 1 quand MAKE décale la saignée (extérieur, trou, tracé ouvert d’un côté : `Lead.offset`), 0 pour une coupe sur le trait (confirmé à la coupe, A-023) ; arcs exacts en `Bow` (arrondi du DXF), découpés en demi-tours au plus. Le premier point est le point de départ : sur la machine, c’est là que se trouve la buse au lancement.
- Application de bureau : ⌘O, ⌘Z, ⇧⌘Z et ⌘A y sont des raccourcis du menu natif ; l’interface les ignore au clavier pour ne pas agir deux fois. F et ? restent des touches de l’interface (sans raccourci de menu), libres dans les champs.
- Interface : aucune couleur en dur dans les composants ni dans le dessin de l’espace de travail, tout passe par les jetons de `ui/src/app.css` ; textes par `t('clé')`, `fr.ts` est la référence et `en.ts` a les mêmes clés.
- Mettre à jour la documentation dans le même changement que le comportement.

## Commandes

- Installation : `cd ui && npm install` (Rust 1.88 par `rust-toolchain.toml`, `wasm-pack`, Node.js).
- Tests : `cargo test --workspace` (fichiers d’exemple SVG et DXF dans `crates/fabomx-core/tests/fixtures/`), `cargo clippy --workspace --all-targets`, `cargo fmt --all --check`, `cd ui && npm run check`.
- Moteur pour l’interface : `cd ui && npm run wasm` (wasm-pack, sortie dans `ui/src/wasm/`, hors Git).
- Développement : `cd ui && npm run dev` → http://localhost:5177 (relancer `npm run wasm` après une modification du moteur).
- Build : `cd ui && npm run build` (moteur compris), puis `npm run preview`.
- Application macOS : `cd ui && npm run desktop` (Tauri, `dist-desktop/FabOMX.app`, à lancer sur place ; pas de `.dmg` pour les essais). Le crate `fabomx-desktop` est hors du workspace : les tests ci-dessus ne le compilent pas ; le vérifier avec `cd crates/fabomx-desktop && cargo clippy --release`.
