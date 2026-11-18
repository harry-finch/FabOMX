# Illustrations

Figures Hairline de FabOMX, dessinées avec la skill `hairline-create` (moteur `kernel.js`, MIT, © Lucas Marques).

## waterjet

- **Concept retenu** (18/11/2026, choisi par l'utilisateur parmi trois) : une tête de découpe jet d'eau au-dessus d'une tôle posée sur les lattes du bac. Le contour de la pièce est le parcours d'outil, en pointillé, depuis un point de perçage hors de la pièce. Le pointeur, projeté sur la tôle, choisit le point du parcours le plus proche ; la tête y va en suivant le contour, et la saignée déjà coupée (arête vive) la suit. Au repos, la découpe est aux deux tiers.
- **Curseur** : la largeur du trait de coupe (1,6 à 4,6).
- **Lecture** : `cut <n>%`, l'avancement le long du parcours ; `rest` au repos.
- **Emplacement** : carte du projet dans FabBoard (`figure = "waterjet"` dans `roadmap/projet.toml`). Pas encore dans l'application.
- **Fichiers** : `waterjet.js` (la source), `hairline-waterjet.html` (page d'essai autonome ; `?play=1` lance le parcours). Contrôle : `node ../../.agents/skills/hairline-create/look.mjs waterjet.js --answer 58,92,3 --edge 0,120,-14 --edge 150,120,-14`.
