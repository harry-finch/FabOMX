//! Moteur FabOMX pour l'interface : appelé depuis un Web Worker. Les
//! données passent en JSON.

use wasm_bindgen::prelude::*;

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
    fn machine_json_carries_bed_size() {
        let v: serde_json::Value = serde_json::from_str(&machine()).unwrap();
        assert_eq!(v["id"], "protomax");
        assert_eq!(v["width"], 305.0);
        assert_eq!(v["height"], 305.0);
    }
}
