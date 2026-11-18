---
id: A-004
type: tester
titre: "Importer tes propres dessins dans FabOMX"
element: R-001
statut: a-faire
cree: 2026-11-18
par: agent
fait:
---

Où : branche r-001-import. Lancer « cd ui && npm install && npm run wasm && npm run dev », puis ouvrir http://localhost:5177 (bouton « Version web » de FabBoard).
Quoi faire : importer quelques-uns de tes fichiers SVG et DXF (Inkscape, Fusion, Illustrator…) par le bouton Importer de la palette, ⌘O ou en les déposant sur la fenêtre.
Ce qu'on regarde : l'unité (lue ou supposée) et les dimensions correspondent-elles au dessin réel ? La liste « À nettoyer » et les avertissements sont-ils justes ? Les arcs et cercles sont-ils bien ronds sur le plateau ?
Répondre : « ça marche », ou pour chaque fichier faux, ce qui ne va pas (et joindre le fichier si possible).
