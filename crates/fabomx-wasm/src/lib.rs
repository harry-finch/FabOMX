//! Moteur FabOMX pour l'interface : appelé depuis un Web Worker. Les
//! données passent en JSON.

use fabomx_core::drawing::Piece;
use fabomx_core::geom::Affine;
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

thread_local! {
    /// Géométrie d'origine des dessins posés, par identifiant de l'interface.
    static DRAWINGS: RefCell<HashMap<u32, Vec<Piece>>> = RefCell::new(HashMap::new());
}

/// Version du moteur.
#[wasm_bindgen]
pub fn engine_version() -> String {
    fabomx_core::VERSION.to_string()
}

/// L'état d'un dessin SVG ou DXF pour le dialogue d'import, en JSON. `unit`
/// vide : l'unité du fichier. Un refus donne `{"error": "<code stable>"}`.
#[wasm_bindgen]
pub fn inspect(text: &str, unit: &str) -> String {
    let unit = fabomx_core::Unit::parse(unit);
    match fabomx_core::import(text, unit, &fabomx_core::PROTOMAX) {
        Ok(report) => serde_json::to_string(&report).expect("rapport sérialisable"),
        Err(e) => serde_json::json!({ "error": e.code() }).to_string(),
    }
}

/// Garde la géométrie d'origine d'un dessin importé sous `id`, pour le
/// placer ensuite. Rend ses points d'accrochage `[x, y, …]` (mm), en JSON,
/// ou `{"error": "<code stable>"}`.
#[wasm_bindgen]
pub fn load(id: u32, text: &str, unit: &str) -> String {
    match fabomx_core::import::parse(text, fabomx_core::Unit::parse(unit)) {
        Ok((_, parsed, _, _)) => {
            let snaps: Vec<f64> = fabomx_core::place::snap_points(&parsed.pieces)
                .into_iter()
                .flat_map(|p| [p.x, p.y])
                .collect();
            DRAWINGS.with(|d| d.borrow_mut().insert(id, parsed.pieces));
            serde_json::json!({ "snaps": snaps }).to_string()
        }
        Err(e) => serde_json::json!({ "error": e.code() }).to_string(),
    }
}

/// Le dessin `id` transformé par la matrice `[a, b, c, d, e, f]` : sa boîte,
/// ses dimensions et ses avertissements, en JSON.
#[wasm_bindgen]
pub fn place(id: u32, m: &[f64]) -> String {
    DRAWINGS.with(|d| {
        let d = d.borrow();
        let (Some(pieces), [a, b, c, dd, e, f]) = (d.get(&id), m) else {
            return serde_json::json!({ "error": "place.unknown" }).to_string();
        };
        let p = fabomx_core::place::place(
            pieces,
            Affine([*a, *b, *c, *dd, *e, *f]),
            &fabomx_core::PROTOMAX,
        );
        serde_json::to_string(&p).expect("placement sérialisable")
    })
}

/// Machine cible (zone de coupe en mm), en JSON.
#[wasm_bindgen]
pub fn machine() -> String {
    serde_json::to_string(&fabomx_core::PROTOMAX).expect("machine sérialisable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_reports_or_refuses() {
        let ok: serde_json::Value = serde_json::from_str(&inspect(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10"><circle cx="5" cy="5" r="5"/></svg>"#,
            "",
        ))
        .unwrap();
        assert_eq!(ok["unit"], "mm");
        assert_eq!(ok["counts"]["circle"], 1);
        let err: serde_json::Value = serde_json::from_str(&inspect("", "")).unwrap();
        assert_eq!(err["error"], "import.empty");
    }

    #[test]
    fn load_then_place() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10"><circle cx="5" cy="5" r="5"/></svg>"#;
        let snaps: serde_json::Value = serde_json::from_str(&load(7, svg, "")).unwrap();
        assert!(snaps["snaps"].as_array().unwrap().len() >= 4);
        let p: serde_json::Value =
            serde_json::from_str(&place(7, &[2.0, 0.0, 0.0, 2.0, 0.0, 0.0])).unwrap();
        assert!((p["width"].as_f64().unwrap() - 20.0).abs() < 1e-9);
        let err: serde_json::Value =
            serde_json::from_str(&place(8, &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0])).unwrap();
        assert_eq!(err["error"], "place.unknown");
    }

    #[test]
    fn machine_json_carries_bed_size() {
        let v: serde_json::Value = serde_json::from_str(&machine()).unwrap();
        assert_eq!(v["id"], "protomax");
        assert_eq!(v["width"], 305.0);
        assert_eq!(v["height"], 305.0);
    }
}
