//! Lecture d'un DXF texte : section ENTITIES, entités LINE, ARC, CIRCLE,
//! LWPOLYLINE, POLYLINE (arrondis compris), SPLINE et ELLIPSE. Les autres
//! entités (textes, blocs, hachures, cotes…) sont ignorées et comptées.
//! L'unité vient de `$INSUNITS` ; absente ou sans équivalent, le millimètre
//! est supposé.

use crate::drawing::{Builder, Kind, Parsed};
use crate::geom::{bulge_arc, Affine, Seg, P};
use crate::import::ImportError;
use crate::units::{Source, Unit};
use std::collections::HashMap;
use std::f64::consts::TAU;

/// Minuscules sans accents, pour comparer des noms de calques.
fn fold(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect()
}

/// Une entité : son nom et ses groupes (code, valeur), dans l'ordre du fichier.
struct Entity<'a> {
    name: &'a str,
    groups: Vec<(i32, &'a str)>,
}

impl Entity<'_> {
    fn f(&self, code: i32) -> Option<f64> {
        self.groups
            .iter()
            .find(|(c, _)| *c == code)
            .and_then(|(_, v)| v.parse().ok())
    }

    fn get(&self, code: i32) -> f64 {
        self.f(code).unwrap_or(0.0)
    }

    fn all(&self, code: i32) -> Vec<f64> {
        self.groups
            .iter()
            .filter(|(c, _)| *c == code)
            .filter_map(|(_, v)| v.parse().ok())
            .collect()
    }

    fn text(&self, code: i32) -> Option<&str> {
        self.groups
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, v)| *v)
    }

    /// Repère ou trait de construction : calque « Defpoints », calque dont le
    /// nom contient « construction » ou « repère », ou trait non continu
    /// (celui de l'entité, sinon celui de son calque).
    fn construction(&self, layers: &HashMap<String, String>) -> bool {
        let layer = self.text(8).unwrap_or("0");
        let folded = fold(layer);
        if folded == "defpoints" || folded.contains("construction") || folded.contains("repere") {
            return true;
        }
        let own = self.text(6).unwrap_or("BYLAYER");
        let ltype = if own.eq_ignore_ascii_case("BYLAYER") {
            layers.get(&layer.to_uppercase()).map_or("", String::as_str)
        } else {
            own
        };
        !(ltype.is_empty()
            || ltype.eq_ignore_ascii_case("CONTINUOUS")
            || ltype.eq_ignore_ascii_case("BYBLOCK")
            || ltype.eq_ignore_ascii_case("BYLAYER"))
    }

    fn flags(&self) -> i32 {
        self.get(70) as i32
    }

    /// Repère d'objet retourné (extrusion vers −Z) : X en miroir.
    fn ocs(&self, m: Affine) -> Affine {
        if self.f(230).is_some_and(|z| z < 0.0) {
            m.then_after(Affine([-1.0, 0.0, 0.0, 1.0, 0.0, 0.0]))
        } else {
            m
        }
    }
}

