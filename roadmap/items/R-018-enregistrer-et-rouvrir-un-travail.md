---
id: R-018
titre: "Enregistrer et rouvrir un travail"
resume: "Fichier de travail .fabomx autonome (géométrie, placements, réglages faits à la main, point de départ, ordre) : Nouveau, Ouvrir, Enregistrer, Enregistrer sous, récents, dépôt ; repère « modifié » et demande avant de perdre des changements."
statut: a-tester
jalon: v1.0
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

Aujourd'hui, tout est perdu à la fermeture : les dessins, leur placement, les côtés, qualités, amorces et onglets réglés à la main, le point de départ et l'ordre. Pour reprendre une découpe le lendemain, ou la refaire un mois plus tard, l'utilisateur doit pouvoir enregistrer son travail dans un fichier et le rouvrir tel quel. Le fichier doit se suffire à lui-même, même si les SVG ou DXF d'origine ont bougé.

## Inclus

- **Fichier de travail `.fabomx`**, autonome, en JSON lisible :
  - la géométrie de chaque dessin telle que le moteur la garde, nettoyages compris, avec son nom, son format et son unité ;
  - sa matrice et son point de référence ;
  - ses réglages faits à la main : côtés, qualités, amorces, points d'entrée, onglets, tracés ouverts ;
  - le point de départ et l'ordre de coupe ;
  - la version du format et celle de FabOMX ;
  - les réglages par défaut du poste au moment de l'enregistrement (qualité, amorces, largeur d'onglet).
- **Commandes**, dans le menu Fichier de l'application, au clavier et dans l'aide :
  - **Nouveau travail** (⌘N) ;
  - **Ouvrir un travail…**, ou ⌘O : le dialogue Ouvrir accepte SVG, DXF et FABOMX, et un `.fabomx` s'ouvre comme un travail ;
  - **Enregistrer** (⌘S) ;
  - **Enregistrer sous…** (⇧⌘S).

  Le dépôt d'un `.fabomx` sur la fenêtre l'ouvre aussi, et les travaux entrent dans les fichiers récents. Dans le navigateur, Enregistrer télécharge le fichier et Ouvrir passe par le sélecteur de fichier.
- **Modifications non enregistrées** :
  - le titre de la fenêtre porte le nom du travail et un repère « modifié » ;
  - ouvrir un autre travail, en commencer un nouveau ou fermer l'application demande d'abord « Enregistrer, Ne pas enregistrer, Annuler » ;
  - dans le navigateur, une alerte du navigateur prévient avant de quitter.
- **À l'ouverture** :
  - le plateau, les réglages et le parcours sont ceux de l'enregistrement, et l'historique d'annulation repart de zéro ;
  - si les réglages par défaut du fichier diffèrent de ceux du poste, un avis le dit et propose « Reprendre les réglages du travail ». Rien n'est changé en silence.
- **Refus, avec un code stable et un message** :
  - un fichier illisible ou d'un autre format (`job.unreadable`) ;
  - un fichier d'une version plus récente de FabOMX (`job.newer`).
- Aide, README et documentation mis à jour.

## Hors périmètre

- Enregistrement automatique et reprise après un plantage.
- Plusieurs travaux ouverts à la fois (onglets ou fenêtres).
- Lien vers les fichiers d'origine, ou leur mise à jour quand ils changent.
- Garder l'historique d'annulation dans le fichier.

## Dépendances

- R-001 à R-017 (livrés).

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check`, `cd ui && npm run check` et `cd crates/fabomx-desktop && cargo clippy --release` passent sans erreur ni avertissement.
- Tests du moteur :
  - une géométrie exportée puis réimportée donne les mêmes contours, côtés détectés et rapport ;
  - un dessin nettoyé garde sa géométrie nettoyée ;
  - un fichier illisible donne `job.unreadable`, une version future `job.newer`.
- Dans le navigateur :
  - deux dessins placés, nettoyés et réglés (un côté, une qualité, des amorces, des onglets, un tracé décalé, un point de départ, un ordre à la main) sont enregistrés, puis rouverts après un rechargement : le plan et l'OMX exporté sont **identiques** octet pour octet (hors date) ;
  - un changement après l'enregistrement met le repère « modifié », et Nouveau travail demande d'enregistrer ;
  - l'avis des réglages par défaut apparaît quand ils diffèrent.
- Textes en français et en anglais.

## Vérifications de l'utilisateur

- Enregistrer un vrai travail dans l'application Mac, quitter, relancer, le rouvrir depuis les fichiers récents, et vérifier que tout est revenu, OMX compris.
