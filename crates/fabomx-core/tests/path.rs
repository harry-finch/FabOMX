//! Parcours complet : critères d'acceptation de R-011.
//! Les SVG sont écrits avec un `viewBox` décalé de -200 en Y : y du fichier = -y du plateau,
//! donc y_svg = -y. Pour lire facilement, les dessins sont donnés en coordonnées du plateau.

use fabomx_core::cut::{sides, Contour, Side};
use fabomx_core::geom::{Affine, P};
use fabomx_core::import::parse;
use fabomx_core::lead::{leads, LeadSpec, Leads, Shape, LEAD_IN, LEAD_OUT};
use fabomx_core::path::{plan, Job, MoveKind, PathWarning, Plan};

/// Un dessin : ses contours, côtés, amorces et qualités (3 partout).
struct Drawing {
    contours: Vec<Contour>,
    sides: Vec<Side>,
    leads: Leads,
    qualities: Vec<Option<u8>>,
}

const ID: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

/// `body` en SVG, axe Y du fichier vers le bas : on retourne pour retrouver le plateau.
fn drawing(body: &str) -> Drawing {
    let t = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400mm" height="400mm" viewBox="0 0 400 400"><g transform="scale(1,-1)">{body}</g></svg>"#
    );
    let contours = sides(&parse(&t, None).unwrap().1.pieces).contours;
    let sides: Vec<Side> = contours.iter().map(|c| c.side).collect();
    let specs: Vec<LeadSpec> = sides
        .iter()
        .map(|&side| LeadSpec {
            side,
            shape: Shape::Line,
            lead_in: LEAD_IN,
            lead_out: LEAD_OUT,
            entry: None,
        })
        .collect();
    let leads = leads(&contours, &specs, ID);
    let qualities = sides
        .iter()
        .map(|&s| (s != Side::Skip).then_some(3))
        .collect();
    Drawing {
        contours,
        sides,
        leads,
        qualities,
    }
}

fn job(key: u32, d: &Drawing) -> Job<'_> {
    Job {
        key,
        m: ID,
        contours: &d.contours,
        sides: &d.sides,
        leads: &d.leads.leads,
        qualities: &d.qualities,
        warnings: &d.leads.warnings,
    }
}

fn run(d: &Drawing, start: (f64, f64)) -> Plan {
    plan(&[job(1, d)], P::new(start.0, start.1), None).unwrap()
}

fn sides_in_order(d: &Drawing, p: &Plan) -> Vec<Side> {
    p.order.iter().map(|s| d.sides[s.contour]).collect()
}

#[test]
fn holes_and_slots_come_before_the_outside() {
    let d = drawing(
        r#"<rect x="0" y="0" width="100" height="60"/><circle cx="25" cy="30" r="10"/><circle cx="75" cy="30" r="10"/><line x1="40" y1="10" x2="60" y2="10"/>"#,
    );
    let p = run(&d, (0.0, 0.0));
    let order = sides_in_order(&d, &p);
    assert_eq!(order.len(), 4);
    assert_eq!(order[3], Side::Outside);
}

#[test]
fn a_part_inside_a_hole_comes_before_that_hole() {
    // Plaque, grand trou, pièce dans le trou, petit trou dans la pièce.
    let d = drawing(
        r#"<rect x="0" y="0" width="200" height="200"/><rect x="20" y="20" width="160" height="160"/><rect x="40" y="40" width="120" height="120"/><rect x="60" y="60" width="80" height="80"/>"#,
    );
    let p = run(&d, (0.0, 0.0));
    assert_eq!(
        sides_in_order(&d, &p),
        [Side::Inside, Side::Outside, Side::Inside, Side::Outside]
    );
    // Du plus petit au plus grand : chacun après ceux qu'il contient.
    let areas: Vec<f64> = p
        .order
        .iter()
        .map(|s| d.contours[s.contour].area.abs())
        .collect();
    assert!(areas.windows(2).all(|w| w[0] < w[1]));
}

fn three_squares() -> Drawing {
    drawing(
        r#"<rect x="40" y="0" width="20" height="20"/><rect x="140" y="0" width="20" height="20"/><rect x="240" y="0" width="20" height="20"/>"#,
    )
}

fn xs(d: &Drawing, p: &Plan) -> Vec<f64> {
    p.order
        .iter()
        .map(|s| {
            let pts = &d.contours[s.contour].pts;
            pts.iter().step_by(2).copied().fold(f64::INFINITY, f64::min)
        })
        .collect()
}

