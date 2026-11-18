---
id: R-003
titre: "Poser l'interface"
resume: "Squelette de l'application suivant fab-design-system : palette d'outils en icônes dans le panneau de gauche, paramètres et formulaires dans le panneau de droite, dessin au centre."
statut: specifiee
jalon: v0.1
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

Avoir le squelette de FabOMX dans lequel les éléments suivants (import, transformations, nettoyage, puis la découpe) viendront se ranger, sans revenir sur la disposition. L'utilisateur ouvre l'application, reconnaît une application Fab, voit le plateau de la ProtoMAX et trouve la palette d'outils de v0.1 à gauche et les paramètres à droite.

## Inclus

- Même pile que FabVents et FabWeave : moteur Rust pur (`fabomx-core`) et sa version WebAssembly (`fabomx-wasm`) dans un workspace Cargo, interface Svelte 5 dans `ui/` à partir du kit de `fab-design-system`, servie en développement sur http://localhost:5177.
- Barre d'application et barre d'état de la famille : liseré à la couleur d'identité (#2D22BF), logo provisoire du kit, nom et sous-titre, langue FR/EN, thème sombre/clair, unités mm/pouces ; barre d'état avec l'état et la version du moteur.
- **Disposition propre à FabOMX** (écart voulu à `fab-design-system`, qui met les réglages à gauche) : à gauche une barre étroite d'icônes (environ 44 px), au centre l'espace de travail, à droite le panneau des paramètres et des formulaires.
- Palette d'outils : tous les outils de v0.1, une icône chacun, avec infobulle et raccourci : Sélection et Main (déplacer la vue) actifs ; Importer, Déplacer, Tourner, Mettre à l'échelle et Nettoyer présents mais grisés, l'infobulle disant « à venir ».
- Espace de travail : le plateau de la ProtoMAX à l'échelle (305 × 305 mm, soit 12 × 12 pouces), une grille en millimètres et l'origine ; zoom à la molette autour du pointeur, déplacement de la vue avec l'outil Main ou espace + glisser, `F` recadre sur le plateau ; coordonnées du pointeur en mm dans la barre d'état.
- Panneau droit : sans sélection, une section « Plateau » (dimensions, en lecture seule) et une phrase qui dit quoi faire (« Importez un dessin pour commencer »).
- Préférences mémorisées d'une session à l'autre : thème, langue, unités, outil actif.
- Textes en français et en anglais, par clés `t('…')`.
- `README.md`, `AGENTS.md` (commandes, règles reprises de FabVents qui s'appliquent : moteur pur, mm et degrés, aucune couleur en dur, textes par clés) et `STATUS.md` mis à jour ; `[[test]]` « Version web » déclaré dans `roadmap/projet.toml`.

## Hors périmètre

- Le fonctionnement des outils grisés : importer (R-001), déplacer, tourner, mettre à l'échelle (R-002), nettoyer (R-004).
- L'application de bureau Tauri : élément à part, en fin de v0.1.
- Le logo définitif (le logo provisoire du kit, à la couleur d'identité, en tient lieu).
- La figure au trait dans l'application (état vide, À propos).
- Tout ce qui touche à la découpe : parcours, trait de coupe, amorces, export OMX (jalon v0.2).
- Annuler / rétablir : rien n'est encore modifiable ; arrive avec le premier outil qui modifie le dessin.
- Ouverture et enregistrement d'un document de projet.

## Dépendances

Aucune.

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets` et `cd ui && npm run check` passent sans erreur ni avertissement.
- `cd ui && npm run dev` ouvre l'application sur http://localhost:5177 ; la barre d'état affiche le moteur prêt et sa version, lue depuis `fabomx-wasm`.
- À 1440 et à 1100 px de large, en thème sombre et clair : barre d'icônes à gauche, plateau au centre, panneau des paramètres à droite, aucun défilement horizontal.
- Les sept outils sont dans la palette ; Sélection et Main s'activent au clic et par leur raccourci, l'outil actif est marqué ; les cinq autres sont grisés, ne réagissent pas, et leur infobulle dit « à venir ».
- Le plateau occupe l'espace de travail à l'ouverture ; après un zoom ou un déplacement, `F` le recadre ; la position du pointeur s'affiche en mm (et en pouces quand l'unité est changée), origine du plateau à (0, 0).
- Thème, langue, unités et outil actif sont retrouvés après un rechargement de la page.
- Aucune couleur en dur dans les composants ni dans le dessin de l'espace de travail ; `en.ts` a les mêmes clés que `fr.ts` (vérifié par TypeScript).

## Vérifications de l'utilisateur

- Ouvrir l'application dans le navigateur et dire si la disposition (palette à gauche, paramètres à droite) convient à la suite du travail.
- Vérifier que les outils de la palette et leurs icônes sont les bons pour v0.1.
