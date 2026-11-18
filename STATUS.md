# État du projet

**Jalon** : **v0.1 — Interface et dessin**, en cours. R-003 (squelette de l’interface) livré le 18/11/2026. Suite : voir [ROADMAP.md](ROADMAP.md).

| Élément | État |
| --- | --- |
| Moteur | `fabomx-core` : la machine (ProtoMAX, plateau 305 × 305 mm) et la version ; `fabomx-wasm` les donne à l’interface |
| Interface | Barre d’application, palette de sept outils (Sélection et Main actifs, cinq grisés « à venir »), plateau à l’échelle avec grille et origine, zoom, déplacement de la vue, recadrage, coordonnées, panneau Plateau, FR/EN, sombre/clair, mm/pouces |
| Tests | `cargo test --workspace` : 2 verts ; `cargo clippy` et `cargo fmt --check` propres ; `npm run check` : 0 erreur, 0 avertissement (18/11/2026) |
| Vérifié | Dans Chrome piloté, à 1440 px (sombre) et 1100 px (clair) : disposition, aucun défilement horizontal, origine lue à (0, 0) et coin à (305, 305), zoom à la molette, F, espace + glisser, outils grisés inertes, préférences retrouvées après rechargement, console sans erreur (18/11/2026) ; disposition et outils de la palette validés par l’utilisateur (A-002, A-003, 18/11/2026) |
| Non vérifié | Safari et Firefox |

## Points ouverts

- Logo provisoire du kit (R-006).

## À suivre

- Idées à spécifier : R-001, R-002, R-004, R-005, R-006. Actions en attente : voir FabBoard.
