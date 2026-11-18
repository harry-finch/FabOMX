# État du projet

**Jalon** : **v0.1 — Interface et dessin**, en cours. R-003 (squelette de l’interface) livré le 18/11/2026 ; R-001 (import SVG et DXF) développé, à tester. Suite : voir [ROADMAP.md](ROADMAP.md).

| Élément | État |
| --- | --- |
| Moteur | `fabomx-core` : la machine (ProtoMAX, plateau 305 × 305 mm) ; import SVG et DXF (lignes, arcs et cercles exacts, courbes à 0,01 mm), unité lue ou supposée, contours fermés et ouverts, défauts à nettoyer, avertissements ; `fabomx-wasm` le donne à l’interface |
| Interface | Barre d’application, palette de sept outils (Sélection, Main et Importer actifs, quatre grisés « à venir »), dialogue d’import (aperçu, unité modifiable, dimensions, éléments, à nettoyer, avertissements, refus), plusieurs dessins centrés sur le plateau, sélection, retrait, annuler / rétablir, dépôt de fichier, plateau à l’échelle avec grille et origine, zoom, déplacement de la vue, recadrage, coordonnées, panneau Plateau, FR/EN, sombre/clair, mm/pouces |
| Tests | `cargo test --workspace` : 20 verts, dont 14 d’import sur fichiers d’exemple ; `cargo clippy` et `cargo fmt --check` propres ; `npm run check` : 0 erreur, 0 avertissement (18/11/2026) |
| Vérifié | Dans Chrome piloté, à 1440 px (sombre) et 1100 px (clair) : disposition, aucun défilement horizontal, origine lue à (0, 0) et coin à (305, 305), zoom à la molette, F, espace + glisser, outils grisés inertes, préférences retrouvées après rechargement, console sans erreur (18/11/2026) ; import par le bouton, I, ⌘O et dépôt, changement d’unité, refus d’un fichier étranger, sélection au clic, Suppr, ⌘Z, ⇧⌘Z (Chrome piloté, 18/11/2026) ; disposition et outils de la palette validés par l’utilisateur (A-002, A-003, 18/11/2026) |
| Non vérifié | Fichiers réels de l’utilisateur (Inkscape, Fusion, Illustrator) ; jugement du dialogue d’import ; Safari et Firefox |

## Points ouverts

- Logo provisoire du kit (R-006).
- Les dessins importés ne sont pas retrouvés après un rechargement de la page (hors périmètre de R-001).
- Blocs DXF (`INSERT`) et calques : ignorés ou mis à plat.

## À suivre

- À tester : R-001. Idées à spécifier : R-002, R-004, R-005, R-006. Actions en attente : voir FabBoard.
