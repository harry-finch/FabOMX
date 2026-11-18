---
id: R-011
titre: "Ordre de coupe et aperçu du parcours"
resume: "Outil Parcours : point de départ (origine, point choisi ou coordonnées) et retour, ordre automatique (contenu avant contenant, plus proche voisin) retouchable à la main, déplacements jet coupé signalés au-dessus de ce qui est découpé, bilan chiffré et lecture pas à pas."
statut: idee
jalon: v0.2
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

Avant de produire le fichier, l'utilisateur doit voir ce que fera la machine : par où la buse part, dans quel ordre elle coupe les contours, où elle perce, et par où elle passe jet coupé. FabOMX calcule un ordre sûr, les trous d'une pièce avant son extérieur, avec des déplacements courts. L'utilisateur peut le retoucher et choisir le point de départ, celui où il placera la buse sur sa plaque, par exemple pour percer un trou à un endroit précis d'une plaque déjà à la cote. C'est ce parcours que R-013 écrira dans l'OMX.

## Inclus

- **Outil Parcours (P)**, nouveau dans la palette. Avec lui, le plateau montre le parcours complet, tous dessins confondus :
  - les coupes à la couleur de leur côté ;
  - les amorces et les perçages ;
  - les déplacements jet coupé en pointillés ;
  - le point de départ.

  Le panneau droit contient la section « Parcours ».
- **Point de départ** :
  - par défaut l'origine du plateau (0, 0) ;
  - on peut le choisir sur le plateau, accroché aux coins du plateau, aux coins des boîtes des dessins et à leurs points (extrémités, centres), ou le saisir en X et Y ;
  - le panneau rappelle de placer la buse au-dessus de ce point de la plaque avant de lancer ;
  - le parcours revient au point de départ à la fin ;
  - un changement est une étape d'annulation.
- **Ordre automatique** :
  - seuls les contours coupés comptent : extérieurs, trous et fentes, pas les Ne pas couper ;
  - un contour est toujours coupé après tous ceux qu'il contient, dans le même dessin : les trous et fentes d'une pièce avant son extérieur, une pièce posée dans un trou avant ce trou ;
  - parmi les contours permis, le suivant est celui dont le perçage est le plus proche de la position de la buse, en partant du point de départ.
- **Ordre à la main** :
  - la section liste les contours dans l'ordre, avec leur dessin, leur côté et leur rang ;
  - un clic sur une ligne met le contour en évidence sur le plateau ;
  - « Plus tôt » et « Plus tard » déplacent le contour d'un rang. Un déplacement qui couperait un contour avant l'un de ceux qu'il contient est refusé, et un message dit lequel (code stable `path.order`) ;
  - « Ordre automatique » revient à l'ordre calculé ;
  - un changement est une étape d'annulation ;
  - les contours d'un dessin ajouté ensuite se placent à la suite, dans l'ordre automatique.
- **Déplacements jet coupé** : en ligne droite, de la fin de l'amorce de sortie, ou du point de départ, au perçage suivant. Un déplacement qui passe au-dessus d'une pièce ou d'un trou déjà découpés est marqué sur le plateau et signalé comme une remarque (code `path.over_cut`, pas un blocage). Un clic sur la remarque centre la vue.
- **Bilan chiffré** :
  - la longueur de coupe par qualité, amorces à part ;
  - la longueur des déplacements jet coupé ;
  - le nombre de perçages.
- **Lecture pas à pas** :
  - un curseur et un bouton lecture / pause font avancer la buse le long du parcours ;
  - ce qui est déjà coupé est en trait plein, le reste estompé ;
  - le parcours entier défile en 20 s au plus, quelle que soit sa longueur ;
  - le curseur peut aussi se glisser à la main.
- Le parcours suit les changements de dessin, de côté, de qualité, d'amorce et de placement.
- Aide et documentation mises à jour.

## Hors périmètre

- **Écriture de l'OMX** : c'est R-013. Le parcours calculé ici en est la source.
- **Inversion du sens de coupe**, pièce à droite du jet : c'est R-012. Le parcours suivra.
- **Contourner ce qui est déjà découpé**, plutôt que de le signaler.
- **Numéros d'ordre sur le plateau** et **temps de coupe estimé** : le temps dépend du matériau et de l'épaisseur, que MAKE connaît.
- **Ordre par glisser-déposer dans la liste.**
- **Mémoriser le parcours après un rechargement** : les dessins ne le sont pas non plus.
- **Ordre qui tient compte des chevauchements d'un dessin à l'autre** : ils sont déjà signalés.

## Dépendances

- R-008, R-009 et R-010 (livrés).
- Hypothèse à vérifier sur la machine (vérification de l'utilisateur ci-dessous) : MAKE pose le premier point de l'OMX à la position de la buse au lancement.

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Tests du moteur :
  - **Trous d'abord** : une plaque avec deux trous et une fente coupe les trois avant l'extérieur.
  - **Pièce dans un trou** : la pièce, et ses trous à elle, passent avant le trou qui la contient.
  - **Plus proche voisin** : trois carrés alignés à 50, 150 et 250 mm, départ en (0, 0), sont coupés dans cet ordre. Avec un départ en (305, 0), ils sont coupés dans l'ordre inverse.
  - **Ordre à la main** : un ordre permis est respecté. Un ordre qui met un extérieur avant son trou est refusé avec `path.order`.
  - **Départ et retour** : le parcours commence et finit au point de départ.
  - **Bilan** : un carré de 50 mm en qualité 3 donne 200 mm de coupe en qualité 3, un perçage, et la longueur exacte des déplacements.
  - **Survol** : un déplacement qui traverse un trou déjà coupé est signalé avec `path.over_cut`, et un déplacement qui le contourne de loin ne l'est pas.
- Dans le navigateur :
  - le parcours se voit avec l'outil Parcours et suit une rotation d'un dessin ;
  - un point de départ choisi sur un coin de dessin, ou saisi, change le premier déplacement ;
  - « Plus tard » change l'ordre, ⌘Z le rend, et un déplacement interdit affiche son message ;
  - la lecture avance la buse et s'arrête à la fin ;
  - le bilan change quand on change une qualité.
- Textes en français et en anglais ; aucune couleur en dur.

## Vérifications de l'utilisateur

- Regarder le parcours proposé pour deux ou trois de tes vrais dessins, et dire si l'ordre te paraît bon pour la ProtoMAX.
- Juger l'outil Parcours : point de départ, ordre à la main, bilan et lecture pas à pas.
- Vérifier sur la machine, avec un OMX de LAYOUT, que MAKE pose le premier point du fichier là où se trouve la buse au lancement (ou dire comment MAKE place le départ).
