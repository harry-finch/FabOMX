//! Côté de coupe d'un tracé ouvert : critères d'acceptation de R-017.
//! Dessins en coordonnées du plateau (axe Y du SVG retourné par le groupe).

use fabomx_core::cut::{sides, Contour, Side};
use fabomx_core::geom::{Affine, P};
use fabomx_core::import::parse;
use fabomx_core::lead::{leads, LeadSpec, Part, Shape, TabSpec, LEAD_IN, LEAD_OUT};
use fabomx_core::omx::{from_plan, Omx};
use fabomx_core::path::{plan, Job, MoveKind, Plan};

const ID: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
const LINE: &str = r#"<line x1="0" y1="10" x2="80" y2="10"/>"#;

fn contours(body: &str) -> Vec<Contour> {
    let t = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400mm" height="400mm" viewBox="0 0 400 400"><g transform="scale(1,-1)">{body}</g></svg>"#
    );
    sides(&parse(&t, None).unwrap().1.pieces).contours
}

fn run(body: &str, part: Option<Part>, extend: bool) -> (Vec<Contour>, Plan, Omx) {
    let cs = contours(body);
    let ss: Vec<Side> = cs.iter().map(|c| c.side).collect();
    let specs: Vec<LeadSpec> = ss
        .iter()
        .map(|&side| LeadSpec {
            side,
            shape: Shape::Line,
            lead_in: LEAD_IN,
            lead_out: LEAD_OUT,
            entry: None,
            tabs: TabSpec::None,
            tab_width: 1.0,
            part: if side == Side::Line { part } else { None },
            extend: side == Side::Line && extend,
        })
        .collect();
    let l = leads(&cs, &specs, ID);
    let q = vec![Some(3); cs.len()];
    let job = Job {
        key: 1,
        m: ID,
        contours: &cs,
        sides: &ss,
        leads: &l.leads,
        qualities: &q,
        warnings: &l.warnings,
    };
    let p = plan(&[job], P::new(0.0, 0.0), None).unwrap();
    let o = from_plan(&p, 46000.0, "test");
    (cs, p, o)
}

/// Les lignes de coupe (Q 3) de l'OMX, en mm.
fn cut_rows(o: &Omx) -> Vec<(f64, f64, u8)> {
    o.rows
        .iter()
        .filter(|r| r.q == 3)
        .map(|r| (r.x * 25.4, r.y * 25.4, r.side))
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

#[test]
fn on_the_line_is_unchanged() {
    let (_, _, o) = run(LINE, None, false);
    let rows = cut_rows(&o);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].2, 0);
}

#[test]
fn part_above_cuts_from_right_to_left_with_offset() {
    let (_, p, o) = run(LINE, Some(Part::Left), false);
    let rows = cut_rows(&o);
    assert_eq!(rows.len(), 1);
    let (x, y, side) = rows[0];
    assert_eq!(side, 1);
    assert!(close(x, 80.0) && close(y, 10.0), "départ en ({x}, {y})");
    let cut = p.moves.iter().find(|m| m.kind == MoveKind::Cut).unwrap();
    assert!(close(cut.segs[0].end().x, 0.0));
}

#[test]
fn part_below_cuts_from_left_to_right_with_offset() {
    let (_, _, o) = run(LINE, Some(Part::Right), false);
    let (x, y, side) = cut_rows(&o)[0];
    assert_eq!(side, 1);
    assert!(close(x, 0.0) && close(y, 10.0), "départ en ({x}, {y})");
}

#[test]
fn extensions_pierce_before_and_leave_after_in_line() {
    let (_, p, o) = run(LINE, Some(Part::Right), true);
    assert_eq!(p.stats.pierces, 1);
    let pierce = p.order[0].pierce;
    assert!(
        close(pierce[0], -3.8) && close(pierce[1], 10.0),
        "{pierce:?}"
    );
    let out = p
        .moves
        .iter()
        .find(|m| m.kind == MoveKind::LeadOut)
        .unwrap()
        .segs
        .last()
        .unwrap()
        .end();
    assert!(close(out.x, 81.3) && close(out.y, 10.0), "{out:?}");
    // Amorces en Q 9, du côté décalé.
    assert!(o.rows.iter().filter(|r| r.q == 9).all(|r| r.side == 1));
    assert_eq!(o.rows.iter().filter(|r| r.q == 9).count(), 2);
    // Sur la ligne, des prolongements au côté 0.
    let (_, _, o) = run(LINE, None, true);
    assert!(o.rows.iter().filter(|r| r.q == 9).all(|r| r.side == 0));
}

#[test]
fn an_extension_near_another_contour_is_shortened() {
    // Un carré juste avant le début de la ligne : l'entrée prolongée le toucherait.
    let (cs, p, _) = run(
        r#"<line x1="0" y1="10" x2="80" y2="10"/><rect x="-20" y="0" width="18" height="20"/>"#,
        Some(Part::Right),
        true,
    );
    let line = cs.iter().position(|c| !c.closed).unwrap();
    let step = p.order.iter().position(|s| s.contour == line).unwrap();
    let lead_in = p
        .moves
        .iter()
        .find(|m| m.step == Some(step) && m.kind == MoveKind::LeadIn)
        .unwrap();
    let l: f64 = lead_in.segs.iter().map(|s| s.length()).sum();
    assert!(l < 2.0, "entrée de {l} mm");
}

#[test]
fn closed_contours_are_unchanged() {
    let body = r#"<rect x="100" y="100" width="50" height="50"/>"#;
    let (_, _, a) = run(body, None, false);
    let (_, _, b) = run(body, Some(Part::Left), true);
    assert_eq!(a.rows, b.rows);
    assert!(a.rows.iter().filter(|r| r.q == 3).all(|r| r.side == 1));
}