#[test]
fn nearest_neighbour_from_the_start_point() {
    let d = three_squares();
    assert_eq!(xs(&d, &run(&d, (0.0, 0.0))), [40.0, 140.0, 240.0]);
    assert_eq!(xs(&d, &run(&d, (305.0, 0.0))), [240.0, 140.0, 40.0]);
}

#[test]
fn a_manual_order_is_kept_and_a_wrong_one_refused() {
    let d = three_squares();
    let auto = run(&d, (0.0, 0.0));
    let mut order: Vec<(u32, usize)> = auto.order.iter().map(|s| (s.key, s.contour)).collect();
    order.swap(0, 2);
    let p = plan(&[job(1, &d)], P::new(0.0, 0.0), Some(&order)).unwrap();
    assert_eq!(xs(&d, &p), [240.0, 140.0, 40.0]);

    let plate =
        drawing(r#"<rect x="0" y="0" width="100" height="60"/><circle cx="50" cy="30" r="10"/>"#);
    let outside = plate
        .sides
        .iter()
        .position(|&s| s == Side::Outside)
        .unwrap();
    let hole = plate.sides.iter().position(|&s| s == Side::Inside).unwrap();
    let err = plan(&[job(7, &plate)], P::new(0.0, 0.0), Some(&[(7, outside)])).unwrap_err();
    assert_eq!(err.contour, (7, outside));
    assert_eq!(err.inner, (7, hole));
    assert_eq!(err.code(), "path.order");
    // Un ordre partiel permis : le reste suit automatiquement.
    let p = plan(&[job(7, &plate)], P::new(0.0, 0.0), Some(&[(7, hole)])).unwrap();
    assert_eq!(p.order.len(), 2);
}

#[test]
fn the_path_starts_and_ends_at_the_start_point() {
    let d = three_squares();
    let p = run(&d, (10.0, 5.0));
    let first = p.moves.first().unwrap();
    let last = p.moves.last().unwrap();
    assert_eq!(first.kind, MoveKind::Traverse);
    assert_eq!(first.segs[0].start(), P::new(10.0, 5.0));
    assert_eq!(last.kind, MoveKind::Traverse);
    assert_eq!(last.segs.last().unwrap().end(), P::new(10.0, 5.0));
    assert_eq!(last.step, None);
}

#[test]
fn stats_count_cut_length_leads_traverses_and_pierces() {
    let d = drawing(r#"<rect x="100" y="100" width="50" height="50"/>"#);
    let p = run(&d, (0.0, 0.0));
    assert_eq!(p.stats.pierces, 1);
    assert!((p.stats.cut[&3] - 200.0).abs() < 1e-9);
    assert!((p.stats.leads - (3.8 + 1.3)).abs() < 1e-9);
    let pierce = P::new(p.order[0].pierce[0], p.order[0].pierce[1]);
    let out_end = p
        .moves
        .iter()
        .find(|m| m.kind == MoveKind::LeadOut)
        .unwrap()
        .segs
        .last()
        .unwrap()
        .end();
    let expected = P::new(0.0, 0.0).dist(pierce) + out_end.dist(P::new(0.0, 0.0));
    assert!((p.stats.traverse - expected).abs() < 1e-9);
}

#[test]
fn a_traverse_over_a_cut_hole_is_reported() {
    // Plaque avec un trou au milieu, et deux pièces de part et d'autre : la
    // plaque est coupée avant la pièce de droite, imposée à la main.
    let plate = drawing(
        r#"<rect x="0" y="0" width="100" height="100"/><rect x="40" y="40" width="20" height="20"/>"#,
    );
    let right = drawing(r#"<rect x="300" y="45" width="10" height="10"/>"#);
    let hole = plate.sides.iter().position(|&s| s == Side::Inside).unwrap();
    let outside = plate
        .sides
        .iter()
        .position(|&s| s == Side::Outside)
        .unwrap();
    // Départ à gauche de la plaque : trou, plaque, puis la pièce de droite (le
    // déplacement depuis la plaque traverse la plaque découpée).
    let jobs = [job(1, &plate), job(2, &right)];
    let p = plan(
        &jobs,
        P::new(-50.0, 50.0),
        Some(&[(1, hole), (1, outside), (2, 0)]),
    )
    .unwrap();
    assert!(
        p.warnings
            .iter()
            .any(|w| matches!(w, PathWarning::OverCut { step: 2, .. })),
        "{:?}",
        p.warnings
    );
    let json = serde_json::to_value(&p.warnings[0]).unwrap();
    assert_eq!(json["code"], "path.over_cut");

    // Deux carrés éloignés, sans rien entre eux : aucune remarque.
    let d = three_squares();
    assert!(run(&d, (0.0, 0.0)).warnings.is_empty());
}
