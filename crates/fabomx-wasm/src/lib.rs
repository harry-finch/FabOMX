//! Moteur FabOMX pour l'interface : appelé depuis un Web Worker. Les
//! données passent en JSON. Les dessins posés sont gardés ici, chacun sous
//! un identifiant de géométrie que l'interface choisit : un nettoyage range
//! une nouvelle géométrie sous un nouvel identifiant, et l'ancienne reste
//! pour annuler.

use fabomx_core::clean::Options;
use fabomx_core::cut::Sides;
use fabomx_core::drawing::Parsed;
use fabomx_core::geom::Affine;
use fabomx_core::import::{analyze, parse, Format};
use fabomx_core::{Source, Unit, PROTOMAX};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Une géométrie gardée, avec ce qu'il faut pour en refaire le rapport.
struct Stored {
    parsed: Parsed,
    format: Format,
    unit: Unit,
    detected: Unit,
    source: Source,
    /// Côtés détectés, calculés à la première demande.
    sides: Option<Sides>,
}

thread_local! {
    static GEOMETRIES: RefCell<HashMap<u32, Stored>> = RefCell::new(HashMap::new());
}

fn error(code: &str) -> String {
    serde_json::json!({ "error": code }).to_string()
}

/// Réglages de nettoyage en JSON ; vide : pas de nettoyage.
fn options(json: &str) -> Option<Options> {
    (!json.is_empty())
        .then(|| serde_json::from_str(json).ok())
        .flatten()
}

fn snaps(parsed: &Parsed) -> Vec<f64> {
    fabomx_core::place::snap_points(&parsed.pieces)
        .into_iter()
        .flat_map(|p| [p.x, p.y])
        .collect()
}

/// Version du moteur.
#[wasm_bindgen]
pub fn engine_version() -> String {
    fabomx_core::VERSION.to_string()
}

/// L'état d'un dessin SVG ou DXF pour le dialogue d'import, en JSON. `unit`
/// vide : l'unité du fichier. `clean` : réglages de « Nettoyer à l'import »,
/// ou vide. Un refus donne `{"error": "<code stable>"}`.
#[wasm_bindgen]
pub fn inspect(text: &str, unit: &str, clean: &str) -> String {
    let unit = Unit::parse(unit);
    match fabomx_core::import::import_with(text, unit, options(clean).as_ref(), &PROTOMAX) {
        Ok(report) => serde_json::to_string(&report).expect("rapport sérialisable"),
        Err(e) => error(e.code()),
    }
}

/// Garde la géométrie d'origine d'un dessin importé sous `geo` (nettoyée
/// avec `clean` s'il n'est pas vide). Rend ses points d'accrochage
/// `{"snaps": [x, y, …]}` (mm), ou `{"error": "<code stable>"}`.
#[wasm_bindgen]
pub fn load(geo: u32, text: &str, unit: &str, clean: &str) -> String {
    let unit = Unit::parse(unit);
    match parse(text, unit) {
        Ok((format, mut parsed, detected, source)) => {
            if let Some(o) = options(clean) {
                parsed.pieces = fabomx_core::clean::clean(&parsed.pieces, &o, 1.0).0;
            }
            let out = serde_json::json!({ "snaps": snaps(&parsed) }).to_string();
            let stored = Stored {
                parsed,
                format,
                unit: unit.unwrap_or(detected),
                detected,
                source,
                sides: None,
            };
            GEOMETRIES.with(|g| g.borrow_mut().insert(geo, stored));
            out
        }
        Err(e) => error(e.code()),
    }
}

/// La géométrie `geo` transformée par la matrice `[a, b, c, d, e, f]` : sa
/// boîte, ses dimensions et ses avertissements, en JSON.
#[wasm_bindgen]
pub fn place(geo: u32, m: &[f64]) -> String {
    GEOMETRIES.with(|g| {
        let g = g.borrow();
        let (Some(s), [a, b, c, d, e, f]) = (g.get(&geo), m) else {
            return error("place.unknown");
        };
        let p = fabomx_core::place::place(
            &s.parsed.pieces,
            Affine([*a, *b, *c, *d, *e, *f]),
            &PROTOMAX,
        );
        serde_json::to_string(&p).expect("placement sérialisable")
    })
}

/// Ce que ferait le nettoyage de `geo` avec `opts` (JSON), à l'échelle `k`
/// (mm du plateau par mm d'origine) : comptes, bilan, défauts.
#[wasm_bindgen]
pub fn clean_preview(geo: u32, opts: &str, k: f64) -> String {
    let Some(o) = options(opts) else {
        return error("clean.options");
    };
    GEOMETRIES.with(|g| match g.borrow().get(&geo) {
        Some(s) => serde_json::to_string(&fabomx_core::clean::preview(&s.parsed.pieces, &o, k))
            .expect("aperçu sérialisable"),
        None => error("clean.unknown"),
    })
}

