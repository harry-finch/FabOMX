//! Onglets de maintien : critères d'acceptation de R-015.
//! Dessins en coordonnées du plateau (axe Y du SVG retourné par le groupe).

use fabomx_core::cut::{sides, Contour, Side};
use fabomx_core::geom::{Affine, P};
use fabomx_core::import::parse;
use fabomx_core::lead::{leads, LeadSpec, LeadWarning, Leads, Shape, TabSpec, LEAD_IN, LEAD_OUT};
use fabomx_core::omx::from_plan;
use fabomx_core::path::{plan, Job, MoveKind, Plan};

const ID: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
const SQUARE: &str = r#"<rect x="100" y="100" width="50" height="50"/>"#;

fn contours(body: &str) -> Vec<Contour> {
    let t = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400mm" height="400mm" viewBox="0 0 400 400"><g transform="scale(1,-1)">{body}</g></svg>"#
    );
    sides(&parse(&t, None).unwrap().1.pieces).contours
}

fn run(body: &str, tabs: TabSpec) -> (Vec<Contour>, Leads, Plan) {
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
            tabs: tabs.clone(),
            tab_width: 1.0,
            part: None,
            extend: false,
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
    (cs, l, p)
}

fn cut_length(p: &Plan) -> f64 {
    p.stats.cut.values().sum()
}

#[test]
fn an_end_tab_costs_no_pierce() {
    let (_, l, p) = run(SQUARE, TabSpec::Count { n: 1 });
    assert_eq!(l.leads[0].as_ref().unwrap().tabs.len(), 1);
    assert!((cut_length(&p) - 199.0).abs() < 1e-9, "{}", cut_length(&p));
    assert_eq!(p.stats.pierces, 1);
    assert!(l.warnings.is_empty(), "{:?}", l.warnings);
}

#[test]
fn three_tabs_cost_two_more_pierces_in_the_scrap() {
    let (_, l, p) = run(SQUARE, TabSpec::Count { n: 3 });
    let tabs = &l.leads[0].as_ref().unwrap().tabs;
    assert_eq!(tabs.len(), 3);
    assert_eq!(p.stats.pierces, 3);
    assert!((cut_length(&p) - 197.0).abs() < 1e-9, "{}", cut_length(&p));
    // Chaque reperçage est dans la chute (hors du carré).
    let outside = |q: P| q.x < 100.0 || q.x > 150.0 || q.y < 100.0 || q.y > 150.0;
    for m in p.moves.iter().filter(|m| m.kind == MoveKind::LeadIn) {
        assert!(outside(m.segs[0].start()), "{:?}", m.segs[0].start());
    }
    // Chaque onglet laisse sa place non coupée.
    for t in tabs {
        let at = P::new(t.at[0], t.at[1]);
        let near = p
            .moves
            .iter()
            .filter(|m| m.kind == MoveKind::Cut)
            .flat_map(|m| m.segs.iter())
            .map(|s| s.closest(at).0.dist(at))
            .fold(f64::INFINITY, f64::min);
        assert!(near >= 0.5 - 1e-9, "onglet coupé : {near}");
    }
    assert!(l.warnings.is_empty(), "{:?}", l.warnings);
}

#[test]
fn placed_tabs_are_projected_on_the_contour() {
    let (_, l, p) = run(
        SQUARE,
        TabSpec::At {
            end: false,
            points: vec![[125.0, 150.3], [150.4, 125.0]],
        },
    );
    let tabs = &l.leads[0].as_ref().unwrap().tabs;
    assert_eq!(tabs.len(), 2);
    let at: Vec<P> = tabs.iter().map(|t| P::new(t.at[0], t.at[1])).collect();
    assert!(at.iter().any(|q| q.dist(P::new(125.0, 150.0)) < 1e-9));
    assert!(at.iter().any(|q| q.dist(P::new(150.0, 125.0)) < 1e-9));
    assert_eq!(p.stats.pierces, 3);
    assert!((cut_length(&p) - 198.0).abs() < 1e-9);
}

#[test]
fn tabs_too_close_are_reported() {
    let (_, l, _) = run(
        SQUARE,
        TabSpec::At {
            end: false,
            points: vec![[125.0, 150.0], [125.6, 150.0]],
        },
    );
    assert!(l
        .warnings
        .iter()
        .any(|w| matches!(w, LeadWarning::TabTooClose { .. })));
    let json = serde_json::to_value(&l.warnings[0]).unwrap();
    assert_eq!(json["code"], "tab.too_close");
    // Un onglet sur le point d'entrée (les amorces).
    let entry = run(SQUARE, TabSpec::None).1.leads[0]
        .as_ref()
        .unwrap()
        .entry;
    let (_, l, _) = run(
        SQUARE,
        TabSpec::At {
            end: false,
            points: vec![entry],
        },
    );
    assert!(l
        .warnings
        .iter()
        .any(|w| matches!(w, LeadWarning::TabTooClose { .. })));
}

#[test]
fn holes_never_get_tabs() {
    let (cs, l, _) = run(
        r#"<rect x="100" y="100" width="50" height="50"/><circle cx="125" cy="125" r="10"/>"#,
        TabSpec::Count { n: 3 },
    );
    let hole = cs.iter().position(|c| c.side == Side::Inside).unwrap();
    assert!(l.leads[hole].as_ref().unwrap().tabs.is_empty());
    let outside = cs.iter().position(|c| c.side == Side::Outside).unwrap();
    assert_eq!(l.leads[outside].as_ref().unwrap().tabs.len(), 3);
}

#[test]
fn tabs_are_written_as_traverses_and_new_pierces() {
    let (_, _, p) = run(SQUARE, TabSpec::Count { n: 3 });
    let o = from_plan(&p, 46000.0, "test");
    let pierces = o
        .rows
        .windows(2)
        .filter(|w| w[0].q == 0 && w[1].q == 9)
        .count();
    assert_eq!(pierces, 3);
}
