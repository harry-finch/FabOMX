//! Des arcs à la place des petits segments, à précision égale.
//!
//! - `biarcs` : une courbe paramétrée (Bézier, spline, ellipse) en arcs
//!   tangents qui se raccordent sans cassure (biarcs), à `tol` près ; une
//!   partie presque droite devient une ligne, un détail plus petit que
//!   `MIN_RADIUS` reste en segments.
//! - `fit` : dans une suite de segments (polyligne), les suites qui suivent
//!   un arc deviennent cet arc : ses sommets à `tol` près, ses cordes pas
//!   plus loin que `CHORD` écarts de l'arc. Les coins (changement de direction de plus
//!   de `CORNER`) ne sont jamais arrondis, les segments alignés restent une
//!   ligne, les arcs et les lignes isolées ne changent pas.

use crate::geom::{flatten, Seg, P};
use std::f64::consts::{PI, TAU};

/// Rayon minimal d'un arc reconnu (mm) : en dessous, le détail reste en segments.
pub const MIN_RADIUS: f64 = 0.1;
/// Changement de direction à partir duquel deux segments font un coin.
pub const CORNER: f64 = 30.0 * PI / 180.0;
/// Flèche max d'une corde de la polyligne sous l'arc, en écarts : une
/// polyligne de CAO suit son arc à quelques centièmes près, un long segment
/// droit en est loin.
const CHORD: f64 = 10.0;
/// Nombre minimal de segments remplacés par un arc.
const MIN_RUN: usize = 3;

fn unit(v: P) -> P {
    let l = v.len();
    if l < 1e-300 {
        P::new(0.0, 0.0)
    } else {
        v * (1.0 / l)
    }
}

fn left(v: P) -> P {
    P::new(-v.y, v.x)
}

fn dot(a: P, b: P) -> f64 {
    a.x * b.x + a.y * b.y
}

/// Arc qui part de `a` avec la tangente `t` et arrive en `b` ; une ligne si
/// `b` est dans l'axe de la tangente.
fn tangent_arc(a: P, t: P, b: P) -> Seg {
    let ab = b - a;
    let k = dot(ab, left(t));
    if k.abs() <= 1e-12 * ab.len().max(1e-300) {
        return Seg::Line { a, b };
    }
    let r = dot(ab, ab) / (2.0 * k);
    let c = a + left(t) * r;
    let a0 = (a - c).angle();
    let a1 = (b - c).angle();
    let sweep = if r > 0.0 {
        (a1 - a0).rem_euclid(TAU)
    } else {
        -(a0 - a1).rem_euclid(TAU)
    };
    Seg::Arc {
        c,
        r: r.abs(),
        a0,
        sweep,
    }
}

/// Biarc de `p0` (tangente `t0`) à `p1` (tangente `t1`), les deux arcs de
/// même longueur de levier ; `None` si la construction est dégénérée.
fn biarc(p0: P, t0: P, p1: P, t1: P) -> Option<[Seg; 2]> {
    let v = p1 - p0;
    let t = t0 + t1;
    let denom = 2.0 * (1.0 - dot(t0, t1));
    let d = if denom.abs() < 1e-12 {
        let vt = dot(v, t1);
        if vt.abs() < 1e-12 {
            return None;
        }
        dot(v, v) / (4.0 * vt)
    } else {
        let vt = dot(v, t);
        let disc = vt * vt + denom * dot(v, v);
        (-vt + disc.max(0.0).sqrt()) / denom
    };
    if !d.is_finite() || d <= 0.0 {
        return None;
    }
    let q1 = p0 + t0 * d;
    let q2 = p1 - t1 * d;
    let pm = q1.lerp(q2, 0.5);
    let tm = unit(q2 - q1);
    if tm.len() == 0.0 {
        return None;
    }
    Some([tangent_arc(p0, t0, pm), tangent_arc(pm, tm, p1)])
}

/// Distance d'un point à un segment exact.
fn distance(s: &Seg, p: P) -> f64 {
    s.closest(p).0.dist(p)
}

fn radius_ok(s: &Seg, min_radius: f64) -> bool {
    match *s {
        Seg::Line { .. } => true,
        Seg::Arc { r, .. } => r >= min_radius,
    }
}

