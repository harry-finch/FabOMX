//! Machines cibles. Une seule pour l'instant : la ProtoMAX d'OMAX.

use serde::Serialize;

/// Zone de coupe d'une machine, en millimètres, origine en bas à gauche.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Machine {
    /// Identifiant stable, en minuscules.
    pub id: &'static str,
    /// Nom affiché (nom commercial, non traduit).
    pub name: &'static str,
    /// Largeur de la zone de coupe (X), en mm.
    pub width: f64,
    /// Profondeur de la zone de coupe (Y), en mm.
    pub height: f64,
}

/// OMAX ProtoMAX : zone de coupe de 12 × 12 pouces, arrondie à 305 × 305 mm.
pub const PROTOMAX: Machine = Machine {
    id: "protomax",
    name: "ProtoMAX",
    width: 305.0,
    height: 305.0,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protomax_bed_is_twelve_inches_square() {
        assert_eq!(PROTOMAX.width, 305.0);
        assert_eq!(PROTOMAX.height, 305.0);
        assert!((PROTOMAX.width - 12.0 * 25.4).abs() < 0.5);
    }
}
