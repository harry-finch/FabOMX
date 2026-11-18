//! Côtés de coupe : pour chaque contour d'un dessin, de quel côté du trait
//! est la pièce. Détection par imbrication, dessin par dessin : un contour
//! fermé dans aucun autre est Extérieur, puis le côté alterne à chaque
//! niveau (trou, pièce dans le trou…). Un tracé ouvert, ou un contour qui
//! se croise ou en croise un autre, est coupé sur la ligne ; un repère
//! (d'après le fichier) n'est pas coupé. Le sens de parcours est fixé : la
//! pièce reste à gauche du jet.
//!
//! Qualité de coupe : de 1 (rapide, bord rugueux) à 5 (lent, bord net), celle
//! du contour s'il en a une, sinon la qualité par défaut ; un contour qui
//! n'est pas coupé n'en a pas.

use crate::drawing::Piece;
use crate::geom::{Affine, BBox, P};
use serde::{Deserialize, Serialize};

/// Tolérance des contours mis en segments pour ces calculs (mm d'origine).
const TOL: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    /// La pièce est dedans.
    Outside,
    /// La pièce est autour.
    Inside,
    /// Fente : coupé sur le trait, sans décalage.
    Line,
    /// Repère ou trait de construction : pas coupé.
    Skip,
}

/// Un contour mis en segments, dans le repère d'origine du dessin.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Contour {
    pub closed: bool,
    /// Repère d'après le fichier.
    pub construction: bool,
    /// Côté détecté.
    pub side: Side,
    /// Niveau d'imbrication (contours fermés qui le contiennent).
    pub depth: usize,
    /// Aire signée (mm², positive dans le sens trigonométrique) ; 0 si ouvert.
    pub area: f64,
    /// Premier croisement trouvé, s'il y en a un.
    pub crossing: Option<[f64; 2]>,
    /// Points `[x, y, …]` dans le sens du tracé ; un contour fermé ne répète pas le premier.
    pub pts: Vec<f64>,
}

impl Contour {
    fn points(&self) -> Vec<P> {
        self.pts.chunks(2).map(|c| P::new(c[0], c[1])).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "code")]
pub enum CutWarning {
    /// Un contour se croise lui-même ou en croise un autre : coupé sur la ligne.
    #[serde(rename = "cut.crossing")]
    Crossing { contour: usize, x: f64, y: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Sides {
    pub contours: Vec<Contour>,
    pub warnings: Vec<CutWarning>,
}

/// Qualités de coupe valides, comme dans LAYOUT.
pub const QUALITIES: std::ops::RangeInclusive<u8> = 1..=5;

/// Refus d'un réglage de découpe, par code stable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutError {
    /// Qualité hors de 1 à 5.
    Quality,
}

impl CutError {
    pub fn code(self) -> &'static str {
        match self {
            CutError::Quality => "cut.quality",
        }
    }
}

/// La qualité `q` si elle est valide.
pub fn check_quality(q: u8) -> Result<u8, CutError> {
    if QUALITIES.contains(&q) {
        Ok(q)
    } else {
        Err(CutError::Quality)
    }
}

/// Qualité retenue d'un contour de côté `side` : la sienne (`own`), sinon
/// `default` ; aucune s'il n'est pas coupé. Une qualité invalide est refusée.
pub fn quality(side: Side, own: Option<u8>, default: u8) -> Result<Option<u8>, CutError> {
    let default = check_quality(default)?;
    let own = own.map(check_quality).transpose()?;
    Ok((side != Side::Skip).then(|| own.unwrap_or(default)))
}

