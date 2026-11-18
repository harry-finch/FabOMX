---
id: R-004
titre: "Nettoyer le dessin"
resume: "Section Nettoyer : doublons, superpositions, minuscules, jonctions sous un seuil, segments alignés, avec comptes et bilan en direct, liste des défauts et option à l'import ; rien n'est inventé."
statut: livree
jalon: v0.1
cree: 2026-11-18
maj: 2026-11-18
livree: 2026-11-18
---

## But

Rendre un dessin importé bon pour la découpe sans repasser par le logiciel de dessin quand les défauts sont des défauts de fichier : doublons, superpositions, extrémités presque jointes, éléments minuscules, segments alignés en trop. L'utilisateur voit ce qui sera corrigé avant de le faire, choisit les seuils, et sait ce qui restera à reprendre à la main. Rien n'est inventé : un contour qui ne se ferme pas par une jonction sous le seuil reste ouvert et signalé.

## Inclus

- **Outil Nettoyer (C)** actif. Il ouvre une section « Nettoyer » dans le panneau droit pour le dessin sélectionné. Sans sélection, une phrase dit de choisir un dessin.
- **Cinq opérations à cocher**, chacune avec le nombre de corrections qu'elle fera, en direct :
  - retirer les doublons exacts ;
  - fusionner les superpositions partielles (lignes alignées qui se chevauchent, arcs d'un même cercle qui se chevauchent) ;
  - retirer les éléments minuscules, sous une taille max (0,1 mm par défaut, réglable de 0,001 à 1 mm). Un élément minuscule pris dans un contour est remplacé par la jonction de ses voisins, pour ne pas ouvrir le contour ;
  - joindre les extrémités, sous un écart max (0,01 mm par défaut, réglable de 0,001 à 1 mm). Les deux bouts se rejoignent en leur milieu ;
  - simplifier les segments alignés : une suite de lignes alignées (à 0,001 mm près) devient une seule ligne. Les arcs ne sont pas touchés.
- **Seuils** : en mm sur le plateau, quelle que soit l'échelle du dessin. Ils sont mémorisés sur le poste, avec les cases cochées.
- **Ce qui restera** :
  - le bilan après nettoyage (contours fermés et ouverts) est affiché avant d'appliquer ;
  - les contours qui resteront ouverts sont dits, avec la raison (« écart de 0,3 mm, au-dessus du seuil »).
- **Voir chaque défaut** :
  - une liste dans la section, groupée par type et numérotée ;
  - quand l'outil Nettoyer est actif, les défauts sont marqués sur le plateau ;
  - un clic sur un défaut de la liste centre la vue dessus et le met en évidence.
- **Appliquer** : le bouton Nettoyer corrige la géométrie d'origine du dessin. La position, la rotation et l'échelle sont gardées, et les informations du dessin (éléments, à nettoyer) sont recalculées. Un nettoyage est une seule étape d'annulation.
- **Nettoyer à l'import** : une case dans le dialogue d'import, avec les réglages du dernier nettoyage. Cochée, l'aperçu, les comptes et le dessin posé sont ceux du dessin nettoyé, et le dialogue dit ce qui a été corrigé. La case est mémorisée sur le poste.
- Aide et documentation mises à jour.

## Hors périmètre

- Fermer un contour en ajoutant un segment qui n'est pas dans le dessin.
- Croisements et auto-intersections, sens des contours (intérieur / extérieur), décalage du trait de coupe : la découpe (jalon v0.2).
- Reconnaître des arcs dans une suite de segments (courbes converties) : idée possible pour plus tard.
- Nettoyer plusieurs dessins d'un coup.
- Défauts entre deux dessins différents (doublons d'un dessin à l'autre).

## Dépendances

- R-001 et R-002 (livrés).

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Tests du moteur, sur des fichiers d'exemple :
  - **Jonction** : `gap.dxf` nettoyé avec l'écart par défaut donne 1 contour fermé et 0 ouvert. Un carré dont l'écart est de 0,3 mm reste ouvert à 0,01 mm et se ferme à 0,5 mm.
  - **Doublons** : `duplicate.dxf` nettoyé n'a plus de doublon et garde 1 contour fermé.
  - **Superposition** : deux lignes de 0 à 60 mm et de 40 à 100 mm sur le même axe deviennent une ligne de 0 à 100 mm.
  - **Minuscules** : un carré avec un segment de 0,05 mm dans un coin perd ce segment et reste un contour fermé.
  - **Simplification** : une polyligne de 10 segments alignés devient 1 ligne, et un arc voisin reste un arc.
  - **Rien d'inventé** : `single-line.dxf` nettoyé garde 1 contour ouvert et l'avertissement « aucun contour fermé ».
  - **Bilan** : les comptes annoncés avant le nettoyage sont ceux obtenus après.
  - **Seuil à l'échelle** : un dessin mis à l'échelle 10 % a un écart de 0,05 mm (0,5 mm d'origine) qui se joint avec un seuil de 0,1 mm.
- Dans le navigateur :
  - cocher ou décocher une opération, ou changer un seuil, met à jour les comptes et le bilan sans appliquer ;
  - Nettoyer applique, la position du dessin ne change pas, et ⌘Z rend le dessin d'avant en une étape ;
  - un clic sur un défaut de la liste centre la vue dessus ;
  - « Nettoyer à l'import » coché pose un dessin déjà nettoyé, et la case est retrouvée après un rechargement.
- Textes en français et en anglais ; aucune couleur en dur.

## Vérifications de l'utilisateur

- Nettoyer un ou deux de tes fichiers « sales » réels et dire si le résultat est bon pour la découpe.
- Juger la section Nettoyer et la liste des défauts : clarté des comptes, du bilan et des seuils.
