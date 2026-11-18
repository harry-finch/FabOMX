//! Import d'un dessin SVG ou DXF : lecture, puis l'état du dessin pour le
//! dialogue d'import. Contours fermés et ouverts, ce qu'il faudra nettoyer
//! (contours ouverts, doublons, extrémités à joindre, éléments minuscules)
//! et les avertissements. Rien n'est réparé ici : seulement compté et situé.

use crate::drawing::{Kind, Parsed, Piece};
use crate::geom::{BBox, Seg, P};
use crate::machine::Machine;
use crate::units::{Source, Unit};
use crate::{dxf, svg};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

/// Deux extrémités plus proches que ceci sont jointes (mm).
pub const JOINED: f64 = 1e-4;
/// Deux extrémités libres plus proches que ceci sont « à joindre » (mm).
pub const GAP: f64 = 0.01;
/// Un élément plus court que ceci est « minuscule » (mm).
pub const TINY: f64 = 0.1;
/// Tolérance des tracés envoyés à l'affichage (mm).
const DISPLAY_TOL: f64 = 0.02;

/// Refus d'un fichier, par code stable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// Ni SVG ni DXF reconnaissable.
    UnknownFormat,
    /// Syntaxe illisible.
    Unreadable,
    /// DXF binaire, non pris en charge.
    BinaryDxf,
    /// Aucun élément de dessin.
    Empty,
}

impl ImportError {
    pub fn code(self) -> &'static str {
        match self {
            ImportError::UnknownFormat => "import.unknown_format",
            ImportError::Unreadable => "import.unreadable",
            ImportError::BinaryDxf => "import.binary_dxf",
            ImportError::Empty => "import.empty",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Svg,
    Dxf,
}

/// Ce qu'il faudra nettoyer, compté.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Cleanup {
    pub open: usize,
    pub duplicates: usize,
    pub gaps: usize,
    pub tiny: usize,
}

/// Où se trouve un défaut, pour le marquer sur l'aperçu.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Marker {
    pub kind: &'static str,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "code")]
pub enum Warning {
    #[serde(rename = "import.no_closed_contour")]
    NoClosedContour,
    #[serde(rename = "import.larger_than_bed")]
    LargerThanBed { width: f64, height: f64 },
    #[serde(rename = "import.ignored")]
    Ignored { count: usize },
}

/// Une pièce prête à dessiner : ses points (mm) et si elle est d'un contour ouvert.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Trace {
    pub open: bool,
    pub pts: Vec<f64>,
}

/// L'état d'un dessin importé, tel que le montre le dialogue.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub format: Format,
    /// Unité retenue pour la lecture.
    pub unit: Unit,
    /// Unité lue dans le fichier, ou supposée.
    pub detected: Unit,
    pub source: Source,
    pub width: f64,
    pub height: f64,
    /// Boîte englobante `[x0, y0, x1, y1]` (mm).
    pub bbox: [f64; 4],
    pub counts: BTreeMap<Kind, usize>,
    pub closed: usize,
    pub open: usize,
    pub cleanup: Cleanup,
    pub markers: Vec<Marker>,
    pub warnings: Vec<Warning>,
    pub ignored: BTreeMap<String, usize>,
    pub traces: Vec<Trace>,
}

/// Lit `text` (SVG ou DXF) dans l'unité `unit`, ou celle du fichier.
pub fn import(text: &str, unit: Option<Unit>, machine: &Machine) -> Result<Report, ImportError> {
    let (format, parsed, detected, source) = parse(text, unit)?;
    if parsed.pieces.is_empty() {
        return Err(ImportError::Empty);
    }
    Ok(analyze(
        format,
        parsed,
        unit.unwrap_or(detected),
        detected,
        source,
        machine,
    ))
}

/// Lecture seule : le format, les pièces exactes (mm), l'unité du fichier et sa source.
pub fn parse(
    text: &str,
    unit: Option<Unit>,
) -> Result<(Format, Parsed, Unit, Source), ImportError> {
    let head = text.trim_start_matches('\u{feff}').trim_start();
    let (format, (parsed, detected, source)) = if head.starts_with('<') {
        (Format::Svg, svg::read(head, unit)?)
    } else if head.starts_with("AutoCAD Binary DXF") {
        return Err(ImportError::BinaryDxf);
    } else if head.starts_with('0') || head.starts_with("999") {
        (Format::Dxf, dxf::read(head, unit)?)
    } else if head.is_empty() {
        return Err(ImportError::Empty);
    } else {
        return Err(ImportError::UnknownFormat);
    };
    Ok((format, parsed, detected, source))
}

