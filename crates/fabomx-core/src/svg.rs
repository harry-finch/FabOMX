//! Lecture d'un SVG : chemins, formes simples, groupes et transformations.
//! Les textes, images et réutilisations (`use`) sont ignorés et comptés.
//!
//! L'unité est celle de `width` (sinon le pixel, supposé). Les coordonnées
//! sont ramenées en nombres de cette unité par la `viewBox`, puis en mm par
//! l'unité retenue : changer d'unité revient à relire `width` dans une autre.

use crate::drawing::{Builder, Kind, Parsed};
use crate::geom::{Affine, P};
use crate::import::ImportError;
use crate::units::{Source, Unit};
use std::f64::consts::{PI, TAU};
use std::str::FromStr;

/// Éléments qui ne dessinent rien par eux-mêmes.
const SKIPPED: &[&str] = &[
    "defs",
    "clipPath",
    "mask",
    "symbol",
    "metadata",
    "pattern",
    "marker",
    "title",
    "desc",
    "style",
    "script",
    "linearGradient",
    "radialGradient",
    "filter",
];
/// Éléments ignorés et signalés.
const IGNORED: &[&str] = &["text", "image", "use", "foreignObject"];

pub fn read(text: &str, unit: Option<Unit>) -> Result<(Parsed, Unit, Source), ImportError> {
    let doc = roxmltree::Document::parse(text).map_err(|_| ImportError::Unreadable)?;
    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        return Err(ImportError::Unreadable);
    }
    let length = |k: &str| {
        root.attribute(k)
            .and_then(|v| svgtypes::Length::from_str(v).ok())
    };
    let (w, h) = (length("width"), length("height"));
    use svgtypes::LengthUnit as U;
    let (detected, source) = match w.map(|l| l.unit) {
        Some(U::Mm) => (Unit::Mm, Source::File),
        Some(U::Cm) => (Unit::Cm, Source::File),
        Some(U::In) => (Unit::In, Source::File),
        Some(U::Pt) => (Unit::Pt, Source::File),
        Some(U::Pc) => (Unit::Pc, Source::File),
        Some(U::Px) => (Unit::Px, Source::File),
        _ => (Unit::Px, Source::Assumed),
    };
    let vb: Option<[f64; 4]> = root.attribute("viewBox").and_then(|v| {
        let n: Vec<f64> = v
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        (n.len() == 4 && n[2] > 0.0 && n[3] > 0.0).then(|| [n[0], n[1], n[2], n[3]])
    });
    // Unités utilisateur → nombres de l'unité du fichier, Y retourné.
    let (kx, ky, tx, ty) = match (vb, w, h) {
        (Some(b), Some(w), Some(h)) if w.unit != U::Percent && h.unit != U::Percent => {
            (w.number / b[2], h.number / b[3], b[0], b[1])
        }
        (Some(b), _, _) => (1.0, 1.0, b[0], b[1]),
        _ => {
            let k = Unit::Px.mm() / detected.mm();
            (k, k, 0.0, 0.0)
        }
    };
    let k = unit.unwrap_or(detected).mm();
    let base = Affine([k * kx, 0.0, 0.0, -k * ky, -k * kx * tx, k * ky * ty]);

    let mut out = Parsed::default();
    walk(root, base, &mut out);
    Ok((out, detected, source))
}

fn hidden(node: roxmltree::Node) -> bool {
    node.attribute("display") == Some("none")
        || node
            .attribute("style")
            .is_some_and(|s| s.replace(' ', "").contains("display:none"))
}

/// Trait en pointillés : `stroke-dasharray` (attribut ou style) autre que
/// `none`, sur l'élément ou hérité d'un parent.
fn dashed(node: roxmltree::Node) -> bool {
    for n in node.ancestors().filter(|n| n.is_element()) {
        let style = n.attribute("style").and_then(|s| {
            s.split(';').find_map(|decl| {
                let (k, v) = decl.split_once(':')?;
                (k.trim() == "stroke-dasharray").then(|| v.trim())
            })
        });
        if let Some(v) = style.or(n.attribute("stroke-dasharray")) {
            let v = v.trim();
            if v != "inherit" {
                return !(v.is_empty() || v == "none");
            }
        }
    }
    false
}

