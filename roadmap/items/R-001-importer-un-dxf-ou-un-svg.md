---
id: R-001
titre: "Importer un DXF ou un SVG"
resume: "Importer un dessin SVG ou DXF par un dialogue qui montre l'unité, les dimensions, les éléments, ce qu'il faudra nettoyer et les avertissements ; plusieurs dessins centrés sur le plateau, sélection, retrait, annuler."
statut: a-tester
jalon: v0.1
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

Faire entrer un dessin SVG ou DXF dans FabOMX en sachant tout de suite ce qu'on récupère : dans quelle unité, à quelle taille, et s'il est bon pour la découpe. Le dialogue d'import montre l'état du dessin avant de le poser sur le plateau. Il signale ce qu'il faudra nettoyer, mais ne le répare pas : c'est R-004.

## Inclus

- **Ouvrir** : le bouton Importer de la palette (raccourci I), ⌘O, ou un fichier déposé sur l'espace de travail. Importer est une action, pas un outil : il ouvre le choix du fichier sans changer l'outil actif.
- **Formats** :
  - SVG : `path`, `rect`, `circle`, `ellipse`, `line`, `polyline`, `polygon`, transformations et groupes compris.
  - DXF : `LINE`, `ARC`, `CIRCLE`, `LWPOLYLINE` et `POLYLINE` (arrondis compris), `SPLINE`, `ELLIPSE`.
  - Lignes, arcs et cercles restent exacts. Les courbes de Bézier et les splines sont converties en segments, à 0,01 mm près.
- **Dialogue d'import**, avant que rien ne soit posé :
  - nom du fichier et format ;
  - aperçu du dessin ;
  - **unité** lue dans le fichier (`$INSUNITS`, `width`/`height` du SVG) ou supposée, avec sa source écrite. Un sélecteur mm / cm / pouces / px (96 par pouce) permet de la changer ;
  - **dimensions** de la boîte englobante, recalculées en direct quand l'unité change ;
  - **éléments** comptés par type, avec les contours fermés et ouverts.
- **À nettoyer**, compté et listé dans le dialogue, et marqué dans l'aperçu :
  - contours ouverts ;
  - éléments en double (superposés) ;
  - extrémités à moins de 0,01 mm l'une de l'autre et non jointes ;
  - éléments de moins de 0,1 mm.
- **Avertissements** :
  - aucun contour fermé ;
  - dessin plus grand que le plateau (305 × 305 mm) ;
  - éléments ignorés (textes, images, hachures, cotes, blocs), comptés par type.
- **Refus** : un fichier illisible, ou sans aucun élément de dessin, affiche la raison, et Importer reste grisé. Les messages ont un code stable et sont traduits.
- **Plusieurs dessins** :
  - chaque import s'ajoute aux dessins déjà présents, centré sur le plateau ;
  - sur le plateau, un dessin a le style des objets de l'utilisateur, et ses contours ouverts sont en orange.
- **Sélection** : l'outil Sélection choisit un dessin d'un clic, et Échap ou un clic dans le vide le désélectionne. Le panneau droit montre alors les informations du dialogue, en lecture seule, avec un bouton Retirer (touche Suppr).
- **Annuler / rétablir** (⌘Z / ⇧⌘Z) pour un import et un retrait.
- Aide (raccourcis) et documentation mises à jour, avec des fichiers d'exemple SVG et DXF pour les tests.

## Hors périmètre

- Réparer le dessin (fermer, dédoublonner, joindre, retirer les éléments minuscules) : R-004, qui ajoutera aussi une case « Nettoyer à l'import » au dialogue.
- Déplacer, tourner, mettre à l'échelle un dessin posé : R-002.
- Calques : tout est importé à plat, sans tenir compte des calques ni de leur visibilité.
- Blocs DXF (`INSERT`), textes et hachures : ignorés, comptés et signalés.
- Retrouver les dessins après un rechargement de la page, et enregistrer un document de projet.
- Autres formats (PDF, AI, DWG).

## Dépendances

- R-003 (livré).

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets` et `cd ui && npm run check` passent sans erreur ni avertissement.
- Des tests du moteur, sur des fichiers d'exemple versionnés, vérifient les points suivants.
  - **Lecture** : un carré de 100 mm en DXF (`$INSUNITS` = mm) et le même carré en SVG (`width="100mm"`) se lisent tous deux 100 × 100 mm, un contour fermé, rien à nettoyer.
  - **Unités** : un DXF sans `$INSUNITS` est lu en mm, « supposée ». Un SVG en px de 378 × 378 se lit en px, soit 100 × 100 mm, et en pouces si on choisit cette unité.
  - **Défauts détectés** :
    - un contour dont les extrémités sont à 0,005 mm compte 1 extrémité à joindre ;
    - un segment répété compte 1 doublon ;
    - une ligne seule donne « aucun contour fermé » ;
    - un dessin de 400 mm donne « plus grand que le plateau » ;
    - un texte et un bloc DXF sont comptés comme ignorés.
  - **Courbes** : un arc reste un arc. Une courbe de Bézier convertie ne s'écarte pas de la courbe de plus de 0,01 mm.
- Un fichier vide ou corrompu affiche sa raison dans le dialogue, Importer est grisé, et rien n'est ajouté.
- Dans le navigateur :
  - deux imports successifs donnent deux dessins centrés sur le plateau ;
  - un clic sur l'un le sélectionne, et le panneau droit montre ses informations ;
  - Suppr le retire, ⌘Z le remet, ⇧⌘Z le retire de nouveau ;
  - le bouton Importer, ⌘O et le dépôt d'un fichier ouvrent le même dialogue ;
  - l'outil actif ne change pas.
- Changer l'unité dans le dialogue met à jour les dimensions sans fermer le dialogue. Le dessin posé a la taille de l'unité choisie.
- Textes du dialogue en français et en anglais ; aucune couleur en dur.

## Vérifications de l'utilisateur

- Importer quelques-uns de tes propres fichiers SVG et DXF (Inkscape, Fusion, Illustrator…) et dire si les dimensions, l'unité et la liste « à nettoyer » sont justes.
- Juger le dialogue d'import : lisibilité, ordre des informations, ce qui manque.
