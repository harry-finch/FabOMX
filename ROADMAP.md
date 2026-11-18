<!-- Généré par FabBoard depuis roadmap/ : ne pas modifier à la main. -->

Objectif de FabOMX : à partir d'un dessin vectoriel (SVG ou DXF), produire un parcours d'outil (fichier OMX) pour la découpeuse jet d'eau ProtoMAX.

On avance par étapes : d'abord l'interface et le dessin (v0.1), ensuite la découpe et les spécificités du jet d'eau (v0.2).

**Avancement : 4 livrés sur 7** (57 %)

## Restant à faire

1. **[R-005 Application de bureau macOS](roadmap/items/R-005-application-de-bureau-macos.md)** · Spécifiée · jalon v0.1
   Application de bureau macOS (Tauri) : .app et .dmg, dialogue Ouvrir natif, fichiers récents, menu natif FR/EN ; mêmes fonctions que la version web.
2. **[R-006 Logo de FabOMX](roadmap/items/R-006-logo-de-fabomx.md)** · Idée · jalon v0.1
   Logo définitif de la famille Fab (tuile graphite et symbole à la couleur d'identité), à la place du logo provisoire du kit.
3. **[R-007 Reconnaître des arcs dans les segments](roadmap/items/R-007-reconnaitre-des-arcs-dans-les-segments.md)** · Idée · jalon v0.2
   Remplacer les suites de petits segments issues des courbes converties par de vrais arcs, pour un parcours de jet d'eau plus fluide.

## Livré

### v0.1 : Interface et dessin

- [R-004 Nettoyer le dessin](roadmap/items/R-004-nettoyer-le-dessin.md) · livrée le 2026-11-18
  Section Nettoyer : doublons, superpositions, minuscules, jonctions sous un seuil, segments alignés, avec comptes et bilan en direct, liste des défauts et option à l'import ; rien n'est inventé.
- [R-003 Poser l'interface](roadmap/items/R-003-poser-l-interface.md) · livrée le 2026-11-18
  Squelette de l'application suivant fab-design-system : palette d'outils en icônes dans le panneau de gauche, paramètres et formulaires dans le panneau de droite, dessin au centre.
- [R-002 Redimensionner, déplacer et tourner le dessin](roadmap/items/R-002-redimensionner-deplacer-tourner-et-nettoyer-le.md) · livrée le 2026-11-18
  Déplacer, tourner, mettre à l'échelle (proportionnelle ou non) et retourner un dessin, à la souris et par valeurs, autour d'un point de référence choisi ; calcul depuis la géométrie d'origine, débord signalé.
- [R-001 Importer un DXF ou un SVG](roadmap/items/R-001-importer-un-dxf-ou-un-svg.md) · livrée le 2026-11-18
  Importer un dessin SVG ou DXF par un dialogue qui montre l'unité, les dimensions, les éléments, ce qu'il faudra nettoyer et les avertissements ; plusieurs dessins centrés sur le plateau, sélection, retrait, annuler.
