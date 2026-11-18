<!-- Généré par FabBoard depuis roadmap/ : ne pas modifier à la main. -->

Objectif de FabOMX : à partir d'un dessin vectoriel (SVG ou DXF), produire un parcours d'outil (fichier OMX) pour la découpeuse jet d'eau ProtoMAX.

On avance par étapes : d'abord l'interface et le dessin (v0.1), ensuite la découpe et les spécificités du jet d'eau (v0.2).

**Avancement : 8 livrés sur 14** (57 %)

## Restant à faire

1. **[R-011 Ordre de coupe et aperçu du parcours](roadmap/items/R-011-ordre-de-coupe-et-apercu-du-parcours.md)** · Idée · jalon v0.2
   Outil Parcours : point de départ (origine, point choisi ou coordonnées) et retour, ordre automatique (contenu avant contenant, plus proche voisin) retouchable à la main, déplacements jet coupé signalés au-dessus de ce qui est découpé, bilan chiffré et lecture pas à pas.
2. **[R-012 Comprendre le format OMX](roadmap/items/R-012-comprendre-le-format-omx.md)** · Idée · jalon v0.2
   Décrire et reproduire le format OMX de la ProtoMAX à partir de fichiers réels produits par LAYOUT, avec des tests qui relisent ces fichiers.
3. **[R-007 Reconnaître des arcs dans les segments](roadmap/items/R-007-reconnaitre-des-arcs-dans-les-segments.md)** · Idée · jalon v0.2
   Remplacer les suites de petits segments issues des courbes converties par de vrais arcs, pour un parcours de jet d'eau plus fluide.
4. **[R-013 Exporter l'OMX](roadmap/items/R-013-exporter-l-omx.md)** · Idée · jalon v0.2
   Écrire le fichier OMX du parcours, prêt à ouvrir dans MAKE ; la compensation de la saignée reste à MAKE, qui reçoit le côté de coupe.
5. **[R-014 Gravure](roadmap/items/R-014-gravure.md)** · Idée · jalon v0.2
   Marquer un trait sans traverser la plaque (gravure), dès que son code OMX est connu par un exemple de LAYOUT.
6. **[R-006 Logo de FabOMX](roadmap/items/R-006-logo-de-fabomx.md)** · Idée · jalon v1.0
   Logo définitif de la famille Fab (tuile graphite et symbole à la couleur d'identité), à la place du logo provisoire du kit.

## Actions à faire

- [ ] [A-015](roadmap/actions/A-015-fournir-un-omx-de-layout-avec-un-trait-grave.md) Fournir un OMX de LAYOUT avec un trait gravé (fournir, R-014)

## Livré

### v0.1 : Interface et dessin

- [R-005 Application de bureau macOS](roadmap/items/R-005-application-de-bureau-macos.md) · livrée le 2026-11-18
  Application de bureau macOS (Tauri) : .app et .dmg, dialogue Ouvrir natif, fichiers récents, menu natif FR/EN ; mêmes fonctions que la version web.
- [R-004 Nettoyer le dessin](roadmap/items/R-004-nettoyer-le-dessin.md) · livrée le 2026-11-18
  Section Nettoyer : doublons, superpositions, minuscules, jonctions sous un seuil, segments alignés, avec comptes et bilan en direct, liste des défauts et option à l'import ; rien n'est inventé.
- [R-003 Poser l'interface](roadmap/items/R-003-poser-l-interface.md) · livrée le 2026-11-18
  Squelette de l'application suivant fab-design-system : palette d'outils en icônes dans le panneau de gauche, paramètres et formulaires dans le panneau de droite, dessin au centre.
- [R-002 Redimensionner, déplacer et tourner le dessin](roadmap/items/R-002-redimensionner-deplacer-tourner-et-nettoyer-le.md) · livrée le 2026-11-18
  Déplacer, tourner, mettre à l'échelle (proportionnelle ou non) et retourner un dessin, à la souris et par valeurs, autour d'un point de référence choisi ; calcul depuis la géométrie d'origine, débord signalé.
- [R-001 Importer un DXF ou un SVG](roadmap/items/R-001-importer-un-dxf-ou-un-svg.md) · livrée le 2026-11-18
  Importer un dessin SVG ou DXF par un dialogue qui montre l'unité, les dimensions, les éléments, ce qu'il faudra nettoyer et les avertissements ; plusieurs dessins centrés sur le plateau, sélection, retrait, annuler.

### v0.2 : Découpe jet d'eau

- [R-010 Amorces et perçages](roadmap/items/R-010-amorces-et-percages.md) · livrée le 2026-11-18
  Amorces d'entrée et de sortie (droites ou en arc, longueurs par défaut ou par contour) et point de perçage dans la chute, placées sur un coin ou à la main ; petit trou percé au centre, amorce raccourcie ou signalée quand elle ne tient pas.
- [R-009 Qualité de coupe](roadmap/items/R-009-qualite-de-coupe.md) · livrée le 2026-11-18
  Qualité de coupe de 1 à 5 : une qualité par défaut mémorisée sur le poste, et une qualité propre pour les contours choisis dans l'outil Découpe, affichée près de leur flèche.
- [R-008 Côtés de coupe](roadmap/items/R-008-cotes-de-coupe.md) · livrée le 2026-11-18
  Outil Découpe : pour chaque contour, Extérieur, Trou, Sur la ligne (fente) ou Ne pas couper (repère), détecté par imbrication et d'après le fichier (pointillés, calques), modifiable, avec sens de parcours fixé et croisements signalés.
