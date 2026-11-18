//! Amorces et perçages. Un contour Extérieur ou Trou commence par une
//! amorce d'entrée, du point de perçage (dans la chute) au point d'entrée
//! sur le contour, et finit par une amorce de sortie vers la chute. La
//! chute est dehors pour un Extérieur, dedans pour un Trou.
//!
//! - Point d'entrée : celui choisi à la main, sinon le coin le plus marqué
//!   (au moins `CORNER`), le plus en bas à gauche à égalité ; sans coin, le
//!   milieu du plus long segment.
//! - Amorce droite : à un coin, l'entrée prolonge le premier segment et la
//!   sortie le dernier, quand ce prolongement est dans la chute ; sinon le
//!   long de la bissectrice du coin, ou perpendiculaire au contour.
//! - Amorce en arc : un quart de cercle tangent au contour, de la longueur réglée.
//! - Petit trou (l'entrée irait au-delà du centre) : perçage au centre.
//! - Une amorce qui croise un contour coupé du dessin ou sort de la chute
//!   est raccourcie ; sous `MIN_LEAD`, elle est signalée.
//! - Fente ou contour coupé sur la ligne : pas d'amorce, perçage au point d'entrée.
//!
//! Tout se calcule dans le repère d'origine du dessin ; les longueurs sont
//! en mm du plateau, ramenées à l'origine par l'échelle de la matrice. Le
//! sens de parcours est celui de `cut::reversed`.

use crate::cut::{area, inside, reversed, Contour, Side};
use crate::geom::{Affine, Seg, P};
use serde::{Deserialize, Serialize, Serializer};
use std::f64::consts::FRAC_PI_2;

/// Amorce d'entrée par défaut (mm), celle des exemples LAYOUT (0,15 po).
pub const LEAD_IN: f64 = 3.8;
/// Amorce de sortie par défaut (mm), celle des exemples LAYOUT (0,05 po).
pub const LEAD_OUT: f64 = 1.3;
/// En dessous, une amorce raccourcie est signalée (mm).
pub const MIN_LEAD: f64 = 0.5;
/// Bornes des longueurs réglables (mm).
pub const LEAD_RANGE: std::ops::RangeInclusive<f64> = 0.5..=20.0;
/// Changement de direction minimal d'un coin.
const CORNER: f64 = 30.0 * std::f64::consts::PI / 180.0;
/// Facteur de raccourcissement d'une amorce qui ne tient pas.
const SHRINK: f64 = 0.85;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    Line,
    Arc,
}

/// Réglages d'un contour : son côté retenu, la forme et les longueurs
/// (mm du plateau) de ses amorces, et son point d'entrée choisi (origine).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct LeadSpec {
    pub side: Side,
    pub shape: Shape,
    pub lead_in: f64,
    pub lead_out: f64,
    pub entry: Option<[f64; 2]>,
}

/// Les amorces d'un contour, dans le repère d'origine.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Lead {
    pub pierce: [f64; 2],
    pub entry: [f64; 2],
    #[serde(serialize_with = "as_points")]
    pub lead_in: Vec<Seg>,
    #[serde(serialize_with = "as_points")]
    pub lead_out: Vec<Seg>,
    /// Parcouru à l'envers de son tracé.
    pub reversed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "code")]
