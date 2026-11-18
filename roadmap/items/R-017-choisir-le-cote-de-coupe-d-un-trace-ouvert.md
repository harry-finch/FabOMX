---
id: R-017
titre: "Choisir le côté de coupe d'un tracé ouvert"
resume: "Pour un tracé ouvert : sur la ligne, pièce d'un côté ou de l'autre (saignée vers la chute, côté 1 dans l'OMX, sens imposé), avec hachures sur le plateau ; amorces au choix dans le prolongement du tracé pour percer hors de la plaque."
statut: livree
jalon: v0.3
cree: 2026-11-18
maj: 2026-11-18
livree: 2026-11-18
---

## But

Un tracé ouvert est aujourd'hui toujours coupé sur la ligne : la saignée se partage des deux côtés du trait. Pour couper une plaque en deux à la cote, ou détourer un bord droit, la pièce doit garder sa dimension exacte d'un côté, la saignée partant de l'autre côté, dans la chute. L'utilisateur choisit de quel côté du tracé est la pièce, le voit sur le plateau, et peut percer hors de la plaque grâce à des amorces dans le prolongement du tracé.

## Inclus

- **Trois choix pour un tracé ouvert**, dans l'outil Découpe :
  - **Sur la ligne**, par défaut et comme aujourd'hui : côté 0 dans l'OMX, sans décalage ;
  - **Pièce d'un côté** et **Pièce de l'autre côté** : MAKE décale la saignée vers l'autre côté, la chute ;
  - un bouton **Changer de côté** passe d'un côté à l'autre.

  Un changement est une étape d'annulation, pour un ou plusieurs tracés sélectionnés.
- **Aperçu sur le plateau** : pour un tracé décalé, de petites hachures le long du trait marquent le côté de la saignée (la chute). La flèche de sens et le chiffre de qualité restent.
- **Écriture** : comme pour les contours fermés, la pièce est à droite du jet (côté 1, exemples LAYOUT). Le côté choisi fixe donc le sens de parcours, et donc le bout où l'on perce. « Changer de côté » perce à l'autre bout. Le point d'entrée choisi à la main ne s'applique plus à un tracé décalé : le panneau le dit.
- **Amorces dans le prolongement**, en option (case à cocher) pour les tracés ouverts sélectionnés, décalés ou non :
  - l'entrée prolonge le tracé avant son début, la sortie le prolonge après sa fin ;
  - longueurs et forme droite reprises des amorces du contour (R-010). Une amorce qui toucherait un autre contour est raccourcie ou signalée, comme les autres ;
  - elles permettent de percer hors de la plaque et d'en sortir proprement quand on coupe de bord à bord ;
  - elles s'écrivent en Q 9, du côté du tracé.
- Le parcours, le bilan, la lecture et l'export suivent, sans changement pour les contours fermés.
- **Durée de vie** : comme les autres réglages faits à la main, le choix suit les transformations du dessin et se perd au nettoyage, avec l'avis.
- Aide et documentation mises à jour.

## Hors périmètre

- Choisir le bout de perçage indépendamment du côté. Il faudrait un autre code de côté dans l'OMX, à confirmer par un exemple de LAYOUT.
- Décaler un contour fermé coupé sur la ligne : les contours fermés ont déjà Extérieur et Trou.
- Onglets sur un tracé ouvert.
- Couper une plaque dans un coin du stock (R-016), mis de côté.

## Dépendances

- R-008, R-010 et R-012 (livrés).

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Tests du moteur :
  - **Sur la ligne** : une ligne de (0, 10) à (80, 10) s'écrit comme aujourd'hui, côté 0.
  - **Pièce d'un côté** : la même ligne, pièce au-dessus, s'écrit côté 1, parcourue pour que la pièce soit à droite du jet, de (80, 10) vers (0, 10).
  - **Pièce de l'autre côté** : pièce en dessous, elle s'écrit côté 1, de (0, 10) vers (80, 10).
  - **Prolongements** : avec les amorces dans le prolongement, le perçage est à 3,8 mm avant le début du tracé et la sortie s'arrête à 1,3 mm après sa fin, dans l'axe, en Q 9, avec un seul perçage.
  - **Contour voisin** : un prolongement qui toucherait un autre contour est raccourci.
  - **Fermés inchangés** : les contours fermés et leurs OMX ne changent pas.
- Dans le navigateur :
  - un tracé ouvert sélectionné propose les trois choix, et « Changer de côté » passe d'un côté à l'autre ;
  - les hachures passent de côté, ⌘Z revient ;
  - la case des prolongements dessine les amorces dans l'axe ;
  - l'OMX exporté contient le côté 1 pour le tracé décalé.
- Textes en français et en anglais ; aucune couleur en dur.

## Vérifications de l'utilisateur

- Couper une plaque en deux avec un tracé décalé, de bord à bord avec les prolongements, et mesurer la pièce gardée.
- Juger le choix du côté et les hachures dans l'outil Découpe.

## Notes pour la spécification

- Demande de l'utilisateur (18/11/2026) : une fente reste sur la ligne par défaut (OffsetSide 0, confirmé à la coupe, A-023), mais on doit pouvoir choisir le côté de la pièce, par exemple pour couper une plaque en deux à la cote.
- Piste : dans l'OMX, le décalage s'écrit avec OffsetSide = 1, la pièce à droite du jet (exemples LAYOUT). Pour garder la pièce de l'autre côté, il suffit de parcourir le tracé dans l'autre sens, sans nouveau code à deviner.
- À trancher : comment montrer et choisir le côté sur le plateau (une flèche, un clic du côté de la pièce), et les amorces d'un tracé ouvert décalé.
