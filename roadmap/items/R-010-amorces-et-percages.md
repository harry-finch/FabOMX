---
id: R-010
titre: "Amorces et perçages"
resume: "Amorces d'entrée et de sortie (droites ou en arc, longueurs par défaut ou par contour) et point de perçage dans la chute, placées sur un coin ou à la main ; petit trou percé au centre, amorce raccourcie ou signalée quand elle ne tient pas."
statut: a-tester
jalon: v0.2
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

Le jet ne doit pas percer sur le trait de la pièce : le perçage fait un trou plus large que la saignée et abîme le bord. Chaque contour coupé commence donc par une amorce d'entrée qui part d'un point de perçage pris dans la chute, et se termine par une courte amorce de sortie. FabOMX les place tout seul, comme LAYOUT, à un endroit où la marque se voit peu. L'utilisateur les voit sur le plateau, règle leur forme et leur longueur, et déplace le point d'entrée d'un contour s'il le faut.

## Inclus

- **Amorces et perçage** pour chaque contour Extérieur ou Trou :
  - une amorce d'entrée part du point de perçage, dans la chute, et rejoint le contour au point d'entrée ;
  - une amorce de sortie quitte le contour à la fin de sa coupe, vers la chute ;
  - la chute est dehors pour un Extérieur, dedans pour un Trou ;
  - le sens de parcours est celui de l'outil Découpe et suivra son inversion prévue avec R-012.
- **Forme** :
  - **droite par défaut**, comme LAYOUT. À un coin, l'entrée prolonge le premier segment quand ce prolongement est dans la chute ; sinon elle arrive le long de la bissectrice du coin, ou perpendiculairement au contour. La sortie prolonge de même le dernier segment à un coin, sinon elle est perpendiculaire, comme dans l'exemple `rectangle` de LAYOUT ;
  - **en option, en arc** : un quart de cercle tangent au contour, de la longueur réglée.
- **Longueurs** : entrée 3,8 mm et sortie 1,3 mm par défaut, les valeurs des exemples LAYOUT. Elles sont réglables de 0,5 à 20 mm, en mm du plateau quelle que soit l'échelle du dessin.
- **Réglages** :
  - la forme et les longueurs par défaut sont dans la section Découpe et mémorisées sur le poste. Ce n'est pas une étape d'annulation ;
  - les contours sélectionnés peuvent avoir une forme et des longueurs propres, avec un retour aux réglages par défaut. Un changement est une étape d'annulation.
- **Point d'entrée** :
  - **automatique** : sur un coin, c'est-à-dire un changement de direction d'au moins 30°, où la marque d'amorce se voit le moins. On prend le coin le plus marqué, puis à égalité le plus en bas à gauche. Sans coin, c'est le milieu du plus long segment ;
  - **à la main** : avec un seul contour sélectionné, « Choisir le point d'entrée » permet de cliquer un point du contour. Le point s'accroche aux coins et aux extrémités, sinon il prend le point le plus proche sur le trait. Échap annule. C'est une étape d'annulation, et « Point d'entrée automatique » revient au point calculé.
- **Quand l'amorce ne tient pas** :
  - **petit trou** : si l'entrée ne tient pas dans le trou, le perçage se fait en son centre, comme dans l'exemple LAYOUT, quand le segment du centre au point d'entrée reste dans le trou ;
  - **contour trop proche** : une amorce qui toucherait un autre contour coupé du même dessin, ou sortirait de la chute, est raccourcie. En dessous de 0,5 mm, un avertissement à code stable (`lead.too_short`) est marqué sur le plateau et listé dans la section, et un clic dessus centre la vue.
- **Fentes et contours coupés sur la ligne** : pas d'amorce. Le perçage se fait au point d'entrée : l'extrémité de départ pour un tracé ouvert, ou l'autre extrémité si on la choisit à la main.
- **Sur le plateau**, avec l'outil Découpe : les amorces sont en trait fin, de la couleur de leur contour, et le point de perçage est marqué d'un petit cercle.
- **Durée de vie** : les points d'entrée et les réglages propres suivent le dessin quand il est transformé. Un nettoyage les perd, et l'avis existant le dit.
- Aide et documentation mises à jour.

## Hors périmètre

- **Ordre de coupe, déplacements jet coupé et aperçu du parcours complet** : c'est R-011.
- **Écriture des amorces dans l'OMX** : c'est R-013, en qualité d'amorce (code 9, vu dans les exemples).
- **Surcoupe ou recouvrement en fin de contour, micro-attaches qui retiennent la pièce.**
- **Qualité propre des amorces, perçage en mouvement ou perçage spécial** : ce sont des réglages de MAKE.
- **Amorce qui évite les contours des autres dessins.** Les chevauchements de dessins sont déjà signalés.
- **Amorce sur une fente.**

## Dépendances

- R-008 et R-009 (livrés).
- Le sens réel de la ProtoMAX (pièce à droite du jet) sera appliqué avec R-012 ; les amorces suivront ce changement.

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Tests du moteur :
  - **Rectangle extérieur** : un rectangle de 50 × 30 mm est percé hors de la pièce, à 3,8 mm d'un coin, dans le prolongement du premier segment. Sa sortie fait 1,3 mm, perpendiculaire, vers l'extérieur.
  - **Petit trou** : dans un disque, un trou de rayon 3,5 mm est percé en son centre.
  - **Grand trou** : un trou de rayon 20 mm a une entrée de 3,8 mm à l'intérieur du trou.
  - **Arc** : en arc, l'entrée est tangente au contour au point d'entrée, et son perçage est dans la chute.
  - **Contour voisin** : à 2 mm d'un autre contour, l'amorce est raccourcie sous 2 mm. À 0,3 mm, l'avertissement `lead.too_short` est donné.
  - **Fente** : une ligne ouverte n'a pas d'amorce et est percée à son extrémité.
  - **Point choisi** : un point d'entrée choisi sur un coin est celui utilisé.
  - **Échelle** : un dessin mis à l'échelle × 2 garde une entrée de 3,8 mm sur le plateau.
- Dans le navigateur :
  - les amorces et les perçages se voient avec l'outil Découpe et suivent une rotation du dessin ;
  - changer une longueur par défaut les met à jour, et elle est retrouvée après un rechargement ;
  - une longueur propre donnée à un contour ne change que lui, et ⌘Z la rend ;
  - « Choisir le point d'entrée » puis un clic sur un coin y déplace l'amorce ;
  - un avertissement cliqué centre la vue.
- Textes en français et en anglais ; aucune couleur en dur.

## Vérifications de l'utilisateur

- Regarder les amorces et les perçages proposés sur deux ou trois de tes vrais dessins, et dire s'ils sont bien placés pour la ProtoMAX.
- Juger les réglages d'amorce et le choix du point d'entrée dans l'outil Découpe.
