//! Placement d'un dessin sur le plateau : la transformation (une matrice
//! affine) s'applique toujours à la géométrie d'origine, jamais à un
//! résultat déjà transformé. Les similitudes gardent lignes, arcs et cercles
//! exacts ; une échelle non proportionnelle change les arcs en ellipses,
//! mises en segments à `CURVE_TOL` près.

use crate::drawing::{Builder, Parsed, Piece};
use crate::geom::{Affine, BBox, Seg, P};
use crate::machine::Machine;
use serde::Serialize;

/// Le dessin transformé.
pub fn transform(pieces: &[Piece], m: Affine) -> Vec<Piece> {
    let mut out = Parsed::default();
    for p in pieces {
        let mut b = Builder::new(m);
        for s in &p.segs {
            match *s {
                Seg::Line { a, b: e } => b.line(a, e),
                Seg::Arc { c, r, a0, sweep } => b.arc(c, r, a0, sweep),
            }
        }
        b.finish(p.kind, p.closed, &mut out);
    }
    out.pieces
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "code")]
pub enum PlaceWarning {
    /// Le dessin sort du plateau : de combien, en X et en Y (mm).
    #[serde(rename = "place.overflow")]
    Overflow { x: f64, y: f64 },
    /// Échelle non proportionnelle : arcs et cercles devenus ellipses.
    #[serde(rename = "place.distorted")]
    Distorted,
}

/// Ce que l'interface montre d'un dessin placé.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Placement {
    pub bbox: [f64; 4],
    pub width: f64,
    pub height: f64,
    pub warnings: Vec<PlaceWarning>,
}

pub fn place(pieces: &[Piece], m: Affine, machine: &Machine) -> Placement {
    let mut b = BBox::EMPTY;
    for s in transform(pieces, m).iter().flat_map(|p| &p.segs) {
        s.extend(&mut b);
    }
    let mut warnings = Vec::new();
    let over = |lo: f64, hi: f64, size: f64| (-lo).max(0.0) + (hi - size).max(0.0);
    let (ox, oy) = (
        over(b.x0, b.x1, machine.width),
        over(b.y0, b.y1, machine.height),
    );
    if ox > 1e-9 || oy > 1e-9 {
        warnings.push(PlaceWarning::Overflow { x: ox, y: oy });
    }
    let has_arcs = pieces
        .iter()
        .flat_map(|p| &p.segs)
        .any(|s| matches!(s, Seg::Arc { .. }));
    if has_arcs && !m.is_similarity() {
        warnings.push(PlaceWarning::Distorted);
    }
    Placement {
        bbox: [b.x0, b.y0, b.x1, b.y1],
        width: b.width(),
        height: b.height(),
        warnings,
    }
}

/// Points d'accrochage d'un dessin (géométrie d'origine) : extrémités des
/// segments, centres des arcs et des cercles, sans doublon.
pub fn snap_points(pieces: &[Piece]) -> Vec<P> {
    let mut out: Vec<P> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut push = |p: P| {
        if seen.insert(((p.x * 1e4).round() as i64, (p.y * 1e4).round() as i64)) {
            out.push(p);
        }
    };
    for s in pieces.iter().flat_map(|p| &p.segs) {
        push(s.start());
        push(s.end());
        if let Seg::Arc { c, .. } = *s {
            push(c);
        }
    }
    out
}