/// La courbe `f` sur [t0, t1] en lignes et arcs tangents, à `tol` près.
pub fn biarcs(
    f: &dyn Fn(f64) -> P,
    t0: f64,
    t1: f64,
    tol: f64,
    min_radius: f64,
    min_steps: usize,
) -> Vec<Seg> {
    let tangent = |t: f64| {
        let h = (t1 - t0).abs() * 1e-6;
        let (a, b) = ((t - h).max(t0.min(t1)), (t + h).min(t0.max(t1)));
        let d = f(b) - f(a);
        let d = if t1 < t0 { d * -1.0 } else { d };
        unit(d)
    };
    let mut out = Vec::new();
    let n = min_steps.max(1);
    for k in 0..n {
        let ta = t0 + (t1 - t0) * k as f64 / n as f64;
        let tb = t0 + (t1 - t0) * (k + 1) as f64 / n as f64;
        fit_interval(f, &tangent, ta, tb, tol, min_radius, 0, &mut out);
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn fit_interval(
    f: &dyn Fn(f64) -> P,
    tangent: &dyn Fn(f64) -> P,
    ta: f64,
    tb: f64,
    tol: f64,
    min_radius: f64,
    depth: u32,
    out: &mut Vec<Seg>,
) {
    let (pa, pb) = (f(ta), f(tb));
    if pa.dist(pb) < 1e-12 {
        return;
    }
    let samples: Vec<P> = (1..8).map(|i| f(ta + (tb - ta) * i as f64 / 8.0)).collect();
    // Presque droit : une ligne.
    let line = Seg::Line { a: pa, b: pb };
    if samples.iter().all(|&q| distance(&line, q) <= tol) {
        out.push(line);
        return;
    }
    if let Some(arcs) = biarc(pa, tangent(ta), pb, tangent(tb)) {
        let close = samples.iter().all(|&q| {
            arcs.iter()
                .map(|s| distance(s, q))
                .fold(f64::INFINITY, f64::min)
                <= tol
        });
        if close && arcs.iter().all(|s| radius_ok(s, min_radius)) {
            out.extend(arcs.into_iter().filter(|s| s.length() > 1e-12));
            return;
        }
    }
    if depth >= 16 || (tb - ta).abs() < 1e-9 {
        // Détail trop fin pour des arcs : des segments.
        let pts = flatten(f, ta, tb, tol, 1);
        out.extend(pts.windows(2).map(|w| Seg::Line { a: w[0], b: w[1] }));
        return;
    }
    let tm = (ta + tb) / 2.0;
    fit_interval(f, tangent, ta, tm, tol, min_radius, depth + 1, out);
    fit_interval(f, tangent, tm, tb, tol, min_radius, depth + 1, out);
}

/// Cercle passant par trois points : centre et rayon.
fn circle(a: P, b: P, c: P) -> Option<(P, f64)> {
    let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
    if d.abs() < 1e-18 {
        return None;
    }
    let (a2, b2, c2) = (dot(a, a), dot(b, b), dot(c, c));
    let x = (a2 * (b.y - c.y) + b2 * (c.y - a.y) + c2 * (a.y - b.y)) / d;
    let y = (a2 * (c.x - b.x) + b2 * (a.x - c.x) + c2 * (b.x - a.x)) / d;
    let o = P::new(x, y);
    Some((o, o.dist(a)))
}

/// Changement de direction signé entre deux segments consécutifs.
fn turn(a: P, b: P, c: P) -> f64 {
    let (u, v) = (b - a, c - b);
    (u.x * v.y - u.y * v.x).atan2(dot(u, v))
}

/// L'arc qui passe par les points `pts` (dans l'ordre), s'il les suit à
/// `tol` près, tourne toujours du même côté et n'est ni trop petit ni droit.
fn arc_through(pts: &[P], tol: f64, min_radius: f64) -> Option<Seg> {
    let n = pts.len();
    let (a, m, b) = (pts[0], pts[n / 2], pts[n - 1]);
    let (c, r) = circle(a, m, b)?;
    if r < min_radius || a.dist(b) < 1e-12 {
        return None;
    }
    if pts.iter().any(|p| (p.dist(c) - r).abs() > tol) {
        return None;
    }
    // Les sommets sont sur l'arc ; ses cordes ne doivent pas s'en écarter de
    // plus de `CHORD` écarts : un long segment droit ne devient pas un arc.
    if pts
        .windows(2)
        .any(|w| r - (r * r - w[0].dist(w[1]).powi(2) / 4.0).max(0.0).sqrt() > CHORD * tol)
    {
        return None;
    }
    // Pas une ligne : la flèche de l'arc dépasse l'écart.
    let chord = Seg::Line { a, b };
    if pts.iter().all(|&p| distance(&chord, p) <= tol) {
        return None;
    }
    let turns: Vec<f64> = pts.windows(3).map(|w| turn(w[0], w[1], w[2])).collect();
    let ccw = turns.iter().sum::<f64>() > 0.0;
    if turns.iter().any(|&t| (t > 1e-12) != ccw && t.abs() > 1e-12) {
        return None;
    }
    let a0 = (a - c).angle();
    let a1 = (b - c).angle();
    let sweep = if ccw {
        (a1 - a0).rem_euclid(TAU)
    } else {
        -(a0 - a1).rem_euclid(TAU)
    };
    // Le balayage doit couvrir les points dans l'ordre (pas le grand arc par erreur).
    let mid = (pts[n / 2] - c).angle() - a0;
    let mid = if ccw {
        mid.rem_euclid(TAU)
    } else {
        -(-mid).rem_euclid(TAU)
    };
    if mid.abs() > sweep.abs() {
        return None;
    }
    Some(Seg::Arc { c, r, a0, sweep })
}

/// Remplace dans `segs` les suites d'au moins trois lignes qui suivent un arc
/// par cet arc ; rend les segments et le nombre d'arcs créés.
pub fn fit(segs: &[Seg], closed: bool, tol: f64, min_radius: f64) -> (Vec<Seg>, usize) {
    let n = segs.len();
    if n < MIN_RUN {
        return (segs.to_vec(), 0);
    }
    let is_line = |i: usize| matches!(segs[i], Seg::Line { .. });
    // Coupure avant le segment i : il n'est pas une ligne, ou il fait un coin avec le précédent.
    let cut = |i: usize| -> bool {
        let prev = if i == 0 {
            if closed {
                n - 1
            } else {
                return true;
            }
        } else {
            i - 1
        };
        if !is_line(i) || !is_line(prev) {
            return true;
        }
        turn(segs[prev].start(), segs[i].start(), segs[i].end()).abs() > CORNER
    };
    // Départ : une coupure, pour ne pas ouvrir un arc au milieu d'une boucle.
    let first_cut = (0..n).find(|&i| cut(i));
    let order: Vec<usize> = match first_cut {
        Some(s) => (0..n).map(|k| (s + k) % n).collect(),
        None => (0..n).collect(),
    };
    // Suites de lignes entre deux coupures.
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for &i in &order {
        if cut(i) || runs.is_empty() {
            runs.push(Vec::new());
        }
        runs.last_mut().unwrap().push(i);
    }
    // Une boucle sans coin : deux moitiés (un arc ne fait pas un tour complet).
    if first_cut.is_none() && runs.len() == 1 {
        let all = runs.pop().unwrap();
        let (a, b) = all.split_at(all.len() / 2);
        runs = vec![a.to_vec(), b.to_vec()];
    }

    let mut out = Vec::with_capacity(n);
    let mut arcs = 0;
    for run in runs {
        if run.iter().any(|&i| !is_line(i)) {
            out.extend(run.iter().map(|&i| segs[i]));
            continue;
        }
        let pts: Vec<P> = std::iter::once(segs[run[0]].start())
            .chain(run.iter().map(|&i| segs[i].end()))
            .collect();
        let mut i = 0;
        while i < run.len() {
            // L'arc le plus long qui part du point i.
            let mut best: Option<(usize, Seg)> = None;
            let mut j = i + MIN_RUN;
            while j <= run.len() {
                match arc_through(&pts[i..=j], tol, min_radius) {
                    Some(a) => best = Some((j, a)),
                    None => break,
                }
                j += 1;
            }
            match best {
                Some((j, a)) => {
                    out.push(a);
                    arcs += 1;
                    i = j;
                }
                None => {
                    out.push(segs[run[i]]);
                    i += 1;
                }
            }
        }
    }
    (out, arcs)
}