/// Nettoie `geo` et range le résultat sous `new_geo`, sans toucher à `geo`.
/// Rend `{"report", "snaps", "counts"}` du dessin nettoyé, en JSON.
#[wasm_bindgen]
pub fn clean_apply(geo: u32, opts: &str, k: f64, new_geo: u32) -> String {
    let Some(o) = options(opts) else {
        return error("clean.options");
    };
    GEOMETRIES.with(|g| {
        let mut g = g.borrow_mut();
        let Some(s) = g.get(&geo) else {
            return error("clean.unknown");
        };
        let (pieces, counts, _) = fabomx_core::clean::clean(&s.parsed.pieces, &o, k);
        let parsed = Parsed {
            pieces,
            ignored: s.parsed.ignored.clone(),
        };
        let report = analyze(
            s.format,
            parsed.clone(),
            s.unit,
            s.detected,
            s.source,
            &PROTOMAX,
        );
        let out =
            serde_json::json!({ "report": report, "snaps": snaps(&parsed), "counts": counts })
                .to_string();
        let stored = Stored {
            parsed,
            format: s.format,
            unit: s.unit,
            detected: s.detected,
            source: s.source,
            sides: None,
        };
        g.insert(new_geo, stored);
        out
    })
}

/// Les contours de `geo` et leurs côtés détectés, en JSON (repère d'origine).
#[wasm_bindgen]
pub fn sides(geo: u32) -> String {
    GEOMETRIES.with(|g| {
        let mut g = g.borrow_mut();
        let Some(s) = g.get_mut(&geo) else {
            return error("cut.unknown");
        };
        let sides = s
            .sides
            .get_or_insert_with(|| fabomx_core::cut::sides(&s.parsed.pieces));
        serde_json::to_string(sides).expect("côtés sérialisables")
    })
}

/// Les paires de dessins placés qui se chevauchent. `placed` : JSON
/// `[{"geo": n, "m": [a, b, c, d, e, f]}, …]` ; rend `[[i, j], …]` (indices).
#[wasm_bindgen]
pub fn overlaps(placed: &str) -> String {
    #[derive(serde::Deserialize)]
    struct Placed {
        geo: u32,
        m: [f64; 6],
    }
    let Ok(list) = serde_json::from_str::<Vec<Placed>>(placed) else {
        return error("cut.placed");
    };
    GEOMETRIES.with(|g| {
        let mut g = g.borrow_mut();
        for p in &list {
            if let Some(s) = g.get_mut(&p.geo) {
                s.sides
                    .get_or_insert_with(|| fabomx_core::cut::sides(&s.parsed.pieces));
            }
        }
        let contours = |geo: u32| {
            g.get(&geo)
                .and_then(|s| s.sides.as_ref())
                .map(|s| &s.contours)
        };
        let mut out = Vec::new();
        for i in 0..list.len() {
            for j in i + 1..list.len() {
                if let (Some(a), Some(b)) = (contours(list[i].geo), contours(list[j].geo)) {
                    if fabomx_core::cut::overlap(a, Affine(list[i].m), b, Affine(list[j].m)) {
                        out.push([i, j]);
                    }
                }
            }
        }
        serde_json::to_string(&out).expect("paires sérialisables")
    })
}

/// Les amorces et perçages des contours de `geo`, placée par la matrice `m`.
/// `specs` : JSON `[{"side", "shape", "lead_in", "lead_out", "entry"}, …]`,
/// un par contour (longueurs en mm du plateau, `entry` en repère d'origine
/// ou `null`). Rend `{"leads": [...], "warnings": [...]}` (repère d'origine).
#[wasm_bindgen]
pub fn leads(geo: u32, m: &[f64], specs: &str) -> String {
    let Ok(specs) = serde_json::from_str::<Vec<fabomx_core::lead::LeadSpec>>(specs) else {
        return error("lead.specs");
    };
    let [a, b, c, d, e, f] = m else {
        return error("lead.matrix");
    };
    GEOMETRIES.with(|g| {
        let mut g = g.borrow_mut();
        let Some(s) = g.get_mut(&geo) else {
            return error("lead.unknown");
        };
        let sides = s
            .sides
            .get_or_insert_with(|| fabomx_core::cut::sides(&s.parsed.pieces));
        let out =
            fabomx_core::lead::leads(&sides.contours, &specs, Affine([*a, *b, *c, *d, *e, *f]));
        serde_json::to_string(&out).expect("amorces sérialisables")
    })
}