pub enum LeadWarning {
    /// L'amorce ne tient pas, même raccourcie à `MIN_LEAD`.
    #[serde(rename = "lead.too_short")]
    TooShort { contour: usize, x: f64, y: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Leads {
    /// Une entrée par contour ; rien pour un contour qui n'est pas coupé.
    pub leads: Vec<Option<Lead>>,
    pub warnings: Vec<LeadWarning>,
}

/// Points `[x, y, …]` d'amorces, pour l'affichage.
fn as_points<S: Serializer>(segs: &[Seg], s: S) -> Result<S::Ok, S::Error> {
    let mut pts = Vec::new();
    for seg in segs {
        let before = pts.len();
        seg.points(0.01, &mut pts);
        if before > 0 {
            pts.remove(before);
        }
    }
    s.collect_seq(pts.into_iter().flat_map(|p| [p.x, p.y]))
}

/// Les amorces des contours d'un dessin placé par `m`, avec un réglage par
/// contour (`specs[i]` pour `contours[i]`).
pub fn leads(contours: &[Contour], specs: &[LeadSpec], m: Affine) -> Leads {
    let k = m.factor().max(1e-12);
    let polys: Vec<Vec<P>> = contours.iter().map(Contour::points).collect();
    let closed_of: Vec<bool> = contours.iter().map(|c| c.closed).collect();
    // Contours coupés : ceux qu'une amorce ne doit pas croiser.
    let obstacles: Vec<usize> = (0..contours.len())
        .filter(|&i| specs.get(i).is_some_and(|s| s.side != Side::Skip))
        .collect();
    let mut out = Leads {
        leads: Vec::with_capacity(contours.len()),
        warnings: Vec::new(),
    };
    for (i, c) in contours.iter().enumerate() {
        let Some(spec) = specs.get(i).copied() else {
            out.leads.push(None);
            continue;
        };
        let lead = match spec.side {
            Side::Skip => None,
            Side::Line => Some(on_line(c, &polys[i], &spec, m)),
            Side::Outside | Side::Inside if c.closed => {
                let ctx = Ctx {
                    polys: &polys,
                    closed: &closed_of,
                    obstacles: &obstacles,
                    own: i,
                    hole: spec.side == Side::Inside,
                };
                let (lead, short) = closed(c, &polys[i], &spec, m, k, &ctx);
                if short {
                    out.warnings.push(LeadWarning::TooShort {
                        contour: i,
                        x: lead.entry[0],
                        y: lead.entry[1],
                    });
                }
                Some(lead)
            }
            // Extérieur ou Trou sur un tracé ouvert : pas d'amorce possible.
            _ => Some(on_line(c, &polys[i], &spec, m)),
        };
        out.leads.push(lead);
    }
    out
}

/// Fente ou contour coupé sur la ligne : perçage au point d'entrée.
fn on_line(c: &Contour, poly: &[P], spec: &LeadSpec, m: Affine) -> Lead {
    let (at, rev) = if c.closed {
        let (q, _) = travel(poly, false);
        let (q, _) = start_at(q, spec.entry, m);
        (q[0], false)
    } else {
        // Tracé ouvert : son début, ou sa fin si le point choisi en est plus près.
        let (a, b) = (poly[0], poly[poly.len() - 1]);
        match spec.entry {
            Some([x, y]) if P::new(x, y).dist(b) < P::new(x, y).dist(a) => (b, true),
            _ => (a, false),
        }
    };
    Lead {
        pierce: [at.x, at.y],
        entry: [at.x, at.y],
        lead_in: Vec::new(),
        lead_out: Vec::new(),
        reversed: rev,
    }
}

struct Ctx<'a> {
    polys: &'a [Vec<P>],
    closed: &'a [bool],
    obstacles: &'a [usize],
    own: usize,
    hole: bool,
}

impl Ctx<'_> {
    /// Le point `p` est-il dans la chute du contour ?
    fn scrap(&self, p: P) -> bool {
        inside(p, &self.polys[self.own]) == self.hole
    }

