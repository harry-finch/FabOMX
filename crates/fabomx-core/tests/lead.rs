//! Amorces et perçages : critères d'acceptation de R-010.
//! Rappel : un SVG sans décalage de `viewBox` retourne l'axe Y (y → -y).

use fabomx_core::cut::{sides, Contour, Side};
use fabomx_core::geom::{Affine, Seg, P};
use fabomx_core::import::parse;
use fabomx_core::lead::{leads, Lead, LeadSpec, LeadWarning, Leads, Shape, LEAD_IN, LEAD_OUT};

fn contours(body: &str) -> Vec<Contour> {
    let t = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="200mm" height="200mm" viewBox="0 0 200 200">{body}</svg>"#
    );
    sides(&parse(&t, None).unwrap().1.pieces).contours
}

fn specs(cs: &[Contour], shape: Shape) -> Vec<LeadSpec> {
    cs.iter()
        .map(|c| LeadSpec {
            side: c.side,
            shape,
            lead_in: LEAD_IN,
            lead_out: LEAD_OUT,
            entry: None,
            tabs: Default::default(),
            tab_width: 1.0,
            part: None,
            extend: false,
        })
        .collect()
}

fn run(body: &str, shape: Shape, m: Affine) -> (Vec<Contour>, Leads) {
    let cs = contours(body);
    let l = leads(&cs, &specs(&cs, shape), m);
    (cs, l)
}

fn p(a: [f64; 2]) -> P {
    P::new(a[0], a[1])
}

fn len(segs: &[Seg]) -> f64 {
    segs.iter().map(Seg::length).sum()
}

/// L'amorce du contour de côté `side`.
fn of(cs: &[Contour], l: &Leads, side: Side) -> Lead {
    let i = cs.iter().position(|c| c.side == side).unwrap();
    l.leads[i].clone().unwrap()
}

const ID: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

#[test]
fn rectangle_is_pierced_outside_in_line_with_a_corner() {
    let (cs, l) = run(
        r#"<rect x="10" y="10" width="50" height="30"/>"#,
        Shape::Line,
        ID,
    );
    let lead = of(&cs, &l, Side::Outside);
    let (e, pierce) = (p(lead.entry), p(lead.pierce));
    // Un coin du rectangle.
    assert!([10.0, 60.0].iter().any(|x| (e.x - x).abs() < 1e-9));
    assert!([-10.0, -40.0].iter().any(|y| (e.y - y).abs() < 1e-9));
    // À 3,8 mm, dans le prolongement d'un côté, hors de la pièce.
    assert!((pierce.dist(e) - 3.8).abs() < 1e-9);
    assert!((pierce.x - e.x).abs() < 1e-9 || (pierce.y - e.y).abs() < 1e-9);
    assert!(pierce.x < 10.0 || pierce.x > 60.0 || pierce.y > -10.0 || pierce.y < -40.0);
    // Sortie de 1,3 mm, d'équerre, vers l'extérieur.
    assert!((len(&lead.lead_out) - 1.3).abs() < 1e-9);
    let out = lead.lead_out[0].end();
    assert!((out.x - e.x).abs() < 1e-9 || (out.y - e.y).abs() < 1e-9);
    assert!(out.x < 10.0 || out.x > 60.0 || out.y > -10.0 || out.y < -40.0);
    assert!(l.warnings.is_empty());
}

#[test]
fn a_small_hole_is_pierced_at_its_centre() {
    let (cs, l) = run(
        r#"<circle cx="50" cy="50" r="15"/><circle cx="50" cy="50" r="3.5"/>"#,
        Shape::Line,
        ID,
    );
    let lead = of(&cs, &l, Side::Inside);
    assert!(
        p(lead.pierce).dist(P::new(50.0, -50.0)) < 1e-3,
        "{:?}",
        lead.pierce
    );
}

#[test]
fn a_large_hole_gets_a_full_lead_inside() {
    let (cs, l) = run(
        r#"<rect x="0" y="0" width="100" height="100"/><circle cx="50" cy="50" r="20"/>"#,
        Shape::Line,
        ID,
    );
    let lead = of(&cs, &l, Side::Inside);
    assert!((len(&lead.lead_in) - 3.8).abs() < 1e-9);
    let d = p(lead.pierce).dist(P::new(50.0, -50.0));
    assert!(d < 20.0 - 3.0, "perçage dans le trou : {d}");
}

