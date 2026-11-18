//! Parcours complet, tous dessins confondus, sur le plateau : depuis le
//! point de départ, pour chaque contour coupé dans l'ordre, un déplacement
//! jet coupé jusqu'au perçage, l'amorce d'entrée, la coupe, l'amorce de
//! sortie ; puis le retour au point de départ.
//!
//! Ordre : un contour est toujours coupé après tous ceux qu'il contient, dans
//! le même dessin (trous et fentes avant l'extérieur de leur pièce, pièce
//! dans un trou avant ce trou). Automatique : parmi les contours permis, le
//! perçage le plus proche de la buse. Un ordre donné à la main est respecté
//! s'il tient cette règle, sinon refusé (`path.order`) ; les contours qu'il
//! ne cite pas suivent, dans l'ordre automatique.
//!
//! Sur la machine, le départ d'un OMX est là où se trouve la buse quand
//! l'usinage est lancé (vérifié par l'utilisateur) : `start` en sera le
//! premier point.
//!
//! Un déplacement qui passe au-dessus d'une pièce ou d'un trou déjà
//! découpés (hors le contour qu'il quitte) est signalé (`path.over_cut`).

use crate::cut::{inside, Contour, Side};
use crate::geom::{Affine, Seg, P};
use crate::lead::{route, Lead, LeadWarning};
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

/// Un dessin placé, tel que le parcours le voit.
pub struct Job<'a> {
    /// Identifiant du dessin (celui de l'interface).
    pub key: u32,
    pub m: Affine,
    pub contours: &'a [Contour],
    /// Côté retenu de chaque contour.
    pub sides: &'a [Side],
    /// Amorces de chaque contour (rien s'il n'est pas coupé).
    pub leads: &'a [Option<Lead>],
    /// Qualité retenue de chaque contour.
    pub qualities: &'a [Option<u8>],
    /// Amorces trop courtes, reprises dans les remarques du parcours.
    pub warnings: &'a [LeadWarning],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveKind {
    Traverse,
    LeadIn,
    Cut,
    LeadOut,
}

/// Un mouvement de la buse, sur le plateau.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Move {
    pub kind: MoveKind,
    /// Rang du contour dans l'ordre (le déplacement qui y mène compris) ; aucun pour le retour.
    pub step: Option<usize>,
    pub side: Option<Side>,
    pub quality: Option<u8>,
    /// Segments exacts (lignes et arcs) sur le plateau ; pour l'affichage, des points `pts`.
    #[serde(rename = "pts", serialize_with = "flat")]
    pub segs: Vec<Seg>,
}

impl Move {
    pub fn length(&self) -> f64 {
        self.segs.iter().map(Seg::length).sum()
    }
}

/// Points d'une suite de segments, à 0,01 mm près.
pub(crate) fn points(segs: &[Seg]) -> Vec<P> {
    let mut pts = Vec::new();
    for s in segs {
        let before = pts.len();
        s.points(0.01, &mut pts);
        if before > 0 {
            pts.remove(before);
        }
    }
    pts
}

fn flat<S: Serializer>(segs: &[Seg], s: S) -> Result<S::Ok, S::Error> {
    s.collect_seq(points(segs).into_iter().flat_map(|p| [p.x, p.y]))
}

/// Un contour dans l'ordre de coupe.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Step {
    pub key: u32,
    pub contour: usize,
    pub side: Side,
    pub pierce: [f64; 2],
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Stats {
    /// Longueur coupée par qualité (mm), amorces à part.
    pub cut: BTreeMap<u8, f64>,
    pub leads: f64,
    pub traverse: f64,
    pub pierces: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "code")]
pub enum PathWarning {
    /// Le déplacement vers le contour `step` passe au-dessus de ce qui est déjà découpé.
    #[serde(rename = "path.over_cut")]
    OverCut { step: usize, x: f64, y: f64 },
    /// Une amorce du contour (`key`, `contour`) ne tient pas, même raccourcie (sur le plateau).
    #[serde(rename = "lead.too_short")]
    LeadTooShort {
        key: u32,
        contour: usize,
        x: f64,
        y: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Plan {
    pub start: [f64; 2],
    pub order: Vec<Step>,
    pub moves: Vec<Move>,
    pub stats: Stats,
    pub warnings: Vec<PathWarning>,
}

/// Ordre refusé : `contour` serait coupé avant `inner`, qu'il contient.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderError {
    pub contour: (u32, usize),
    pub inner: (u32, usize),
}

impl OrderError {
    pub fn code(&self) -> &'static str {
        "path.order"
    }
}

