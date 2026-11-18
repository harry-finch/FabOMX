//! Nettoyage d'un dessin : éléments minuscules, doublons, superpositions,
//! extrémités à joindre, segments alignés. Les seuils sont donnés en mm sur
//! le plateau et ramenés à la géométrie d'origine par `k` (mm du plateau
//! par mm d'origine). Rien n'est inventé : seules les jonctions sous le
//! seuil ferment des contours ; un pont n'est posé qu'entre deux arcs, et
//! il est plus court que le seuil.
//!
//! Ordre : minuscules (sur les pièces du fichier), doublons, superpositions,
//! jonctions, puis les segments sont rechaînés en contours, les lignes
//! alignées fusionnées au passage.

use crate::drawing::{Kind, Piece};
use crate::geom::{Seg, P};
use crate::import::{Grid, SegKey, Topology, JOINED};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

/// Écart toléré pour dire trois points alignés (mm du plateau).
pub const ALIGNED: f64 = 0.001;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Options {
    pub duplicates: bool,
    pub overlaps: bool,
    pub tiny: bool,
    /// Taille max d'un élément minuscule (mm du plateau).
    pub tiny_size: f64,
    pub join: bool,
    /// Écart max entre deux extrémités à joindre (mm du plateau).
    pub join_gap: f64,
    pub simplify: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            duplicates: true,
            overlaps: true,
            tiny: true,
            tiny_size: 0.1,
            join: true,
            join_gap: 0.01,
            simplify: true,
        }
    }
}

impl Options {
    fn all(self) -> Options {
        Options {
            duplicates: true,
            overlaps: true,
            tiny: true,
            join: true,
            simplify: true,
            ..self
        }
    }
}

/// Corrections faites (ou à faire), par opération.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Counts {
    pub duplicates: usize,
    pub overlaps: usize,
    pub tiny: usize,
    pub joins: usize,
    pub simplified: usize,
}

/// Un défaut situé (géométrie d'origine). `gap` : pour une extrémité qui
/// restera libre, l'écart à l'extrémité libre la plus proche (mm du plateau).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Defect {
    pub kind: &'static str,
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f64>,
}

/// Ce que ferait le nettoyage, avant de l'appliquer.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Preview {
    /// Pour une opération cochée, ce qu'elle fera ; sinon ce qu'elle ferait seule… avec les autres.
    pub counts: Counts,
    pub closed: usize,
    pub open: usize,
    pub defects: Vec<Defect>,
}

/// Un segment et le type de l'élément d'où il vient.
#[derive(Clone, Copy, Debug)]
struct E {
    s: Seg,
    kind: Kind,
    construction: bool,
}

fn reversed(s: Seg) -> Seg {
    match s {
        Seg::Line { a, b } => Seg::Line { a: b, b: a },
        Seg::Arc { c, r, a0, sweep } => Seg::Arc {
            c,
            r,
            a0: a0 + sweep,
            sweep: -sweep,
        },
    }
}

fn is_arc(s: &Seg) -> bool {
    matches!(s, Seg::Arc { .. })
}