pub fn read(text: &str, unit: Option<Unit>) -> Result<(Parsed, Unit, Source), ImportError> {
    if text.starts_with("AutoCAD Binary DXF") {
        return Err(ImportError::BinaryDxf);
    }
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let mut pairs = Vec::with_capacity(lines.len() / 2);
    for c in lines.chunks(2) {
        if c.len() < 2 {
            break;
        }
        let code = c[0].parse::<i32>().map_err(|_| ImportError::Unreadable)?;
        pairs.push((code, c[1]));
    }
    if !pairs.iter().any(|&(c, v)| c == 0 && v == "SECTION") {
        return Err(ImportError::Unreadable);
    }

    // $INSUNITS : 1 po, 2 pi, 4 mm, 5 cm, 6 m.
    let mut detected = (Unit::Mm, Source::Assumed);
    for w in pairs.windows(2) {
        if w[0] == (9, "$INSUNITS") && w[1].0 == 70 {
            detected = match w[1].1.parse::<i32>().unwrap_or(0) {
                1 => (Unit::In, Source::File),
                2 => (Unit::Ft, Source::File),
                4 => (Unit::Mm, Source::File),
                5 => (Unit::Cm, Source::File),
                6 => (Unit::M, Source::File),
                _ => (Unit::Mm, Source::Assumed),
            };
        }
    }

    // Calques de la table LAYER : leur type de trait.
    let mut layers: HashMap<String, String> = HashMap::new();
    let mut k = 0;
    while k < pairs.len() {
        if pairs[k] == (0, "LAYER") {
            let (mut name, mut ltype) = ("", "");
            k += 1;
            while k < pairs.len() && pairs[k].0 != 0 {
                match pairs[k].0 {
                    2 => name = pairs[k].1,
                    6 => ltype = pairs[k].1,
                    _ => {}
                }
                k += 1;
            }
            layers.insert(name.to_uppercase(), ltype.to_string());
        } else {
            k += 1;
        }
    }

    // Les entités de la section ENTITIES, jusqu'à son ENDSEC.
    let start = pairs.iter().position(|&(c, v)| c == 2 && v == "ENTITIES");
    let mut ents: Vec<Entity> = Vec::new();
    if let Some(start) = start {
        for &(c, v) in &pairs[start + 1..] {
            if c == 0 {
                if v == "ENDSEC" {
                    break;
                }
                ents.push(Entity {
                    name: v,
                    groups: Vec::new(),
                });
            } else if let Some(e) = ents.last_mut() {
                e.groups.push((c, v));
            }
        }
    }

    let m = Affine::scale(unit.unwrap_or(detected.0).mm());
    let mut out = Parsed::default();
    let mut i = 0;
    while i < ents.len() {
        let e = &ents[i];
        i += 1;
        let mut b = Builder::new(e.ocs(m));
        let first = out.pieces.len();
        match e.name {
            "LINE" => {
                b.line(P::new(e.get(10), e.get(20)), P::new(e.get(11), e.get(21)));
                b.finish(Kind::Line, false, &mut out);
            }
            "ARC" => {
                let (a0, a1) = (e.get(50).to_radians(), e.get(51).to_radians());
                let mut sweep = (a1 - a0).rem_euclid(TAU);
                if sweep < 1e-12 {
                    sweep = TAU;
                }
                b.arc(P::new(e.get(10), e.get(20)), e.get(40), a0, sweep);
                b.finish(Kind::Arc, false, &mut out);
            }
            "CIRCLE" => {
                b.arc(P::new(e.get(10), e.get(20)), e.get(40), 0.0, TAU);
                b.finish(Kind::Circle, true, &mut out);
            }
            "ELLIPSE" => {
                let c = P::new(e.get(10), e.get(20));
                let major = P::new(e.get(11), e.get(21));
                let ratio = e.f(40).unwrap_or(1.0);
                let t0 = e.get(41);
                let mut dt = e.f(42).unwrap_or(TAU) - t0;
                if dt <= 1e-12 {
                    dt += TAU;
                }
                let full = (dt - TAU).abs() < 1e-9;
                b.ellipse(c, major.len(), major.len() * ratio, major.angle(), t0, dt);
                let kind = if (ratio - 1.0).abs() < 1e-12 {
                    if full {
                        Kind::Circle
                    } else {
                        Kind::Arc
                    }
                } else {
                    Kind::Ellipse
                };
                b.finish(kind, full, &mut out);
            }
            "LWPOLYLINE" => {
                // Chaque sommet (10, 20) peut être suivi de son arrondi (42).
                let mut v: Vec<(P, f64)> = Vec::new();
                let mut x = None;
                for &(c, val) in &e.groups {
                    let n: f64 = val.parse().unwrap_or(0.0);
                    match c {
                        10 => x = Some(n),
                        20 => {
                            if let Some(x) = x.take() {
                                v.push((P::new(x, n), 0.0));
                            }
                        }
                        42 => {
                            if let Some(last) = v.last_mut() {
                                last.1 = n;
                            }
                        }
                        _ => {}
                    }
                }
                polyline(&mut b, &v, e.flags() & 1 == 1, &mut out);
            }
            "POLYLINE" => {
                let mut v = Vec::new();
                while i < ents.len() && ents[i].name == "VERTEX" {
                    let p = &ents[i];
                    v.push((P::new(p.get(10), p.get(20)), p.get(42)));
                    i += 1;
                }
                if i < ents.len() && ents[i].name == "SEQEND" {
                    i += 1;
                }
                // Maillages et faces (drapeaux 16, 64) : pas des tracés 2D.
                if e.flags() & (16 | 64) != 0 {
                    out.ignore("POLYLINE");
                } else {
                    polyline(&mut b, &v, e.flags() & 1 == 1, &mut out);
                }
            }
            "SPLINE" => spline(e, &mut b, &mut out),
            "VERTEX" | "SEQEND" => {}
            other => out.ignore(other),
        }
        out.mark_construction_if(first, e.construction(&layers));
    }
    Ok((out, detected.0, detected.1))
}

