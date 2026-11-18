//! Côtés de coupe : critères d'acceptation de R-008, sur les fichiers de `fixtures/`.

use fabomx_core::clean::{clean, Options};
use fabomx_core::cut::{overlap, quality, reversed, sides, CutError, CutWarning, Side};
use fabomx_core::drawing::Piece;
use fabomx_core::geom::Affine;
use fabomx_core::import::{import, parse};
use fabomx_core::PROTOMAX;

fn text(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn pieces(name: &str) -> Vec<Piece> {
    parse(&text(name), None).unwrap().1.pieces
}

fn svg(body: &str) -> Vec<Piece> {
    let t = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="100mm" viewBox="0 0 100 100">{body}</svg>"#
    );
    parse(&t, None).unwrap().1.pieces
}

/// Les côtés, du plus grand contour (en aire) au plus petit.
fn by_size(p: &[Piece]) -> Vec<Side> {
    let mut cs = sides(p).contours;
    cs.sort_by(|a, b| b.area.abs().total_cmp(&a.area.abs()));
    cs.into_iter().map(|c| c.side).collect()
}

#[test]
fn a_lone_square_is_outside() {
    assert_eq!(by_size(&pieces("square-100mm.dxf")), [Side::Outside]);
}

#[test]
fn a_circle_in_a_square_is_a_hole() {
    assert_eq!(
        by_size(&pieces("plate-hole.svg")),
        [Side::Outside, Side::Inside]
    );
}

#[test]
fn sides_alternate_with_nesting() {
    let s = sides(&pieces("nested.svg"));
    assert!(s.warnings.is_empty());
    assert_eq!(
        by_size(&pieces("nested.svg")),
        [Side::Outside, Side::Inside, Side::Outside, Side::Inside]
    );
}

#[test]
fn an_open_path_is_cut_on_the_line() {
    assert_eq!(by_size(&pieces("single-line.dxf")), [Side::Line]);
}

#[test]
fn dxf_marks_come_from_layers_and_linetypes() {
    let p = pieces("marks.dxf");
    // Lignes à y = 10, 20, … 60 : seules 10 (calque 0) et 60 (continu forcé) se coupent.
    let mut marks: Vec<(f64, bool)> = p
        .iter()
        .map(|p| (p.segs[0].start().y, p.construction))
        .collect();
    marks.sort_by(|a, b| a.0.total_cmp(&b.0));
    let flags: Vec<bool> = marks.iter().map(|m| m.1).collect();
    assert_eq!(flags, [false, true, true, true, true, false]);
    let s = sides(&p);
    assert_eq!(
        s.contours.iter().filter(|c| c.side == Side::Skip).count(),
        4
    );
    assert_eq!(
        s.contours.iter().filter(|c| c.side == Side::Line).count(),
        2
    );
    let r = import(&text("marks.dxf"), None, &PROTOMAX).unwrap();
    assert_eq!(r.construction, 4);
}

#[test]
fn svg_dashes_are_marks() {
    let p = pieces("dashed.svg");
    let mut marks: Vec<(f64, bool)> = p
        .iter()
        .map(|p| (p.segs[0].start().y, p.construction))
        .collect();
    marks.sort_by(|a, b| b.0.total_cmp(&a.0));
    // y du fichier 10, 20, 30, 40 → retourné : 90, 80, 70, 60.
    let flags: Vec<bool> = marks.iter().map(|m| m.1).collect();
    assert_eq!(flags, [false, true, true, false]);
}

#[test]
fn marks_survive_cleaning_and_never_join_cuts() {
    // Un repère qui prolonge une ligne coupée reste un contour à part.
    let p = svg(
        r#"<line x1="0" y1="50" x2="50" y2="50"/><line x1="50" y1="50" x2="100" y2="50" stroke-dasharray="2"/>"#,
    );
    let (out, _, _) = clean(&p, &Options::default(), 1.0);
    let s = sides(&out);
    let mut got: Vec<Side> = s.contours.iter().map(|c| c.side).collect();
    got.sort_by_key(|s| *s as u8);
    assert_eq!(got, [Side::Line, Side::Skip]);
}

