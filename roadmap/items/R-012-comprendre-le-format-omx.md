---
id: R-012
titre: "Exporter l'OMX"
resume: "Écrire le parcours en OMX à l'identique de LAYOUT (pouces, Q, côté, arcs exacts en Bow), avec la pièce à droite du jet comme la machine ; bouton d'export dans l'outil Parcours, dialogue Enregistrer dans l'application Mac, remarques listées avant l'export."
statut: livree
jalon: v0.2
cree: 2026-11-18
maj: 2026-11-18
livree: 2026-11-18
---

## But

C'est l'aboutissement de FabOMX : produire le fichier que la ProtoMAX coupe. Le parcours préparé dans FabOMX s'écrit en OMX, dans le format exact de LAYOUT, et MAKE l'ouvre sans retouche. On n'a plus besoin de repasser par LAYOUT. Les arcs et cercles restent des arcs, pour un mouvement fluide. Le sens de coupe devient celui de la machine, la pièce à droite du jet, pour que MAKE décale la saignée du bon côté.

## Inclus

- **Format OMX dans le moteur** (lecture et écriture), à l'identique de LAYOUT :
  - en-tête de commentaires, identification, `2` = parcours, version `2`, date en jours depuis 1900, six `[Reserved]`, `[COMMENT]` … `[END]`, fins de ligne CRLF ;
  - une ligne par point : `[0], X, Y, Z, TiltStart, TiltEnd, Bow, Q, OffsetSide, R, R, R, 0, 0, 0,[END]`, en **pouces** à 10 décimales, alignée comme LAYOUT.
- **Sens de la machine** : la pièce est à droite du jet, l'extérieur coupé dans le sens horaire et le trou dans le sens trigonométrique, comme les exemples. Ça vaut pour le moteur, les flèches et les amorces de l'outil Découpe, le parcours, et le texte d'aide (« la pièce est toujours à droite du jet »).
- **Écriture du parcours**, celui de l'outil Parcours :
  - le premier point est le point de départ, là où sera la buse au lancement, et le parcours y revient ;
  - un déplacement jet coupé s'écrit Q = 0, côté 0 ;
  - une amorce s'écrit Q = 9 ;
  - une coupe s'écrit à sa qualité (1 à 5) ;
  - le côté vaut 1 pour un extérieur ou un trou et ses amorces, et 0 pour une fente. Le 0 est une hypothèse, à confirmer par un exemple (A-023) ;
  - les lignes, arcs et cercles du dessin restent exacts : un arc s'écrit avec son `Bow` (arrondi du DXF, positif dans le sens trigonométrique). Les courbes déjà mises en segments à l'import (Bézier, splines, ellipses) restent des segments, jusqu'à R-007.
- **Exporter** :
  - un bouton « Exporter l'OMX » dans l'outil Parcours, actif dès qu'il y a un contour à couper ;
  - dans le navigateur, le fichier se télécharge ; dans l'application Mac, un dialogue Enregistrer natif s'ouvre, filtré sur OMX ;
  - le nom proposé est celui du premier dessin, avec l'extension `.omx` ;
  - le commentaire du fichier dit « Créé avec FabOMX », sa version et la date.
- **Avant d'exporter** : s'il reste des remarques, un dialogue les liste et propose « Exporter quand même » ou « Annuler ». Les remarques comptées sont les débords du plateau, les contours qui se croisent, les dessins qui se chevauchent, les amorces trop courtes et les survols. Rien n'est bloqué.
- Aide et documentation mises à jour.

## Hors périmètre

- Reconnaître des arcs dans les courbes mises en segments : c'est R-007.
- Gravure, pauses, délais et autres XData : c'est R-014, ou plus tard.
- Inclinaison de la tête et axe Z : la ProtoMAX n'en a pas, ils valent 0.
- Ouvrir un OMX pour le modifier dans FabOMX : le moteur sait le lire, mais l'interface ne l'importe pas.
- Envoyer le fichier à la machine : MAKE le fait.

## Dépendances

