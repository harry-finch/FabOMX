---
id: R-015
titre: "Onglets de maintien"
resume: "Onglets sur les contours extérieurs choisis : onglet de fin sans perçage, onglets le long du contour avec arrêt du jet et reperçage dans la chute ; nombre réparti ou placés au clic ; largeur 1 mm par défaut, réglable."
statut: a-tester
jalon: v0.3
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

Une petite pièce découpée peut tomber entre les lattes, basculer sous le jet ou bouger avant la fin de la coupe. Des onglets, de petits ponts de matière laissés sur son contour, la gardent accrochée à la plaque. On la détache ensuite à la main et on lime la trace. L'utilisateur choisit les pièces qui en ont besoin, leur nombre ou leur place, et leur largeur. FabOMX écrit dans le parcours les arrêts du jet et les reperçages nécessaires.

## Inclus

- **Deux sortes d'onglets**, sur les contours Extérieur seulement : un trou tombe dans la chute, une fente n'a pas de pièce à tenir.
  - **Onglet de fin** : la coupe s'arrête avant de revenir au point d'entrée, à la largeur de l'onglet, puis sort dans la chute par l'amorce de sortie. Il ne coûte aucun perçage.
  - **Onglets le long du contour** : à chacun, la coupe s'arrête avant l'onglet et sort dans la chute par une courte amorce de sortie, le jet se coupe, puis il perce à nouveau dans la chute et rejoint le contour après l'onglet par une amorce d'entrée. Les amorces sont celles du contour (forme et longueurs, R-010).
- **Placer**, dans l'outil Découpe, pour les contours Extérieur sélectionnés :
  - « Onglets » donne un nombre de 0 à 20 par contour, répartis régulièrement le long du contour. Le premier est l'onglet de fin, au point d'entrée, et les autres sont des onglets avec reperçage ;
  - à la main, avec un seul contour sélectionné : « Placer des onglets », puis un clic sur le contour ajoute un onglet, et un clic sur un onglet le retire. Échap termine ;
  - aucun onglet par défaut ;
  - un changement est une étape d'annulation.
- **Largeur** :
  - 1 mm par défaut, réglable de 0,2 à 10 mm, mesurée le long du contour sur le trajet du jet ;
  - dans les réglages par défaut du poste, avec une largeur propre possible pour les contours sélectionnés.

  Le pont de matière réel est plus étroit que cette largeur, de la saignée du jet, que MAKE compense ; le panneau le rappelle.
- **Sur le plateau** : dans les outils Découpe et Parcours, chaque onglet est marqué sur son contour. Dans l'outil Parcours, les arrêts, déplacements et reperçages apparaissent avec le reste.
- **Bilan et OMX** :
  - le nombre de perçages compte les reperçages ;
  - l'OMX écrit ces arrêts comme LAYOUT écrit tout déplacement : Q 0 entre la sortie et le nouveau perçage, Q 9 pour les amorces, puis la qualité du contour.
- **Remarques** : deux onglets qui se touchent, ou un onglet qui touche l'amorce d'un autre, sont signalés (code stable `tab.too_close`), marqués sur le plateau et repris dans les remarques avant l'export.
- **Durée de vie** : les onglets suivent le dessin quand il est transformé. Un nettoyage les perd, comme les autres réglages faits à la main, et l'avis le dit.
- Aide et documentation mises à jour.

## Hors périmètre

- Onglets sur les trous et les fentes.
- Onglets sans reperçage ailleurs qu'en fin de contour, par exemple une coupe aller-retour.
- Déplacer un onglet en le glissant : on le retire et on en ajoute un.
- Calculer la saignée, ou donner la largeur du pont de matière après coupe : MAKE seul connaît la saignée.
- Onglets automatiques selon la taille de la pièce.
- Couper une plaque dans un coin du stock : c'est R-016.

## Dépendances

- R-008 à R-012 (livrés).
- Hypothèse à vérifier à la coupe : une largeur de 1 mm laisse un pont de matière suffisant une fois la saignée retirée. Sinon, on changera la valeur par défaut.

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Tests du moteur :
  - **Onglet de fin** : un carré de 50 mm avec un onglet de 1 mm est coupé sur 199 mm et ne compte qu'un perçage.
  - **Trois onglets** : le même carré avec 3 onglets compte 3 perçages et 197 mm de coupe. Chaque reperçage est dans la chute, et chaque onglet laisse 1 mm non coupé à sa place.
  - **Placés à la main** : des onglets donnés par leurs points sont projetés sur le contour et utilisés.
  - **Trop proches** : deux onglets à moins de leur largeur, ou un onglet sur une amorce, donnent `tab.too_close`.
  - **Pas sur les trous** : un trou n'a jamais d'onglet, même demandé.
  - **OMX** : un carré à 3 onglets s'écrit avec 3 passages de Q 0 à Q 9, sans compter le départ.
- Dans le navigateur :
  - « Onglets » à 3 marque 3 onglets sur le contour et change le bilan à 3 perçages ;
  - un clic avec « Placer des onglets » en ajoute un, un autre clic le retire, et ⌘Z revient ;
  - un trou sélectionné ne propose pas d'onglets ;
  - la largeur par défaut est retrouvée après un rechargement.
- Textes en français et en anglais ; aucune couleur en dur.

## Vérifications de l'utilisateur

- Couper une petite pièce avec 2 ou 3 onglets de 1 mm, et dire si elle tient pendant la coupe et se détache bien ensuite (et si 1 mm est la bonne largeur par défaut).
- Juger le placement des onglets dans l'outil Découpe et leur aspect dans l'outil Parcours.
