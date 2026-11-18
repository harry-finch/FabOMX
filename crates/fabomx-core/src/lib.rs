//! Moteur FabOMX : pur (aucune IO, aucun réseau, aucune dépendance
//! d'interface) et déterministe. Millimètres et degrés ; repère de la
//! machine : origine en bas à gauche du plateau, Y vers le haut.

pub mod clean;
pub mod cut;
pub mod drawing;
mod dxf;
pub mod geom;
pub mod import;
pub mod lead;
pub mod machine;
pub mod place;
mod svg;
pub mod units;

pub use import::{import, ImportError, Report};
pub use machine::{Machine, PROTOMAX};
pub use units::{Source, Unit};

/// Version du moteur (celle du workspace).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