fn walk(node: roxmltree::Node, parent: Affine, out: &mut Parsed) {
    for child in node.children().filter(|n| n.is_element()) {
        let name = child.tag_name().name();
        let foreign = child
            .tag_name()
            .namespace()
            .is_some_and(|ns| ns != "http://www.w3.org/2000/svg");
        if foreign || SKIPPED.contains(&name) || hidden(child) {
            continue;
        }
        if IGNORED.contains(&name) {
            out.ignore(name);
            continue;
        }
        let m = match child
            .attribute("transform")
            .and_then(|t| svgtypes::Transform::from_str(t).ok())
        {
            Some(t) => parent.then_after(Affine([t.a, t.b, t.c, t.d, t.e, t.f])),
            None => parent,
        };
        match name {
            "g" | "a" | "switch" | "svg" => walk(child, m, out),
            _ => {
                let n = out.pieces.len();
                shape(child, m, out);
                out.mark_construction_if(n, dashed(child));
            }
        }
    }
}

fn shape(node: roxmltree::Node, m: Affine, out: &mut Parsed) {
    let num = |k: &str| {
        node.attribute(k)
            .and_then(|v| svgtypes::Length::from_str(v).ok())
            .map(|l| l.number)
    };
    let get = |k: &str| num(k).unwrap_or(0.0);
    let mut b = Builder::new(m);
    match node.tag_name().name() {
        "path" => {
            if let Some(d) = node.attribute("d") {
                path(d, m, out);
            }
        }
        "line" => {
            b.line(P::new(get("x1"), get("y1")), P::new(get("x2"), get("y2")));
            b.finish(Kind::Line, false, out);
        }
        "polyline" | "polygon" => {
            let closed = node.tag_name().name() == "polygon";
            let pts: Vec<P> = node
                .attribute("points")
                .map(|s| {
                    svgtypes::PointsParser::from(s)
                        .map(|(x, y)| P::new(x, y))
                        .collect()
                })
                .unwrap_or_default();
            for w in pts.windows(2) {
                b.line(w[0], w[1]);
            }
            if closed && pts.len() > 2 {
                b.line(pts[pts.len() - 1], pts[0]);
            }
            b.finish(Kind::Polyline, closed, out);
        }
        "rect" => {
            let (x, y, w, h) = (get("x"), get("y"), get("width"), get("height"));
            if w <= 0.0 || h <= 0.0 {
                return;
            }
            let (rx, ry) = match (num("rx"), num("ry")) {
                (Some(rx), Some(ry)) => (rx, ry),
                (Some(r), None) | (None, Some(r)) => (r, r),
                (None, None) => (0.0, 0.0),
            };
            let (rx, ry) = (rx.clamp(0.0, w / 2.0), ry.clamp(0.0, h / 2.0));
            let corner = |b: &mut Builder, cx: f64, cy: f64, a: f64| {
                if rx > 0.0 && ry > 0.0 {
                    b.ellipse(P::new(cx, cy), rx, ry, 0.0, a, PI / 2.0);
                }
            };
            b.line(P::new(x + rx, y), P::new(x + w - rx, y));
            corner(&mut b, x + w - rx, y + ry, -PI / 2.0);
            b.line(P::new(x + w, y + ry), P::new(x + w, y + h - ry));
            corner(&mut b, x + w - rx, y + h - ry, 0.0);
            b.line(P::new(x + w - rx, y + h), P::new(x + rx, y + h));
            corner(&mut b, x + rx, y + h - ry, PI / 2.0);
            b.line(P::new(x, y + h - ry), P::new(x, y + ry));
            corner(&mut b, x + rx, y + ry, PI);
            b.finish(Kind::Rect, true, out);
        }
        "circle" => {
            let r = get("r");
            if r > 0.0 {
                b.arc(P::new(get("cx"), get("cy")), r, 0.0, TAU);
                b.finish(Kind::Circle, true, out);
            }
        }
        "ellipse" => {
            let (rx, ry) = (get("rx"), get("ry"));
            if rx > 0.0 && ry > 0.0 {
                b.ellipse(P::new(get("cx"), get("cy")), rx, ry, 0.0, 0.0, TAU);
                let kind = if (rx - ry).abs() <= 1e-12 * rx {
                    Kind::Circle
                } else {
                    Kind::Ellipse
                };
                b.finish(kind, true, out);
            }
        }
        _ => {}
    }
}

