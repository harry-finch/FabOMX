//! Fichier de travail `.fabomx` : un JSON autonome que l'interface compose
//! (dessins, placements, réglages faits à la main, point de départ, ordre).
//! Le moteur fournit la géométrie de chaque dessin telle qu'il la garde,
//! nettoyages compris (`Geometry`), et vérifie l'en-tête d'un fichier avant
//! qu'il soit ouvert (`check`).

use crate::drawing::{Parsed, Piece};
use crate::import::Format;
use crate::units::{Source, Unit};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Valeur du champ `format` d'un fichier de travail.
pub const FORMAT: &str = "fabomx";
/// Version du format écrite par ce moteur ; un fichier plus récent est refusé.
pub const VERSION: u32 = 1;

/// La géométrie d'un dessin, de quoi la ranger à nouveau dans le moteur.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Geometry {
    pub format: Format,
    pub unit: Unit,
    pub detected: Unit,
    pub source: Source,
    pub pieces: Vec<Piece>,
    #[serde(default)]
    pub ignored: BTreeMap<String, usize>,
}

impl Geometry {
    pub fn parsed(&self) -> Parsed {
        Parsed {
            pieces: self.pieces.clone(),
            ignored: self.ignored.clone(),
        }
    }
}

/// Refus d'un fichier de travail, par code stable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobError {
    /// Pas un fichier de travail FabOMX lisible.
    Unreadable,
    /// Écrit par une version plus récente de FabOMX.
    Newer,
}

impl JobError {
    pub fn code(self) -> &'static str {
        match self {
            JobError::Unreadable => "job.unreadable",
            JobError::Newer => "job.newer",
        }
    }
}

/// Le contenu d'un fichier de travail, si c'en est un que ce moteur sait lire.
pub fn check(text: &str) -> Result<serde_json::Value, JobError> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|_| JobError::Unreadable)?;
    if v.get("format").and_then(|f| f.as_str()) != Some(FORMAT) {
        return Err(JobError::Unreadable);
    }
    let version = v
        .get("version")
        .and_then(|n| n.as_u64())
        .ok_or(JobError::Unreadable)?;
    if version > VERSION as u64 {
        return Err(JobError::Newer);
    }
    if !v.get("drawings").is_some_and(|d| d.is_array()) {
        return Err(JobError::Unreadable);
    }
    Ok(v)
}
