---
id: R-019
titre: "Version 1.0"
resume: "Version 1.0.0 : CHANGELOG, guide d'utilisation illustré dans le README, raccourcis Ctrl sous Windows, releases Mac (Apple Silicon) et Windows construites par GitHub Actions, sans signature payante."
statut: en-cours
jalon: v1.0
cree: 2026-11-18
maj: 2026-11-18
livree:
---

## But

FabOMX fait ce pour quoi il a été conçu, du dessin à la pièce coupée sur la ProtoMAX. La version 1.0 le rend utilisable par d'autres :
- une version numérotée 1.0.0 ;
- des notes qui disent ce qu'elle contient ;
- un guide qui explique le parcours complet ;
- des applications à télécharger pour Mac et pour Windows, construites automatiquement à partir du dépôt public.

## Inclus

- **Version 1.0.0** partout : moteur, interface (barre d'état), application de bureau, `CHANGELOG.md`.
- **`CHANGELOG.md`** : ce que contient la 1.0, par grandes fonctions, de l'import à l'export OMX, avec les limites connues (gravure, plaque dans un coin du stock).
- **Guide d'utilisation dans le README** :
  - **installation** sur Mac et sur Windows. Les applications ne portent pas de signature payante : sur Mac, clic droit > Ouvrir la première fois ; sur Windows, « Informations complémentaires > Exécuter quand même » ;
  - **le parcours complet**, en quelques étapes illustrées de captures d'écran : importer, nettoyer, régler la découpe, vérifier le parcours, exporter, placer la buse au-dessus du point de départ et lancer dans MAKE ;
  - **enregistrer un travail** (R-018) ;
  - les captures sont dans `docs/images/`.
- **Windows** :
  - les raccourcis affichés dans l'aide, les infobulles et les menus disent Ctrl au lieu de ⌘ ;
  - le menu de l'application est la barre de menus de la fenêtre, sans les entrées propres à macOS ;
  - les chemins de fichiers récents et les messages fonctionnent avec les chemins Windows.
- **Releases construites par GitHub Actions** :
  - un flux `.github/workflows/release.yml`, lancé par une étiquette `v*`, construit l'application Mac (Apple Silicon, `.dmg`) et l'application Windows (installateur `.exe`) ;
  - il crée un brouillon de release GitHub avec les deux fichiers, que l'utilisateur publie.
- **Vérifications continues** : un flux `.github/workflows/ci.yml` lance les tests, clippy, fmt et `npm run check` à chaque poussée sur `main`.
- La release **v1.0.0** est préparée : l'étiquette, le brouillon et les notes reprises du CHANGELOG.

## Hors périmètre

- Signature et notarisation payantes (Apple Developer, certificat Windows).
- Application Mac universelle (Intel) et application Linux.
- Version web en ligne.
- Mise à jour automatique de l'application.

## Dépendances

- R-018 (Enregistrer et rouvrir un travail), décrite par le guide.
- Le dépôt public `harry-finch/FabOMX`, où les flux s'exécutent.

## Critères d'acceptation

- La barre d'état, l'application Mac et le paquet affichent 1.0.0.
- Les tests, clippy, fmt et `npm run check` passent en local, et le flux `ci.yml` est vert sur GitHub.
- Le flux `release.yml`, lancé par l'étiquette `v1.0.0`, est vert. Le brouillon de release contient un `.dmg` Mac et un installateur Windows, avec les notes de la 1.0.
- Dans le navigateur, avec un agent utilisateur Windows simulé, l'aide affiche Ctrl ; sur Mac, ⌘.
- Le README se lit d'un bout à l'autre comme un guide, avec ses captures, et ses liens fonctionnent sur GitHub.

## Vérifications de l'utilisateur

- Installer l'application Windows de la release sur un PC, faire un parcours complet et ouvrir l'OMX dans MAKE.
- Installer l'application Mac depuis la release (et non depuis la construction locale) et la lancer.
- Relire le guide d'utilisation et le CHANGELOG, puis publier la release.
