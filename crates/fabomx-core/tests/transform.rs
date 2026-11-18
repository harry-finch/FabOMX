//! Placement : critères d'acceptation de R-002, sur les fichiers de `fixtures/`.

use fabomx_core::drawing::Piece;
use fabomx_core::geom::{Affine, Seg, P};
use fabomx_core::import::parse;
use fabomx_core::place::{place, snap_points, transform, PlaceWarning};
use fabomx_core::PROTOMAX;

fn pieces(name: &str) -> Vec<Piece> {
    let text = std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    parse(&text, None).unwrap().1.pieces
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

#[test]
fn quarter_turn_swaps_width_and_height_around_the_centre() {
    let rect = pieces("rect-100x40.dxf");
    let p = place(
        &rect,
        Affine::rotate_about(P::new(50.0, 20.0), 90.0),
        &PROTOMAX,
    );
    assert!(
        close(p.width, 40.0, 1e-9) && close(p.height, 100.0, 1e-9),
        "{} × {}",
        p.width,
        p.height
    );
    let [x0, y0, x1, y1] = p.bbox;
    assert!(close((x0 + x1) / 2.0, 50.0, 1e-9) && close((y0 + y1) / 2.0, 20.0, 1e-9));
}

#[test]
fn similarities_keep_arcs_exact() {
    let arc = pieces("arc.dxf");
    let c = P::new(50.0, 50.0);
    let m = Affine::scale_about(c, 1.5, 1.5)
        .then_after(Affine::mirror_about(c, true))
        .then_after(Affine::rotate_about(c, 30.0));
    let out = transform(&arc, m);
    let [Seg::Arc { r, .. }] = out[0].segs[..] else {
        panic!("arc attendu : {:?}", out[0].segs)
    };
    assert!(close(r, 37.5, 1e-9));
    assert!(place(&arc, m, &PROTOMAX).warnings.is_empty());
}

#[test]
fn free_scale_turns_a_circle_into_an_ellipse_and_warns() {
    let circle = pieces("circle-r10.dxf");
    let p = place(
        &circle,
        Affine::scale_about(P::new(0.0, 0.0), 2.0, 1.0),
        &PROTOMAX,
    );
    assert!(
        close(p.width, 40.0, 0.01) && close(p.height, 20.0, 0.01),
        "{} × {}",
        p.width,
        p.height
    );
    assert!(p.warnings.contains(&PlaceWarning::Distorted));
    let out = transform(&circle, Affine::scale_about(P::new(0.0, 0.0), 2.0, 1.0));
    // L'ellipse est approchée par des arcs tangents (R-007), à 0,01 mm près.
    let segs = &out[0].segs;
    assert!(segs.iter().filter(|s| matches!(s, Seg::Arc { .. })).count() >= 4);
    let c = circle[0].segs[0];
    let Seg::Arc { c: o, r, .. } = c else {
        panic!()
    };
    let worst = (0..720)
        .map(|k| {
            let a = k as f64 * std::f64::consts::TAU / 720.0;
            let q = P::new(2.0 * (o.x + r * a.cos()), o.y + r * a.sin());
            segs.iter()
                .map(|s| s.closest(q).0.dist(q))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0, f64::max);
    assert!(worst <= 0.01, "écart {worst} mm");
}

#[test]
fn thirty_six_turns_of_ten_degrees_come_back_home() {
    let rect = pieces("rect-100x40.dxf");
    let c = P::new(13.0, 7.0);
    let mut m = Affine::IDENTITY;
    for _ in 0..36 {
        m = Affine::rotate_about(c, 10.0).then_after(m);
    }
    for (a, b) in rect[0].segs.iter().zip(&transform(&rect, m)[0].segs) {
        assert!(a.start().dist(b.start()) < 1e-3 && a.end().dist(b.end()) < 1e-3);
    }
    let twice = Affine::mirror_about(c, false).then_after(Affine::mirror_about(c, false));
    assert_eq!(transform(&rect, twice), rect);
}

#[test]
fn turning_about_a_corner_keeps_the_corner() {
    let rect = pieces("rect-100x40.dxf");
    let corner = P::new(100.0, 40.0);
    let p = place(&rect, Affine::rotate_about(corner, 90.0), &PROTOMAX);
    // Le coin (100, 40) reste en place : il devient le coin haut gauche de la boîte tournée.
    let [x0, y0, x1, y1] = p.bbox;
    assert!(
        close(x0, 100.0, 1e-9) && close(y1, 40.0, 1e-9),
        "{:?}",
        p.bbox
    );
    assert!(
        close(x1, 140.0, 1e-9) && close(y0, -60.0, 1e-9),
        "{:?}",
        p.bbox
    );
    let snaps = snap_points(&transform(&rect, Affine::rotate_about(corner, 90.0)));
    assert!(snaps.iter().any(|q| q.dist(corner) < 1e-9));
}

#[test]
fn square_at_250_overflows_by_45mm() {
    let square = pieces("square-100mm.dxf");
    let p = place(&square, Affine::translate(250.0, 0.0), &PROTOMAX);
    assert!(p.warnings.iter().any(
        |w| matches!(w, PlaceWarning::Overflow { x, y } if close(*x, 45.0, 1e-9) && *y == 0.0)
    ));
    assert!(place(&square, Affine::IDENTITY, &PROTOMAX)
        .warnings
        .is_empty());
}

#[test]
fn snap_points_include_arc_centres() {
    let snaps = snap_points(&pieces("circle-r10.dxf"));
    assert!(snaps.iter().any(|q| q.dist(P::new(0.0, 0.0)) < 1e-12));
}
