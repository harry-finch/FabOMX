---
id: R-017
titre: "Choisir le côté de coupe d'un tracé ouvert"
resume: "Pour un tracé ouvert, pouvoir garder la coupe sur la ligne ou décaler la saignée d'un côté choisi, par exemple pour couper précisément une plaque en deux."
statut: idee
jalon: v0.3
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## Notes pour la spécification

- Demande de l'utilisateur (18/11/2026) : une fente reste sur la ligne par défaut (OffsetSide 0, confirmé à la coupe, A-023), mais on doit pouvoir choisir le côté de la pièce, par exemple pour couper une plaque en deux à la cote.
- Piste : dans l'OMX, le décalage s'écrit avec OffsetSide = 1, la pièce à droite du jet (exemples LAYOUT). Pour garder la pièce de l'autre côté, il suffit de parcourir le tracé dans l'autre sens, sans nouveau code à deviner.
- À trancher : comment montrer et choisir le côté sur le plateau (une flèche, un clic du côté de la pièce), et les amorces d'un tracé ouvert décalé.
