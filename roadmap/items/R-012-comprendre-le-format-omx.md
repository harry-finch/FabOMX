---
id: R-012
titre: "Comprendre le format OMX"
resume: "Décrire et reproduire le format OMX de la ProtoMAX à partir de fichiers réels produits par LAYOUT, avec des tests qui relisent ces fichiers."
statut: idee
jalon: v0.2
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## Notes tirées des exemples (A-012)

Deux paires DXF + OMX produites par LAYOUT, dans `omx/` : `rectangle` (un rectangle avec amorces) et `test` (un trou rond dans un disque, avec amorces).

- En-tête fixe en texte (lignes `//` de commentaires, phrase d'identification, `2` = parcours, `2` = version du format, date en jours depuis 1900, six `[Reserved]`, `[COMMENT]` … `[END]`), fins de ligne CRLF.
- Une ligne par point : `[0], X, Y, Z, TiltAngleStart, TiltAngleEnd, Bow, Q, OffsetSide, R, R, R, 0, 0, 0,[END]`. Coordonnées en **pouces**.
- Le `Q` et l'`OffsetSide` d'une ligne valent pour le segment qui part de ce point : `0` = déplacement jet coupé (côté 0), `9` = amorce, `1` à `5` = qualité de coupe (les exemples n'ont que la qualité 3) ; `OffsetSide` vaut `1` sur les coupes et les amorces des deux exemples.
- `Bow` est l'arrondi (bulge) du DXF : 0 = ligne, 1 = demi-cercle, positif = sens trigonométrique. Les arcs restent donc exacts.
- **Sens** : l'extérieur est coupé dans le sens horaire, le trou dans le sens trigonométrique : la pièce est à **droite** du jet. R-008 la met à gauche : à inverser ici (décision de l'utilisateur, 18/11/2026), dans `cut::reversed`, dans `ui/src/lib/cutting.svelte.ts` et dans le texte d'aide de l'outil Découpe.
- Les DXF fournis sont des parcours réexportés par LAYOUT : la qualité y est le nom du calque (`0`, `3`, `9`).
- Encore inconnus : le code de la gravure, ceux des pauses et délais (XData), le sens d'une fente.
