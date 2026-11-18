// Matrices affines `[a, b, c, d, e, f]` : x' = a x + c y + e, y' = b x + d y + f.
// Mêmes formules que `Affine` du moteur (`fabomx-core/src/geom.rs`), pour
// composer les gestes sans attendre le moteur. Un dessin garde une seule
// matrice, appliquée à sa géométrie d'origine.

export type M = [number, number, number, number, number, number];
export type Pt = [number, number];

export const IDENTITY: M = [1, 0, 0, 1, 0, 0];

/** `m` appliquée après `o`. */
export function then(m: M, o: M): M {
  const [a, b, c, d, e, f] = m;
  const [a2, b2, c2, d2, e2, f2] = o;
  return [a * a2 + c * b2, b * a2 + d * b2, a * c2 + c * d2, b * c2 + d * d2, a * e2 + c * f2 + e, b * e2 + d * f2 + f];
}

export function apply(m: M, x: number, y: number): Pt {
  return [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]];
}

export function invert(m: M): M {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  return [d / det, -b / det, -c / det, a / det, (c * f - d * e) / det, (b * e - a * f) / det];
}

export const translate = (dx: number, dy: number): M => [1, 0, 0, 1, dx, dy];

/** Arrondit les quarts de tour exacts (pas de 6e-17 parasite). */
const tidy = (v: number) => (Math.abs(v - Math.round(v)) < 1e-15 ? Math.round(v) : v);

/** Rotation de `deg` degrés (sens trigonométrique) autour de `c`. */
export function rotateAbout([cx, cy]: Pt, deg: number): M {
  const r = (deg * Math.PI) / 180;
  const s = tidy(Math.sin(r));
  const k = tidy(Math.cos(r));
  return [k, s, -s, k, cx - k * cx + s * cy, cy - s * cx - k * cy];
}

export function scaleAbout([cx, cy]: Pt, kx: number, ky: number): M {
  return [kx, 0, 0, ky, cx - kx * cx, cy - ky * cy];
}

/** Miroir autour de `c` : `horizontal` retourne la gauche et la droite. */
export const mirrorAbout = (c: Pt, horizontal: boolean): M => (horizontal ? scaleAbout(c, -1, 1) : scaleAbout(c, 1, -1));

/** Angle de rotation (degrés, de −180 à 180) : celui de l'axe X transformé. */
export function angleOf(m: M): number {
  const deg = (Math.atan2(m[1], m[0]) * 180) / Math.PI;
  return Math.abs(deg) < 1e-9 ? 0 : deg;
}

/** Facteur d'échelle moyen. */
export const factorOf = (m: M) => Math.sqrt(Math.abs(m[0] * m[3] - m[1] * m[2]));

/** Déterminant : négatif quand la matrice retourne le dessin (miroir). */
export const det = (m: M) => m[0] * m[3] - m[1] * m[2];
