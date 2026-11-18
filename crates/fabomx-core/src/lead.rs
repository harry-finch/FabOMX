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
//! - Onglets (contours Extérieur) : l'onglet de fin arrête la coupe avant le
//!   point d'entrée, à la largeur de l'onglet, et sort dans la chute ; un
//!   onglet le long du contour arrête la coupe (sortie dans la chute), puis
//!   la reprend après lui par un nouveau perçage et une entrée. Positions en
//!   longueur le long du trajet de coupe, depuis le point d'entrée.
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
/// Distance au point d'entrée sous laquelle un contact avec le contour lui-même ne compte pas.
const NEAR_ENTRY: f64 = 0.05;
/// Facteur de raccourcissement d'une amorce qui ne tient pas.
const SHRINK: f64 = 0.85;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    Line,
    Arc,
}

/// Largeur d'onglet par défaut (mm du plateau), et bornes.
pub const TAB_WIDTH: f64 = 1.0;
pub const TAB_RANGE: std::ops::RangeInclusive<f64> = 0.2..=10.0;
/// Nombre maximal d'onglets répartis.
pub const TAB_MAX: u8 = 20;

/// Onglets demandés pour un contour : aucun, un nombre réparti (le premier est
/// l'onglet de fin), ou placés (onglet de fin, et points du repère d'origine).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TabSpec {
    #[default]
    None,
    Count {
        n: u8,
    },
    At {
        end: bool,
        points: Vec<[f64; 2]>,
    },
}

fn tab_width() -> f64 {
    TAB_WIDTH
}

/// Réglages d'un contour : son côté retenu, la forme et les longueurs
/// (mm du plateau) de ses amorces, son point d'entrée choisi (origine), et
/// ses onglets (largeur en mm du plateau).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LeadSpec {
    pub side: Side,
    pub shape: Shape,
    pub lead_in: f64,
    pub lead_out: f64,
    pub entry: Option<[f64; 2]>,
    #[serde(default)]
    pub tabs: TabSpec,
    #[serde(default = "tab_width")]
    pub tab_width: f64,
}

/// Un onglet placé : son centre (origine), et l'intervalle non coupé le long
/// du trajet de coupe (longueurs depuis le point d'entrée, origine).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Tab {
    pub at: [f64; 2],
    pub end: bool,
    pub start: f64,
    pub stop: f64,
    /// Sortie dans la chute au début de l'onglet.
    #[serde(serialize_with = "as_points")]
    pub lead_out: Vec<Seg>,
    /// Entrée après l'onglet (vide pour l'onglet de fin).
    #[serde(serialize_with = "as_points")]
    pub lead_in: Vec<Seg>,
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
    /// Onglets, dans l'ordre du trajet.
    pub tabs: Vec<Tab>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "code")]
