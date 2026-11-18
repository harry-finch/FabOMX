// Identité de l'application : nom affiché et préfixe des clés de stockage
// local (`fabomx.theme.v1`, `fabomx.section.bed`…).

export const APP_ID = 'fabomx';
export const APP_NAME = 'FabOMX';

/** Clé du stockage local, préfixée par l'application. */
export const storageKey = (key: string) => `${APP_ID}.${key}`;
