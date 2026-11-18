---
id: R-007
titre: "Reconnaître des arcs"
resume: "Courbes de Bézier, splines et ellipses lues en arcs tangents à l'import ; opération « Reconnaître des arcs » au nettoyage pour les polylignes en segments ; coins vifs, lignes droites et petits détails intacts ; arcs écrits en Bow dans l'OMX."
statut: livree
jalon: v0.2
cree: 2026-11-18
maj: 2026-11-18
livree: 2026-11-18
---

## But

Une courbe faite de centaines de petits segments donne un OMX de centaines de lignes, et la ProtoMAX ralentit à chaque raccord. Les courbes de Bézier, les splines et les ellipses du dessin, comme les polylignes déjà découpées en segments par certains logiciels, doivent devenir de vrais arcs. Le fichier est alors plus court et le mouvement plus fluide, à la même précision. Les coins vifs, les lignes droites et les petits détails du dessin restent tels quels.

## Inclus

- **À l'import**, sans action de l'utilisateur :
  - les courbes de Bézier (SVG), les splines (DXF) et les ellipses (SVG, DXF) sont lues en arcs tangents qui se raccordent sans cassure (biarcs), à 0,01 mm près (`CURVE_TOL`) ;
  - de même pour les cercles et arcs déformés par une échelle non proportionnelle ;
  - les types comptés au dialogue d'import ne changent pas (Bézier, spline, ellipse).
- **Au nettoyage**, une sixième opération de l'outil Nettoyer : « Reconnaître des arcs ».
  - Elle remplace les suites de segments d'une polyligne qui suivent un arc par cet arc.
  - Elle est cochée par défaut, avec son nombre d'arcs créés en direct, comme les autres opérations.
  - Son seuil « Écart max » vaut 0,01 mm par défaut, réglable de 0,001 à 0,1 mm, en mm du plateau. Il est mémorisé avec les autres réglages.
  - Elle s'applique aussi avec « Nettoyer à l'import », et reste une étape d'annulation comme tout nettoyage.
- **Ce qui reste intact** :
  - **coins vifs** : un changement de direction de plus de 30° entre deux segments n'est jamais arrondi, les arcs s'arrêtent aux coins ;
  - **lignes droites** : des segments alignés à l'écart près restent une ligne, jamais un arc de très grand rayon ;
  - **petits détails** : pas d'arc de rayon inférieur à 0,1 mm, un tel détail reste en segments ;
  - les lignes, arcs et cercles du fichier, déjà exacts.
- Dans l'OMX, ces arcs s'écrivent avec leur `Bow`, comme ceux du dessin (R-012).
- Aide et documentation mises à jour.

## Hors périmètre

- Remplacer des arcs par d'autres arcs, ou fusionner des arcs voisins d'un même cercle (déjà fait par « Fusionner les superpositions »).
- Arcs dans les amorces (déjà exacts) et dans les déplacements jet coupé (des lignes).
- Lisser un dessin en arrondissant ses coins.
- Reconnaître des arcs dans les repères Ne pas couper : ils ne sont pas coupés.

## Dépendances

- R-001, R-004 et R-012 (livrés).

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Tests du moteur :
  - **Cercle en Bézier** : un cercle SVG tracé en 4 courbes de Bézier est lu en 8 arcs au plus, sans ligne, à 0,01 mm du vrai cercle, en un contour fermé.
  - **Courbe en S** : une Bézier en S donne des arcs qui se raccordent tangents (écart d'angle inférieur à 0,1°) et restent à 0,01 mm de la courbe.
  - **Spline et ellipse DXF** : elles sont lues en arcs, à 0,01 mm près.
  - **Échelle non proportionnelle** : un cercle étiré s'écrit en arcs dans l'OMX, et non plus en segments.
  - **Polyligne en cercle** : une polyligne de 72 segments sur un cercle de rayon 20 mm devient au plus 2 arcs au nettoyage, comptés.
  - **Rectangle arrondi** en polyligne : ses côtés restent des lignes et ses coins deviennent des arcs.
  - **Coins vifs** : un carré et un octogone en polyligne ne changent pas.
  - **Petit détail** : une polyligne qui suit un arc de rayon 0,05 mm reste en segments.
  - **Échelle** : au nettoyage, l'écart max suit l'échelle du dessin, comme les autres seuils.
  - **OMX** : un cercle en Bézier s'écrit en moins de 20 lignes, avec des `Bow`.
- Dans le navigateur :
  - « Reconnaître des arcs » affiche son compte en direct et change le compte quand on change le seuil ;
  - après le nettoyage, le bilan du parcours et l'OMX ont moins de lignes ;
  - le seuil est retrouvé après un rechargement.
- Textes en français et en anglais.

## Vérifications de l'utilisateur

- Importer deux ou trois de tes fichiers avec des courbes (Bézier, splines, polylignes de CAO), les nettoyer, et vérifier que la forme n'a pas bougé.
- Couper ou simuler dans MAKE un OMX avec des courbes, et dire si le mouvement est plus fluide.