/// Un contour coupé, prêt pour le parcours (plateau).
struct Item {
    key: u32,
    contour: usize,
    side: Side,
    quality: Option<u8>,
    pierce: P,
    lead_in: Vec<Seg>,
    cut: Vec<Seg>,
    lead_out: Vec<Seg>,
    /// Polygone fermé (plateau), pour le survol ; vide pour un tracé ouvert.
    poly: Vec<P>,
    /// Indices (dans `items`) des contours qu'il contient.
    inner: Vec<usize>,
}

/// Segments du repère d'origine placés sur le plateau (arcs exacts pour une similitude).
fn placed(segs: &[Seg], m: Affine) -> Vec<Seg> {
    let mut out = Vec::new();
    for s in segs {
        s.transformed(m, 0.01, &mut out);
    }
    out
}

fn items(jobs: &[Job]) -> Vec<Item> {
    let mut out: Vec<Item> = Vec::new();
    for job in jobs {
        let first = out.len();
        for (i, c) in job.contours.iter().enumerate() {
            let side = job.sides.get(i).copied().unwrap_or(c.side);
            let Some(Some(lead)) = job.leads.get(i) else {
                continue;
            };
            if side == Side::Skip {
                continue;
            }
            let at = |p: [f64; 2]| job.m.apply(P::new(p[0], p[1]));
            out.push(Item {
                key: job.key,
                contour: i,
                side,
                quality: job.qualities.get(i).copied().flatten(),
                pierce: at(lead.pierce),
                lead_in: placed(&lead.lead_in, job.m),
                cut: placed(&route(c, lead), job.m),
                lead_out: placed(&lead.lead_out, job.m),
                poly: if c.closed {
                    c.points().into_iter().map(|p| job.m.apply(p)).collect()
                } else {
                    Vec::new()
                },
                inner: Vec::new(),
            });
        }
        // Contenance dans le dessin : un contour fermé contient ceux dont un
        // point est à l'intérieur et qui sont plus petits que lui.
        for b in first..out.len() {
            if out[b].poly.is_empty() {
                continue;
            }
            let area_b = crate::cut::area(&out[b].poly).abs();
            let inner: Vec<usize> = (first..out.len())
                .filter(|&a| {
                    a != b && {
                        let probe = out[a].cut.first().map_or(out[a].pierce, Seg::mid);
                        let area_a = if out[a].poly.is_empty() {
                            0.0
                        } else {
                            crate::cut::area(&out[a].poly).abs()
                        };
                        area_a < area_b && inside(probe, &out[b].poly)
                    }
                })
                .collect();
            out[b].inner = inner;
        }
    }
    out
}