    /// Les segments de `segs` restent-ils dans la chute, sans croiser de contour coupé ?
    fn fits(&self, segs: &[Seg]) -> bool {
        let mut pts = Vec::new();
        for s in segs {
            s.points(0.01, &mut pts);
        }
        // Le bout sur le contour ne compte pas : tester un peu avant.
        let probe = |a: P, b: P| a.lerp(b, 0.5);
        if pts.len() < 2 || !pts.windows(2).all(|w| self.scrap(probe(w[0], w[1]))) {
            return false;
        }
        for &o in self.obstacles {
            let poly = &self.polys[o];
            let n = poly.len();
            if n < 2 {
                continue;
            }
            let edges = if self.closed[o] { n } else { n - 1 };
            for w in pts.windows(2) {
                for j in 0..edges {
                    let (c, d) = (poly[j], poly[(j + 1) % n]);
                    if hits(w[0], w[1], c, d) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Le contour fermé dans le sens de parcours.
fn travel(poly: &[P], rev: bool) -> (Vec<P>, bool) {
    let mut q = poly.to_vec();
    if rev {
        q.reverse();
    }
    (q, rev)
}

/// Changement de direction au sommet `i` d'un contour fermé (radians).
fn turn(q: &[P], i: usize) -> f64 {
    let n = q.len();
    let (a, b, c) = (q[(i + n - 1) % n], q[i], q[(i + 1) % n]);
    let (u, v) = (b - a, c - b);
    let (lu, lv) = (u.len(), v.len());
    if lu < 1e-12 || lv < 1e-12 {
        return 0.0;
    }
    ((u.x * v.x + u.y * v.y) / (lu * lv))
        .clamp(-1.0, 1.0)
        .acos()
}

/// Le contour qui commence au point d'entrée (choisi, ou automatique).
fn start_at(mut q: Vec<P>, entry: Option<[f64; 2]>, m: Affine) -> (Vec<P>, bool) {
    let n = q.len();
    let idx = match entry {
        Some([x, y]) => {
            // Le point du contour le plus proche du point choisi.
            let p = P::new(x, y);
            let mut best = (f64::INFINITY, 0, p);
            for j in 0..n {
                let (a, b) = (q[j], q[(j + 1) % n]);
                let ab = b - a;
                let l2 = ab.x * ab.x + ab.y * ab.y;
                let t = if l2 > 0.0 {
                    (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let f = a + ab * t;
                if f.dist(p) < best.0 {
                    best = (f.dist(p), j, f);
                }
            }
            let (_, j, f) = best;
            insert(&mut q, j, f)
        }
        None => {
            let corners: Vec<(usize, f64)> = (0..n)
                .map(|i| (i, turn(&q, i)))
                .filter(|&(_, a)| a >= CORNER)
                .collect();
            if corners.is_empty() {
                // Milieu du plus long segment.
                let j = (0..n)
                    .max_by(|&a, &b| {
                        q[a].dist(q[(a + 1) % n])
                            .total_cmp(&q[b].dist(q[(b + 1) % n]))
                            .then(b.cmp(&a))
                    })
                    .unwrap_or(0);
                let mid = q[j].lerp(q[(j + 1) % n], 0.5);
                insert(&mut q, j, mid)
            } else {
                let key = |i: usize| {
                    let p = m.apply(q[i]);
                    (p.y, p.x)
                };
                corners
                    .iter()
                    .copied()
                    .max_by(|&(i, a), &(j, b)| {
                        if (a - b).abs() > 1e-6 {
                            a.total_cmp(&b)
                        } else {
                            // À égalité, le plus en bas à gauche sur le plateau.
                            let (ki, kj) = (key(i), key(j));
                            kj.0.total_cmp(&ki.0).then(kj.1.total_cmp(&ki.1))
                        }
                    })
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            }
        }
    };
    let corner = turn(&q, idx) >= CORNER;
    q.rotate_left(idx);
    (q, corner)
}

/// Place `f` sur l'arête `j` (de `q[j]` à `q[j + 1]`) ; rend son indice.
fn insert(q: &mut Vec<P>, j: usize, f: P) -> usize {
    let n = q.len();
    let eps = 1e-9;
    if f.dist(q[j]) <= eps {
        j
    } else if f.dist(q[(j + 1) % n]) <= eps {
        (j + 1) % n
    } else {
        q.insert(j + 1, f);
        j + 1
    }
}

fn unit(v: P) -> P {
    let l = v.len();
    if l < 1e-300 {
        P::new(1.0, 0.0)
    } else {
        v * (1.0 / l)
    }
}

/// Normale à gauche de `t`.
fn left(t: P) -> P {
    P::new(-t.y, t.x)
}

/// Amorce d'entrée droite, de longueur `l`, depuis la direction `u` (du point d'entrée vers le perçage).
fn line_in(e: P, u: P, l: f64) -> Vec<Seg> {
    vec![Seg::Line { a: e + u * l, b: e }]
}

/// Amorce de sortie droite, de longueur `l`, dans la direction `u`.
fn line_out(e: P, u: P, l: f64) -> Vec<Seg> {
    vec![Seg::Line { a: e, b: e + u * l }]
}

/// Quart de cercle d'entrée de longueur `l`, tangent à `t` au point `e`,
/// du côté de la normale `n`.
fn arc_in(e: P, t: P, n: P, l: f64) -> Vec<Seg> {
    let r = l / FRAC_PI_2;
    let c = e + n * r;
    let ccw = n.x * left(t).x + n.y * left(t).y > 0.0;
    let a0 = (t * -1.0).angle();
    vec![Seg::Arc {
        c,
        r,
        a0,
        sweep: if ccw { FRAC_PI_2 } else { -FRAC_PI_2 },
    }]
}

/// Quart de cercle de sortie de longueur `l`, tangent à `t` au point `e`,
/// du côté de la normale `n`.
fn arc_out(e: P, t: P, n: P, l: f64) -> Vec<Seg> {
    let r = l / FRAC_PI_2;
    let c = e + n * r;
    let ccw = n.x * left(t).x + n.y * left(t).y > 0.0;
    let a0 = (n * -1.0).angle();
    vec![Seg::Arc {
        c,
        r,
        a0,
        sweep: if ccw { FRAC_PI_2 } else { -FRAC_PI_2 },
    }]
}

/// Raccourcit `make(l)` jusqu'à ce qu'elle tienne ; rend l'amorce et vrai si
/// elle ne tient pas même à `min`.
fn shrink(ctx: &Ctx, l: f64, min: f64, make: &dyn Fn(f64) -> Vec<Seg>) -> (Vec<Seg>, bool) {
    let mut len = l;
    loop {
        let segs = make(len);
        if ctx.fits(&segs) {
            return (segs, false);
        }
        let next = len * SHRINK;
        if next < min {
            return (make(min.min(l)), true);
        }
        len = next;
    }
}

fn closed(c: &Contour, poly: &[P], spec: &LeadSpec, m: Affine, k: f64, ctx: &Ctx) -> (Lead, bool) {
    let rev = reversed(spec.side, c.area, m.det());
    let (q, _) = travel(poly, rev);
    let (q, corner) = start_at(q, spec.entry, m);
    let n = q.len();
    let e = q[0];
    let t_first = unit(q[1 % n] - e);
    let t_last = unit(e - q[n - 1]);
    let (l_in, l_out, min) = (spec.lead_in / k, spec.lead_out / k, MIN_LEAD / k);
    // Petit pas pour tester de quel côté est la chute.
    let probe = (l_in.min(l_out) * 0.25).max(1e-6);
    let toward = |v: P| {
        if ctx.scrap(e + v * probe) {
            v
        } else {
            v * -1.0
        }
    };

    // Entrée.
    let (lead_in, short_in) = match spec.shape {
        Shape::Line => {
            let mut dirs = Vec::new();
            if corner {
                dirs.push(t_first * -1.0);
                dirs.push(toward(unit(t_first - t_last)));
            }
            dirs.push(toward(left(t_first)));
            // Petit trou : percer au centre si l'entrée irait au-delà.
            let center = ctx.hole.then(|| centroid(poly)).filter(|&g| {
                inside(g, poly) && e.dist(g) <= l_in && ctx.fits(&[Seg::Line { a: g, b: e }])
            });
            match center {
                Some(g) => (vec![Seg::Line { a: g, b: e }], false),
                None => {
                    let u = dirs
                        .iter()
                        .copied()
                        .find(|&u| ctx.fits(&line_in(e, u, l_in)))
                        .unwrap_or(*dirs.last().unwrap());
                    shrink(ctx, l_in, min, &|l| line_in(e, u, l))
                }
            }
        }
        Shape::Arc => {
            let nrm = toward(left(t_first));
            shrink(ctx, l_in, min, &|l| arc_in(e, t_first, nrm, l))
        }
    };

    // Sortie.
    let (lead_out, short_out) = match spec.shape {
        Shape::Line => {
            let mut dirs = Vec::new();
            if corner {
                dirs.push(t_last);
            }
            dirs.push(toward(left(t_last)));
            let u = dirs
                .iter()
                .copied()
                .find(|&u| ctx.fits(&line_out(e, u, l_out)))
                .unwrap_or(*dirs.last().unwrap());
            shrink(ctx, l_out, min, &|l| line_out(e, u, l))
        }
        Shape::Arc => {
            let nrm = toward(left(t_last));
            shrink(ctx, l_out, min, &|l| arc_out(e, t_last, nrm, l))
        }
    };

    let pierce = lead_in.first().map_or(e, Seg::start);
    (
        Lead {
            pierce: [pierce.x, pierce.y],
            entry: [e.x, e.y],
            lead_in,
            lead_out,
            reversed: rev,
        },
        short_in || short_out,
    )
}

/// Centre de gravité d'un polygone fermé.
fn centroid(poly: &[P]) -> P {
    let n = poly.len();
    let a = area(poly);
    if a.abs() < 1e-300 {
        return poly[0];
    }
    let (mut cx, mut cy) = (0.0, 0.0);
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let w = p.x * q.y - q.x * p.y;
        cx += (p.x + q.x) * w;
        cy += (p.y + q.y) * w;
    }
    P::new(cx / (6.0 * a), cy / (6.0 * a))
}

/// L'amorce [a, b] touche-t-elle l'arête [c, d] ailleurs qu'à ses propres
/// bouts ? Les bouts de l'arête comptent : passer par le coin d'un autre
/// contour, c'est le toucher.
fn hits(a: P, b: P, c: P, d: P) -> bool {
    let r = b - a;
    let s = d - c;
    let den = r.x * s.y - r.y * s.x;
    if den.abs() <= 1e-12 * (r.len() * s.len()).max(1e-300) {
        return false;
    }
    let w = c - a;
    let t = (w.x * s.y - w.y * s.x) / den;
    let u = (w.x * r.y - w.y * r.x) / den;
    const EPS: f64 = 1e-9;
    t > EPS && t < 1.0 - EPS && (-EPS..=1.0 + EPS).contains(&u)
}

/// Le trajet de coupe d'un contour (repère d'origine), dans son sens de
/// parcours, du point d'entrée de `lead` jusqu'à sa fin : un contour fermé
/// revient au point d'entrée.
pub fn route(c: &Contour, lead: &Lead, m: Affine) -> Vec<P> {
    let poly = c.points();
    if !c.closed {
        let mut q = poly;
        if lead.reversed {
            q.reverse();
        }
        return q;
    }
    let (q, _) = travel(&poly, lead.reversed);
    let (mut q, _) = start_at(q, Some(lead.entry), m);
    if let Some(&first) = q.first() {
        q.push(first);
    }
    q
}
