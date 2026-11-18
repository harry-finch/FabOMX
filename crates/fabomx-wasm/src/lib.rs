//! Moteur FabOMX pour l'interface : appelé depuis un Web Worker. Les
//! données passent en JSON.

use wasm_bindgen::prelude::*;

/// Version du moteur.
#[wasm_bindgen]
pub fn engine_version() -> String {
    fabomx_core::VERSION.to_string()
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
    fn machine_json_carries_bed_size() {
        let v: serde_json::Value = serde_json::from_str(&machine()).unwrap();
        assert_eq!(v["id"], "protomax");
        assert_eq!(v["width"], 305.0);
        assert_eq!(v["height"], 305.0);
    }
}