pub enum LeadWarning {
    /// L'amorce ne tient pas, même raccourcie à `MIN_LEAD`.
    #[serde(rename = "lead.too_short")]
    TooShort { contour: usize, x: f64, y: f64 },
    /// Un onglet touche un autre onglet ou l'amorce du point d'entrée.
    #[serde(rename = "tab.too_close")]
    TabTooClose { contour: usize, x: f64, y: f64 },
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
        let Some(spec) = specs.get(i).cloned() else {
            out.leads.push(None);
            continue;
        };
        let lead = match spec.side {
            Side::Skip => None,
            Side::Line => Some(on_line(c, &polys[i], &spec, m)),
            Side::Outside | Side::Inside if c.closed => {
                let ctx = Ctx {
                    entry: P::new(f64::NAN, f64::NAN),
                    polys: &polys,
                    closed: &closed_of,
                    obstacles: &obstacles,
                    own: i,
                    hole: spec.side == Side::Inside,
                };
                let (mut lead, short) = closed(c, &polys[i], &spec, m, k, &ctx);
                if short {
                    out.warnings.push(LeadWarning::TooShort {
                        contour: i,
                        x: lead.entry[0],
                        y: lead.entry[1],
                    });
                }
                if spec.side == Side::Outside {
                    lead.tabs = tabs(c, &lead, &spec, k, &ctx, i, &mut out.warnings);
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
        (on_segs(&c.segs, q[0]), false)
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
        tabs: Vec::new(),
    }
}

#[derive(Clone, Copy)]
struct Ctx<'a> {
    /// Point d'entrée : les contacts avec le contour lui-même tout près de lui
    /// ne comptent pas (il est sur la géométrie exacte, le polygone en diffère
    /// de la tolérance de mise en segments).
    entry: P,
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
                    if let Some(x) = hits(w[0], w[1], c, d) {
                        if !(o == self.own && x.dist(self.entry) <= NEAR_ENTRY) {
                            return false;
                        }
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
    // Le point d'entrée sur la géométrie exacte (les arcs ne sont pas des polygones).
    let e = on_segs(&c.segs, q[0]);
    let ctx = &Ctx { entry: e, ..*ctx };
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
            tabs: Vec::new(),
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
fn hits(a: P, b: P, c: P, d: P) -> Option<P> {
    let r = b - a;
    let s = d - c;
    let den = r.x * s.y - r.y * s.x;
    if den.abs() <= 1e-12 * (r.len() * s.len()).max(1e-300) {
        return None;
    }
    let w = c - a;
    let t = (w.x * s.y - w.y * s.x) / den;
    let u = (w.x * r.y - w.y * r.x) / den;
    const EPS: f64 = 1e-9;
    (t > EPS && t < 1.0 - EPS && (-EPS..=1.0 + EPS).contains(&u)).then(|| a + r * t)
}

/// Le point de la géométrie exacte `segs` le plus proche de `p`.
fn on_segs(segs: &[Seg], p: P) -> P {
    segs.iter()
        .map(|s| s.closest(p).0)
        .min_by(|a, b| a.dist(p).total_cmp(&b.dist(p)))
        .unwrap_or(p)
}

/// Le trajet de coupe exact d'un contour (repère d'origine), dans son sens
/// de parcours, du point d'entrée de `lead` jusqu'à sa fin : un contour
/// fermé est coupé au point d'entrée et y revient.
pub fn route(c: &Contour, lead: &Lead) -> Vec<Seg> {
    let mut segs: Vec<Seg> = if lead.reversed {
        c.segs.iter().rev().map(Seg::reversed).collect()
    } else {
        c.segs.clone()
    };
    if !c.closed || segs.is_empty() {
        return segs;
    }
    let e = P::new(lead.entry[0], lead.entry[1]);
    let (j, (_, t)) = segs
        .iter()
        .map(|s| s.closest(e))
        .enumerate()
        .min_by(|a, b| a.1 .0.dist(e).total_cmp(&b.1 .0.dist(e)))
        .unwrap();
    let eps = 1e-9;
    if t <= eps {
        segs.rotate_left(j);
    } else if t >= 1.0 - eps {
        let n = segs.len();
        segs.rotate_left((j + 1) % n);
    } else {
        let (a, b) = segs[j].split(t);
        segs[j] = b;
        segs.rotate_left(j);
        segs.push(a);
    }
    segs
}

/// Longueur d'une suite de segments.
pub(crate) fn length(segs: &[Seg]) -> f64 {
    segs.iter().map(Seg::length).sum()
}

/// Le point et la tangente à la longueur `s` le long de `segs`.
fn at_length(segs: &[Seg], s: f64) -> (P, P) {
    let mut run = 0.0;
    for seg in segs {
        let l = seg.length();
        if run + l >= s || std::ptr::eq(seg, segs.last().unwrap()) {
            let t = if l > 0.0 {
                ((s - run) / l).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let (a, _) = seg.split(t);
            let p = if t <= 0.0 { seg.start() } else { a.end() };
            let tan = match *seg {
                Seg::Line { a, b } => unit(b - a),
                Seg::Arc { c, sweep, .. } => {
                    let r = p - c;
                    let t = left(unit(r));
                    if sweep >= 0.0 {
                        t
                    } else {
                        t * -1.0
                    }
                }
            };
            return (p, tan);
        }
        run += l;
    }
    (P::new(0.0, 0.0), P::new(1.0, 0.0))
}

/// Le morceau de `segs` entre les longueurs `a` et `b`.
pub(crate) fn range(segs: &[Seg], a: f64, b: f64) -> Vec<Seg> {
    let mut out = Vec::new();
    let mut run = 0.0;
    for seg in segs {
        let l = seg.length();
        let (s0, s1) = (run, run + l);
        run = s1;
        if s1 <= a + 1e-12 || s0 >= b - 1e-12 || l <= 0.0 {
            continue;
        }
        let (t0, t1) = (((a - s0) / l).max(0.0), ((b - s0) / l).min(1.0));
        let mut piece = *seg;
        if t1 < 1.0 {
            piece = piece.split(t1).0;
        }
        if t0 > 0.0 {
            piece = piece.split(t0 / t1).1;
        }
        out.push(piece);
    }
    out
}

/// Les onglets d'un contour Extérieur, avec leurs sorties et reprises.
fn tabs(
    c: &Contour,
    lead: &Lead,
    spec: &LeadSpec,
    k: f64,
    ctx: &Ctx,
    contour: usize,
    warnings: &mut Vec<LeadWarning>,
) -> Vec<Tab> {
    let route = route(c, lead);
    let total = length(&route);
    let w = spec.tab_width.clamp(*TAB_RANGE.start(), *TAB_RANGE.end()) / k;
    if total <= 2.0 * w {
        return Vec::new();
    }
    // Centres (longueur) des onglets le long du contour, l'onglet de fin à part.
    let (end, mut centres): (bool, Vec<f64>) = match &spec.tabs {
        TabSpec::None => return Vec::new(),
        TabSpec::Count { n } => {
            let n = (*n).min(TAB_MAX) as usize;
            if n == 0 {
                return Vec::new();
            }
            (true, (1..n).map(|i| total * i as f64 / n as f64).collect())
        }
        TabSpec::At { end, points } => {
            let along = |p: P| {
                let mut run = 0.0;
                let mut best = (f64::INFINITY, 0.0);
                for seg in &route {
                    let (q, t) = seg.closest(p);
                    if q.dist(p) < best.0 {
                        best = (q.dist(p), run + t * seg.length());
                    }
                    run += seg.length();
                }
                best.1
            };
            (
                *end,
                points.iter().map(|&[x, y]| along(P::new(x, y))).collect(),
            )
        }
    };
    centres.sort_by(f64::total_cmp);
    let (l_in, l_out, min) = (spec.lead_in / k, spec.lead_out / k, MIN_LEAD / k);
    let mut out = Vec::new();
    for s in centres {
        let (start, stop) = (s - w / 2.0, s + w / 2.0);
        let (p0, t0) = at_length(&route, start);
        let (p1, t1) = at_length(&route, stop);
        let (lead_out, short_out) = side_lead(ctx, p0, t0, spec.shape, l_out, min, false);
        let (lead_in, short_in) = side_lead(ctx, p1, t1, spec.shape, l_in, min, true);
        let at = at_length(&route, s).0;
        if short_out || short_in {
            warnings.push(LeadWarning::TooShort {
                contour,
                x: at.x,
                y: at.y,
            });
        }
        out.push(Tab {
            at: [at.x, at.y],
            end: false,
            start,
            stop,
            lead_out,
            lead_in,
        });
    }
    if end {
        let start = total - w;
        let (p0, t0) = at_length(&route, start);
        let (lead_out, short) = side_lead(ctx, p0, t0, spec.shape, l_out, min, false);
        let at = at_length(&route, total - w / 2.0).0;
        if short {
            warnings.push(LeadWarning::TooShort {
                contour,
                x: at.x,
                y: at.y,
            });
        }
        out.push(Tab {
            at: [at.x, at.y],
            end: true,
            start,
            stop: total,
            lead_out,
            lead_in: Vec::new(),
        });
    }
    // Trop proches : moins d'une largeur d'onglet de coupe entre deux onglets,
    // ou entre un onglet et le point d'entrée (où sont les amorces).
    let has_end = out.last().is_some_and(|t| t.end);
    let mut prev = 0.0;
    for tab in &out {
        let after = if tab.end { tab.start } else { tab.stop + w };
        let limit = if has_end && !tab.end {
            total - w
        } else {
            total
        };
        if tab.start < prev + w - 1e-9 || after > limit + 1e-9 {
            warnings.push(LeadWarning::TabTooClose {
                contour,
                x: tab.at[0],
                y: tab.at[1],
            });
        }
        prev = tab.stop;
    }
    out
}

/// Amorce d'un point du contour vers la chute (sortie) ou de la chute vers lui
/// (entrée), d'équerre ou en arc, raccourcie si elle ne tient pas.
fn side_lead(
    ctx: &Ctx,
    p: P,
    t: P,
    shape: Shape,
    l: f64,
    min: f64,
    entering: bool,
) -> (Vec<Seg>, bool) {
    let ctx = &Ctx { entry: p, ..*ctx };
    let probe = (l * 0.25).max(1e-6);
    let n = if ctx.scrap(p + left(t) * probe) {
        left(t)
    } else {
        left(t) * -1.0
    };
    match (shape, entering) {
        (Shape::Line, true) => shrink(ctx, l, min, &|l| line_in(p, n, l)),
        (Shape::Line, false) => shrink(ctx, l, min, &|l| line_out(p, n, l)),
        (Shape::Arc, true) => shrink(ctx, l, min, &|l| arc_in(p, t, n, l)),
        (Shape::Arc, false) => shrink(ctx, l, min, &|l| arc_out(p, t, n, l)),
    }
}
