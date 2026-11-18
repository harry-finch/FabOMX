// Raccourcis affichés : ⌘ et ⇧ sur Mac, Ctrl+ et Shift+ ailleurs (Windows).
// Les textes et l'aide sont écrits avec les symboles du Mac ; `keys` les
// traduit pour les autres systèmes.

export const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/i.test(navigator.platform || navigator.userAgent);

export function keys(text: string): string {
  if (isMac) return text;
  return text.replace(/⇧⌘/g, 'Ctrl+Shift+').replace(/⌘/g, 'Ctrl+').replace(/⇧/g, 'Shift+');
}