- R-008 à R-011 (livrés).
- **A-023** : un OMX de LAYOUT avec une fente (et, si possible, un trou carré et une pièce dans un trou) pour confirmer le côté 0 des fentes. Ce n'est pas bloquant pour développer, mais à confirmer avant de livrer.

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check` et `cd ui && npm run check` passent sans erreur ni avertissement ; le crate de bureau passe `cargo clippy --release`.
- Tests du moteur :
  - **Relecture** : `omx/rectangle.omx` et `omx/test.omx` sont relus puis réécrits **octet pour octet**.
  - **Sens** : un rectangle extérieur est parcouru dans le sens horaire, un trou dans le sens trigonométrique.
  - **Écriture** : le parcours d'un carré de 50 mm avec un trou rond s'écrit ainsi :
    - le premier et le dernier point sont le point de départ ;
    - les codes Q valent 0, 9 et la qualité ;
    - le côté vaut 1 sur les coupes et les amorces, 0 sur les déplacements ;
    - les coordonnées sont en pouces ;
    - le cercle s'écrit avec des `Bow` non nuls, sans petits segments.
  - **Fente** : une ligne ouverte s'écrit avec le côté 0.
  - **Arc à l'échelle** : un dessin mis à l'échelle proportionnellement garde ses arcs. Une échelle non proportionnelle les écrit en segments.
- Dans le navigateur :
  - « Exporter l'OMX » télécharge un fichier dont le nom est celui du dessin ;
  - s'il y a une remarque, le dialogue la liste avant l'export ;
  - les flèches de l'outil Découpe tournent dans le nouveau sens.
- Textes en français et en anglais ; aucune couleur en dur.

## Vérifications de l'utilisateur

- Ouvrir dans MAKE l'OMX exporté d'une plaque avec un trou et une fente, et vérifier que le parcours, les côtés de coupe et les dimensions sont bons.
- Couper cette pièce sur la ProtoMAX et mesurer la pièce et son trou.
- Fournir un OMX de LAYOUT avec une fente (A-023).

## Notes tirées des exemples (A-012)

Deux paires DXF + OMX produites par LAYOUT, dans `omx/` : `rectangle` (un rectangle avec amorces) et `test` (un trou rond dans un disque, avec amorces).

- En-tête fixe en texte (lignes `//` de commentaires, phrase d'identification, `2` = parcours, `2` = version du format, date en jours depuis 1900, six `[Reserved]`, `[COMMENT]` … `[END]`), fins de ligne CRLF.
- Une ligne par point : `[0], X, Y, Z, TiltAngleStart, TiltAngleEnd, Bow, Q, OffsetSide, R, R, R, 0, 0, 0,[END]`. Coordonnées en **pouces**.
- Le `Q` et l'`OffsetSide` d'une ligne valent pour le segment qui part de ce point : `0` = déplacement jet coupé (côté 0), `9` = amorce, `1` à `5` = qualité de coupe (les exemples n'ont que la qualité 3) ; `OffsetSide` vaut `1` sur les coupes et les amorces des deux exemples.
- `Bow` est l'arrondi (bulge) du DXF : 0 = ligne, 1 = demi-cercle, positif = sens trigonométrique. Les arcs restent donc exacts.
- **Sens** : l'extérieur est coupé dans le sens horaire, le trou dans le sens trigonométrique : la pièce est à **droite** du jet. R-008 la met à gauche : à inverser ici (décision de l'utilisateur, 18/11/2026), dans `cut::reversed`, dans `ui/src/lib/cutting.svelte.ts` et dans le texte d'aide de l'outil Découpe.
- Les DXF fournis sont des parcours réexportés par LAYOUT : la qualité y est le nom du calque (`0`, `3`, `9`).
- **Départ** (vérifié par l'utilisateur sur la machine, A-022, 18/11/2026) : le premier point de l'OMX correspond à l'endroit où se trouve la buse quand l'usinage est lancé. Les coordonnées du fichier sont donc relatives à ce départ : le point de départ de l'outil Parcours doit devenir le premier point écrit.
- Encore inconnus : le code de la gravure, ceux des pauses et délais (XData), le sens d'une fente.