#[test]
fn a_figure_eight_is_cut_on_the_line_and_reported() {
    let s = sides(&pieces("figure-eight.svg"));
    assert_eq!(s.contours.len(), 1);
    assert_eq!(s.contours[0].side, Side::Line);
    let [w] = s.warnings.as_slice() else {
        panic!("un avertissement attendu : {:?}", s.warnings)
    };
    let CutWarning::Crossing { contour, x, y } = *w;
    assert_eq!(contour, 0);
    assert!(
        (x - 50.0).abs() < 1e-6 && (y + 50.0).abs() < 1e-6,
        "croisement en ({x}, {y})"
    );
    let json = serde_json::to_value(w).unwrap();
    assert_eq!(json["code"], "cut.crossing");
}

#[test]
fn two_crossing_contours_are_both_reported() {
    let p = svg(
        r#"<rect x="10" y="10" width="40" height="40"/><rect x="30" y="30" width="40" height="40"/>"#,
    );
    let s = sides(&p);
    assert!(s.contours.iter().all(|c| c.side == Side::Line));
    assert_eq!(s.warnings.len(), 2);
}

/// Sens de parcours sur le plateau : positif = trigonométrique.
fn travel(area: f64, side: Side, det: f64) -> f64 {
    let a = area * det.signum();
    if reversed(side, area, det) {
        -a
    } else {
        a
    }
}

#[test]
fn direction_does_not_depend_on_the_file() {
    let ccw = svg(r#"<polygon points="0,0 0,100 100,100 100,0"/>"#);
    let cw = svg(r#"<polygon points="0,0 100,0 100,100 0,100"/>"#);
    let (a, b) = (sides(&ccw).contours[0].area, sides(&cw).contours[0].area);
    assert!(
        a * b < 0.0,
        "les deux carrés sont tracés en sens contraires"
    );
    for area in [a, b] {
        // Pièce à droite du jet : extérieur dans le sens horaire, trou dans le sens trigonométrique.
        assert!(travel(area, Side::Outside, 1.0) < 0.0);
        assert!(travel(area, Side::Inside, 1.0) > 0.0);
        // Après un miroir, toujours pareil par rapport à la pièce.
        assert!(travel(area, Side::Outside, -1.0) < 0.0);
        assert!(travel(area, Side::Inside, -1.0) > 0.0);
    }
    assert!(!reversed(Side::Line, a, 1.0) && !reversed(Side::Skip, b, -1.0));
}

#[test]
fn placed_drawings_overlap_only_when_their_material_meets() {
    let plate = sides(&pieces("plate-hole.svg")).contours; // carré 100, trou r 20 au centre
                                                           // L'axe Y est retourné : la plaque va de (0, -100) à (100, 0), trou en (50, -50) ;
                                                           // le carré de 10 mm de (0, -10) à (10, 0).
    let small = sides(&svg(r#"<rect x="0" y="0" width="10" height="10"/>"#)).contours;
    let id = Affine::translate(0.0, 0.0);
    // Dans le trou : pas de chevauchement.
    assert!(!overlap(&plate, id, &small, Affine::translate(45.0, -40.0)));
    // Dans la matière : chevauchement.
    assert!(overlap(&plate, id, &small, Affine::translate(5.0, -5.0)));
    // À cheval sur le bord : les traits se croisent.
    assert!(overlap(&plate, id, &small, Affine::translate(95.0, -40.0)));
    // Ailleurs : rien.
    assert!(!overlap(&plate, id, &small, Affine::translate(150.0, 0.0)));
}

#[test]
fn quality_is_own_else_default_and_none_when_not_cut() {
    assert_eq!(quality(Side::Outside, None, 3), Ok(Some(3)));
    assert_eq!(quality(Side::Inside, Some(5), 3), Ok(Some(5)));
    assert_eq!(quality(Side::Line, Some(1), 4), Ok(Some(1)));
    assert_eq!(quality(Side::Skip, Some(5), 3), Ok(None));
}

#[test]
fn quality_outside_1_to_5_is_refused() {
    for bad in [0, 6, 9] {
        assert_eq!(quality(Side::Outside, Some(bad), 3), Err(CutError::Quality));
        assert_eq!(quality(Side::Outside, None, bad), Err(CutError::Quality));
    }
    assert_eq!(CutError::Quality.code(), "cut.quality");
}
