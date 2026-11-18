---
id: R-002
titre: "Redimensionner, déplacer et tourner le dessin"
resume: "Déplacer, tourner, mettre à l'échelle (proportionnelle ou non) et retourner un dessin, à la souris et par valeurs, autour d'un point de référence choisi ; calcul depuis la géométrie d'origine, débord signalé."
statut: specifiee
jalon: v0.1
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

Poser chaque dessin importé là où il doit être découpé, dans la bonne orientation et à la bonne taille, avant de passer à la découpe. On le fait à la souris sur le plateau pour aller vite, et par valeurs exactes dans le panneau droit quand la cote compte. Les cotes du dessin ne se dégradent jamais au fil des retouches.

## Inclus

- **Outils actifs** : Déplacer (M), Tourner (R) et Mettre à l'échelle (S). Ils agissent sur le dessin sélectionné, et un clic sur un autre dessin le sélectionne.
- **Point de référence**, commun à la rotation, à la mise à l'échelle et au miroir :
  - un sélecteur à 9 points de la boîte (centre par défaut) ;
  - ses coordonnées X/Y ;
  - « Choisir sur le plateau » : un clic s'accroche, à 8 px près, aux extrémités, aux centres d'arcs et aux centres de cercles du dessin.
  - Le point est marqué sur le plateau et suit le dessin quand on le déplace.
- **Déplacer** :
  - glisser le dessin ;
  - flèches du clavier (1 mm, 10 mm avec Maj) ;
  - X/Y du point de référence ;
  - boutons « À l'origine » (coin bas gauche de la boîte en (0, 0)) et « Centrer sur le plateau ».
- **Tourner** :
  - glisser autour du point de référence, par pas de 15° avec Maj ;
  - champ d'angle en degrés (sens trigonométrique) ;
  - boutons −90° et +90°.
- **Mettre à l'échelle** :
  - poignées aux coins de la boîte ;
  - largeur, hauteur et pourcentage dans le panneau ;
  - un cadenas, fermé par défaut, garde le ratio. Ouvert, largeur et hauteur sont indépendantes, et les poignées des côtés n'agissent que sur un axe.
- **Miroir** horizontal et vertical, autour du point de référence.
- **Exactitude** : chaque transformation se calcule depuis la géométrie d'origine du dessin, jamais par retouches successives.
  - Déplacement, rotation, miroir et échelle proportionnelle gardent les lignes, arcs et cercles exacts.
  - Une échelle non proportionnelle change les arcs et cercles en ellipses, mises en segments à 0,01 mm près. Un avertissement le dit.
- **Débord** : un dessin qui sort du plateau est permis. Un avertissement dit de combien il déborde (en X, en Y), et sa boîte passe en orange. Rien n'est corrigé en silence.
- **Panneau droit** : une section « Transformer » pour le dessin sélectionné, avec point de référence, position, angle, échelle et miroir. Les valeurs suivent les gestes à la souris en direct, et les informations du dessin (dimensions) suivent ses transformations.
- **Annuler** : un geste (un glissé, un champ validé, un bouton) est une étape d'annulation.
- Aide (raccourcis) et documentation mises à jour.

## Hors périmètre

- Sélection de plusieurs dessins et transformation d'un groupe.
- Accrochage du déplacement à la grille ou aux autres dessins, alignement et répartition entre dessins.
- Inclinaison (cisaillement), dupliquer un dessin.
- Retrouver les dessins et leurs transformations après un rechargement, enregistrer un document de projet.
- Imbrication automatique des pièces sur la tôle.

## Dépendances

- R-001 (livré).

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Tests du moteur, sur les fichiers d'exemple :
  - **Quart de tour** : un rectangle de 100 × 40 mm tourné de 90° autour de son centre mesure 40 × 100 mm, et son centre n'a pas bougé.
  - **Arcs gardés** : après rotation de 30°, miroir et échelle de 150 %, un arc est toujours un arc. Son rayon passe de 25 à 37,5 mm.
  - **Échelle libre** : un cercle de rayon 10 mm mis à l'échelle 200 % × 100 % devient une ellipse de 40 × 20 mm, à 0,01 mm près, et l'avertissement est présent.
  - **Pas de dérive** : tourner 36 fois de 10° ramène le dessin à sa position d'origine à 0,001 mm près. Deux miroirs successifs aussi.
  - **Point de référence** : tourner de 90° autour d'un coin de la boîte laisse ce coin en place.
  - **Débord** : un carré de 100 mm dont le coin bas gauche est en (250, 0) déborde de 45 mm en X, et l'avertissement le dit.
- Dans le navigateur :
  - glisser déplace le dessin, et X/Y suivent en direct ;
  - les flèches déplacent de 1 mm, et de 10 mm avec Maj ;
  - Maj pendant une rotation arrondit l'angle à 15° ;
  - le cadenas fermé garde le ratio ;
  - « Choisir sur le plateau » puis un clic près du centre d'un cercle y place le point de référence ;
  - ⌘Z après un glissé rend la position d'avant en une seule étape.
- Textes en français et en anglais ; aucune couleur en dur.

## Vérifications de l'utilisateur

- Manipuler un de tes dessins à la souris (déplacer, tourner, mettre à l'échelle, miroir) et dire si les gestes sont naturels.
- Sur un dessin réel, tourner autour d'un point choisi (un trou, un coin) et vérifier les cotes obtenues.