#[test]
fn an_arc_lead_is_tangent_and_starts_in_the_scrap() {
    let (cs, l) = run(
        r#"<rect x="10" y="10" width="50" height="30"/>"#,
        Shape::Arc,
        ID,
    );
    let lead = of(&cs, &l, Side::Outside);
    let [Seg::Arc { c, r, .. }] = lead.lead_in.as_slice() else {
        panic!("un arc attendu : {:?}", lead.lead_in)
    };
    let e = p(lead.entry);
    assert!((lead.lead_in[0].end().dist(e)) < 1e-9);
    assert!((r * std::f64::consts::FRAC_PI_2 - 3.8).abs() < 1e-9);
    // Rayon au point d'entrée perpendiculaire au côté (horizontal ou vertical) : tangent.
    let radial = e - *c;
    assert!(radial.x.abs() < 1e-9 || radial.y.abs() < 1e-9);
    let pierce = p(lead.pierce);
    assert!(pierce.x < 10.0 || pierce.x > 60.0 || pierce.y > -10.0 || pierce.y < -40.0);
}

/// Un rectangle entouré d'un tracé ouvert à `gap` mm.
fn boxed(gap: f64) -> String {
    let (a, b) = (10.0 - gap, 60.0 + gap);
    format!(
        r#"<rect x="10" y="10" width="50" height="50"/><polyline points="{a},{a} {b},{a} {b},{b} {a},{b} {a},{}"/>"#,
        a + 1.0
    )
}

#[test]
fn a_lead_near_another_contour_is_shortened() {
    let (cs, l) = run(&boxed(2.0), Shape::Line, ID);
    let lead = of(&cs, &l, Side::Outside);
    let d = len(&lead.lead_in);
    assert!((0.5..2.0).contains(&d), "amorce de {d} mm");
    assert!(l.warnings.is_empty());
}

#[test]
fn a_lead_that_cannot_fit_is_reported() {
    let (_, l) = run(&boxed(0.3), Shape::Line, ID);
    assert!(matches!(
        l.warnings.as_slice(),
        [LeadWarning::TooShort { .. }]
    ));
    let json = serde_json::to_value(&l.warnings[0]).unwrap();
    assert_eq!(json["code"], "lead.too_short");
}

#[test]
fn a_slot_has_no_lead_and_is_pierced_at_its_end() {
    let (cs, l) = run(r#"<line x1="0" y1="10" x2="80" y2="10"/>"#, Shape::Line, ID);
    let lead = of(&cs, &l, Side::Line);
    assert!(lead.lead_in.is_empty() && lead.lead_out.is_empty());
    assert_eq!(lead.pierce, [0.0, -10.0]);
}

#[test]
fn a_chosen_entry_point_is_used() {
    let cs = contours(r#"<rect x="10" y="10" width="50" height="30"/>"#);
    let mut sp = specs(&cs, Shape::Line);
    sp[0].entry = Some([60.0, -10.0]);
    let l = leads(&cs, &sp, ID);
    assert_eq!(l.leads[0].as_ref().unwrap().entry, [60.0, -10.0]);
    // Un point au milieu d'un côté : projeté sur le trait.
    sp[0].entry = Some([35.0, -9.0]);
    let l = leads(&cs, &sp, ID);
    assert_eq!(l.leads[0].as_ref().unwrap().entry, [35.0, -10.0]);
}

#[test]
fn lengths_are_bed_millimetres_whatever_the_scale() {
    let (cs, l) = run(
        r#"<rect x="10" y="10" width="50" height="30"/>"#,
        Shape::Line,
        Affine([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]),
    );
    let lead = of(&cs, &l, Side::Outside);
    assert!((len(&lead.lead_in) * 2.0 - 3.8).abs() < 1e-9);
}

#[test]
fn skipped_contours_have_no_lead() {
    let cs = contours(r#"<rect x="10" y="10" width="50" height="30"/>"#);
    let mut sp = specs(&cs, Shape::Line);
    sp[0].side = Side::Skip;
    assert!(leads(&cs, &sp, ID).leads[0].is_none());
}