/// Nettoie `pieces` ; rend le dessin nettoyé, les corrections et leurs positions.
pub fn clean(pieces: &[Piece], o: &Options, k: f64) -> (Vec<Piece>, Counts, Vec<Defect>) {
    let k = k.max(1e-12);
    let mut counts = Counts::default();
    let mut defects = Vec::new();
    let mut es: Vec<E> = Vec::new();

    // Minuscules : les pièces du fichier plus courtes que le seuil. Celles qui
    // touchent d'autres segments sont remplacées par la jonction de ces voisins.
    let tiny_size = o.tiny_size / k;
    let mut collapse: Vec<(P, P)> = Vec::new();
    for p in pieces {
        if o.tiny && p.length() < tiny_size {
            counts.tiny += 1;
            let m = p.segs[0].mid();
            defects.push(Defect {
                kind: "tiny",
                x: m.x,
                y: m.y,
                gap: None,
            });
            if !p.closed {
                collapse.push((p.segs[0].start(), p.segs[p.segs.len() - 1].end()));
            }
        } else {
            es.extend(p.segs.iter().map(|&s| E {
                s,
                kind: p.kind,
                construction: p.construction,
            }));
        }
    }
    for (a, b) in collapse {
        snap_pair(&mut es, a, b);
    }

    if o.duplicates {
        let mut seen = std::collections::HashSet::new();
        es.retain(|e| {
            let fresh = seen.insert(SegKey::of(&e.s));
            if !fresh {
                counts.duplicates += 1;
                let m = e.s.mid();
                defects.push(Defect {
                    kind: "duplicate",
                    x: m.x,
                    y: m.y,
                    gap: None,
                });
            }
            fresh
        });
    }

    if o.overlaps {
        let (merged, n, at) = merge_overlaps(es);
        es = merged;
        counts.overlaps += n;
        defects.extend(at.into_iter().map(|m| Defect {
            kind: "overlap",
            x: m.x,
            y: m.y,
            gap: None,
        }));
    }

    if o.join {
        let gap = o.join_gap / k;
        let free = free_ends(&es);
        let grid = Grid::new(&free, gap.max(JOINED));
        let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
        for (i, &p) in free.iter().enumerate() {
            for j in grid.near(p) {
                let d = p.dist(free[j]);
                if j > i && d <= gap && d > JOINED {
                    pairs.push((d, i, j));
                }
            }
        }
        pairs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        let mut used = vec![false; free.len()];
        for (_, i, j) in pairs {
            if !used[i] && !used[j] {
                used[i] = true;
                used[j] = true;
                counts.joins += 1;
                let m = free[i].lerp(free[j], 0.5);
                defects.push(Defect {
                    kind: "gap",
                    x: m.x,
                    y: m.y,
                    gap: None,
                });
                snap_pair(&mut es, free[i], free[j]);
            }
        }
    }

    let (out, simplified) = rechain(es, if o.simplify { Some(ALIGNED / k) } else { None });
    counts.simplified = simplified;
    (out, counts, defects)
}

/// Ce que ferait `clean`, sans rien changer : comptes, bilan, défauts, et
/// les extrémités qui resteront libres avec leur écart.
pub fn preview(pieces: &[Piece], o: &Options, k: f64) -> Preview {
    let (after, mut counts, mut defects) = clean(pieces, o, k);
    // Les opérations décochées : ce qu'elles trouveraient, toutes cochées.
    let (_, all, all_defects) = clean(pieces, &o.all(), k);
    let off = |on: bool, kind: &str| !on && ["duplicate", "overlap", "tiny", "gap"].contains(&kind);
    for d in all_defects {
        let unchecked = match d.kind {
            "duplicate" => off(o.duplicates, d.kind),
            "overlap" => off(o.overlaps, d.kind),
            "tiny" => off(o.tiny, d.kind),
            "gap" => off(o.join, d.kind),
            _ => false,
        };
        if unchecked {
            defects.push(d);
        }
    }
    if !o.duplicates {
        counts.duplicates = all.duplicates;
    }
    if !o.overlaps {
        counts.overlaps = all.overlaps;
    }
    if !o.tiny {
        counts.tiny = all.tiny;
    }
    if !o.join {
        counts.joins = all.joins;
    }
    if !o.simplify {
        counts.simplified = all.simplified;
    }
    let topo = Topology::build(&after);
    let free: Vec<P> = topo
        .nodes
        .iter()
        .filter(|n| n.degree == 1)
        .map(|n| n.p)
        .collect();
    for (i, &p) in free.iter().enumerate() {
        let gap = free
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .map(|(_, q)| p.dist(*q) * k)
            .fold(None, |m: Option<f64>, d| Some(m.map_or(d, |m| m.min(d))));
        defects.push(Defect {
            kind: "open",
            x: p.x,
            y: p.y,
            gap,
        });
    }
    Preview {
        counts,
        closed: topo.closed,
        open: topo.open,
        defects,
    }
}

/// Extrémités libres : nœuds (à `JOINED` près) touchés par un seul bout.
fn free_ends(es: &[E]) -> Vec<P> {
    let pieces: Vec<Piece> = es
        .iter()
        .map(|e| Piece {
            kind: e.kind,
            segs: vec![e.s],
            closed: false,
            construction: false,
        })
        .collect();
    Topology::build(&pieces)
        .nodes
        .into_iter()
        .filter(|n| n.degree == 1)
        .map(|n| n.p)
        .collect()
}

