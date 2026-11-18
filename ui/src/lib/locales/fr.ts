// Dictionnaire de référence (français). Toute clé ajoutée ici doit l'être
// aussi dans en.ts : TypeScript le vérifie.
// Typographie : espace insécable avant « : ; ! ? % », guillemets « », apostrophe ’.

const fr = {
  app: {
    tagline: 'Parcours d’outils pour la ProtoMAX',
    units: 'Unités',
    inches: 'pouces',
    language: 'Langue',
    theme: 'Thème',
    themes: { system: 'Thème du système', light: 'Thème clair', dark: 'Thème sombre' },
    help: 'Aide',
  },
  common: {
    close: 'Fermer',
    cancel: 'Annuler',
    apply: 'Appliquer',
  },
  tools: {
    label: 'Outils',
    soon: 'À venir',
    select: { name: 'Sélection', hint: 'Choisir un élément du dessin.' },
    hand: { name: 'Main', hint: 'Glisser pour déplacer la vue. Espace + glisser fait de même avec tout outil.' },
    import: { name: 'Importer', hint: 'Ouvrir un dessin SVG ou DXF.' },
    move: { name: 'Déplacer', hint: 'Déplacer le dessin sur le plateau.' },
    rotate: { name: 'Tourner', hint: 'Faire tourner le dessin.' },
    scale: { name: 'Mettre à l’échelle', hint: 'Agrandir ou réduire le dessin.' },
    clean: { name: 'Nettoyer', hint: 'Fermer les contours, retirer les doublons, joindre les segments.' },
  },
  bed: {
    title: 'Plateau',
    machine: 'Machine',
    width: 'Largeur (X)',
    height: 'Profondeur (Y)',
    origin: 'Origine',
    originValue: 'en bas à gauche',
    empty: 'Importez un dessin pour commencer.',
  },
  workspace: {
    zoom: 'Zoom {v}',
  },
  status: {
    ready: 'Moteur prêt',
    loading: 'Chargement du moteur…',
    failed: 'Moteur indisponible',
    engine: 'Moteur {v}',
  },
  alerts: {
    info: '{n} remarque',
    infos: '{n} remarques',
  },
  help: {
    title: 'Aide',
    shortcuts: 'Raccourcis',
    keys: {
      help: 'Ouvrir l’aide',
      select: 'Outil Sélection',
      hand: 'Outil Main',
      pan: 'Maintenir pour déplacer la vue',
      wheel: 'Zoomer autour du pointeur',
      fit: 'Recadrer sur le plateau',
    },
    wheel: 'Molette',
    space: 'Espace',
  },
};

export default fr;