/// Les contours d'un dessin et leurs côtés détectés.
pub fn sides(pieces: &[Piece]) -> Sides {
    let mut contours: Vec<Contour> = crate::clean::contours(pieces)
        .into_iter()
        .map(|p| {
            let mut pts: Vec<P> = Vec::new();
            for s in &p.segs {
                let before = pts.len();
                s.points(TOL, &mut pts);
                if before > 0 {
                    pts.remove(before);
                }
            }
            if p.closed && pts.len() > 1 {
                pts.pop();
            }
            let area = if p.closed { area(&pts) } else { 0.0 };
            Contour {
                closed: p.closed,
                construction: p.construction,
                side: Side::Line,
                depth: 0,
                area,
                crossing: None,
                pts: pts.into_iter().flat_map(|q| [q.x, q.y]).collect(),
            }
        })
        .collect();

    let polys: Vec<Vec<P>> = contours.iter().map(Contour::points).collect();
    let cut: Vec<usize> = (0..contours.len())
        .filter(|&i| !contours[i].construction)
        .collect();

    // Croisements, entre contours coupés seulement.
    let mut edges = Vec::new();
    for &i in &cut {
        push_edges(&mut edges, i, &polys[i], contours[i].closed);
    }
    let closed_of: Vec<bool> = contours.iter().map(|c| c.closed).collect();
    let found = crossings(&mut edges, &|x: &Edge, y: &Edge| {
        x.contour == y.contour && adjacent(x, y, polys[x.contour].len(), closed_of[x.contour])
    });
    for (a, b, at) in found {
        for c in [a, b] {
            contours[c].crossing.get_or_insert([at.x, at.y]);
        }
    }

    // Imbrication des contours fermés coupés.
    let closed: Vec<usize> = cut
        .iter()
        .copied()
        .filter(|&i| contours[i].closed)
        .collect();
    let boxes: Vec<BBox> = polys.iter().map(|p| bbox(p)).collect();
    for &a in &closed {
        let q = sample(&polys[a]);
        let depth = closed
            .iter()
            .filter(|&&b| {
                b != a
                    && contours[b].area.abs() > contours[a].area.abs()
                    && contains_box(&boxes[b], &boxes[a])
                    && inside(q, &polys[b])
            })
            .count();
        contours[a].depth = depth;
    }

    let mut warnings = Vec::new();
    for (i, c) in contours.iter_mut().enumerate() {
        c.side = if c.construction {
            Side::Skip
        } else if !c.closed || c.crossing.is_some() {
            Side::Line
        } else if c.depth % 2 == 0 {
            Side::Outside
        } else {
            Side::Inside
        };
        if let Some([x, y]) = c.crossing {
            warnings.push(CutWarning::Crossing { contour: i, x, y });
        }
    }
    Sides { contours, warnings }
}

/// Faut-il parcourir le contour à l'envers de son tracé, pour qu'avec
/// `side` la pièce reste à gauche du jet ? Extérieur dans le sens
/// trigonométrique, Trou dans le sens horaire, sur le plateau : une
/// matrice de déterminant négatif (miroir) retourne le sens du dessin.
pub fn reversed(side: Side, area: f64, det: f64) -> bool {
    let ccw = area * det.signum() > 0.0;
    match side {
        Side::Outside => !ccw,
        Side::Inside => ccw,
        Side::Line | Side::Skip => false,
    }
}

/// Deux dessins placés (contours de `sides`, matrices) se chevauchent-ils ?
/// Oui si leurs traits se croisent, ou si un trait de l'un est dans la
/// matière de l'autre. Les repères ne comptent pas.
pub fn overlap(a: &[Contour], ma: Affine, b: &[Contour], mb: Affine) -> bool {
    let place = |cs: &[Contour], m: Affine| -> Vec<(Vec<P>, bool)> {
        cs.iter()
            .filter(|c| !c.construction)
            .map(|c| {
                (
                    c.points().into_iter().map(|p| m.apply(p)).collect(),
                    c.closed,
                )
            })
            .collect()
    };
    let (pa, pb) = (place(a, ma), place(b, mb));
    let total = |ps: &[(Vec<P>, bool)]| {
        let mut bb = BBox::EMPTY;
        for (p, _) in ps {
            for &q in p {
                bb.add(q);
            }
        }
        bb
    };
    let (ba, bb) = (total(&pa), total(&pb));
    if ba.is_empty()
        || bb.is_empty()
        || ba.x1 < bb.x0
        || bb.x1 < ba.x0
        || ba.y1 < bb.y0
        || bb.y1 < ba.y0
    {
        return false;
    }
    let mut edges = Vec::new();
    for (i, (p, closed)) in pa.iter().enumerate() {
        push_edges(&mut edges, i, p, *closed);
    }
    let shift = pa.len();
    for (i, (p, closed)) in pb.iter().enumerate() {
        push_edges(&mut edges, shift + i, p, *closed);
    }
    let crossed = crossings(&mut edges, &|x: &Edge, y: &Edge| {
        (x.contour < shift) == (y.contour < shift)
    });
    if !crossed.is_empty() {
        return true;
    }
    // Dans la matière : contenu par un nombre impair de contours fermés de l'autre.
    let in_material = |q: P, ps: &[(Vec<P>, bool)]| {
        ps.iter()
            .filter(|(p, closed)| *closed && inside(q, p))
            .count()
            % 2
            == 1
    };
    pb.iter()
        .any(|(p, _)| !p.is_empty() && in_material(sample(p), &pa))
        || pa
            .iter()
            .any(|(p, _)| !p.is_empty() && in_material(sample(p), &pb))
}