/// Réunit les extrémités en `a` et en `b` en un seul point. Les arcs ne
/// bougent pas : le point est l'extrémité d'un arc s'il y en a un ; entre
/// deux arcs, un pont droit (plus court que le seuil) les relie.
fn snap_pair(es: &mut Vec<E>, a: P, b: P) {
    let near = |p: P, q: P| p.dist(q) <= JOINED;
    let arc_at = |es: &[E], p: P| {
        es.iter()
            .find(|e| is_arc(&e.s) && (near(e.s.start(), p) || near(e.s.end(), p)))
            .map(|e| {
                if near(e.s.start(), p) {
                    e.s.start()
                } else {
                    e.s.end()
                }
            })
    };
    let target = match (arc_at(es, a), arc_at(es, b)) {
        (Some(pa), Some(pb)) => {
            es.push(E {
                s: Seg::Line { a: pa, b: pb },
                kind: Kind::Line,
                construction: false,
            });
            return;
        }
        (Some(pa), None) => pa,
        (None, Some(pb)) => pb,
        (None, None) => a.lerp(b, 0.5),
    };
    for e in es.iter_mut() {
        if let Seg::Line {
            a: ref mut s,
            b: ref mut t,
        } = e.s
        {
            if near(*s, a) || near(*s, b) {
                *s = target;
            }
            if near(*t, a) || near(*t, b) {
                *t = target;
            }
        }
    }
    es.retain(|e| e.s.length() > 1e-12);
}

