//! Le dessin lu dans un fichier : des pièces (un élément SVG, une entité
//! DXF, un sous-chemin), chacune faite de segments exacts, en millimètres.
//! `Builder` les construit depuis les coordonnées du fichier : les lignes,
//! arcs et cercles restent exacts, les autres courbes sont mises en segments
//! à `CURVE_TOL` près.

use crate::geom::{flatten, Affine, Seg, P};
use serde::Serialize;
use std::collections::BTreeMap;
use std::f64::consts::TAU;

/// Écart maximal entre une courbe convertie et ses segments (mm).
pub const CURVE_TOL: f64 = 0.01;

/// Type d'un élément du fichier, pour les comptes du dialogue d'import.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Line,
    Arc,
    Circle,
    Ellipse,
    Polyline,
    Rect,
    Spline,
    Path,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub kind: Kind,
    pub segs: Vec<Seg>,
    /// Fermée par le fichier (`Z`, drapeau de polyligne, cercle).
    pub closed: bool,
    /// Repère ou trait de construction d'après le fichier (pointillés,
    /// calque de construction) : proposé « Ne pas couper ».
    pub construction: bool,
}

impl Piece {
    pub fn length(&self) -> f64 {
        self.segs.iter().map(Seg::length).sum()
    }
}

/// Ce que donne la lecture d'un fichier : les pièces, et les éléments
/// ignorés, comptés par nom (`text`, `INSERT`…).
#[derive(Clone, Debug, Default)]
pub struct Parsed {
    pub pieces: Vec<Piece>,
    pub ignored: BTreeMap<String, usize>,
}

impl Parsed {
    pub fn ignore(&mut self, name: &str) {
        *self.ignored.entry(name.to_string()).or_default() += 1;
    }

    /// Comme `mark_construction`, seulement si `on`.
    pub fn mark_construction_if(&mut self, from: usize, on: bool) {
        if on {
            self.mark_construction(from);
        }
    }

    /// Marque comme repères les pièces ajoutées depuis l'indice `from`.
    pub fn mark_construction(&mut self, from: usize) {
        for p in &mut self.pieces[from..] {
            p.construction = true;
        }
    }
}

/// Construit une pièce dans le repère final (mm) à partir de primitives
/// exprimées dans le repère du fichier, transformées par `m`.
pub struct Builder {
    m: Affine,
    /// Tolérance des courbes dans le repère du fichier.
    tol: f64,
    segs: Vec<Seg>,
}

impl Builder {
    pub fn new(m: Affine) -> Self {
        Builder {
            m,
            tol: CURVE_TOL / m.factor().max(1e-300),
            segs: Vec::new(),
        }
    }

    pub fn line(&mut self, a: P, b: P) {
        let (a, b) = (self.m.apply(a), self.m.apply(b));
        if a.dist(b) > 1e-12 {
            self.segs.push(Seg::Line { a, b });
        }
    }

    fn polyline(&mut self, pts: &[P]) {
        for w in pts.windows(2) {
            self.line(w[0], w[1]);
        }
    }

    /// Arc de cercle (repère du fichier) : exact si la transformation garde
    /// les cercles, sinon mis en segments.
    pub fn arc(&mut self, c: P, r: f64, a0: f64, sweep: f64) {
        if r <= 0.0 || sweep == 0.0 {
            return;
        }
        if self.m.is_similarity() {
            self.segs.push(self.m.arc(c, r, a0, sweep));
        } else {
            self.ellipse(c, r, r, 0.0, a0, sweep);
        }
    }

    /// Arc d'ellipse : centre, demi-axes, rotation de l'axe `rx`, paramètre
    /// de départ et balayage. Cercle exact quand les demi-axes sont égaux.
    pub fn ellipse(&mut self, c: P, rx: f64, ry: f64, phi: f64, t0: f64, dt: f64) {
        if (rx - ry).abs() <= 1e-12 * rx.max(ry) && self.m.is_similarity() {
            self.segs.push(self.m.arc(c, rx, t0 + phi, dt));
            return;
        }
        let (cs, sn) = (phi.cos(), phi.sin());
        let f = |t: f64| {
            let (x, y) = (rx * t.cos(), ry * t.sin());
            P::new(c.x + x * cs - y * sn, c.y + x * sn + y * cs)
        };
        let steps = ((dt.abs() / TAU) * 16.0).ceil() as usize;
        let pts = flatten(&f, t0, t0 + dt, self.tol, steps.max(2));
        self.polyline(&pts);
    }

    pub fn cubic(&mut self, p0: P, p1: P, p2: P, p3: P) {
        let f = |t: f64| {
            let u = 1.0 - t;
            p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
        };
        let pts = flatten(&f, 0.0, 1.0, self.tol, 2);
        self.polyline(&pts);
    }

    /// Courbe déjà paramétrée (spline) : `f` sur [t0, t1].
    pub fn curve(&mut self, f: &dyn Fn(f64) -> P, t0: f64, t1: f64, min_steps: usize) {
        let pts = flatten(f, t0, t1, self.tol, min_steps);
        self.polyline(&pts);
    }

    /// Termine la pièce ; rien si elle n'a aucun segment.
    pub fn finish(&mut self, kind: Kind, closed: bool, out: &mut Parsed) {
        let segs = std::mem::take(&mut self.segs);
        if !segs.is_empty() {
            out.pieces.push(Piece {
                kind,
                segs,
                closed,
                construction: false,
            });
        }
    }

    pub fn is_empty(&self) -> bool {
        self.segs.is_empty()
    }
}
