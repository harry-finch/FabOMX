---
id: A-026
type: tester
titre: "Vérifier les arcs sur tes fichiers à courbes"
element: R-007
statut: a-faire
cree: 2026-11-18
par: agent
fait:
---

Où : `dist-desktop/FabOMX.app` (à lancer sur place) ou la version web (`cd ui && npm run dev`, http://localhost:5177).

Gestes : importe deux ou trois de tes fichiers avec des courbes (Bézier en SVG, splines DXF, polylignes de CAO). Outil Nettoyer (C) : regarde le compte de « Reconnaître des arcs », change son écart max, puis Nettoyer.

À observer : la forme n'a pas bougé (coins vifs, lignes droites et petits détails intacts) ; dans l'outil Parcours, l'OMX exporté est nettement plus court.

Réponds « ça marche », ou quel fichier pose problème.
