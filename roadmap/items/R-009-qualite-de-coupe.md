---
id: R-009
titre: "Qualité de coupe"
resume: "Qualité de coupe de 1 à 5 : une qualité par défaut mémorisée sur le poste, et une qualité propre pour les contours choisis dans l'outil Découpe, affichée près de leur flèche."
statut: idee
jalon: v0.2
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

La qualité de coupe règle la vitesse du jet : 1 coupe vite avec un bord rugueux, 5 coupe lentement avec un bord net. On ne veut pas tout couper en 5, trop lent, ni tout en 1, trop grossier pour les bords qui comptent. L'utilisateur fixe une qualité par défaut pour tout le travail et la change pour les contours qui le demandent. Il voit sur le plateau la qualité de chaque contour avant de produire le fichier.

## Inclus

- **Qualités de 1 à 5**, nommées comme dans LAYOUT. Une infobulle rappelle que 1 est rapide et rugueux, 5 lent et net.
- **Qualité par défaut** :
  - dans la section Découpe, valable pour tous les contours coupés qui n'ont pas de qualité propre ;
  - 3 au départ, mémorisée sur le poste ;
  - la changer met à jour tous ces contours. Ce n'est pas une étape d'annulation, c'est un réglage.
- **Qualité propre** :
  - pour les contours sélectionnés dans l'outil Découpe, à côté du choix du côté ;
  - une sélection mêlée n'affiche aucune valeur ;
  - « Qualité par défaut » rend aux contours sélectionnés la qualité par défaut ;
  - un changement est une étape d'annulation.
- **Tout sélectionner** : ⌘A, dans l'outil Découpe, prend tous les contours du dessin sélectionné. Dans l'application Mac, Édition > Tout sélectionner fait de même, et sélectionne toujours le texte quand on est dans un champ de saisie. Un bouton « Tout sélectionner » est aussi dans la section.
- **Contours non coupés** : un contour Ne pas couper n'a pas de qualité. S'il est sélectionné avec d'autres, le choix de qualité ne vaut que pour les autres.
- **Sur le plateau** : avec l'outil Découpe, un petit chiffre (1 à 5) près de la flèche de chaque contour coupé. Il est en accent quand la qualité est propre au contour.
- **Dans le panneau** : le nombre de contours de chaque qualité est affiché, pour le dessin sélectionné.
- **Durée de vie** : les qualités propres suivent le dessin quand il est transformé. Un nettoyage les perd, comme les côtés choisis à la main, et l'avis existant le dit.
- Aide et documentation mises à jour.

## Hors périmètre

- **Gravure** : c'est R-014, en attente d'un exemple OMX (A-015).
- **Qualité des amorces** : elle est propre à l'amorce dans l'OMX (code 9) et viendra avec R-010.
- **Matériau, épaisseur, vitesses calculées, temps de coupe** : ce sont MAKE et la ProtoMAX qui les traitent.
- **Qualité par défaut distincte pour les trous et pour les extérieurs.**
- **Qualité différente sur une partie seulement d'un contour.**
- **Écriture dans l'OMX** : c'est R-013. Seules les qualités 1 à 5 sont valides, comme l'exemple (Q = 3) le confirme.

## Dépendances

- R-008 (livré).

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Moteur : une qualité hors de 1 à 5 est refusée avec un code stable (`cut.quality`). La qualité retenue d'un contour est sa qualité propre, sinon la qualité par défaut, et un contour Ne pas couper n'en a pas. Tout cela est vérifié par des tests.
- Dans le navigateur :
  - la qualité par défaut passée de 3 à 4 change le chiffre de tous les contours sans qualité propre, et elle est retrouvée après un rechargement ;
  - un contour mis en 5 garde 5 quand la qualité par défaut change, et son chiffre est en accent ;
  - ⌘Z rend la qualité d'avant en une étape ;
  - ⌘A sélectionne tous les contours du dessin, et le choix de qualité s'applique à tous sauf aux Ne pas couper ;
  - « Qualité par défaut » enlève la qualité propre ;
  - après un nettoyage, les qualités propres sont perdues et l'avis le dit ;
  - ⌘A dans un champ de saisie sélectionne son texte.
- Textes en français et en anglais ; aucune couleur en dur.

## Vérifications de l'utilisateur

- Régler la qualité par défaut et celle de quelques contours sur un de tes dessins, et juger le panneau et les chiffres sur le plateau.
- Dans l'application Mac, vérifier ⌘A (contours dans l'outil Découpe, texte dans un champ).
