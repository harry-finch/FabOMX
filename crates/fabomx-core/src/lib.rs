//! Moteur FabOMX : pur (aucune IO, aucun réseau, aucune dépendance
//! d'interface) et déterministe. Millimètres et degrés ; repère de la
//! machine : origine en bas à gauche du plateau, Y vers le haut.

pub mod machine;

pub use machine::{Machine, PROTOMAX};

/// Version du moteur (celle du workspace).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
