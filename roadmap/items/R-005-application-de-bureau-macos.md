---
id: R-005
titre: "Application de bureau macOS"
resume: "Application de bureau macOS (Tauri) : .app et .dmg, dialogue Ouvrir natif, fichiers récents, menu natif FR/EN ; mêmes fonctions que la version web."
statut: a-tester
jalon: v0.1
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

Lancer FabOMX comme une application du Mac, sans terminal ni navigateur : une icône dans le Dock, les menus habituels, et les derniers dessins à portée de main. L'application fait exactement ce que fait la version web, au même niveau, avec la même interface.

## Inclus

- **Application** : une application de bureau Tauri, comme FabVents et FabWeave.
  - Elle contient l'interface et son moteur WebAssembly, et fonctionne sans réseau.
  - Fenêtre de 1440 × 900, 1000 px de large au minimum.
  - Préférences (thème, langue, unités, outil, réglages de nettoyage) gardées d'un lancement à l'autre.
- **Paquet** :
  - `cd ui && npm run desktop` produit un `.app` et un `.dmg` dans `dist-desktop/`, version 0.1.0 ;
  - signature locale (ad hoc), comme FabVents ;
  - icône tirée du logo provisoire ; R-006 la remplacera.
- **Import natif** : le bouton Importer, I et ⌘O ouvrent le dialogue Ouvrir du Mac, filtré sur SVG et DXF. Un fichier déposé depuis le Finder ouvre le dialogue d'import, comme dans la version web.
- **Fichiers récents** :
  - les 10 derniers dessins importés, du plus récent au plus ancien, gardés d'un lancement à l'autre ;
  - ils sont dans Fichier > Importer un fichier récent, avec « Effacer la liste » ;
  - un fichier qui n'existe plus le dit et sort de la liste.
- **Menu natif macOS** :
  - FabOMX : À propos, Masquer, Quitter ;
  - Fichier : Importer… (⌘O), Importer un fichier récent, Fermer la fenêtre ;
  - Édition : Annuler (⌘Z), Rétablir (⇧⌘Z), Couper, Copier, Coller, Tout sélectionner. Annuler et Rétablir agissent sur le dessin, sauf dans un champ de saisie, où ils agissent sur le texte ;
  - Présentation : Recadrer sur le plateau (F), Plein écran ;
  - Fenêtre ;
  - Aide : Raccourcis (?).

  Les menus sont en français ou en anglais selon la langue de l'application. Un raccourci du menu ne déclenche jamais son action deux fois.
- `README.md` (installer et lancer l'application) et `AGENTS.md` (commande `npm run desktop`) mis à jour ; le `.dmg` déclaré dans FabBoard comme build à tester.

## Hors périmètre

- Windows et Linux.
- Signature avec un certificat Apple et notarisation.
- Mises à jour automatiques.
- « Ouvrir avec FabOMX » depuis le Finder (association des fichiers SVG et DXF).
- Logo et icône définitifs (R-006).
- Enregistrer et ouvrir un document de projet.

## Dépendances

- R-003, R-001, R-002, R-004 (livrés).
- Outils de construction présents sur le poste : Rust, Node.js, wasm-pack, Xcode Command Line Tools.

## Critères d'acceptation

- `cargo test --workspace`, `cargo clippy --workspace --all-targets` et `cd ui && npm run check` passent sans erreur ni avertissement. Le crate de bureau est hors du workspace, comme dans FabWeave.
- `cd ui && npm run desktop` produit `dist-desktop/FabOMX_0.1.0_aarch64.dmg`.
- L'application lancée montre le moteur prêt et sa version dans la barre d'état. Importer, transformer et nettoyer un dessin s'y font comme dans la version web.
- ⌘O, le bouton Importer et le menu Fichier > Importer… ouvrent le dialogue natif. Un fichier choisi arrive dans le dialogue d'import.
- **Fichiers récents** :
  - après trois imports, Fichier > Importer un fichier récent les liste, le dernier en tête ;
  - la liste est retrouvée après avoir quitté et relancé l'application ;
  - un fichier déplacé entre-temps affiche un message et sort de la liste.
- **Menus** : Édition > Annuler annule une transformation du dessin. Dans un champ numérique, ⌘Z annule la saisie et pas le dessin. Un appui sur ⌘Z n'annule qu'une étape.
- Les menus passent en anglais quand la langue de l'application est l'anglais.

## Vérifications de l'utilisateur

- Installer le `.dmg`, ouvrir l'application (la première fois, clic droit > Ouvrir, à cause de la signature locale), et faire un parcours complet : importer, transformer, nettoyer.
- Juger les menus et les fichiers récents.
