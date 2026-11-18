//! Format OMX de la ProtoMAX (OMAX), tel que l'écrit LAYOUT : un en-tête de
//! commentaires fixe, la date de création (jours depuis 1900), un
//! commentaire libre, puis une ligne par point du parcours. Fins de ligne
//! CRLF, coordonnées en pouces à 10 décimales.
//!
//! Une ligne : `[0], X, Y, Z, TiltStart, TiltEnd, Bow, Q, OffsetSide, R, R,
//! R, XType, XData, 0,[END]`. Son `Q` et son côté valent pour le segment qui
//! part de ce point : 0 = déplacement jet coupé, 9 = amorce, 1 à 5 =
//! qualité de coupe. `Bow` est l'arrondi d'un arc (celui du DXF) : 0 =
//! ligne, 1 = demi-cercle, positif dans le sens trigonométrique.
//!
//! `write` (de `read`) rend les exemples de LAYOUT à l'octet près ; `from_plan`
//! écrit un parcours de FabOMX.

use crate::geom::{Seg, P};
use crate::path::{MoveKind, Plan};
use std::f64::consts::PI;

/// Un pouce, en mm.
const INCH: f64 = 25.4;
/// Codes Q des exemples LAYOUT.
pub const Q_TRAVERSE: u8 = 0;
pub const Q_LEAD: u8 = 9;

/// Un point du parcours et ce qui vaut pour le segment qui en part.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    /// Position (pouces).
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub tilt_start: f64,
    pub tilt_end: f64,
    pub bow: f64,
    pub q: u8,
    pub side: u8,
    pub xtype: i32,
    pub xdata: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Omx {
    /// Date de création, en jours depuis 1900 (comme le tableur : 1 = 1/1/1900).
    pub date: f64,
    /// Lignes du commentaire libre.
    pub comment: Vec<String>,
    pub rows: Vec<Row>,
}

/// Refus d'un fichier OMX, par code stable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OmxError {
    /// Pas un OMX de parcours lisible.
    Unreadable,
}

impl OmxError {
    pub fn code(self) -> &'static str {
        match self {
            OmxError::Unreadable => "omx.unreadable",
        }
    }
}

const HEAD: &[&str] = &[
    "// Slash Slash marks (//) represent remarks that are ignored by the computer",
    "// if the slashes are the first characters in the line",
    "// For additional information about .OMX files, consult the OMAX Interactive Reference.",
    "// The line below identifies this file as a .OMX file.  Do not modify it, or the file will not open.",
    IDENT,
    "2",
    "// Above is a flag that indicates the kind of data this file contains \"2\" means tool path data.",
    "// Below is the version of this file format. Do not modify it, unless you are confident you know what you are doing.",
    "2",
    "// Below is the date that this file was first created, in units of \"days since 1900\".",
];

const IDENT: &str = "This is an OMAX (.OMX) file.  Do not modify the first 2 lines of this file. For information on this file format contact softwareengineering@omax.com or visit http://www.omax.com.";

const MID: &[&str] = &[
    "// The following fields are reserved for future use.  Do not delete or modify them.",
    "[Reserved]",
    "[Reserved]",
    "[Reserved]",
    "[Reserved]",
    "[Reserved]",
    "[Reserved]",
    "// Insert any comments here.  These may be displayed in some dialogs to the user,",
    "// Such as the dialog for the parametric shape library.",
    "[COMMENT]",
];

const TAIL: &[&str] = &[
    "[END]",
    "// [MACROS]",
    "// Insert constant or macro definitions here",
    "// (typically these are used when making a parametric file.)",
    "// Format is:",
    "// ConstantName=Value",
    "// or:",
    "// MacroName=Macro Definition",
    "// For example:",
    "// MyConstant=123.456",
    "// MyFunction=Sin(PrevX/27)*27",
    "// [END]",
    "// [VARIABLES]",
    "// Insert variable declarations here if making a parametric file or other algorithmically generated file.",
    "// The format for variables to expose to the user as part of the UI is:",
    "//   Variable Name, Min Value, Max Value, Default Value, LengthFlag (True or False), Optional Description",
    "// The format for variables to use internally (hidden from user) use the key-word \"Private\" in front, and is:",
    "//   Private Variable Name, Optional Initial value",
    "//   (If initial value is not specified, then it is assumed to be \"0\".)",
    "// [END]",
    "// Tool path data is below.",
    "// Items marked with \"R\" are reserved for future use",
    "// ",
    "//    X,               Y,              Z,              TiltAngleStart, TiltAngleEnd,   Bow,          Q, OffsetSide, Reserved, TiltLagStart, TiltLagEnd, (..More Reserved items...), XType, XData, [END]",
];

/// Un nombre comme LAYOUT : 10 décimales, aligné à droite sur 15 caractères.
fn num(v: f64) -> String {
    let v = if v == 0.0 { 0.0 } else { v };
    let s = format!("{v:.10}");
    // Pas de « -0.0000000000 ».
    let s = if s
        .trim_start_matches('-')
        .chars()
        .all(|c| c == '0' || c == '.')
    {
        s.trim_start_matches('-').to_string()
    } else {
        s
    };
    format!("{s:>15}")
}

