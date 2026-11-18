//! Géométrie 2D du moteur : points, segments (lignes et arcs exacts),
//! boîtes englobantes, transformations affines et mise en segments des
//! courbes à une tolérance donnée. Millimètres, Y vers le haut.

use serde::Serialize;
use std::f64::consts::{PI, TAU};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct P {
    pub x: f64,
    pub y: f64,
}

impl P {
    pub const fn new(x: f64, y: f64) -> Self {
        P { x, y }
    }

    pub fn len(self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn dist(self, o: P) -> f64 {
        (self - o).len()
    }

    pub fn angle(self) -> f64 {
        self.y.atan2(self.x)
    }

    pub fn lerp(self, o: P, t: f64) -> P {
        self + (o - self) * t
    }
}

impl std::ops::Add for P {
    type Output = P;
    fn add(self, o: P) -> P {
        P::new(self.x + o.x, self.y + o.y)
    }
}

impl std::ops::Sub for P {
    type Output = P;
    fn sub(self, o: P) -> P {
        P::new(self.x - o.x, self.y - o.y)
    }
}

impl std::ops::Mul<f64> for P {
    type Output = P;
    fn mul(self, k: f64) -> P {
        P::new(self.x * k, self.y * k)
    }
}

/// Point d'un cercle de centre `c` et de rayon `r` à l'angle `a` (radians).
pub fn polar(c: P, r: f64, a: f64) -> P {
    P::new(c.x + r * a.cos(), c.y + r * a.sin())
}

/// Un segment exact : une ligne, ou un arc de cercle (angle de départ et
/// balayage signé, en radians ; positif = sens trigonométrique).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Seg {
    Line { a: P, b: P },
    Arc { c: P, r: f64, a0: f64, sweep: f64 },
}

impl Seg {
    pub fn start(&self) -> P {
        match *self {
            Seg::Line { a, .. } => a,
            Seg::Arc { c, r, a0, .. } => polar(c, r, a0),
        }
    }

    pub fn end(&self) -> P {
        match *self {
            Seg::Line { b, .. } => b,
            Seg::Arc { c, r, a0, sweep } => polar(c, r, a0 + sweep),
        }
    }

    pub fn length(&self) -> f64 {
        match *self {
            Seg::Line { a, b } => a.dist(b),
            Seg::Arc { r, sweep, .. } => r * sweep.abs(),
        }
    }

    /// Le point au milieu du segment.
    pub fn mid(&self) -> P {
        match *self {
            Seg::Line { a, b } => a.lerp(b, 0.5),
            Seg::Arc { c, r, a0, sweep } => polar(c, r, a0 + sweep / 2.0),
        }
    }

    pub fn extend(&self, b: &mut BBox) {
        b.add(self.start());
        b.add(self.end());
        if let Seg::Arc { c, r, a0, sweep } = *self {
            // Les extrémités des axes que l'arc traverse.
            for k in 0..4 {
                let q = k as f64 * PI / 2.0;
                if angle_within(q, a0, sweep) {
                    b.add(polar(c, r, q));
                }
            }
        }
    }

    /// Les points du segment, de son début à sa fin, à `tol` près.
    pub fn points(&self, tol: f64, out: &mut Vec<P>) {
        match *self {
            Seg::Line { a, b } => {
                out.push(a);
                out.push(b);
            }
            Seg::Arc { c, r, a0, sweep } => {
                let n = arc_steps(r, sweep, tol);
                for k in 0..=n {
                    out.push(polar(c, r, a0 + sweep * k as f64 / n as f64));
                }
            }
        }
    }
}

/// Nombre de cordes pour qu'un arc ne s'écarte pas de plus de `tol`.
pub fn arc_steps(r: f64, sweep: f64, tol: f64) -> usize {
    if r <= tol {
        return 1;
    }
    let step = 2.0 * (1.0 - tol / r).clamp(-1.0, 1.0).acos();
    ((sweep.abs() / step.max(1e-6)).ceil() as usize).clamp(1, 4096)
}

