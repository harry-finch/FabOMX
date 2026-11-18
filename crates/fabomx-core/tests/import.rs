//! Import : critères d'acceptation de R-001, sur les fichiers de `fixtures/`.

use fabomx_core::drawing::Kind;
use fabomx_core::geom::{Seg, P};
use fabomx_core::import::{parse, Warning};
use fabomx_core::{import, ImportError, Report, Source, Unit, PROTOMAX};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn open(name: &str, unit: Option<Unit>) -> Report {
    import(&fixture(name), unit, &PROTOMAX).unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn clean_square(r: &Report) {
    assert!(
        close(r.width, 100.0) && close(r.height, 100.0),
        "{} × {}",
        r.width,
        r.height
    );
    assert_eq!((r.closed, r.open), (1, 0));
    assert_eq!(r.cleanup, Default::default());
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn square_reads_100mm_in_dxf_and_svg() {
    let d = open("square-100mm.dxf", None);
    assert_eq!((d.detected, d.source), (Unit::Mm, Source::File));
    clean_square(&d);
    assert_eq!(d.counts[&Kind::Line], 4);
    let s = open("square-100mm.svg", None);
    assert_eq!((s.detected, s.source), (Unit::Mm, Source::File));
    clean_square(&s);
    assert_eq!(s.counts[&Kind::Rect], 1);
}

#[test]
fn dxf_without_units_is_assumed_mm() {
    let r = open("square-no-units.dxf", None);
    assert_eq!((r.unit, r.source), (Unit::Mm, Source::Assumed));
    assert!(close(r.width, 100.0));
}

#[test]
fn svg_in_px_reads_100mm_then_inches() {
    let r = open("square-378px.svg", None);
    assert_eq!((r.detected, r.source), (Unit::Px, Source::File));
    // 378 px à 96 par pouce : 100,0125 mm.
    assert!(
        close(r.width, 378.0 * 25.4 / 96.0) && (r.height - 100.0).abs() < 0.02,
        "{}",
        r.width
    );
    let i = open("square-378px.svg", Some(Unit::In));
    assert!(close(i.width, 378.0 * 25.4), "{}", i.width);
}

#[test]
fn changing_the_unit_scales_the_dimensions() {
    let r = open("square-100mm.dxf", Some(Unit::Cm));
    assert!(close(r.width, 1000.0));
}

#[test]
fn ends_5_microns_apart_are_one_gap() {
    let r = open("gap.dxf", None);
    assert_eq!(r.cleanup.gaps, 1);
    assert_eq!(r.cleanup.open, 1);
    assert_eq!(r.closed, 0);
    assert!(r.markers.iter().any(|m| m.kind == "gap"));
}

#[test]
fn repeated_segment_is_one_duplicate() {
    let r = open("duplicate.dxf", None);
    assert_eq!(r.cleanup.duplicates, 1);
}

#[test]
fn single_line_has_no_closed_contour() {
    let r = open("single-line.dxf", None);
    assert!(r.warnings.contains(&Warning::NoClosedContour));
    assert_eq!((r.closed, r.open), (0, 1));
}

#[test]
fn drawing_of_400mm_is_larger_than_the_bed() {
    let r = open("too-large.dxf", None);
    assert!(r
        .warnings
        .iter()
        .any(|w| matches!(w, Warning::LargerThanBed { .. })));
    assert!(!open("square-100mm.dxf", None)
        .warnings
        .iter()
        .any(|w| matches!(w, Warning::LargerThanBed { .. })));
}

#[test]
fn text_and_block_are_counted_as_ignored() {
    let r = open("ignored.dxf", None);
    assert_eq!(r.ignored.get("TEXT"), Some(&1));
    assert_eq!(r.ignored.get("INSERT"), Some(&1));
    assert!(r.warnings.contains(&Warning::Ignored { count: 2 }));
    let s = open("shapes.svg", None);
    assert_eq!(s.ignored.get("text"), Some(&1));
    assert_eq!(s.ignored.get("image"), Some(&1));
}

#[test]
fn an_arc_stays_an_arc() {
    let (_, parsed, _, _) = parse(&fixture("arc.dxf"), None).unwrap();
    assert_eq!(parsed.pieces.len(), 1);
    let Seg::Arc { c, r, sweep, .. } = parsed.pieces[0].segs[0] else {
        panic!("arc attendu")
    };
    assert!(
        c.dist(P::new(50.0, 50.0)) < 1e-9
            && close(r, 25.0)
            && close(sweep, std::f64::consts::FRAC_PI_2)
    );
    // Cercle SVG et arc de chemin restent des arcs exacts, malgré la transformation.
    let (_, shapes, _, _) = parse(&fixture("shapes.svg"), None).unwrap();
    assert!(matches!(shapes.pieces[0].segs[..], [Seg::Arc { .. }]));
    assert!(shapes.pieces[3]
        .segs
        .iter()
        .any(|s| matches!(s, Seg::Arc { .. })));
}

#[test]
fn bulged_polyline_keeps_its_arc_and_is_closed() {
    let r = open("lwpolyline-bulge.dxf", None);
    assert_eq!((r.closed, r.open), (1, 0));
    // Le demi-cercle d'arrondi (rayon 20) dépasse la boîte des sommets de 20 mm.
    assert!(
        close(r.width, 80.0) && close(r.height, 40.0),
        "{} × {}",
        r.width,
        r.height
    );
}

#[test]
fn bezier_is_read_as_arcs_within_10_microns() {
    let (_, parsed, _, _) = parse(&fixture("bezier.svg"), None).unwrap();
    let segs = &parsed.pieces[0].segs;
    // La courbe en mm (Y retourné) : M10 90 C20 10, 80 10, 90 90.
    let f = |t: f64| {
        let u = 1.0 - t;
        let (x, y) = (
            10.0 * u * u * u + 3.0 * 20.0 * u * u * t + 3.0 * 80.0 * u * t * t + 90.0 * t * t * t,
            90.0 * u * u * u + 3.0 * 10.0 * u * u * t + 3.0 * 10.0 * u * t * t + 90.0 * t * t * t,
        );
        P::new(x, -y)
    };
    let dist_to_segs = |p: P| {
        segs.iter()
            .map(|s| s.closest(p).0.dist(p))
            .fold(f64::INFINITY, f64::min)
    };
    let worst = (0..=2000)
        .map(|k| dist_to_segs(f(k as f64 / 2000.0)))
        .fold(0.0, f64::max);
    assert!(worst <= 0.01, "écart {worst} mm");
    // Des arcs tangents (R-007), pas des dizaines de petits segments.
    let arcs = segs.iter().filter(|s| matches!(s, Seg::Arc { .. })).count();
    assert!(
        arcs >= 2 && segs.len() < 40,
        "{arcs} arcs sur {} segments",
        segs.len()
    );
}

#[test]
fn unreadable_or_empty_files_are_refused() {
    let err = |name: &str| import(&fixture(name), None, &PROTOMAX).unwrap_err();
    assert_eq!(err("empty.svg"), ImportError::Empty);
    assert_eq!(err("garbage.txt"), ImportError::UnknownFormat);
    assert_eq!(err("no-geometry.svg"), ImportError::Empty);
    assert_eq!(
        import("<svg", None, &PROTOMAX).unwrap_err(),
        ImportError::Unreadable
    );
    assert_eq!(
        import(
            "0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n",
            None,
            &PROTOMAX
        )
        .unwrap_err(),
        ImportError::Empty
    );
    assert_eq!(err("empty.svg").code(), "import.empty");
}

#[test]
fn shapes_are_counted_by_kind() {
    let r = open("shapes.svg", None);
    assert_eq!(r.counts[&Kind::Circle], 1);
    assert_eq!(r.counts[&Kind::Ellipse], 1);
    assert_eq!(r.counts[&Kind::Polyline], 1);
    assert_eq!(r.counts[&Kind::Path], 1);
    assert_eq!((r.closed, r.open), (4, 0));
}

#[test]
fn a_geometry_survives_a_job_file() {
    use fabomx_core::clean::{clean, Options};
    use fabomx_core::job::{check, Geometry, JobError};
    let (format, parsed, detected, source) = parse(&fixture("shapes.svg"), None).unwrap();
    let (pieces, _, _) = clean(&parsed.pieces, &Options::default(), 1.0);
    let g = Geometry {
        format,
        unit: detected,
        detected,
        source,
        pieces,
        ignored: parsed.ignored,
    };
    let back: Geometry = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
    assert_eq!(back, g);
    assert_eq!(check("pas du json"), Err(JobError::Unreadable));
    assert_eq!(
        check(r#"{"format":"autre","version":1,"drawings":[]}"#),
        Err(JobError::Unreadable)
    );
    assert_eq!(
        check(r#"{"format":"fabomx","version":2,"drawings":[]}"#),
        Err(JobError::Newer)
    );
    assert_eq!(JobError::Newer.code(), "job.newer");
}