/// Une arête d'un contour mis en segments.
struct Edge {
    contour: usize,
    index: usize,
    a: P,
    b: P,
}

fn push_edges(out: &mut Vec<Edge>, contour: usize, pts: &[P], closed: bool) {
    let n = pts.len();
    let count = if closed && n > 2 {
        n
    } else {
        n.saturating_sub(1)
    };
    for index in 0..count {
        out.push(Edge {
            contour,
            index,
            a: pts[index],
            b: pts[(index + 1) % n],
        });
    }
}

/// Deux arêtes voisines d'un même contour (elles se touchent par construction).
fn adjacent(x: &Edge, y: &Edge, n: usize, closed: bool) -> bool {
    let d = x.index.abs_diff(y.index);
    d <= 1 || (closed && d == n - 1)
}

/// Les croisements francs (à l'intérieur des deux arêtes) entre arêtes que
/// `skip` ne met pas de côté : (contour, contour, point).
fn crossings(edges: &mut [Edge], skip: &dyn Fn(&Edge, &Edge) -> bool) -> Vec<(usize, usize, P)> {
    let lo = |e: &Edge| e.a.x.min(e.b.x);
    edges.sort_by(|x, y| lo(x).total_cmp(&lo(y)));
    let mut out = Vec::new();
    for i in 0..edges.len() {
        let x = &edges[i];
        let x_hi = x.a.x.max(x.b.x);
        let (y0, y1) = (x.a.y.min(x.b.y), x.a.y.max(x.b.y));
        for y in &edges[i + 1..] {
            if lo(y) > x_hi {
                break;
            }
            if y.a.y.max(y.b.y) < y0 || y.a.y.min(y.b.y) > y1 || skip(x, y) {
                continue;
            }
            if let Some(p) = proper(x.a, x.b, y.a, y.b) {
                out.push((x.contour, y.contour, p));
            }
        }
    }
    out
}

/// Point de croisement de [a, b] et [c, d], s'il est à l'intérieur des deux.
fn proper(a: P, b: P, c: P, d: P) -> Option<P> {
    let r = b - a;
    let s = d - c;
    let den = r.x * s.y - r.y * s.x;
    let scale = r.len() * s.len();
    if den.abs() <= 1e-12 * scale.max(1e-300) {
        return None;
    }
    let w = c - a;
    let t = (w.x * s.y - w.y * s.x) / den;
    let u = (w.x * r.y - w.y * r.x) / den;
    const EPS: f64 = 1e-9;
    (t > EPS && t < 1.0 - EPS && u > EPS && u < 1.0 - EPS).then(|| a + r * t)
}

fn area(pts: &[P]) -> f64 {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let (p, q) = (pts[i], pts[(i + 1) % n]);
            p.x * q.y - q.x * p.y
        })
        .sum::<f64>()
        / 2.0
}

fn bbox(pts: &[P]) -> BBox {
    let mut b = BBox::EMPTY;
    for &p in pts {
        b.add(p);
    }
    b
}

fn contains_box(outer: &BBox, inner: &BBox) -> bool {
    outer.x0 <= inner.x0 && outer.y0 <= inner.y0 && outer.x1 >= inner.x1 && outer.y1 >= inner.y1
}

/// Un point du contour pour tester s'il est dans un autre : le milieu de sa
/// première arête (un sommet tombe plus souvent sur un trait voisin).
fn sample(pts: &[P]) -> P {
    if pts.len() > 1 {
        pts[0].lerp(pts[1], 0.5)
    } else {
        pts[0]
    }
}

/// Le point `q` est-il dans le polygone `poly` (pair-impair) ?
fn inside(q: P, poly: &[P]) -> bool {
    let n = poly.len();
    let mut on = false;
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a.y > q.y) != (b.y > q.y) && q.x < a.x + (q.y - a.y) * (b.x - a.x) / (b.y - a.y) {
            on = !on;
        }
    }
    on
}