/// Fusionne les lignes alignées qui se chevauchent, et les arcs d'un même
/// cercle qui se chevauchent. Rend les segments, le nombre de segments
/// absorbés et où.
fn merge_overlaps(es: Vec<E>) -> (Vec<E>, usize, Vec<P>) {
    let q = |v: f64, step: f64| (v / step).round() as i64;
    let mut lines: HashMap<(i64, i64), Vec<E>> = HashMap::new();
    let mut arcs: HashMap<(i64, i64, i64), Vec<E>> = HashMap::new();
    let mut out = Vec::new();
    for e in es {
        match e.s {
            Seg::Line { a, b } => {
                // Droite porteuse : direction (modulo π) et distance à l'origine.
                let mut ang = (b.y - a.y).atan2(b.x - a.x);
                if ang < 0.0 {
                    ang += PI;
                }
                if ang >= PI - 1e-9 {
                    ang -= PI;
                }
                let n = P::new(-ang.sin(), ang.cos());
                let off = a.x * n.x + a.y * n.y;
                lines
                    .entry((q(ang, 1e-7), q(off, JOINED)))
                    .or_default()
                    .push(e);
            }
            Seg::Arc { c, r, .. } => arcs
                .entry((q(c.x, JOINED), q(c.y, JOINED), q(r, JOINED)))
                .or_default()
                .push(e),
        }
    }
    let mut absorbed = 0;
    let mut at = Vec::new();
    let mut keys: Vec<_> = lines.keys().copied().collect();
    keys.sort();
    for key in keys {
        let group = &lines[&key];
        let a0 = group[0].s.start();
        let dir = {
            let e = group[0].s.end();
            let l = a0.dist(e);
            P::new((e.x - a0.x) / l, (e.y - a0.y) / l)
        };
        let t = |p: P| (p.x - a0.x) * dir.x + (p.y - a0.y) * dir.y;
        let mut iv: Vec<(f64, f64, E)> = group
            .iter()
            .map(|e| {
                let (u, v) = (t(e.s.start()), t(e.s.end()));
                (u.min(v), u.max(v), *e)
            })
            .collect();
        iv.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.total_cmp(&y.1)));
        let mut cur: Option<(f64, f64, E, usize)> = None;
        let flush = |cur: (f64, f64, E, usize), out: &mut Vec<E>| {
            if cur.3 == 0 {
                out.push(cur.2);
            } else {
                out.push(E {
                    s: Seg::Line {
                        a: a0 + dir * cur.0,
                        b: a0 + dir * cur.1,
                    },
                    kind: cur.2.kind,
                    construction: cur.2.construction,
                });
            }
        };
        for (lo, hi, e) in iv {
            match cur {
                Some((clo, chi, ce, n)) if lo < chi - JOINED => {
                    absorbed += 1;
                    at.push(a0 + dir * ((lo + chi.min(hi)) / 2.0));
                    cur = Some((clo, chi.max(hi), ce, n + 1));
                }
                Some(c) => {
                    flush(c, &mut out);
                    cur = Some((lo, hi, e, 0));
                }
                None => cur = Some((lo, hi, e, 0)),
            }
        }
        if let Some(c) = cur {
            flush(c, &mut out);
        }
    }
    let mut keys: Vec<_> = arcs.keys().copied().collect();
    keys.sort();
    for key in keys {
        // Intervalles d'angle dans le sens trigonométrique : [début, début + étendue].
        let mut iv: Vec<(f64, f64, E)> = arcs[&key]
            .iter()
            .map(|e| {
                let Seg::Arc { a0, sweep, .. } = e.s else {
                    unreachable!()
                };
                let s = if sweep >= 0.0 { a0 } else { a0 + sweep };
                (s.rem_euclid(TAU), sweep.abs(), *e)
            })
            .collect();
        let mut merged = true;
        while merged {
            merged = false;
            'outer: for i in 0..iv.len() {
                for j in i + 1..iv.len() {
                    let (si, li, ei) = iv[i];
                    let (sj, lj, _) = iv[j];
                    let Seg::Arc { r, c, .. } = ei.s else {
                        unreachable!()
                    };
                    let eps = JOINED / r;
                    // Début de j vu depuis i, ramené dans [0, 2π).
                    let d = (sj - si).rem_euclid(TAU);
                    let union = if d < li - eps {
                        Some((si, li.max(d + lj)))
                    } else {
                        let d2 = (si - sj).rem_euclid(TAU);
                        (d2 < lj - eps).then(|| (sj, lj.max(d2 + li)))
                    };
                    if let Some((s, l)) = union {
                        let l = l.min(TAU);
                        absorbed += 1;
                        at.push(crate::geom::polar(c, r, s + l / 2.0));
                        iv[i] = (
                            s,
                            l,
                            E {
                                s: Seg::Arc {
                                    c,
                                    r,
                                    a0: s,
                                    sweep: l,
                                },
                                kind: ei.kind,
                                construction: ei.construction,
                            },
                        );
                        iv.remove(j);
                        merged = true;
                        break 'outer;
                    }
                }
            }
        }
        out.extend(iv.into_iter().map(|(_, _, e)| e));
    }
    (out, absorbed, at)
}

/// Les contours du dessin, sans rien corriger : ses segments rechaînés.
pub(crate) fn contours(pieces: &[Piece]) -> Vec<Piece> {
    let es = pieces
        .iter()
        .flat_map(|p| {
            p.segs.iter().map(|&s| E {
                s,
                kind: p.kind,
                construction: p.construction,
            })
        })
        .collect();
    rechain(es, None).0
}

/// `chain`, les repères à part : un contour est tout repère ou pas du tout.
fn rechain(es: Vec<E>, aligned: Option<f64>) -> (Vec<Piece>, usize) {
    let (marks, cuts): (Vec<E>, Vec<E>) = es.into_iter().partition(|e| e.construction);
    let (mut out, n) = chain(cuts, aligned);
    let (more, m) = chain(marks, aligned);
    out.extend(more);
    (out, n + m)
}

