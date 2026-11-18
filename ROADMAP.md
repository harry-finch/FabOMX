<!-- Généré par FabBoard depuis roadmap/ : ne pas modifier à la main. -->

Objectif de FabOMX : à partir d'un dessin vectoriel (SVG ou DXF), produire un parcours d'outil (fichier OMX) pour la découpeuse jet d'eau ProtoMAX.

On avance par étapes : d'abord l'interface et le dessin (v0.1), ensuite la découpe et les spécificités du jet d'eau (v0.2).

**Avancement : 15 livrés sur 19** (79 %)

## Restant à faire

1. **[R-019 Version 1.0](roadmap/items/R-019-version-1-0.md)** · À tester · jalon v1.0
   Version 1.0.0 : CHANGELOG, guide d'utilisation illustré dans le README, raccourcis Ctrl sous Windows, releases Mac (Apple Silicon) et Windows construites par GitHub Actions, sans signature payante.
2. **[R-014 Gravure](roadmap/items/R-014-gravure.md)** · Idée
   Marquer un trait sans traverser la plaque (gravure), dès que son code OMX est connu par un exemple de LAYOUT.
3. **[R-016 Couper une plaque dans un coin du stock](roadmap/items/R-016-couper-une-plaque-dans-un-coin-du-stock.md)** · Idée · jalon v0.3
   Couper une plaque dans un coin du stock sans faire le tour complet. À spécifier.
4. **[R-020 Avertissement géométrie trop petite](roadmap/items/R-020-avertissement-geometrie-trop-petite.md)** · Idée
   Emettre un avertissement quand des trous ou des caractèristiques géométriques sont trop petites pour le kerf du jet d’eau (~1mm).

## Actions à faire

- [ ] [A-015](roadmap/actions/A-015-fournir-un-omx-de-layout-avec-un-trait-grave.md) Fournir un OMX de LAYOUT avec un trait gravé (fournir, R-014)
- [ ] [A-034](roadmap/actions/A-034-installer-et-essayer-fabomx-sous-windows.md) Installer et essayer FabOMX sous Windows (tester, R-019)
- [ ] [A-035](roadmap/actions/A-035-installer-fabomx-pour-mac-depuis-la-release.md) Installer FabOMX pour Mac depuis la release (tester, R-019)
- [ ] [A-036](roadmap/actions/A-036-relire-le-guide-le-changelog-et-publier-la.md) Relire le guide, le CHANGELOG et publier la release v1.0.0 (valider, R-019)

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

- [R-012 Exporter l'OMX](roadmap/items/R-012-comprendre-le-format-omx.md) · livrée le 2026-11-18
  Écrire le parcours en OMX à l'identique de LAYOUT (pouces, Q, côté, arcs exacts en Bow), avec la pièce à droite du jet comme la machine ; bouton d'export dans l'outil Parcours, dialogue Enregistrer dans l'application Mac, remarques listées avant l'export.
- [R-011 Ordre de coupe et aperçu du parcours](roadmap/items/R-011-ordre-de-coupe-et-apercu-du-parcours.md) · livrée le 2026-11-18
  Outil Parcours : point de départ (origine, point choisi ou coordonnées) et retour, ordre automatique (contenu avant contenant, plus proche voisin) retouchable à la main, déplacements jet coupé signalés au-dessus de ce qui est découpé, bilan chiffré et lecture pas à pas.
- [R-010 Amorces et perçages](roadmap/items/R-010-amorces-et-percages.md) · livrée le 2026-11-18
  Amorces d'entrée et de sortie (droites ou en arc, longueurs par défaut ou par contour) et point de perçage dans la chute, placées sur un coin ou à la main ; petit trou percé au centre, amorce raccourcie ou signalée quand elle ne tient pas.
- [R-009 Qualité de coupe](roadmap/items/R-009-qualite-de-coupe.md) · livrée le 2026-11-18
  Qualité de coupe de 1 à 5 : une qualité par défaut mémorisée sur le poste, et une qualité propre pour les contours choisis dans l'outil Découpe, affichée près de leur flèche.
- [R-008 Côtés de coupe](roadmap/items/R-008-cotes-de-coupe.md) · livrée le 2026-11-18
  Outil Découpe : pour chaque contour, Extérieur, Trou, Sur la ligne (fente) ou Ne pas couper (repère), détecté par imbrication et d'après le fichier (pointillés, calques), modifiable, avec sens de parcours fixé et croisements signalés.
- [R-007 Reconnaître des arcs](roadmap/items/R-007-reconnaitre-des-arcs-dans-les-segments.md) · livrée le 2026-11-18
  Courbes de Bézier, splines et ellipses lues en arcs tangents à l'import ; opération « Reconnaître des arcs » au nettoyage pour les polylignes en segments ; coins vifs, lignes droites et petits détails intacts ; arcs écrits en Bow dans l'OMX.

### v0.3 : Découpes avancées

- [R-017 Choisir le côté de coupe d'un tracé ouvert](roadmap/items/R-017-choisir-le-cote-de-coupe-d-un-trace-ouvert.md) · livrée le 2026-11-18
  Pour un tracé ouvert : sur la ligne, pièce d'un côté ou de l'autre (saignée vers la chute, côté 1 dans l'OMX, sens imposé), avec hachures sur le plateau ; amorces au choix dans le prolongement du tracé pour percer hors de la plaque.
- [R-015 Onglets de maintien](roadmap/items/R-015-onglets-de-maintien.md) · livrée le 2026-11-18
  Onglets sur les contours extérieurs choisis : onglet de fin sans perçage, onglets le long du contour avec arrêt du jet et reperçage dans la chute ; nombre réparti ou placés au clic ; largeur 1 mm par défaut, réglable.

### v1.0 : Version 1

- [R-018 Enregistrer et rouvrir un travail](roadmap/items/R-018-enregistrer-et-rouvrir-un-travail.md) · livrée le 2026-11-18
  Fichier de travail .fabomx autonome (géométrie, placements, réglages faits à la main, point de départ, ordre) : Nouveau, Ouvrir, Enregistrer, Enregistrer sous, récents, dépôt ; repère « modifié » et demande avant de perdre des changements.
- [R-006 Logo de FabOMX](roadmap/items/R-006-logo-de-fabomx.md) · livrée le 2026-11-18
  Logo définitif : buse, jet d'eau qui frappe la tôle et éclabousse en gouttes (variante A « Gerbe » de la planche), symbole en #2D22BF dans la tuile de la famille ; barre d'application, onglet et icône de l'application Mac.

## Écarté

- [R-013 Exporter l'OMX](roadmap/items/R-013-exporter-l-omx.md)
  Fusionné dans R-012 à la demande de l'utilisateur (18/11/2026) : l'export de l'OMX se fait avec le format.