fn row(r: &Row) -> String {
    format!(
        "[0],{},{},{},{},{},{}, {}, {},          R,        R,              R,{:>13},{:>15},             0,[END]",
        num(r.x),
        num(r.y),
        num(r.z),
        num(r.tilt_start),
        num(r.tilt_end),
        num(r.bow),
        r.q,
        r.side,
        r.xtype,
        r.xdata
    )
}

/// Le texte du fichier (CRLF).
pub fn write(o: &Omx) -> String {
    let mut lines: Vec<String> = HEAD.iter().map(|s| s.to_string()).collect();
    lines.push(format!("{:.10}", o.date));
    lines.extend(MID.iter().map(|s| s.to_string()));
    lines.extend(o.comment.iter().cloned());
    lines.extend(TAIL.iter().map(|s| s.to_string()));
    lines.extend(o.rows.iter().map(row));
    let mut out = lines.join("\r\n");
    out.push_str("\r\n");
    out
}

/// Lit un OMX de parcours.
pub fn read(text: &str) -> Result<Omx, OmxError> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.get(4).copied() != Some(IDENT) || lines.get(5).map(|l| l.trim()) != Some("2") {
        return Err(OmxError::Unreadable);
    }
    let date = lines
        .iter()
        .position(|l| l.starts_with("// Below is the date"))
        .and_then(|i| lines.get(i + 1))
        .and_then(|l| l.trim().parse::<f64>().ok())
        .ok_or(OmxError::Unreadable)?;
    let start = lines
        .iter()
        .position(|l| l.trim() == "[COMMENT]")
        .ok_or(OmxError::Unreadable)?;
    let end = lines[start..]
        .iter()
        .position(|l| l.trim() == "[END]")
        .ok_or(OmxError::Unreadable)?
        + start;
    let comment = lines[start + 1..end]
        .iter()
        .map(|l| l.to_string())
        .collect();
    let mut rows = Vec::new();
    for l in lines.iter().filter(|l| l.starts_with("[0],")) {
        let f: Vec<&str> = l.split(',').map(str::trim).collect();
        if f.len() < 16 {
            return Err(OmxError::Unreadable);
        }
        let n = |i: usize| f[i].parse::<f64>().map_err(|_| OmxError::Unreadable);
        let k = |i: usize| f[i].parse::<i32>().map_err(|_| OmxError::Unreadable);
        rows.push(Row {
            x: n(1)?,
            y: n(2)?,
            z: n(3)?,
            tilt_start: n(4)?,
            tilt_end: n(5)?,
            bow: n(6)?,
            q: k(7)? as u8,
            side: k(8)? as u8,
            xtype: k(12)?,
            xdata: k(13)?,
        });
    }
    Ok(Omx {
        date,
        comment,
        rows,
    })
}

/// Côté de décalage écrit : 1 quand MAKE doit décaler la saignée (extérieur,
/// trou, tracé ouvert d'un côté : pièce à droite du jet, comme les exemples) ;
/// 0 pour une coupe sur le trait (confirmé à la coupe, A-023).
fn offset_side(offset: bool) -> u8 {
    u8::from(offset)
}

/// Un arc découpé en morceaux d'au plus un demi-tour (l'arrondi d'un tour
/// complet serait infini).
fn pieces(s: &Seg) -> Vec<Seg> {
    match *s {
        Seg::Arc { sweep, .. } if sweep.abs() > PI + 1e-12 => {
            let n = (sweep.abs() / PI).ceil() as usize;
            let mut out = Vec::with_capacity(n);
            let mut rest = *s;
            for k in 0..n - 1 {
                let (a, b) = rest.split(1.0 / (n - k) as f64);
                out.push(a);
                rest = b;
            }
            out.push(rest);
            out
        }
        _ => vec![*s],
    }
}

/// L'OMX d'un parcours de FabOMX : le premier point est le point de départ
/// (là où est la buse au lancement), puis chaque segment de chaque
/// mouvement, et le dernier point. `date` en jours depuis 1900.
pub fn from_plan(plan: &Plan, date: f64, comment: &str) -> Omx {
    let inch = |p: P| (p.x / INCH, p.y / INCH);
    let mut rows = Vec::new();
    let mut last = P::new(plan.start[0], plan.start[1]);
    for m in &plan.moves {
        let (q, side) = match m.kind {
            MoveKind::Traverse => (Q_TRAVERSE, 0),
            MoveKind::LeadIn | MoveKind::LeadOut => (Q_LEAD, offset_side(m.offset)),
            MoveKind::Cut => (m.quality.unwrap_or(3), offset_side(m.offset)),
        };
        for s in m.segs.iter().flat_map(pieces) {
            if s.length() < 1e-9 {
                continue;
            }
            let (x, y) = inch(s.start());
            let bow = match s {
                Seg::Line { .. } => 0.0,
                Seg::Arc { sweep, .. } => (sweep / 4.0).tan(),
            };
            rows.push(Row {
                x,
                y,
                z: 0.0,
                tilt_start: 0.0,
                tilt_end: 0.0,
                bow,
                q,
                side,
                xtype: 0,
                xdata: 0,
            });
            last = s.end();
        }
    }
    let (x, y) = inch(last);
    rows.push(Row {
        x,
        y,
        z: 0.0,
        tilt_start: 0.0,
        tilt_end: 0.0,
        bow: 0.0,
        q: Q_TRAVERSE,
        side: 0,
        xtype: 0,
        xdata: 0,
    });
    Omx {
        date,
        comment: vec![comment.to_string()],
        rows,
    }
}