/// Rechaîne les segments en contours : un chemin entre deux nœuds qui ne
/// relient pas exactement deux bouts, ou une boucle. Avec `aligned`, deux
/// lignes consécutives alignées à ce seuil près n'en font qu'une.
fn chain(es: Vec<E>, aligned: Option<f64>) -> (Vec<Piece>, usize) {
    // Nœuds à `JOINED` près.
    let mut reps: Vec<P> = Vec::new();
    let mut cells: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    let mut node = |p: P| -> usize {
        let key = |p: P| ((p.x / JOINED).floor() as i64, (p.y / JOINED).floor() as i64);
        let (kx, ky) = key(p);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(list) = cells.get(&(kx + dx, ky + dy)) {
                    if let Some(&n) = list.iter().find(|&&n| reps[n].dist(p) <= JOINED) {
                        return n;
                    }
                }
            }
        }
        reps.push(p);
        cells.entry((kx, ky)).or_default().push(reps.len() - 1);
        reps.len() - 1
    };
    let ends: Vec<(usize, usize)> = es
        .iter()
        .map(|e| (node(e.s.start()), node(e.s.end())))
        .collect();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); reps.len()];
    for (i, &(a, b)) in ends.iter().enumerate() {
        adj[a].push(i);
        adj[b].push(i);
    }
    let mut used = vec![false; es.len()];
    let mut out = Vec::new();
    let mut simplified = 0;

    // Parcourt depuis le nœud `n` par le segment `i` ; rend les segments orientés et le nœud d'arrivée.
    let walk = |start: usize, first: usize, used: &mut Vec<bool>| -> (Vec<E>, bool) {
        let mut segs = Vec::new();
        let (mut n, mut i) = (start, first);
        loop {
            used[i] = true;
            let (a, b) = ends[i];
            let (s, next) = if a == n {
                (es[i].s, b)
            } else {
                (reversed(es[i].s), a)
            };
            segs.push(E {
                s,
                kind: es[i].kind,
                construction: es[i].construction,
            });
            n = next;
            if n == start {
                return (segs, true);
            }
            if adj[n].len() != 2 {
                return (segs, false);
            }
            match adj[n].iter().find(|&&j| !used[j]) {
                Some(&j) => i = j,
                None => return (segs, false),
            }
        }
    };
    let emit = |segs: Vec<E>, closed: bool, out: &mut Vec<Piece>, simplified: &mut usize| {
        let segs_construction = segs[0].construction;
        // Plusieurs lignes ou arcs rechaînés font une polyligne ; les autres types se gardent.
        let kind = match segs[0].kind {
            Kind::Line | Kind::Arc if segs.len() > 1 => Kind::Polyline,
            k => k,
        };
        let mut s: Vec<Seg> = Vec::with_capacity(segs.len());
        for e in segs {
            if let (Some(tol), Some(Seg::Line { a, b: m }), Seg::Line { b, .. }) =
                (aligned, s.last().copied(), e.s)
            {
                if collinear(a, m, b, tol) {
                    *s.last_mut().unwrap() = Seg::Line { a, b };
                    *simplified += 1;
                    continue;
                }
            }
            s.push(e.s);
        }
        // Une boucle : la dernière ligne et la première peuvent aussi s'aligner.
        if let (Some(tol), true, true) = (aligned, closed, s.len() > 2) {
            if let (Seg::Line { a, b: m }, Seg::Line { b, .. }) = (s[s.len() - 1], s[0]) {
                if collinear(a, m, b, tol) {
                    s.pop();
                    s[0] = Seg::Line { a, b };
                    *simplified += 1;
                }
            }
        }
        out.push(Piece {
            kind,
            segs: s,
            closed,
            construction: segs_construction,
        });
    };
    for (n, around) in adj.iter().enumerate() {
        if around.len() == 2 {
            continue;
        }
        for &i in around {
            if !used[i] {
                let (segs, closed) = walk(n, i, &mut used);
                emit(segs, closed, &mut out, &mut simplified);
            }
        }
    }
    for i in 0..es.len() {
        if !used[i] {
            let (segs, closed) = walk(ends[i].0, i, &mut used);
            emit(segs, closed, &mut out, &mut simplified);
        }
    }
    (out, simplified)
}

/// `m` est-il sur le segment [a, b], à `tol` près ?
fn collinear(a: P, m: P, b: P, tol: f64) -> bool {
    let ab = b - a;
    let l = ab.len();
    if l < 1e-12 {
        return false;
    }
    let dev = ((m.x - a.x) * ab.y - (m.y - a.y) * ab.x).abs() / l;
    let t = ((m.x - a.x) * ab.x + (m.y - a.y) * ab.y) / (l * l);
    dev <= tol && t > 0.0 && t < 1.0
}
