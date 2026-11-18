//! Reconnaître des arcs : critères d'acceptation de R-007.

use fabomx_core::clean::{clean, preview, Options};
use fabomx_core::drawing::Piece;
use fabomx_core::geom::{Seg, P};
use fabomx_core::import::parse;
use std::f64::consts::{PI, TAU};

fn svg(body: &str) -> Vec<Piece> {
    let t = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="200mm" height="200mm" viewBox="0 0 200 200">{body}</svg>"#
    );
    parse(&t, None).unwrap().1.pieces
}

fn dxf(entities: &str) -> Vec<Piece> {
    let t = format!("0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n");
    parse(&t, None).unwrap().1.pieces
}

fn all_segs(p: &[Piece]) -> Vec<Seg> {
    p.iter().flat_map(|p| p.segs.clone()).collect()
}

fn count(segs: &[Seg]) -> (usize, usize) {
    let arcs = segs.iter().filter(|s| matches!(s, Seg::Arc { .. })).count();
    (arcs, segs.len() - arcs)
}

/// Le plus grand écart entre la courbe `f` (échantillonnée) et les segments.
fn worst(segs: &[Seg], f: &dyn Fn(f64) -> P) -> f64 {
    (0..=2000)
        .map(|k| {
            let q = f(k as f64 / 2000.0);
            segs.iter()
                .map(|s| s.closest(q).0.dist(q))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0, f64::max)
}

#[test]
fn a_bezier_circle_is_read_as_a_few_arcs() {
    // Cercle de rayon 20 centré en (100, 100), en 4 courbes de Bézier (k = 0,5523) :
    // la Bézier s'écarte elle-même du cercle de 0,027 % du rayon, 0,005 mm ici.
    let k = 20.0 * 0.552_284_749_8;
    let d = format!(
        "M120,100 C120,{a} {b},120 100,120 C{c},120 80,{a} 80,100 C80,{e} {c},80 100,80 C{b},80 120,{e} 120,100 Z",
        a = 100.0 + k,
        b = 100.0 + k,
        c = 100.0 - k,
        e = 100.0 - k
    );
    let p = svg(&format!(r#"<path d="{d}"/>"#));
    assert_eq!(p.len(), 1);
    assert!(p[0].closed);
    let segs = &p[0].segs;
    let (arcs, lines) = count(segs);
    assert_eq!(lines, 0, "{segs:?}");
    assert!(arcs <= 8, "{arcs} arcs");
    // À 0,01 mm du vrai cercle (l'axe Y du SVG est retourné).
    let circle = |t: f64| {
        P::new(
            100.0 + 20.0 * (t * TAU).cos(),
            -100.0 + 20.0 * (t * TAU).sin(),
        )
    };
    let w = worst(segs, &circle);
    assert!(w <= 0.01, "écart {w} mm");
}

/// Tangente au début et à la fin d'un segment.
fn tangents(s: &Seg) -> (f64, f64) {
    match *s {
        Seg::Line { a, b } => {
            let t = (b - a).angle();
            (t, t)
        }
        Seg::Arc { a0, sweep, .. } => {
            let q = if sweep > 0.0 { PI / 2.0 } else { -PI / 2.0 };
            (a0 + q, a0 + sweep + q)
        }
    }
}

#[test]
fn an_s_curve_gives_tangent_arcs_within_tolerance() {
    let p = svg(r#"<path d="M10,100 C60,0 140,200 190,100"/>"#);
    let segs = &p[0].segs;
    let (arcs, _) = count(segs);
    assert!(arcs >= 2);
    for w in segs.windows(2) {
        let d = (tangents(&w[1]).0 - tangents(&w[0]).1).rem_euclid(TAU);
        let d = d.min(TAU - d);
        assert!(d < 0.1_f64.to_radians(), "cassure de {}°", d.to_degrees());
    }
    let bez = |t: f64| {
        let u = 1.0 - t;
        let x =
            10.0 * u * u * u + 3.0 * 60.0 * u * u * t + 3.0 * 140.0 * u * t * t + 190.0 * t * t * t;
        let y =
            100.0 * u * u * u + 3.0 * 0.0 * u * u * t + 3.0 * 200.0 * u * t * t + 100.0 * t * t * t;
        P::new(x, -y)
    };
    let w = worst(segs, &bez);
    assert!(w <= 0.01 + 3e-4, "écart {w} mm");
}

#[test]
fn dxf_ellipse_and_spline_become_arcs() {
    let ellipse =
        dxf("0\nELLIPSE\n10\n0\n20\n0\n11\n40\n21\n0\n40\n0.5\n41\n0\n42\n6.283185307179586\n");
    let (arcs, lines) = count(&all_segs(&ellipse));
    assert!(arcs >= 4 && lines == 0, "{arcs} arcs, {lines} lignes");
    let e = |t: f64| P::new(40.0 * (t * TAU).cos(), 20.0 * (t * TAU).sin());
    assert!(worst(&all_segs(&ellipse), &e) <= 0.01 + 3e-4);

    // Spline cubique uniforme ouverte à 5 points de contrôle.
    let spline = dxf("0\nSPLINE\n70\n8\n71\n3\n72\n9\n73\n5\n40\n0\n40\n0\n40\n0\n40\n0\n40\n1\n40\n2\n40\n2\n40\n2\n40\n2\n10\n0\n20\n0\n10\n20\n20\n40\n10\n50\n20\n-10\n10\n80\n20\n40\n10\n100\n20\n0\n");
    let (arcs, _) = count(&all_segs(&spline));
    assert!(arcs >= 2, "{arcs} arcs");
}

fn polygon(pts: &[P]) -> String {
    let s: Vec<String> = pts.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
    format!(r#"<polygon points="{}"/>"#, s.join(" "))
}

fn polyline_circle(r: f64, n: usize) -> Vec<P> {
    (0..n)
        .map(|i| {
            let a = i as f64 * TAU / n as f64;
            P::new(100.0 + r * a.cos(), 100.0 + r * a.sin())
        })
        .collect()
}

fn cleaned(p: &[Piece], o: &Options, k: f64) -> (Vec<Seg>, usize) {
    let (out, counts, _) = clean(p, o, k);
    (all_segs(&out), counts.arcs)
}

#[test]
fn a_72_segment_circle_becomes_two_arcs() {
    let p = svg(&polygon(&polyline_circle(20.0, 72)));
    let (segs, n) = cleaned(&p, &Options::default(), 1.0);
    let (arcs, lines) = count(&segs);
    assert!(arcs <= 2 && lines == 0, "{arcs} arcs, {lines} lignes");
    assert_eq!(n, arcs);
    // Le compte annoncé par l'aperçu est celui obtenu.
    assert_eq!(preview(&p, &Options::default(), 1.0).counts.arcs, n);
    // Décochée : rien ne change.
    let off = Options {
        arcs: false,
        ..Options::default()
    };
    assert_eq!(count(&cleaned(&p, &off, 1.0).0), (0, 72));
}

#[test]
fn a_rounded_rectangle_keeps_its_sides_straight() {
    // Rectangle 80 × 40 à coins arrondis de rayon 10, chaque coin en 12 segments.
    let mut pts = Vec::new();
    for (cx, cy, a0) in [
        (170.0, 60.0, -90.0_f64),
        (170.0, 90.0, 0.0),
        (100.0, 90.0, 90.0),
        (100.0, 60.0, 180.0),
    ] {
        for i in 0..=12 {
            let a = (a0 + 90.0 * i as f64 / 12.0).to_radians();
            pts.push(P::new(cx + 10.0 * a.cos(), cy + 10.0 * a.sin()));
        }
    }
    let p = svg(&polygon(&pts));
    let (segs, n) = cleaned(&p, &Options::default(), 1.0);
    let (arcs, lines) = count(&segs);
    assert_eq!((arcs, lines), (4, 4), "{segs:?}");
    assert_eq!(n, 4);
}

#[test]
fn sharp_corners_stay() {
    let square = [
        P::new(0.0, 0.0),
        P::new(50.0, 0.0),
        P::new(50.0, 50.0),
        P::new(0.0, 50.0),
    ];
    let octagon = polyline_circle(30.0, 8);
    for shape in [square.to_vec(), octagon] {
        let p = svg(&polygon(&shape));
        let (segs, n) = cleaned(&p, &Options::default(), 1.0);
        assert_eq!(n, 0);
        assert_eq!(count(&segs).0, 0);
    }
}

#[test]
fn tiny_details_stay_segments() {
    // Un arc de rayon 0,05 mm en 6 segments, au bout d'une longue ligne.
    let mut pts = vec![P::new(0.0, 100.0)];
    for i in 0..=6 {
        let a = (90.0 - 180.0 * i as f64 / 6.0_f64).to_radians();
        pts.push(P::new(
            50.0 + 0.05 * a.cos(),
            100.05 + 0.05 * a.sin() - 0.05,
        ));
    }
    pts.push(P::new(0.0, 99.9));
    let p = svg(&polygon(&pts));
    let (_, n) = cleaned(&p, &Options::default(), 1.0);
    assert_eq!(n, 0);
}

#[test]
fn the_arc_tolerance_follows_the_drawing_scale() {
    // Un cercle de 72 segments de rayon 2 : leurs sommets sont exacts. Avec un
    // bruit de 0,03 mm sur un sommet, l'écart de 0,01 mm du plateau ne suffit
    // pas à l'échelle 1, mais suffit pour un dessin réduit à 10 %.
    let mut pts = polyline_circle(20.0, 72);
    pts[30] = pts[30] + (pts[30] - P::new(100.0, 100.0)) * (0.03 / 20.0);
    let p = svg(&polygon(&pts));
    let at_full = cleaned(&p, &Options::default(), 1.0).1;
    let at_tenth = cleaned(&p, &Options::default(), 0.1).1;
    assert!(
        at_tenth < at_full || at_tenth <= 2,
        "{at_full} → {at_tenth}"
    );
    assert!(at_tenth <= 2);
}

#[test]
fn a_bezier_circle_is_written_in_few_omx_rows() {
    use fabomx_core::cut::{sides, Side};
    use fabomx_core::geom::Affine;
    use fabomx_core::lead::{leads, LeadSpec, Shape, LEAD_IN, LEAD_OUT};
    use fabomx_core::omx::from_plan;
    use fabomx_core::path::{plan, Job};
    let k = 20.0 * 0.552_284_749_8;
    let d = format!(
        "M120,100 C120,{a} {b},120 100,120 C{c},120 80,{a} 80,100 C80,{e} {c},80 100,80 C{b},80 120,{e} 120,100 Z",
        a = 100.0 + k,
        b = 100.0 + k,
        c = 100.0 - k,
        e = 100.0 - k
    );
    let pieces = svg(&format!(r#"<path d="{d}"/>"#));
    let cs = sides(&pieces).contours;
    let ss: Vec<Side> = cs.iter().map(|c| c.side).collect();
    let specs: Vec<LeadSpec> = ss
        .iter()
        .map(|&side| LeadSpec {
            side,
            shape: Shape::Line,
            lead_in: LEAD_IN,
            lead_out: LEAD_OUT,
            entry: None,
            tabs: Default::default(),
            tab_width: 1.0,
            part: None,
            extend: false,
        })
        .collect();
    let id = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    let l = leads(&cs, &specs, id);
    let q = vec![Some(3); cs.len()];
    let job = Job {
        key: 1,
        m: id,
        contours: &cs,
        sides: &ss,
        leads: &l.leads,
        qualities: &q,
        warnings: &l.warnings,
    };
    let o = from_plan(
        &plan(&[job], P::new(0.0, 0.0), None).unwrap(),
        46000.0,
        "test",
    );
    assert!(o.rows.len() < 20, "{} lignes", o.rows.len());
    assert!(o.rows.iter().any(|r| r.bow.abs() > 1e-9));
}