fn analyze(
    format: Format,
    parsed: Parsed,
    unit: Unit,
    detected: Unit,
    source: Source,
    machine: &Machine,
) -> Report {
    let Parsed { pieces, ignored } = parsed;
    let mut bbox = BBox::EMPTY;
    for s in pieces.iter().flat_map(|p| &p.segs) {
        s.extend(&mut bbox);
    }
    let mut counts = BTreeMap::new();
    for p in &pieces {
        *counts.entry(p.kind).or_insert(0) += 1;
    }

    let topo = Topology::build(&pieces);
    let mut markers = Vec::new();
    let mut cleanup = Cleanup {
        open: topo.open,
        ..Cleanup::default()
    };

    // Extrémités libres proches : appariées de la plus proche à la plus lointaine.
    let free: Vec<P> = topo
        .nodes
        .iter()
        .filter(|n| n.degree == 1)
        .map(|n| n.p)
        .collect();
    let grid = Grid::new(&free, GAP);
    let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
    for (i, &p) in free.iter().enumerate() {
        for j in grid.near(p) {
            let d = p.dist(free[j]);
            if j > i && d <= GAP {
                pairs.push((d, i, j));
            }
        }
    }
    pairs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut used = vec![false; free.len()];
    for (_, i, j) in pairs {
        if !used[i] && !used[j] {
            used[i] = true;
            used[j] = true;
            cleanup.gaps += 1;
            let m = free[i].lerp(free[j], 0.5);
            markers.push(Marker {
                kind: "gap",
                x: m.x,
                y: m.y,
            });
        }
    }

    // Segments en double : mêmes extrémités, même forme.
    let mut seen: HashMap<SegKey, ()> = HashMap::new();
    for s in pieces.iter().flat_map(|p| &p.segs) {
        if seen.insert(SegKey::of(s), ()).is_some() {
            cleanup.duplicates += 1;
            let m = s.mid();
            markers.push(Marker {
                kind: "duplicate",
                x: m.x,
                y: m.y,
            });
        }
    }

    for p in &pieces {
        if p.length() < TINY {
            cleanup.tiny += 1;
            let m = p.segs[0].start();
            markers.push(Marker {
                kind: "tiny",
                x: m.x,
                y: m.y,
            });
        }
    }

    let mut warnings = Vec::new();
    if topo.closed == 0 {
        warnings.push(Warning::NoClosedContour);
    }
    if bbox.width() > machine.width + 1e-9 || bbox.height() > machine.height + 1e-9 {
        warnings.push(Warning::LargerThanBed {
            width: bbox.width(),
            height: bbox.height(),
        });
    }
    let n_ignored: usize = ignored.values().sum();
    if n_ignored > 0 {
        warnings.push(Warning::Ignored { count: n_ignored });
    }

    let traces = pieces
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut pts = Vec::new();
            for s in &p.segs {
                let before = pts.len();
                s.points(DISPLAY_TOL, &mut pts);
                // Le premier point d'un segment répète la fin du précédent.
                if before > 0 {
                    pts.remove(before);
                }
            }
            Trace {
                open: topo.piece_open[i],
                pts: pts.into_iter().flat_map(|q| [q.x, q.y]).collect(),
            }
        })
        .collect();

    Report {
        format,
        unit,
        detected,
        source,
        width: bbox.width(),
        height: bbox.height(),
        bbox: [bbox.x0, bbox.y0, bbox.x1, bbox.y1],
        counts,
        closed: topo.closed,
        open: topo.open,
        cleanup,
        markers,
        warnings,
        ignored,
        traces,
    }
}

/// Un nœud : des extrémités confondues (à `JOINED` près).
struct Node {
    p: P,
    degree: usize,
}

/// Les contours : pièces reliées par leurs extrémités confondues. Un
/// contour est fermé quand chacun de ses nœuds relie exactement deux bouts.
struct Topology {
    nodes: Vec<Node>,
    closed: usize,
    open: usize,
    piece_open: Vec<bool>,
}

