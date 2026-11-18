# Journal des versions

## [1.0.0] — 2026-11-18

Première version complète : d’un dessin SVG ou DXF à un fichier OMX que MAKE ouvre et que la ProtoMAX coupe.

### Dessin
- **Import** SVG et DXF : le dialogue montre le dessin, son unité (modifiable), ses dimensions, ce qu’il faudra nettoyer et les avertissements. Lignes, arcs et cercles restent exacts ; les courbes de Bézier, splines et ellipses deviennent des arcs tangents à 0,01 mm près.
- **Placement** sur le plateau de la ProtoMAX (305 × 305 mm) : déplacer, tourner, mettre à l’échelle, retourner, à la souris ou par valeurs, autour d’un point de référence choisi ; plusieurs dessins à la fois.
- **Nettoyage** : doublons, superpositions, éléments minuscules, extrémités à joindre sous un seuil, segments alignés, arcs reconnus dans les polylignes ; comptes en direct, défauts situés, rien n’est inventé.

### Découpe
- **Côtés de coupe** : extérieur, trou, sur la ligne, ne pas couper ; détectés par imbrication, repères reconnus d’après le fichier (pointillés, calques de construction), modifiables. La pièce est à droite du jet, comme sur la machine.
- **Tracés ouverts** coupés sur la ligne ou la pièce d’un côté, avec amorces dans le prolongement.
- **Qualité** de coupe de 1 à 5, par défaut ou par contour.
- **Amorces** droites ou en arc et perçage dans la chute, sur un coin ou à un point choisi ; petit trou percé au centre.
- **Onglets** de maintien : onglet de fin, ou onglets avec reperçage, répartis ou placés au clic.

### Parcours et export
- **Parcours** complet : point de départ (là où l’on place la buse), ordre de coupe automatique (contenu avant contenant, puis plus proche) ou retouché à la main, déplacements au-dessus de ce qui est découpé signalés, bilan, lecture pas à pas.
- **Export OMX** dans le format exact de LAYOUT (pouces, qualités, côtés, arcs exacts), avec les remarques à confirmer avant d’exporter.

### Travail et application
- **Travail** enregistré dans un fichier `.fabomx` autonome et rouvert tel quel.
- **Applications** pour Mac (Apple Silicon) et Windows, en français et en anglais, thème sombre et clair, millimètres ou pouces.

### Limites connues
- Pas de gravure : son code OMX n’est pas encore connu.
- Pas de découpe d’une plaque dans un coin du stock sans en faire le tour.
- Applications sans signature payante : clic droit > Ouvrir la première fois sur Mac, « Exécuter quand même » sous Windows.