/// Un attribut `d` : une pièce par sous-chemin.
fn path(d: &str, m: Affine, out: &mut Parsed) {
    use svgtypes::PathSegment as S;
    let mut b = Builder::new(m);
    let (mut cur, mut start) = (P::new(0.0, 0.0), P::new(0.0, 0.0));
    // Dernier point de contrôle, pour les courbes « lisses » (S, T).
    let mut last_cubic: Option<P> = None;
    let mut last_quad: Option<P> = None;
    for seg in svgtypes::PathParser::from(d) {
        let Ok(seg) = seg else { break };
        let at = |abs: bool, x: f64, y: f64| {
            if abs {
                P::new(x, y)
            } else {
                P::new(cur.x + x, cur.y + y)
            }
        };
        let (mut cubic_ctrl, mut quad_ctrl) = (None, None);
        match seg {
            S::MoveTo { abs, x, y } => {
                b.finish(Kind::Path, false, out);
                cur = at(abs, x, y);
                start = cur;
            }
            S::LineTo { abs, x, y } => {
                let p = at(abs, x, y);
                b.line(cur, p);
                cur = p;
            }
            S::HorizontalLineTo { abs, x } => {
                let p = P::new(if abs { x } else { cur.x + x }, cur.y);
                b.line(cur, p);
                cur = p;
            }
            S::VerticalLineTo { abs, y } => {
                let p = P::new(cur.x, if abs { y } else { cur.y + y });
                b.line(cur, p);
                cur = p;
            }
            S::CurveTo {
                abs,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let (p1, p2, p) = (at(abs, x1, y1), at(abs, x2, y2), at(abs, x, y));
                b.cubic(cur, p1, p2, p);
                cubic_ctrl = Some(p2);
                cur = p;
            }
            S::SmoothCurveTo { abs, x2, y2, x, y } => {
                let p1 = last_cubic.map_or(cur, |c| cur * 2.0 - c);
                let (p2, p) = (at(abs, x2, y2), at(abs, x, y));
                b.cubic(cur, p1, p2, p);
                cubic_ctrl = Some(p2);
                cur = p;
            }
            S::Quadratic { abs, x1, y1, x, y } => {
                let (q, p) = (at(abs, x1, y1), at(abs, x, y));
                quadratic(&mut b, cur, q, p);
                quad_ctrl = Some(q);
                cur = p;
            }
            S::SmoothQuadratic { abs, x, y } => {
                let q = last_quad.map_or(cur, |c| cur * 2.0 - c);
                let p = at(abs, x, y);
                quadratic(&mut b, cur, q, p);
                quad_ctrl = Some(q);
                cur = p;
            }
            S::EllipticalArc {
                abs,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x,
                y,
            } => {
                let p = at(abs, x, y);
                elliptical_arc(
                    &mut b,
                    cur,
                    p,
                    rx,
                    ry,
                    x_axis_rotation.to_radians(),
                    large_arc,
                    sweep,
                );
                cur = p;
            }
            S::ClosePath { .. } => {
                if cur.dist(start) > 1e-12 {
                    b.line(cur, start);
                }
                b.finish(Kind::Path, true, out);
                cur = start;
            }
        }
        last_cubic = cubic_ctrl;
        last_quad = quad_ctrl;
    }
    b.finish(Kind::Path, false, out);
}

fn quadratic(b: &mut Builder, p0: P, q: P, p: P) {
    b.cubic(p0, p0.lerp(q, 2.0 / 3.0), p.lerp(q, 2.0 / 3.0), p);
}

/// Arc SVG donné par ses extrémités (recommandation SVG, annexe B.2.4).
#[allow(clippy::too_many_arguments)]
fn elliptical_arc(
    b: &mut Builder,
    p0: P,
    p: P,
    rx: f64,
    ry: f64,
    phi: f64,
    large: bool,
    sweep: bool,
) {
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if p0.dist(p) < 1e-12 {
        return;
    }
    if rx < 1e-12 || ry < 1e-12 {
        b.line(p0, p);
        return;
    }
    let (cs, sn) = (phi.cos(), phi.sin());
    let (dx, dy) = ((p0.x - p.x) / 2.0, (p0.y - p.y) / 2.0);
    let (x1, y1) = (cs * dx + sn * dy, -sn * dx + cs * dy);
    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        rx *= lambda.sqrt();
        ry *= lambda.sqrt();
    }
    let num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut k = (num / den).max(0.0).sqrt();
    if large == sweep {
        k = -k;
    }
    let (cx1, cy1) = (k * rx * y1 / ry, -k * ry * x1 / rx);
    let c = P::new(
        cs * cx1 - sn * cy1 + (p0.x + p.x) / 2.0,
        sn * cx1 + cs * cy1 + (p0.y + p.y) / 2.0,
    );
    let angle = |ux: f64, uy: f64| uy.atan2(ux);
    let t0 = angle((x1 - cx1) / rx, (y1 - cy1) / ry);
    let t1 = angle((-x1 - cx1) / rx, (-y1 - cy1) / ry);
    let mut dt = t1 - t0;
    if sweep && dt < 0.0 {
        dt += TAU;
    } else if !sweep && dt > 0.0 {
        dt -= TAU;
    }
    b.ellipse(c, rx, ry, phi, t0, dt);
}
