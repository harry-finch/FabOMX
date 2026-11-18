//! Format OMX et export du parcours : critères d'acceptation de R-012.
//! Les exemples de LAYOUT sont dans `omx/` à la racine du dépôt.

use fabomx_core::cut::{sides, Side};
use fabomx_core::geom::{Affine, P};
use fabomx_core::import::parse;
use fabomx_core::lead::{leads, LeadSpec, Shape, LEAD_IN, LEAD_OUT};
use fabomx_core::omx::{from_plan, read, write, OmxError};
use fabomx_core::path::{plan, Job};

fn example(name: &str) -> String {
    std::fs::read_to_string(format!("{}/../../omx/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn layout_examples_are_rewritten_byte_for_byte() {
    for name in ["rectangle.omx", "test.omx"] {
        let text = example(name);
        let o = read(&text).unwrap();
        assert!(!o.rows.is_empty());
        assert_eq!(write(&o), text, "{name}");
    }
}

#[test]
fn layout_examples_carry_qualities_sides_and_bows() {
    let o = read(&example("test.omx")).unwrap();
    let qs: Vec<u8> = o.rows.iter().map(|r| r.q).collect();
    assert!(qs.contains(&0) && qs.contains(&9) && qs.contains(&3));
    assert!(o.rows.iter().filter(|r| r.q != 0).all(|r| r.side == 1));
    assert!(o.rows.iter().any(|r| (r.bow - 1.0).abs() < 1e-9));
}

#[test]
fn a_foreign_file_is_refused() {
    assert_eq!(read("hello").unwrap_err(), OmxError::Unreadable);
    assert_eq!(OmxError::Unreadable.code(), "omx.unreadable");
}

/// Le parcours d'un dessin SVG (axe Y retourné pour lire en coordonnées du plateau).
fn exported(body: &str, m: Affine, start: (f64, f64)) -> fabomx_core::omx::Omx {
    let t = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400mm" height="400mm" viewBox="0 0 400 400"><g transform="scale(1,-1)">{body}</g></svg>"#
    );
    let contours = sides(&parse(&t, None).unwrap().1.pieces).contours;
    let sides_: Vec<Side> = contours.iter().map(|c| c.side).collect();
    let specs: Vec<LeadSpec> = sides_
        .iter()
        .map(|&side| LeadSpec {
            side,
            shape: Shape::Line,
            lead_in: LEAD_IN,
            lead_out: LEAD_OUT,
            entry: None,
        })
        .collect();
    let l = leads(&contours, &specs, m);
    let qualities: Vec<Option<u8>> = sides_
        .iter()
        .map(|&s| (s != Side::Skip).then_some(3))
        .collect();
    let job = Job {
        key: 1,
        m,
        contours: &contours,
        sides: &sides_,
        leads: &l.leads,
        qualities: &qualities,
        warnings: &l.warnings,
    };
    let p = plan(&[job], P::new(start.0, start.1), None).unwrap();
    from_plan(&p, 46000.5, "Created with FabOMX")
}

#[test]
fn a_plate_with_a_hole_is_written_like_layout() {
    let o = exported(
        r#"<rect x="100" y="100" width="50" height="50"/><circle cx="125" cy="125" r="10"/>"#,
        Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        (25.4, 50.8),
    );
    let (first, last) = (o.rows[0], *o.rows.last().unwrap());
    // Départ et retour au point de départ, en pouces.
    assert_eq!((first.x, first.y), (1.0, 2.0));
    assert!((last.x - 1.0).abs() < 1e-12 && (last.y - 2.0).abs() < 1e-12);
    assert_eq!((first.q, first.side), (0, 0));
    let qs: Vec<u8> = o.rows.iter().map(|r| r.q).collect();
    assert!(qs.contains(&9) && qs.contains(&3));
    for r in &o.rows {
        match r.q {
            0 => assert_eq!(r.side, 0),
            _ => assert_eq!(r.side, 1),
        }
    }
    // Le cercle : des arcs (Bow non nul), pas des dizaines de petits segments.
    let arcs = o
        .rows
        .iter()
        .filter(|r| r.q == 3 && r.bow.abs() > 1e-9)
        .count();
    assert!((2..=4).contains(&arcs), "{arcs} arcs");
    assert!(o.rows.len() < 30, "{} lignes", o.rows.len());
    // Le trou est parcouru dans le sens trigonométrique (Bow positifs), comme LAYOUT.
    assert!(o
        .rows
        .iter()
        .filter(|r| r.q == 3 && r.bow.abs() > 1e-9)
        .all(|r| r.bow > 0.0));
    // Relu puis réécrit à l'identique (10 décimales).
    let text = write(&o);
    assert_eq!(write(&read(&text).unwrap()), text);
}

#[test]
fn a_slot_is_written_without_offset() {
    let o = exported(
        r#"<line x1="100" y1="100" x2="150" y2="100"/>"#,
        Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        (0.0, 0.0),
    );
    assert!(o.rows.iter().filter(|r| r.q == 3).all(|r| r.side == 0));
    assert_eq!(o.rows.iter().filter(|r| r.q == 3).count(), 1);
}

#[test]
fn arcs_stay_arcs_even_under_a_stretch() {
    let body = r#"<circle cx="100" cy="100" r="10"/>"#;
    let arcs = |o: &fabomx_core::omx::Omx| {
        o.rows
            .iter()
            .filter(|r| r.q == 3 && r.bow.abs() > 1e-9)
            .count()
    };
    let uniform = exported(body, Affine([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]), (0.0, 0.0));
    assert!(arcs(&uniform) >= 2);
    // Une échelle non proportionnelle donne une ellipse, elle aussi en arcs tangents (R-007).
    let stretched = exported(body, Affine([2.0, 0.0, 0.0, 1.0, 0.0, 0.0]), (0.0, 0.0));
    assert!(arcs(&stretched) >= 4);
    assert!(stretched.rows.len() < 40, "{} lignes", stretched.rows.len());
}

#[test]
fn an_outside_contour_is_cut_clockwise() {
    let o = exported(
        r#"<rect x="100" y="100" width="50" height="30"/>"#,
        Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        (0.0, 0.0),
    );
    // Aire signée des points de coupe : négative = sens horaire.
    let pts: Vec<(f64, f64)> = o
        .rows
        .iter()
        .filter(|r| r.q == 3)
        .map(|r| (r.x, r.y))
        .collect();
    let area: f64 = (0..pts.len())
        .map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum();
    assert!(area < 0.0);
}
