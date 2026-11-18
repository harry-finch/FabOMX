---
id: R-011
titre: "Ordre de coupe et aperçu du parcours"
resume: "Calculer l'ordre de coupe (trous avant l'extérieur, déplacements courts) et montrer le parcours complet : coupes, déplacements jet coupé, perçages."
statut: idee
jalon: v0.2
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## Notes pour la spécification

- **Point de départ choisi** (demande de l'utilisateur, 18/11/2026) : pas forcément le zéro du plateau. Exemple : une plaque déjà aux bonnes dimensions où l'on veut seulement percer un trou à un endroit précis. On pourrait prendre un coin de la plaque, un point du dessin ou des coordonnées. Hypothèse à vérifier sur la machine : MAKE pose le premier point de l'OMX (déplacement jet coupé, Q = 0) à la position de la buse au lancement. L'opérateur amène alors la buse sur le point choisi de la plaque réelle.
- Les déplacements jet coupé s'écrivent en qualité 0, sans côté (exemples `omx/`).
- À trancher : éviter de passer au-dessus d'une pièce ou d'un trou déjà découpés, où une chute peut s'être soulevée.
