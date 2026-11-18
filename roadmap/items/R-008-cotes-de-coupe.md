---
id: R-008
titre: "Côtés de coupe"
resume: "Outil Découpe : pour chaque contour, Extérieur, Trou, Sur la ligne (fente) ou Ne pas couper (repère), détecté par imbrication et d'après le fichier (pointillés, calques), modifiable, avec sens de parcours fixé et croisements signalés."
statut: a-tester
jalon: v0.2
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

Avant de couper, il faut savoir de quel côté de chaque trait se trouve la pièce. Le jet a une largeur, et MAKE décale le trajet du bon côté seulement si on le lui dit. FabOMX le détermine tout seul d'après la forme du dessin : contour extérieur, trou, fente, repère à ne pas couper. L'utilisateur le voit d'un coup d'œil sur le plateau et corrige ce qui ne va pas. C'est la base des qualités, des amorces et de l'ordre de coupe (R-009 à R-011).

## Inclus

- **Outil Découpe (D)**, nouveau dans la palette. Il ouvre une section « Découpe » dans le panneau droit. Les qualités (R-009) et les amorces (R-010) y viendront aussi.
- **Quatre côtés possibles par contour** :
  - **Extérieur** : la pièce est dedans ;
  - **Trou** : la pièce est autour ;
  - **Sur la ligne** : une fente, coupée sans décalage ;
  - **Ne pas couper** : un repère ou un trait de construction, ignoré par le parcours.

  Un tracé ouvert ne peut être que Sur la ligne ou Ne pas couper.
- **Détection automatique, dessin par dessin, par imbrication** :
  - un contour fermé qui n'est dans aucun autre est Extérieur ;
  - le côté alterne ensuite à chaque niveau : un contour dans un Extérieur est un Trou, un contour dans un Trou redevient Extérieur (une pièce dans le trou), et ainsi de suite ;
  - un tracé ouvert est Sur la ligne.
- **Repères d'après le fichier** : à l'import, certains traits sont proposés Ne pas couper, et le dialogue d'import le dit avec leur nombre. Ce sont :
  - les traits en pointillés ou en tirets (type de trait DXF autre que continu, `stroke-dasharray` en SVG) ;
  - les traits du calque DXF « Defpoints » ;
  - les traits d'un calque dont le nom contient « construction » ou « repère » (sans tenir compte des majuscules ni des accents).
- **Croisements** : un contour qui se croise lui-même ou croise un autre contour du même dessin est mis Sur la ligne. Il est signalé par un avertissement à code stable et marqué sur le plateau. On peut quand même lui choisir un autre côté.
- **Dessins qui se chevauchent** : signalés par un avertissement, sans détection d'un dessin à l'autre.
- **Sens de parcours fixé** : chaque contour est orienté pour que la pièce soit toujours du même côté du jet, quel que soit le sens dans le fichier, y compris après un miroir. Une flèche discrète montre le sens sur le plateau.
- **Voir et corriger** :
  - avec l'outil Découpe, chaque contour est coloré selon son côté (Ne pas couper en pointillés) et une légende est dans le panneau ;
  - un clic sélectionne un contour, Maj + clic en ajoute ou en retire un ;
  - le panneau dit le côté de la sélection et permet de le changer. Revenir à la détection automatique est possible ;
  - le panneau compte les contours de chaque côté et liste les avertissements. Un clic sur un avertissement centre la vue dessus ;
  - un changement de côté est une étape d'annulation. Échap vide d'abord la sélection de contours.
- **Durée de vie des choix** :
  - les côtés suivent le dessin quand on le déplace, le tourne, le met à l'échelle ou le retourne ;
  - un nettoyage (R-004) recalcule la détection. Les choix faits à la main sont alors perdus, et la section le dit.
- Aide et documentation mises à jour.

## Hors périmètre

- Décaler le trait de la largeur du jet : c'est MAKE qui le fait, à partir du côté.
- Qualité de coupe (R-009), amorces et perçages (R-010), ordre de coupe et aperçu du parcours (R-011), écriture de l'OMX (R-012, R-013).
- Détection d'un dessin à l'autre (un dessin posé dans le trou d'un autre).
- Inverser le sens d'un contour à la main.
- Réparer un croisement.
- Choisir les calques à ignorer dans une liste à l'import, ou garder les calques au-delà de cette détection.

## Dépendances

- R-001, R-002 et R-004 (livrés).
- Le sens réel attendu par la ProtoMAX dans l'OMX sera confirmé avec les fichiers d'A-012 (R-012). Ici, le sens est fixé de façon cohérente dans le moteur.

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Tests du moteur, sur des fichiers d'exemple :
  - **Carré** : un carré seul est Extérieur.
  - **Trou** : un carré avec un cercle dedans donne un Extérieur et un Trou.
  - **Imbrication** : quatre carrés emboîtés donnent Extérieur, Trou, Extérieur, Trou, de l'extérieur vers l'intérieur.
  - **Ouvert** : `single-line.dxf` donne Sur la ligne.
  - **Repères** : une ligne DXF en pointillés, une ligne du calque Defpoints, une ligne d'un calque « Repères » et un trait SVG avec `stroke-dasharray` sont Ne pas couper.
  - **Croisement** : une forme en huit (contour qui se croise) est Sur la ligne et signalée, avec un code stable.
  - **Sens** : un carré tracé dans un sens et le même carré tracé dans l'autre sortent orientés pareil. Après un miroir, l'orientation reste la même par rapport à la pièce.
- Dans le navigateur :
  - avec l'outil Découpe, les couleurs suivent les côtés ;
  - un clic puis Maj + clic sélectionnent deux contours, et un changement de côté les modifie tous les deux ;
  - ⌘Z rend le côté d'avant en une étape ;
  - après une rotation du dessin, les côtés et les flèches suivent ;
  - un tracé ouvert ne propose pas Extérieur ni Trou ;
  - un clic sur un avertissement centre la vue.
- Textes en français et en anglais ; aucune couleur en dur (nouveaux jetons pour les côtés, dans les deux thèmes).

## Vérifications de l'utilisateur

- Ouvrir deux ou trois de tes vrais dessins et dire si les côtés détectés sont les bons, repères compris.
- Juger l'outil Découpe : couleurs, flèches, sélection et changement de côté.