impl Topology {
    fn build(pieces: &[Piece]) -> Topology {
        let mut nodes: Vec<Node> = Vec::new();
        let mut cells: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        let key = |p: P| ((p.x / JOINED).floor() as i64, (p.y / JOINED).floor() as i64);
        let mut node_of = |p: P, nodes: &mut Vec<Node>| -> usize {
            let (kx, ky) = key(p);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if let Some(list) = cells.get(&(kx + dx, ky + dy)) {
                        if let Some(&n) = list.iter().find(|&&n| nodes[n].p.dist(p) <= JOINED) {
                            return n;
                        }
                    }
                }
            }
            nodes.push(Node { p, degree: 0 });
            cells.entry((kx, ky)).or_default().push(nodes.len() - 1);
            nodes.len() - 1
        };

        // Union-find des pièces par nœud partagé.
        let mut parent: Vec<usize> = (0..pieces.len()).collect();
        fn find(parent: &mut [usize], i: usize) -> usize {
            let mut r = i;
            while parent[r] != r {
                r = parent[r];
            }
            let mut i = i;
            while parent[i] != r {
                let next = parent[i];
                parent[i] = r;
                i = next;
            }
            r
        }
        let mut self_closed = vec![false; pieces.len()];
        let mut ends: Vec<Option<(usize, usize)>> = vec![None; pieces.len()];
        let mut first_piece: HashMap<usize, usize> = HashMap::new();
        for (i, p) in pieces.iter().enumerate() {
            let (a, b) = (p.segs[0].start(), p.segs[p.segs.len() - 1].end());
            if p.closed || a.dist(b) <= JOINED {
                self_closed[i] = true;
                continue;
            }
            let (na, nb) = (node_of(a, &mut nodes), node_of(b, &mut nodes));
            nodes[na].degree += 1;
            nodes[nb].degree += 1;
            ends[i] = Some((na, nb));
            for n in [na, nb] {
                match first_piece.get(&n) {
                    Some(&j) => {
                        let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                        parent[ri] = rj;
                    }
                    None => {
                        first_piece.insert(n, i);
                    }
                }
            }
        }

        // Un composant est ouvert dès qu'un de ses nœuds n'a pas deux bouts.
        let mut comp_open: HashMap<usize, bool> = HashMap::new();
        for (i, e) in ends.iter().enumerate() {
            if let Some((na, nb)) = *e {
                let r = find(&mut parent, i);
                let bad = nodes[na].degree != 2 || nodes[nb].degree != 2;
                *comp_open.entry(r).or_insert(false) |= bad;
            }
        }
        let closed = self_closed.iter().filter(|&&c| c).count()
            + comp_open.values().filter(|&&o| !o).count();
        let open = comp_open.values().filter(|&&o| o).count();
        let piece_open = (0..pieces.len())
            .map(|i| !self_closed[i] && comp_open[&find(&mut parent, i)])
            .collect();
        Topology {
            nodes,
            closed,
            open,
            piece_open,
        }
    }
}

/// Grille de recherche des points voisins.
struct Grid {
    cell: f64,
    cells: HashMap<(i64, i64), Vec<usize>>,
}

impl Grid {
    fn new(pts: &[P], cell: f64) -> Grid {
        let mut cells: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for (i, p) in pts.iter().enumerate() {
            cells.entry(Self::key(*p, cell)).or_default().push(i);
        }
        Grid { cell, cells }
    }

    fn key(p: P, cell: f64) -> (i64, i64) {
        ((p.x / cell).floor() as i64, (p.y / cell).floor() as i64)
    }

    fn near(&self, p: P) -> Vec<usize> {
        let (kx, ky) = Self::key(p, self.cell);
        let mut out = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(list) = self.cells.get(&(kx + dx, ky + dy)) {
                    out.extend(list);
                }
            }
        }
        out
    }
}

/// Clé d'un segment, arrondie à `JOINED`, indépendante de son sens.
#[derive(Hash, PartialEq, Eq)]
struct SegKey(Vec<i64>);

impl SegKey {
    fn of(s: &Seg) -> SegKey {
        let q = |v: f64| (v / JOINED).round() as i64;
        let pt = |p: P| [q(p.x), q(p.y)];
        let (a, b) = (pt(s.start()), pt(s.end()));
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        let mut k = vec![a[0], a[1], b[0], b[1]];
        if let Seg::Arc { c, r, sweep, .. } = *s {
            k.extend([q(c.x), q(c.y), q(r), q(sweep.abs() * 1000.0)]);
            if a == b {
                k.truncate(0);
                k.extend([q(c.x), q(c.y), q(r), q(sweep.abs() * 1000.0)]);
            }
        }
        SegKey(k)
    }
}
