//! Unités des fichiers importés et leur valeur en millimètres.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Mm,
    Cm,
    M,
    In,
    Ft,
    /// Pixel CSS : 96 par pouce.
    Px,
    Pt,
    Pc,
}

impl Unit {
    pub fn mm(self) -> f64 {
        match self {
            Unit::Mm => 1.0,
            Unit::Cm => 10.0,
            Unit::M => 1000.0,
            Unit::In => 25.4,
            Unit::Ft => 304.8,
            Unit::Px => 25.4 / 96.0,
            Unit::Pt => 25.4 / 72.0,
            Unit::Pc => 25.4 / 6.0,
        }
    }

    pub fn parse(s: &str) -> Option<Unit> {
        Some(match s {
            "mm" => Unit::Mm,
            "cm" => Unit::Cm,
            "m" => Unit::M,
            "in" => Unit::In,
            "ft" => Unit::Ft,
            "px" => Unit::Px,
            "pt" => Unit::Pt,
            "pc" => Unit::Pc,
            _ => return None,
        })
    }
}

/// D'où vient l'unité : lue dans le fichier, ou supposée faute d'indication.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    File,
    Assumed,
}