/// Le parcours complet des dessins placés. `job` : JSON
/// `{"start": [x, y], "order": [[clé, contour], …] | null, "drawings":
/// [{"key", "geo", "m": [6], "specs": [LeadSpec, …], "qualities": [q | null, …]}]}`.
/// Rend le plan (`path::Plan`), ou `{"error": "path.order", "contour": [clé,
/// contour], "inner": [clé, contour]}` pour un ordre refusé.
#[wasm_bindgen]
pub fn plan(job: &str) -> String {
    #[derive(serde::Deserialize)]
    struct Drawing {
        key: u32,
        geo: u32,
        m: [f64; 6],
        specs: Vec<fabomx_core::lead::LeadSpec>,
        qualities: Vec<Option<u8>>,
    }
    #[derive(serde::Deserialize)]
    struct Input {
        start: [f64; 2],
        order: Option<Vec<(u32, usize)>>,
        drawings: Vec<Drawing>,
    }
    let Ok(input) = serde_json::from_str::<Input>(job) else {
        return error("path.job");
    };
    GEOMETRIES.with(|g| {
        let mut g = g.borrow_mut();
        for d in &input.drawings {
            if let Some(s) = g.get_mut(&d.geo) {
                s.sides
                    .get_or_insert_with(|| fabomx_core::cut::sides(&s.parsed.pieces));
            }
        }
        // Contours, côtés et amorces de chaque dessin connu.
        let mut prepared = Vec::new();
        for d in &input.drawings {
            let Some(contours) = g
                .get(&d.geo)
                .and_then(|s| s.sides.as_ref())
                .map(|s| &s.contours)
            else {
                continue;
            };
            let m = Affine(d.m);
            let leads = fabomx_core::lead::leads(contours, &d.specs, m);
            let sides: Vec<_> = d.specs.iter().map(|s| s.side).collect();
            prepared.push((d, contours, m, leads, sides));
        }
        let jobs: Vec<fabomx_core::path::Job> = prepared
            .iter()
            .map(|(d, contours, m, leads, sides)| fabomx_core::path::Job {
                key: d.key,
                m: *m,
                contours,
                sides,
                leads: &leads.leads,
                qualities: &d.qualities,
            })
            .collect();
        let start = fabomx_core::geom::P::new(input.start[0], input.start[1]);
        match fabomx_core::path::plan(&jobs, start, input.order.as_deref()) {
            Ok(p) => serde_json::to_string(&p).expect("parcours sérialisable"),
            Err(e) => serde_json::json!({
                "error": e.code(),
                "contour": [e.contour.0, e.contour.1],
                "inner": [e.inner.0, e.inner.1],
            })
            .to_string(),
        }
    })
}

/// Machine cible (zone de coupe en mm), en JSON.
#[wasm_bindgen]
pub fn machine() -> String {
    serde_json::to_string(&PROTOMAX).expect("machine sérialisable")
}

#[cfg(test)]
mod tests {
    use super::*;

    const CIRCLE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10"><circle cx="5" cy="5" r="5"/></svg>"#;
    const OPEN_SQUARE: &str = "0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n10\n21\n0\n0\nLINE\n10\n10\n20\n0\n11\n10\n21\n10\n0\nLINE\n10\n10\n20\n10\n11\n0\n21\n10\n0\nLINE\n10\n0\n20\n10\n11\n0\n21\n0.005\n0\nENDSEC\n0\nEOF\n";

    #[test]
    fn inspect_reports_or_refuses() {
        let ok: serde_json::Value = serde_json::from_str(&inspect(CIRCLE, "", "")).unwrap();
        assert_eq!(ok["unit"], "mm");
        assert_eq!(ok["counts"]["circle"], 1);
        assert!(ok.get("cleaned").is_none());
        let err: serde_json::Value = serde_json::from_str(&inspect("", "", "")).unwrap();
        assert_eq!(err["error"], "import.empty");
    }

    #[test]
    fn inspect_can_clean_at_import() {
        let opts = serde_json::to_string(&Options::default()).unwrap();
        let r: serde_json::Value = serde_json::from_str(&inspect(OPEN_SQUARE, "", &opts)).unwrap();
        assert_eq!(r["cleaned"]["joins"], 1);
        assert_eq!(r["closed"], 1);
        assert_eq!(r["open"], 0);
    }

    #[test]
    fn load_place_clean_keep_versions() {
        let snaps: serde_json::Value = serde_json::from_str(&load(7, CIRCLE, "", "")).unwrap();
        assert!(snaps["snaps"].as_array().unwrap().len() >= 2);
        let p: serde_json::Value =
            serde_json::from_str(&place(7, &[2.0, 0.0, 0.0, 2.0, 0.0, 0.0])).unwrap();
        assert!((p["width"].as_f64().unwrap() - 20.0).abs() < 1e-9);
        let err: serde_json::Value =
            serde_json::from_str(&place(8, &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0])).unwrap();
        assert_eq!(err["error"], "place.unknown");