/// L'angle `q` est-il balayé par l'arc qui part de `a0` et balaie `sweep` ?
fn angle_within(q: f64, a0: f64, sweep: f64) -> bool {
    if sweep.abs() >= TAU - 1e-12 {
        return true;
    }
    let (start, span) = if sweep >= 0.0 {
        (a0, sweep)
    } else {
        (a0 + sweep, -sweep)
    };
    (q - start).rem_euclid(TAU) <= span
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBox {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl BBox {
    pub const EMPTY: BBox = BBox {
        x0: f64::INFINITY,
        y0: f64::INFINITY,
        x1: f64::NEG_INFINITY,
        y1: f64::NEG_INFINITY,
    };

    pub fn add(&mut self, p: P) {
        self.x0 = self.x0.min(p.x);
        self.y0 = self.y0.min(p.y);
        self.x1 = self.x1.max(p.x);
        self.y1 = self.y1.max(p.y);
    }

    pub fn is_empty(&self) -> bool {
        self.x0 > self.x1
    }

    pub fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    pub fn height(&self) -> f64 {
        self.y1 - self.y0
    }
}

/// Transformation affine `[a, b, c, d, e, f]` : x' = a x + c y + e, y' = b x + d y + f.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine(pub [f64; 6]);

impl Affine {
    pub const IDENTITY: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn scale(k: f64) -> Affine {
        Affine([k, 0.0, 0.0, k, 0.0, 0.0])
    }

    /// `self` appliquée après `o`.
    pub fn then_after(self, o: Affine) -> Affine {
        let [a, b, c, d, e, f] = self.0;
        let [a2, b2, c2, d2, e2, f2] = o.0;
        Affine([
            a * a2 + c * b2,
            b * a2 + d * b2,
            a * c2 + c * d2,
            b * c2 + d * d2,
            a * e2 + c * f2 + e,
            b * e2 + d * f2 + f,
        ])
    }

    pub fn apply(&self, p: P) -> P {
        let [a, b, c, d, e, f] = self.0;
        P::new(a * p.x + c * p.y + e, b * p.x + d * p.y + f)
    }

    fn apply_vec(&self, p: P) -> P {
        let [a, b, c, d, _, _] = self.0;
        P::new(a * p.x + c * p.y, b * p.x + d * p.y)
    }

    pub fn det(&self) -> f64 {
        let [a, b, c, d, _, _] = self.0;
        a * d - b * c
    }

    /// Facteur d'échelle moyen.
    pub fn factor(&self) -> f64 {
        self.det().abs().sqrt()
    }

    /// Une similitude (rotation, échelle uniforme, symétrie) garde les cercles.
    pub fn is_similarity(&self) -> bool {
        let [a, b, c, d, _, _] = self.0;
        let (n1, n2) = (a * a + b * b, c * c + d * d);
        let k = n1.max(n2).max(1e-300);
        (n1 - n2).abs() <= 1e-9 * k && (a * c + b * d).abs() <= 1e-9 * k
    }

    /// L'image d'un arc par une similitude.
    pub fn arc(&self, c: P, r: f64, a0: f64, sweep: f64) -> Seg {
        let dir = self.apply_vec(polar(P::new(0.0, 0.0), 1.0, a0));
        Seg::Arc {
            c: self.apply(c),
            r: r * self.factor(),
            a0: dir.angle(),
            sweep: if self.det() < 0.0 { -sweep } else { sweep },
        }
    }
}

/// Met en segments une courbe paramétrée `f` sur [t0, t1] : chaque corde
/// reste à moins de `tol` de la courbe (contrôlé au milieu et aux quarts).
/// Les points rendus vont de `f(t0)` à `f(t1)` compris.
pub fn flatten(f: &dyn Fn(f64) -> P, t0: f64, t1: f64, tol: f64, min_steps: usize) -> Vec<P> {
    fn deviation(p: P, a: P, b: P) -> f64 {
        let ab = b - a;
        let l = ab.len();
        if l < 1e-15 {
            return p.dist(a);
        }
        ((p.x - a.x) * ab.y - (p.y - a.y) * ab.x).abs() / l
    }
    fn split(
        f: &dyn Fn(f64) -> P,
        (ta, pa): (f64, P),
        (tb, pb): (f64, P),
        tol: f64,
        depth: u32,
        out: &mut Vec<P>,
    ) {
        let tm = (ta + tb) / 2.0;
        let pm = f(tm);
        let q1 = f(ta + (tb - ta) / 4.0);
        let q3 = f(ta + 3.0 * (tb - ta) / 4.0);
        let worst = deviation(pm, pa, pb)
            .max(deviation(q1, pa, pb))
            .max(deviation(q3, pa, pb));
        if depth < 24 && worst > tol {
            split(f, (ta, pa), (tm, pm), tol, depth + 1, out);
            split(f, (tm, pm), (tb, pb), tol, depth + 1, out);
        } else {
            out.push(pb);
        }
    }
    let n = min_steps.max(1);
    let mut out = vec![f(t0)];
    for k in 0..n {
        let ta = t0 + (t1 - t0) * k as f64 / n as f64;
        let tb = t0 + (t1 - t0) * (k + 1) as f64 / n as f64;
        let pa = *out.last().unwrap();
        split(f, (ta, pa), (tb, f(tb)), tol, 0, &mut out);
    }
    out
}

/// Arc de balayage `4 atan(bulge)` entre deux points (polylignes DXF).
pub fn bulge_arc(p: P, q: P, bulge: f64) -> Seg {
    let sweep = 4.0 * bulge.atan();
    let d = p.dist(q);
    let mid = p.lerp(q, 0.5);
    let n = P::new(-(q.y - p.y) / d, (q.x - p.x) / d);
    let c = mid + n * (d * (1.0 - bulge * bulge) / (4.0 * bulge));
    Seg::Arc {
        c,
        r: c.dist(p),
        a0: (p - c).angle(),
        sweep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_bbox_includes_crossed_axes() {
        let mut b = BBox::EMPTY;
        Seg::Arc {
            c: P::new(0.0, 0.0),
            r: 10.0,
            a0: 0.0,
            sweep: PI,
        }
        .extend(&mut b);
        assert!((b.y1 - 10.0).abs() < 1e-9 && (b.x0 + 10.0).abs() < 1e-9 && b.y0.abs() < 1e-9);
    }

    #[test]
    fn bulge_one_is_a_half_circle() {
        let s = bulge_arc(P::new(0.0, 0.0), P::new(10.0, 0.0), 1.0);
        let Seg::Arc { c, r, sweep, .. } = s else {
            panic!()
        };
        assert!(
            c.dist(P::new(5.0, 0.0)) < 1e-9 && (r - 5.0).abs() < 1e-9 && (sweep - PI).abs() < 1e-9
        );
        assert!(s.end().dist(P::new(10.0, 0.0)) < 1e-9);
    }

    #[test]
    fn similarity_with_mirror_reverses_arcs() {
        let m = Affine([2.0, 0.0, 0.0, -2.0, 0.0, 0.0]);
        assert!(m.is_similarity());
        let s = m.arc(P::new(0.0, 0.0), 1.0, 0.0, PI / 2.0);
        assert!(s.start().dist(P::new(2.0, 0.0)) < 1e-9 && s.end().dist(P::new(0.0, -2.0)) < 1e-9);
    }
}