fn polyline(b: &mut Builder, v: &[(P, f64)], closed: bool, out: &mut Parsed) {
    let n = v.len();
    let edges = if closed { n } else { n.saturating_sub(1) };
    for k in 0..edges {
        let (p, bulge) = v[k];
        let q = v[(k + 1) % n].0;
        if bulge.abs() > 1e-12 && p.dist(q) > 1e-12 {
            if let Seg::Arc { c, r, a0, sweep } = bulge_arc(p, q, bulge) {
                b.arc(c, r, a0, sweep);
            }
        } else {
            b.line(p, q);
        }
    }
    b.finish(Kind::Polyline, closed, out);
}

/// Spline : B-spline (rationnelle) par ses points de contrôle ; sans eux,
/// une ligne brisée par ses points de passage.
fn spline(e: &Entity, b: &mut Builder, out: &mut Parsed) {
    let closed = e.flags() & 1 == 1;
    let ctrl: Vec<P> = e
        .all(10)
        .into_iter()
        .zip(e.all(20))
        .map(|(x, y)| P::new(x, y))
        .collect();
    let knots = e.all(40);
    let degree = e.get(71) as usize;
    let mut weights = e.all(41);
    if weights.len() != ctrl.len() {
        weights = vec![1.0; ctrl.len()];
    }
    if degree >= 1 && ctrl.len() > degree && knots.len() == ctrl.len() + degree + 1 {
        let (t0, t1) = (knots[degree], knots[ctrl.len()]);
        let f = |t: f64| de_boor(t, degree, &knots, &ctrl, &weights);
        b.curve(&f, t0, t1, 2 * (ctrl.len() - degree));
    } else {
        let fit: Vec<P> = e
            .all(11)
            .into_iter()
            .zip(e.all(21))
            .map(|(x, y)| P::new(x, y))
            .collect();
        for w in fit.windows(2) {
            b.line(w[0], w[1]);
        }
    }
    b.finish(Kind::Spline, closed, out);
}

/// Point d'une B-spline rationnelle au paramètre `t` (algorithme de de Boor).
fn de_boor(t: f64, p: usize, knots: &[f64], ctrl: &[P], w: &[f64]) -> P {
    let n = ctrl.len();
    let mut k = p;
    while k + 1 < n && t >= knots[k + 1] {
        k += 1;
    }
    let mut d: Vec<[f64; 3]> = (0..=p)
        .map(|j| {
            let i = k + j - p;
            [ctrl[i].x * w[i], ctrl[i].y * w[i], w[i]]
        })
        .collect();
    for r in 1..=p {
        for j in (r..=p).rev() {
            let i = k + j - p;
            let den = knots[i + p + 1 - r] - knots[i];
            let a = if den.abs() < 1e-300 {
                0.0
            } else {
                (t - knots[i]) / den
            };
            for c in 0..3 {
                d[j][c] = (1.0 - a) * d[j - 1][c] + a * d[j][c];
            }
        }
    }
    let [x, y, wt] = d[p];
    P::new(x / wt, y / wt)
}