/// Le parcours des dessins `jobs`, depuis `start` (plateau), dans l'ordre
/// donné (`order`, par `(clé, contour)`) ou automatique.
pub fn plan(jobs: &[Job], start: P, order: Option<&[(u32, usize)]>) -> Result<Plan, OrderError> {
    let items = items(jobs);
    let n = items.len();
    let index = |k: u32, c: usize| items.iter().position(|it| it.key == k && it.contour == c);
    let mut done = vec![false; n];
    let mut seq: Vec<usize> = Vec::with_capacity(n);
    let mut at = start;

    // L'ordre donné, d'abord, s'il tient la règle.
    for &(k, c) in order.unwrap_or(&[]) {
        let Some(i) = index(k, c) else { continue };
        if done[i] {
            continue;
        }
        if let Some(&j) = items[i].inner.iter().find(|&&j| !done[j]) {
            return Err(OrderError {
                contour: (k, c),
                inner: (items[j].key, items[j].contour),
            });
        }
        done[i] = true;
        seq.push(i);
        at = end_of(&items[i]);
    }
    // Puis l'ordre automatique : le perçage permis le plus proche.
    while seq.len() < n {
        let next = (0..n)
            .filter(|&i| !done[i] && items[i].inner.iter().all(|&j| done[j]))
            .min_by(|&a, &b| {
                at.dist(items[a].pierce)
                    .total_cmp(&at.dist(items[b].pierce))
                    .then(items[a].key.cmp(&items[b].key))
                    .then(items[a].contour.cmp(&items[b].contour))
            });
        // Une contenance circulaire ne devrait pas exister ; au pire, le premier restant.
        let i = next.unwrap_or_else(|| (0..n).find(|&i| !done[i]).unwrap());
        done[i] = true;
        seq.push(i);
        at = end_of(&items[i]);
    }

    let mut moves = Vec::new();
    let mut stats = Stats::default();
    let mut warnings: Vec<PathWarning> = jobs
        .iter()
        .flat_map(|j| {
            j.warnings.iter().map(move |w| {
                let LeadWarning::TooShort { contour, x, y } = *w;
                let p = j.m.apply(P::new(x, y));
                PathWarning::LeadTooShort {
                    key: j.key,
                    contour,
                    x: p.x,
                    y: p.y,
                }
            })
        })
        .collect();
    let mut at = start;
    for (step, &i) in seq.iter().enumerate() {
        let it = &items[i];
        let traverse = vec![Seg::Line {
            a: at,
            b: it.pierce,
        }];
        // Survol : les contours fermés déjà coupés.
        if let Some(p) = over(&traverse, cut_polys(&items, &seq[..step], at)) {
            warnings.push(PathWarning::OverCut {
                step,
                x: p.x,
                y: p.y,
            });
        }
        stats.traverse += at.dist(it.pierce);
        moves.push(Move {
            kind: MoveKind::Traverse,
            step: Some(step),
            side: None,
            quality: None,
            segs: traverse,
        });
        stats.pierces += 1;
        for (kind, pts) in [
            (MoveKind::LeadIn, &it.lead_in),
            (MoveKind::Cut, &it.cut),
            (MoveKind::LeadOut, &it.lead_out),
        ] {
            if pts.is_empty() {
                continue;
            }
            let mv = Move {
                kind,
                step: Some(step),
                side: Some(it.side),
                quality: it.quality,
                segs: pts.clone(),
            };
            let l = mv.length();
            if kind == MoveKind::Cut {
                *stats.cut.entry(it.quality.unwrap_or(0)).or_default() += l;
            } else {
                stats.leads += l;
            }
            moves.push(mv);
        }
        at = end_of(it);
    }
    // Retour au point de départ.
    if !seq.is_empty() {
        stats.traverse += at.dist(start);
        let back = vec![Seg::Line { a: at, b: start }];
        if let Some(p) = over(&back, cut_polys(&items, &seq, at)) {
            warnings.push(PathWarning::OverCut {
                step: seq.len(),
                x: p.x,
                y: p.y,
            });
        }
        moves.push(Move {
            kind: MoveKind::Traverse,
            step: None,
            side: None,
            quality: None,
            segs: back,
        });
    }

    Ok(Plan {
        start: [start.x, start.y],
        order: seq
            .iter()
            .map(|&i| Step {
                key: items[i].key,
                contour: items[i].contour,
                side: items[i].side,
                pierce: [items[i].pierce.x, items[i].pierce.y],
            })
            .collect(),
        moves,
        stats,
        warnings,
    })
}

/// Les polygones des contours déjà coupés (`cut`, dans l'ordre), sauf le
/// dernier si la buse en part de l'intérieur (un trou qu'elle quitte).
fn cut_polys<'a>(items: &'a [Item], cut: &'a [usize], from: P) -> impl Iterator<Item = &'a Vec<P>> {
    let last = cut.last().copied();
    cut.iter()
        .filter(move |&&j| !(Some(j) == last && inside(from, &items[j].poly)))
        .map(move |&j| &items[j].poly)
}

/// Où la buse se trouve après un contour : la fin de son amorce de sortie,
/// sinon de sa coupe.
fn end_of(it: &Item) -> P {
    it.lead_out
        .last()
        .or(it.cut.last())
        .map_or(it.pierce, Seg::end)
}

/// Le premier point où le déplacement `[a, b]` passe au-dessus d'un des
/// polygones (croise son bord, ou y est tout entier).
fn over<'a>(line: &[Seg], polys: impl Iterator<Item = &'a Vec<P>>) -> Option<P> {
    let (a, b) = (line[0].start(), line[0].end());
    for poly in polys {
        let n = poly.len();
        if n < 3 {
            continue;
        }
        for j in 0..n {
            if let Some(p) = crate::cut::proper(a, b, poly[j], poly[(j + 1) % n]) {
                return Some(p);
            }
        }
        let mid = a.lerp(b, 0.5);
        if a.dist(b) > 1e-9 && inside(mid, poly) {
            return Some(mid);
        }
    }
    None
}
