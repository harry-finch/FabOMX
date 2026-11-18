---
id: R-006
titre: "Logo de FabOMX"
resume: "Logo définitif : buse, jet d'eau qui frappe la tôle et éclabousse en gouttes (variante A « Gerbe » de la planche), symbole en #2D22BF dans la tuile de la famille ; barre d'application, onglet et icône de l'application Mac."
statut: livree
jalon: v1.0
cree: 2026-11-18
maj: 2026-11-18
livree: 2026-11-18
---

## But

FabOMX porte encore le symbole provisoire du kit, un simple disque. Son logo doit dire son métier d'un coup d'œil, comme ceux de la famille Fab : une buse dont le jet d'eau frappe la tôle et éclabousse en gouttes. Il remplace le disque partout : la barre d'application, l'onglet du navigateur et l'icône de l'application Mac.

## Inclus

- **Le logo choisi sur la planche** (`branding/planche/planche.html`, 18/11/2026) : la variante **A « Gerbe »**, dans la tuile commune de la famille (graphite `#181f2a`, bord `#354759`) :
  - une buse en haut ;
  - un jet épais ;
  - l'eau qui jaillit de l'impact sur la tôle en quatre langues et six gouttes ;
  - la tôle en bas.
- **Couleur du symbole** : l'identité actuelle `#2D22BF`. C'est le choix de l'utilisateur, en connaissance de cause : le contraste sur la tuile reste sous la règle de la famille (environ 1,4:1 pour 3:1).
- **Fichiers** :
  - `branding/fabomx-logo.svg` (source, formes nommées) ;
  - `branding/README.md` (variante retenue, date, couleurs, écart assumé à la règle de contraste) ;
  - `ui/public/logo.svg` pour la barre d'application et l'onglet du navigateur ;
  - les icônes de l'application Mac régénérées depuis le SVG.
- La planche reste dans `branding/planche/` comme trace du choix.
- La skill `fab-design-system` est mise à jour : FabOMX entre dans le tableau des couleurs de la famille.

## Hors périmètre

- Changer la couleur d'identité ou le liseré : ils restent `#2D22BF` et `#22198f`.
- Figure au trait pour l'À propos ou l'état vide (skill `fab-illustration-system`).
- Site, README illustré, icônes Windows et Linux.

## Dépendances

- Aucune.

## Critères d'acceptation

- `branding/fabomx-logo.svg` et `ui/public/logo.svg` ont la tuile commune et un seul groupe `#symbol` d'une seule couleur, sans dégradé, texte ni ressource externe. Le symbole est centré et tient dans un disque de rayon 50 environ.
- La barre d'application et l'onglet du navigateur montrent le nouveau logo, en thème sombre et clair, sans erreur dans la console.
- `dist-desktop/FabOMX.app` a la nouvelle icône (Finder et Dock).
- `cd ui && npm run check` passe sans erreur ni avertissement.

## Vérifications de l'utilisateur

- Voir le logo dans l'application Mac (Dock, Finder, barre d'application) et dans le navigateur, et dire s'il convient.