        load(10, OPEN_SQUARE, "", "");
        let opts = serde_json::to_string(&Options::default()).unwrap();
        let pv: serde_json::Value = serde_json::from_str(&clean_preview(10, &opts, 1.0)).unwrap();
        assert_eq!(pv["counts"]["joins"], 1);
        let done: serde_json::Value =
            serde_json::from_str(&clean_apply(10, &opts, 1.0, 11)).unwrap();
        assert_eq!(done["report"]["closed"], 1);
        // L'ancienne géométrie reste, pour annuler.
        let old: serde_json::Value = serde_json::from_str(&clean_preview(10, &opts, 1.0)).unwrap();
        assert_eq!(old["counts"]["joins"], 1);
        let new: serde_json::Value = serde_json::from_str(&clean_preview(11, &opts, 1.0)).unwrap();
        assert_eq!(new["counts"]["joins"], 0);
    }

    #[test]
    fn sides_and_overlaps() {
        const PLATE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="100mm" viewBox="0 0 100 100"><rect width="100" height="100"/><circle cx="50" cy="50" r="20"/></svg>"#;
        load(20, PLATE, "", "");
        let s: serde_json::Value = serde_json::from_str(&sides(20)).unwrap();
        let mut got: Vec<&str> = s["contours"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["side"].as_str().unwrap())
            .collect();
        got.sort();
        assert_eq!(got, ["inside", "outside"]);
        let err: serde_json::Value = serde_json::from_str(&sides(21)).unwrap();
        assert_eq!(err["error"], "cut.unknown");
        let apart = r#"[{"geo":20,"m":[1,0,0,1,0,0]},{"geo":20,"m":[1,0,0,1,200,0]}]"#;
        assert_eq!(overlaps(apart), "[]");
        let on = r#"[{"geo":20,"m":[1,0,0,1,0,0]},{"geo":20,"m":[1,0,0,1,50,0]}]"#;
        assert_eq!(overlaps(on), "[[0,1]]");
    }

    #[test]
    fn leads_for_a_plate() {
        const PLATE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="100mm" viewBox="0 0 100 100"><rect width="100" height="100"/><circle cx="50" cy="50" r="20"/></svg>"#;
        load(30, PLATE, "", "");
        let s: serde_json::Value = serde_json::from_str(&sides(30)).unwrap();
        let specs: Vec<serde_json::Value> = s["contours"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| serde_json::json!({ "side": c["side"], "shape": "line", "lead_in": 3.8, "lead_out": 1.3, "entry": null }))
            .collect();
        let specs = serde_json::to_string(&specs).unwrap();
        let l: serde_json::Value =
            serde_json::from_str(&leads(30, &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0], &specs)).unwrap();
        assert_eq!(l["leads"].as_array().unwrap().len(), 2);
        assert!(l["leads"][0]["lead_in"].as_array().unwrap().len() >= 4);
        let err: serde_json::Value =
            serde_json::from_str(&leads(31, &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0], &specs)).unwrap();
        assert_eq!(err["error"], "lead.unknown");
    }

    #[test]
    fn plan_and_refused_order() {
        const PLATE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="100mm" viewBox="0 0 100 100"><rect width="100" height="100"/><circle cx="50" cy="50" r="20"/></svg>"#;
        load(40, PLATE, "", "");
        let s: serde_json::Value = serde_json::from_str(&sides(40)).unwrap();
        let cs = s["contours"].as_array().unwrap();
        let specs: Vec<serde_json::Value> = cs
            .iter()
            .map(|c| serde_json::json!({ "side": c["side"], "shape": "line", "lead_in": 3.8, "lead_out": 1.3, "entry": null }))
            .collect();
        let job = |order: serde_json::Value| {
            serde_json::json!({ "start": [0, 0], "order": order, "drawings": [{ "key": 1, "geo": 40, "m": [1, 0, 0, 1, 0, 0], "specs": specs, "qualities": [3, 3] }] }).to_string()
        };
        let p: serde_json::Value =
            serde_json::from_str(&plan(&job(serde_json::Value::Null))).unwrap();
        assert_eq!(p["order"].as_array().unwrap().len(), 2);
        assert_eq!(p["order"][1]["side"], "outside");
        assert_eq!(p["stats"]["pierces"], 2);
        let outside = cs.iter().position(|c| c["side"] == "outside").unwrap();
        let e: serde_json::Value =
            serde_json::from_str(&plan(&job(serde_json::json!([[1, outside]])))).unwrap();
        assert_eq!(e["error"], "path.order");
    }

    #[test]
    fn machine_json_carries_bed_size() {
        let v: serde_json::Value = serde_json::from_str(&machine()).unwrap();
        assert_eq!(v["id"], "protomax");
        assert_eq!(v["width"], 305.0);
    }
}
