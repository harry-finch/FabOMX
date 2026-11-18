# État du projet

**Jalon** : **v0.1 — Interface et dessin**, en cours. R-003 (squelette de l’interface) livré le 18/11/2026 ; R-001 (import SVG et DXF) livré le 18/11/2026 ; R-002 (déplacer, tourner, mettre à l’échelle, miroir) livré le 18/11/2026 ; R-004 (nettoyer le dessin) livré le 18/11/2026 ; R-005 (application de bureau macOS) livré le 18/11/2026. Suite : voir [ROADMAP.md](ROADMAP.md).

| Élément | État |
| --- | --- |
| Moteur | `fabomx-core` : la machine (ProtoMAX, plateau 305 × 305 mm) ; import SVG et DXF (lignes, arcs et cercles exacts, courbes à 0,01 mm), unité lue ou supposée, contours fermés et ouverts, défauts à nettoyer, avertissements ; placement d’un dessin par une matrice appliquée à sa géométrie d’origine (arcs exacts sauf échelle non proportionnelle), débord du plateau, points d’accrochage ; nettoyage (doublons, superpositions, minuscules, jonctions sous un seuil, segments alignés) avec aperçu et défauts situés ; `fabomx-wasm` le donne à l’interface |
| Interface | Barre d’application, palette de sept outils (tous actifs), dialogue d’import (aperçu, unité modifiable, dimensions, éléments, à nettoyer, avertissements, refus), plusieurs dessins centrés sur le plateau, sélection, retrait, annuler / rétablir, dépôt de fichier, section Transformer (point de référence à 9 points, choisi sur le plateau ou par coordonnées, position, angle, ±90°, largeur, hauteur, échelle avec cadenas, miroirs), gestes à la souris (glisser avec Sélection ou Déplacer, tourner avec Maj par 15°, poignées d’échelle), flèches du clavier, débord signalé, section Nettoyer (cinq opérations à cocher, seuils, comptes et bilan en direct, liste des défauts qui centre la vue, une étape d’annulation), « Nettoyer à l’import », plateau à l’échelle avec grille et origine, zoom, déplacement de la vue, recadrage, coordonnées, panneau Plateau, FR/EN, sombre/clair, mm/pouces |
| Bureau | Application Tauri : `dist-desktop/FabOMX_0.1.0_aarch64.dmg` (signature locale), dialogue Ouvrir natif, 10 fichiers récents, menu macOS FR/EN, icône du logo provisoire |
| Tests | `cargo test --workspace` : 39 verts, dont 14 d’import, 7 de placement et 10 de nettoyage sur fichiers d’exemple ; `cargo clippy` et `cargo fmt --check` propres ; `npm run check` : 0 erreur, 0 avertissement (18/11/2026) |
| Vérifié | Dans Chrome piloté, à 1440 px (sombre) et 1100 px (clair) : disposition, aucun défilement horizontal, origine lue à (0, 0) et coin à (305, 305), zoom à la molette, F, espace + glisser, outils grisés inertes, préférences retrouvées après rechargement, console sans erreur (18/11/2026) ; import par le bouton, I, ⌘O et dépôt, changement d’unité, refus d’un fichier étranger, sélection au clic, Suppr, ⌘Z, ⇧⌘Z (Chrome piloté, 18/11/2026) ; disposition et outils de la palette validés par l’utilisateur (A-002, A-003, 18/11/2026) ; import de ses propres fichiers et dialogue d’import validés par l’utilisateur (A-004, A-005, 18/11/2026) ; R-002 dans Chrome piloté : champs (position, angle, ±90°, échelle, largeur cadenas ouvert), glisser puis ⌘Z en une étape, flèches, Maj par 15°, poignée d’échelle, accrochage au centre d’un cercle, avertissement d’échelle non proportionnelle, 1440 px sombre et 1100 px clair (18/11/2026) ; gestes et cotes validés par l’utilisateur (A-006, A-007, 18/11/2026) ; R-004 dans Chrome piloté : comptes et bilan qui suivent l’écart max, clic sur un défaut, Nettoyer sans bouger le dessin, ⌘Z / ⇧⌘Z, « Nettoyer à l’import » et sa mémoire, 1440 px sombre et 1100 px clair (18/11/2026) ; nettoyage de ses fichiers et section Nettoyer validés par l’utilisateur (A-008, A-009, 18/11/2026) ; application de bureau (installation, parcours complet, menus, fichiers récents) validée par l’utilisateur (A-010, A-011, 18/11/2026) |
| Non vérifié | Safari et Firefox |

## Points ouverts

- Logo provisoire du kit (R-006).
- Les dessins importés ne sont pas retrouvés après un rechargement de la page (hors périmètre de R-001).
- Blocs DXF (`INSERT`) et calques : ignorés ou mis à plat.

## À suivre

- À tester : R-005. Idées à spécifier : R-006 ; jalon v0.2 : R-007. Actions en attente : voir FabBoard.
