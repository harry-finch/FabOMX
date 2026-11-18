---
id: A-012
type: fournir
titre: "Fournir des fichiers OMX de LAYOUT avec leur DXF"
element: R-012
statut: a-faire
cree: 2026-11-18
par: agent
fait:
---

Pour reproduire exactement le format OMX de la ProtoMAX, j'ai besoin de fichiers réels produits par LAYOUT, chacun avec le DXF dont il vient.

Dépose-les dans `crates/fabomx-core/tests/fixtures/omx/` (ou dis-moi où ils sont). Idéalement, des cas simples, un par fichier :

1. Un carré de 50 mm coupé à l'extérieur, qualité 3.
2. Le même carré avec un trou rond de 20 mm au centre.
3. Une ligne ouverte (fente, sans compensation).
4. Une pièce avec amorces d'entrée et de sortie réglées à la main.
5. Le même contour en qualités 1 et 5, et en gravure.
6. Si possible, un fichier avec une pause ou un délai (XData).

Réponds en disant où sont les fichiers, et la version de LAYOUT utilisée.
