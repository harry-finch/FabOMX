//! Nettoyage : critères d'acceptation de R-004, sur les fichiers de `fixtures/`.

use fabomx_core::clean::{clean, preview, Options};
use fabomx_core::drawing::Piece;
use fabomx_core::geom::{Seg, P};
use fabomx_core::import::parse;

fn pieces(name: &str) -> Vec<Piece> {
    let text = std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    parse(&text, None).unwrap().1.pieces
}

/// Contours fermés et ouverts d'un dessin, par l'aperçu sans aucune opération.
fn contours(p: &[Piece]) -> (usize, usize) {
    let none = Options {
        duplicates: false,
        overlaps: false,
        tiny: false,
        join: false,
        simplify: false,
        ..Options::default()
    };
    let r = preview(p, &none, 1.0);
    (r.closed, r.open)
}

fn segs(p: &[Piece]) -> Vec<Seg> {
    p.iter().flat_map(|p| p.segs.clone()).collect()
}

#[test]
fn gap_of_5_microns_is_joined_by_default() {
    let (out, counts, _) = clean(&pieces("gap.dxf"), &Options::default(), 1.0);
    assert_eq!(counts.joins, 1);
    assert_eq!(contours(&out), (1, 0));
}

#[test]
fn gap_of_0_3mm_needs_a_larger_threshold() {
    let p = pieces("gap-03.dxf");
    let (out, counts, _) = clean(&p, &Options::default(), 1.0);
    assert_eq!((counts.joins, contours(&out)), (0, (0, 1)));
    let wide = Options {
        join_gap: 0.5,
        ..Options::default()
    };
    let (out, counts, _) = clean(&p, &wide, 1.0);
    assert_eq!((counts.joins, contours(&out)), (1, (1, 0)));
    // Ce qui restera ouvert est dit, avec l'écart.
    let pv = preview(&p, &Options::default(), 1.0);
    let open: Vec<_> = pv.defects.iter().filter(|d| d.kind == "open").collect();
    assert_eq!(open.len(), 2);
    assert!(open.iter().all(|d| (d.gap.unwrap() - 0.3).abs() < 1e-9));
}

#[test]
fn duplicates_are_removed_and_the_contour_stays_closed() {
    let (out, counts, _) = clean(&pieces("duplicate.dxf"), &Options::default(), 1.0);
    assert_eq!(counts.duplicates, 1);
    assert_eq!(contours(&out), (1, 0));
    assert_eq!(segs(&out).len(), 4);
    // Quatre lignes rechaînées en un contour : une polyligne.
    assert_eq!(
        (out.len(), out[0].kind),
        (1, fabomx_core::drawing::Kind::Polyline)
    );
}

#[test]
fn partial_overlap_becomes_one_line() {
    let (out, counts, _) = clean(&pieces("overlap.dxf"), &Options::default(), 1.0);
    assert_eq!(counts.overlaps, 1);
    let s = segs(&out);
    assert_eq!(s.len(), 1);
    let Seg::Line { a, b } = s[0] else { panic!() };
    assert!(
        (a.dist(P::new(0.0, 0.0)) < 1e-9 && b.dist(P::new(100.0, 0.0)) < 1e-9)
            || (b.dist(P::new(0.0, 0.0)) < 1e-9 && a.dist(P::new(100.0, 0.0)) < 1e-9)
    );
}

#[test]
fn tiny_corner_segment_is_removed_and_the_square_stays_closed() {
    let p = pieces("tiny-corner.dxf");
    let (out, counts, _) = clean(&p, &Options::default(), 1.0);
    assert_eq!(counts.tiny, 1);
    assert_eq!(contours(&out), (1, 0));
    assert_eq!(segs(&out).len(), 4);
}

#[test]
fn aligned_segments_become_one_line_and_arcs_stay() {
    let (out, counts, _) = clean(&pieces("aligned.dxf"), &Options::default(), 1.0);
    assert_eq!(counts.simplified, 9);
    let s = segs(&out);
    assert_eq!(
        s.iter().filter(|s| matches!(s, Seg::Line { .. })).count(),
        1
    );
    assert_eq!(s.iter().filter(|s| matches!(s, Seg::Arc { .. })).count(), 1);
}

#[test]
fn nothing_is_invented() {
    let p = pieces("single-line.dxf");
    let (out, _, _) = clean(
        &p,
        &Options {
            join_gap: 1.0,
            ..Options::default()
        },
        1.0,
    );
    assert_eq!(contours(&out), (0, 1));
    assert_eq!(segs(&out).len(), 1);
}

#[test]
fn announced_counts_are_the_obtained_ones() {
    for name in [
        "gap.dxf",
        "gap-03.dxf",
        "duplicate.dxf",
        "overlap.dxf",
        "tiny-corner.dxf",
        "aligned.dxf",
        "shapes.svg",
    ] {
        let p = pieces(name);
        let o = Options::default();
        let pv = preview(&p, &o, 1.0);
        let (out, counts, _) = clean(&p, &o, 1.0);
        assert_eq!(pv.counts, counts, "{name}");
        assert_eq!((pv.closed, pv.open), contours(&out), "{name}");
    }
}

#[test]
fn thresholds_are_on_the_bed() {
    // Échelle 10 % : 0,5 mm d'origine font 0,05 mm sur le plateau, sous 0,1 mm.
    let p = pieces("gap-05.dxf");
    let o = Options {
        join_gap: 0.1,
        ..Options::default()
    };
    let (out, counts, _) = clean(&p, &o, 0.1);
    assert_eq!((counts.joins, contours(&out)), (1, (1, 0)));
    let (_, counts, _) = clean(&p, &o, 1.0);
    assert_eq!(counts.joins, 0);
}

#[test]
fn cleaning_a_clean_drawing_changes_nothing_but_alignment() {
    let p = pieces("shapes.svg");
    let (out, counts, _) = clean(&p, &Options::default(), 1.0);
    assert_eq!(
        (
            counts.duplicates,
            counts.overlaps,
            counts.tiny,
            counts.joins
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(contours(&out), contours(&p));
}
