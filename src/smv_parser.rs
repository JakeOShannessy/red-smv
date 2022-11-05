use chid::{Chid, Title};
use std::{
    convert::{TryFrom, TryInto},
    io::{BufRead, BufReader, Read},
    ops::Deref,
    path::Path,
    str::FromStr,
    vec,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Rgbf {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Rgbf {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Rgbaf {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Rgbaf {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
}

pub type GridCoord = i64;
pub type Coord = f64;

/// A sextuple of grid coordinates representing a region of cells.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct GridRegion {
    pub i1: GridCoord,
    pub i2: GridCoord,
    pub j1: GridCoord,
    pub j2: GridCoord,
    pub k1: GridCoord,
    pub k2: GridCoord,
}

impl GridRegion {
    pub fn new(
        i1: GridCoord,
        i2: GridCoord,
        j1: GridCoord,
        j2: GridCoord,
        k1: GridCoord,
        k2: GridCoord,
    ) -> Self {
        Self {
            i1,
            i2,
            j1,
            j2,
            k1,
            k2,
        }
    }
}

/// A sextuple of real coordinates representing a region of 3d space.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Xb {
    pub x1: Coord,
    pub x2: Coord,
    pub y1: Coord,
    pub y2: Coord,
    pub z1: Coord,
    pub z2: Coord,
}

impl Xb {
    pub fn new(x1: Coord, x2: Coord, y1: Coord, y2: Coord, z1: Coord, z2: Coord) -> Self {
        Self {
            x1,
            x2,
            y1,
            y2,
            z1,
            z2,
        }
    }

    /// Test if two Xbs intersect (i.e. their bounding boxes). Two bounding boxes
    /// intersect of all 3 dimensions have overlap. EQ is considered overlap.
    pub fn intersect(&self, b: &Xb) -> bool {
        let intersect_x = (self.x2 > b.x1) && (b.x2 > self.x1);
        let intersect_y = (self.y2 > b.y1) && (b.y2 > self.y1);
        let intersect_z = (self.z2 > b.z1) && (b.z2 > self.z1);
        intersect_x && intersect_y && intersect_z
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Xyz {
    pub x: Coord,
    pub y: Coord,
    pub z: Coord,
}

impl Xyz {
    pub fn new(x: Coord, y: Coord, z: Coord) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Debug)]
pub struct SmvFile {
    pub title: Title,
    pub chid: Chid,
    pub input_filename: String,
    pub endf_filename: Option<String>,
    pub fds_version: Option<String>,
    pub surf_def: Option<String>,
    pub csvfs: Vec<CSVEntry>,
    pub meshes: Vec<SmvMesh>,
    pub surfs: Vec<SmvSurface>,
    pub xyzs: Vec<String>,
    pub solid_ht3d: Option<i64>,
    pub view_times: Option<ViewTimes>,
    pub albedo: Option<f64>,
    pub i_blank: Option<u64>,
    pub gvec: Option<Xyz>,
    pub events: Vec<SmvEvent>,
    pub device_acts: Vec<SmvDeviceAct>,
    pub slcfs: Vec<Slcf>,
    pub prt5s: Vec<Prt5>,
    pub bndfs: Vec<Bndf>,
    pub devcs: Vec<SmvDevice>,
    pub texture_origin: Option<Xyz>,
}

impl SmvFile {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let smv_file = std::fs::File::open(path)?;
        let smv_data = parse_smv_file(smv_file)?;
        Ok(smv_data)
    }
}

pub type SurfIndex = u64;

/// The surface indices for each side of the obst.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Surfaces {
    pub min_x: SurfIndex,
    pub max_x: SurfIndex,
    pub min_y: SurfIndex,
    pub max_y: SurfIndex,
    pub min_z: SurfIndex,
    pub max_z: SurfIndex,
}

impl Surfaces {
    pub fn new(
        min_x: SurfIndex,
        max_x: SurfIndex,
        min_y: SurfIndex,
        max_y: SurfIndex,
        min_z: SurfIndex,
        max_z: SurfIndex,
    ) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SmvSurface {
    pub name: String,
    pub ignition_temperature: f64,
    pub emissivity: f64,
    pub surface_type: i64,
    pub t_width: f64,
    pub t_height: f64,
    pub color: Rgbaf,
    pub texture_file: String,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SmvObst {
    pub xb_exact: Xb,
    pub id: i64,
    pub surfaces: Surfaces,
    pub ijk: GridRegion,
    pub colour_index: i64,
    pub block_type: i64,
}

impl SmvObst {
    pub fn new(half1: ObstFirstHalf, half2: ObstSecondHalf) -> Self {
        Self {
            xb_exact: half1.xb_exact,
            id: half1.blockage_id,
            surfaces: half1.surfaces,
            ijk: half2.ijk,
            colour_index: half2.color_index,
            block_type: half2.block_type,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SmvVent {
    pub xb_exact: Xb,
    pub vent_id: u64,
    pub s_num: u64,
    pub texture_origin: Option<Xyz>,
    pub ijk: GridRegion,
    pub vent_index: i64,
    pub vent_type: i64,
    pub color: Option<Rgbaf>,
}

impl SmvVent {
    pub fn new(half1: VentFirstHalf, half2: VentSecondHalf) -> Self {
        Self {
            xb_exact: half1.xb_exact,
            vent_id: half1.vent_id,
            s_num: half1.s_num,
            texture_origin: half1.texture_origin,
            ijk: half2.ijk,
            vent_index: half2.vent_index,
            vent_type: half2.vent_type,
            color: half2.color,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ViewTimes {
    tour_tstart: f64,
    tour_tstop: f64,
    tour_ntimes: usize,
}

#[derive(Clone, Debug, Default)]
struct PendingSmvFile {
    title: Option<String>,
    fds_version: Option<String>,
    revision: Option<String>,
    n_meshes: Option<u64>,
    input_filename: Option<String>,
    endf_filename: Option<String>,
    surf_def: Option<String>,
    view_times: Option<ViewTimes>,
    albedo: Option<f64>,
    i_blank: Option<u64>,
    gvec: Option<Xyz>,
    chid: Option<String>,
    csvfs: Vec<CSVEntry>,
    offsets: Vec<Xyz>,
    grids: Vec<GridBlock>,
    pdims: Vec<PdimBlock>,
    obsts: Vec<Vec<SmvObst>>,
    vents: Vec<Vec<SmvVent>>,
    surfs: Vec<SmvSurface>,
    events: Vec<SmvEvent>,
    prt5s: Vec<Prt5>,
    bndfs: Vec<Bndf>,
    devcs: Vec<SmvDevice>,
    device_acts: Vec<SmvDeviceAct>,
    xyzs: Vec<String>,
    slcfs: Vec<Slcf>,
    inpfs: Vec<String>,
    trnx: Vec<Vec<TrnEntry>>,
    trny: Vec<Vec<TrnEntry>>,
    trnz: Vec<Vec<TrnEntry>>,
    solid_ht3d: Option<i64>,
    texture_origin: Option<Xyz>,
    smoke_3d: Vec<Smoke3d>,
}

impl PendingSmvFile {
    pub fn new() -> Self {
        PendingSmvFile {
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slcf {
    pub cell_centred: bool,
    pub vs: String,
    pub filename: String,
    pub long_name: String,
    pub short_name: String,
    pub units: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prt5 {
    n: usize,
    filename: String,
    a: i64,
    b: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bndf {
    a: u64,
    b: u64,
    filename: String,
    long_name: String,
    short_name: String,
    units: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct GridBlock {
    name: String,
    i_bar: u64,
    j_bar: u64,
    k_bar: u64,
    mesh_type: u64,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct PdimBlock {
    xbar0: f64,
    xbar: f64,
    ybar0: f64,
    ybar: f64,
    zbar0: f64,
    zbar: f64,
    color: Rgbf,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SmvMesh {
    pub name: String,
    pub i_bar: u64,
    pub j_bar: u64,
    pub k_bar: u64,
    pub mesh_type: u64,
    pub obsts: Vec<SmvObst>,
    pub vents: Vec<SmvVent>,
    pub trnx: Vec<TrnEntry>,
    pub trny: Vec<TrnEntry>,
    pub trnz: Vec<TrnEntry>,
    pub dims: Xb,
    pub color: Rgbf,
    pub offset: Xyz,
}

impl SmvMesh {
    fn new(
        grid: GridBlock,
        obsts: Vec<SmvObst>,
        vents: Vec<SmvVent>,
        trnx: Vec<TrnEntry>,
        trny: Vec<TrnEntry>,
        trnz: Vec<TrnEntry>,
        pdims: PdimBlock,
        offset: Xyz,
    ) -> Self {
        Self {
            name: grid.name,
            i_bar: grid.i_bar,
            j_bar: grid.j_bar,
            k_bar: grid.k_bar,
            mesh_type: grid.mesh_type,
            obsts,
            vents,
            trnx,
            trny,
            trnz,
            dims: Xb {
                x1: pdims.xbar0,
                x2: pdims.xbar,
                y1: pdims.ybar0,
                y2: pdims.ybar,
                z1: pdims.zbar0,
                z2: pdims.zbar,
            },
            color: pdims.color,
            offset,
        }
    }
    pub fn xb_from_grid(&self, ijk: GridRegion) -> Xb {
        Xb {
            x1: self.trnx.get(ijk.i1 as usize).unwrap().f,
            x2: self.trnx.get(ijk.i2 as usize).unwrap().f,
            y1: self.trny.get(ijk.j1 as usize).unwrap().f,
            y2: self.trny.get(ijk.j2 as usize).unwrap().f,
            z1: self.trnz.get(ijk.k1 as usize).unwrap().f,
            z2: self.trnz.get(ijk.k2 as usize).unwrap().f,
        }
    }
}

impl TryFrom<PendingSmvFile> for SmvFile {
    type Error = &'static str;
    fn try_from(pending: PendingSmvFile) -> Result<SmvFile, Self::Error> {
        let n_grids = pending.grids.len();
        let equal_n = [
            n_grids,
            pending.obsts.len(),
            pending.vents.len(),
            pending.trnx.len(),
            pending.trny.len(),
            pending.trnz.len(),
            pending.pdims.len(),
            pending.offsets.len(),
        ]
        .iter()
        .all(|&item| item == n_grids);
        if !equal_n {
            return Err("mesh entries unbalanced");
        }
        let iter = pending
            .grids
            .into_iter()
            .zip(pending.obsts.into_iter())
            .zip(pending.vents.into_iter())
            .zip(pending.trnx.into_iter())
            .zip(pending.trny.into_iter())
            .zip(pending.trnz.into_iter())
            .zip(pending.pdims.into_iter())
            .zip(pending.offsets.into_iter());
        let mut meshes = Vec::new();
        for (((((((grid, obsts), vents), trnx), trny), trnz), pdim), offset) in iter {
            let mesh = SmvMesh::new(grid, obsts, vents, trnx, trny, trnz, pdim, offset);
            meshes.push(mesh);
        }
        Ok(SmvFile {
            title: pending.title.unwrap().parse().unwrap(),
            chid: pending.chid.unwrap().parse().unwrap(),
            csvfs: pending.csvfs,
            surfs: pending.surfs,
            meshes,
            xyzs: pending.xyzs,
            solid_ht3d: pending.solid_ht3d,
            input_filename: pending.input_filename.unwrap(),
            endf_filename: pending.endf_filename,
            fds_version: pending.fds_version,
            surf_def: pending.surf_def,
            view_times: pending.view_times,
            albedo: pending.albedo,
            i_blank: pending.i_blank,
            gvec: pending.gvec,
            events: pending.events,
            device_acts: pending.device_acts,
            slcfs: pending.slcfs,
            prt5s: pending.prt5s,
            bndfs: pending.bndfs,
            devcs: pending.devcs,
            texture_origin: pending.texture_origin,
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ObstFirstHalf {
    pub xb_exact: Xb,
    pub blockage_id: i64,
    pub surfaces: Surfaces,
    pub texture_origin: Option<Xyz>,
}

impl FromStr for ObstFirstHalf {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split_whitespace();
        let x1: f64 = values.next().ok_or(())?.parse().unwrap();
        let x2: f64 = values.next().ok_or(())?.parse().unwrap();
        let y1: f64 = values.next().ok_or(())?.parse().unwrap();
        let y2: f64 = values.next().ok_or(())?.parse().unwrap();
        let z1: f64 = values.next().ok_or(())?.parse().unwrap();
        let z2: f64 = values.next().ok_or(())?.parse().unwrap();
        let blockage_id: i64 = values.next().ok_or(())?.parse().unwrap();
        let s_min_x: u64 = values.next().ok_or(())?.parse().unwrap();
        let s_max_x: u64 = values.next().ok_or(())?.parse().unwrap();
        let s_min_y: u64 = values.next().ok_or(())?.parse().unwrap();
        let s_max_y: u64 = values.next().ok_or(())?.parse().unwrap();
        let s_min_z: u64 = values.next().ok_or(())?.parse().unwrap();
        let s_max_z: u64 = values.next().ok_or(())?.parse().unwrap();
        let texture_origin = if let Some(s) = values.next() {
            let x: f64 = s.parse().unwrap();
            let y: f64 = values.next().ok_or(())?.parse().unwrap();
            let z: f64 = values.next().ok_or(())?.parse().unwrap();
            Some(Xyz::new(x, y, z))
        } else {
            None
        };
        Ok(ObstFirstHalf {
            xb_exact: Xb {
                x1,
                x2,
                y1,
                y2,
                z1,
                z2,
            },
            blockage_id,
            surfaces: Surfaces::new(s_min_x, s_max_x, s_min_y, s_max_y, s_min_z, s_max_z),
            texture_origin,
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ObstSecondHalf {
    pub ijk: GridRegion,
    pub color_index: i64,
    pub block_type: i64,
}

impl FromStr for ObstSecondHalf {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split_whitespace();
        let i1: i64 = values.next().ok_or(())?.parse().unwrap();
        let i2: i64 = values.next().ok_or(())?.parse().unwrap();
        let j1: i64 = values.next().ok_or(())?.parse().unwrap();
        let j2: i64 = values.next().ok_or(())?.parse().unwrap();
        let k1: i64 = values.next().ok_or(())?.parse().unwrap();
        let k2: i64 = values.next().ok_or(())?.parse().unwrap();
        let color_index: i64 = values.next().ok_or(())?.parse().unwrap();
        let block_type: i64 = values.next().ok_or(())?.parse().unwrap();
        Ok(ObstSecondHalf {
            ijk: GridRegion::new(i1, i2, j1, j2, k1, k2),
            color_index,
            block_type,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VentFirstHalf {
    pub xb_exact: Xb,
    pub vent_id: u64,
    pub s_num: u64,
    pub texture_origin: Option<Xyz>,
}

impl FromStr for VentFirstHalf {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split_whitespace();
        let xmin: f64 = values.next().ok_or(())?.parse().unwrap();
        let xmax: f64 = values.next().ok_or(())?.parse().unwrap();
        let ymin: f64 = values.next().ok_or(())?.parse().unwrap();
        let ymax: f64 = values.next().ok_or(())?.parse().unwrap();
        let zmin: f64 = values.next().ok_or(())?.parse().unwrap();
        let zmax: f64 = values.next().ok_or(())?.parse().unwrap();
        let vent_id: u64 = values.next().ok_or(())?.parse().unwrap();
        let s_num: u64 = values.next().ok_or(())?.parse().unwrap();
        let texture_origin = if let Some(s) = values.next() {
            let x: f64 = s.parse().unwrap();
            let y: f64 = values.next().ok_or(())?.parse().unwrap();
            let z: f64 = values.next().ok_or(())?.parse().unwrap();
            Some(Xyz::new(x, y, z))
        } else {
            None
        };
        Ok(VentFirstHalf {
            xb_exact: Xb::new(xmin, xmax, ymin, ymax, zmin, zmax),
            vent_id,
            s_num,
            texture_origin,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VentSecondHalf {
    pub ijk: GridRegion,
    pub vent_index: i64,
    pub vent_type: i64,
    pub color: Option<Rgbaf>,
}

impl VentSecondHalf {
    pub fn new(ijk: GridRegion, vent_index: i64, vent_type: i64, color: Option<Rgbaf>) -> Self {
        Self {
            ijk,
            vent_index,
            vent_type,
            color,
        }
    }
}

impl FromStr for VentSecondHalf {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut values = input.split_whitespace();
        let i1: i64 = values.next().ok_or(())?.parse().unwrap();
        let i2: i64 = values.next().ok_or(())?.parse().unwrap();
        let j1: i64 = values.next().ok_or(())?.parse().unwrap();
        let j2: i64 = values.next().ok_or(())?.parse().unwrap();
        let k1: i64 = values.next().ok_or(())?.parse().unwrap();
        let k2: i64 = values.next().ok_or(())?.parse().unwrap();
        let ijk = GridRegion {
            i1,
            i2,
            j1,
            j2,
            k1,
            k2,
        };
        let vent_index: i64 = values.next().ok_or(())?.parse().unwrap();
        let vent_type: i64 = values.next().ok_or(())?.parse().unwrap();
        let color = if let Some(s) = values.next() {
            let r: f64 = s.parse().unwrap();
            let g: f64 = values.next().ok_or(())?.parse().unwrap();
            let b: f64 = values.next().ok_or(())?.parse().unwrap();
            let a: f64 = values.next().ok_or(())?.parse().unwrap();
            Some(Rgbaf::new(r, g, b, a))
        } else {
            None
        };
        Ok(Self::new(ijk, vent_index, vent_type, color))
    }
}

#[derive(Debug)]
enum ParserState {
    None,
    TitleBlock,
    FdsVersion1,
    FdsVersion2,
    Revision,
    NMeshes,
    ViewTimes,
    Albedo,
    IBlank,
    GVec,
    Material,
    ClassOfParticles,
    Outline,
    TOffset,
    HrrPuvCut,
    Ramp,
    Prop,
    Device1,
    Device2(String, String),
    Offset,
    Grid(String),
    Pdim,
    Vent1,
    Vent2(usize, Vec<VentFirstHalf>, usize, Vec<VentFirstHalf>),
    Vent3(
        usize,
        Vec<VentFirstHalf>,
        Vec<VentSecondHalf>,
        usize,
        Vec<VentFirstHalf>,
        Vec<VentSecondHalf>,
    ),
    CVent,
    Smoke3d1(Smoke3dType, u64),
    Smoke3d2(Smoke3dType, u64, String),
    Smoke3d3(Smoke3dType, u64, String, String),
    Smoke3d4(Smoke3dType, u64, String, String, String),
    Bndf1(u64, u64),
    Bndf2(u64, u64, String),
    Bndf3(u64, u64, String, String),
    Bndf4(u64, u64, String, String, String),
    Bndf5(u64, u64, String, String, String, String),
    Slcf1(bool, String),
    Slcf2(bool, String, String),
    Slcf3(bool, String, String, String),
    Slcf4(bool, String, String, String, String),
    Slcf5(bool, String, String, String, String, String),
    Prt51(usize),
    Prt52(usize, String),
    Prt53(usize, String, i64),
    DeviceAct(String),
    ChidBlock,
    SolidHt3d,
    CsvfBlock1,
    CsvfBlock2(String),
    InpfBlock,
    Endf,
    SurfDef,
    ObstBlock1,
    ObstBlock2(usize, Vec<ObstFirstHalf>),
    ObstBlock3(usize, Vec<ObstFirstHalf>, Vec<ObstSecondHalf>),
    Surface1,
    Surface2(String),
    Surface3(String, f64, f64),
    Surface4(String, f64, f64, i64, f64, f64, Rgbaf),
    Trn1(Axis),
    Trn2(Axis, usize, Vec<TrnEntry>),
    Xyz,
    Pl3d,
    CloseVent(usize),
    OpenVent(usize),
    HideObst(usize),
    ShowObst(usize),
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct TrnEntry {
    pub i: usize,
    pub f: f64,
}

impl FromStr for TrnEntry {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split_whitespace();
        let i = values.next().ok_or(())?.parse().unwrap();
        let f = values.next().ok_or(())?.parse().unwrap();
        Ok(TrnEntry { i, f })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum SmvEvent {
    OpenVent { n: usize, i: usize, t: f64 },
    CloseVent { n: usize, i: usize, t: f64 },
    ShowObst { n: usize, i: usize, t: f64 },
    HideObst { n: usize, i: usize, t: f64 },
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SmvDeviceAct {
    name: String,
    n: usize,
    i: usize,
    v: f64,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SmvDevice {
    name: String,
    quantity: String,
    p1: Xyz,
    p2: Xyz,
    ps: Option<(Xyz, Xyz)>,
    beam_type: String,
    state0: i32,
    nparams: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Smoke3dType {
    F,
    G,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Smoke3d {
    pub smoke_type: Smoke3dType,
    pub mesh: u64,
    pub file_name: String,
    pub long_name: String,
    pub short_name: String,
    pub units: String,
}

pub fn parse_smv_file<R: Read>(input: R) -> Result<SmvFile, Box<dyn std::error::Error>> {
    let reader = BufReader::new(input);
    let lines = reader.lines();
    let mut state: ParserState = ParserState::None;
    let mut pending_file = PendingSmvFile::new();
    for line in lines {
        let line = line?;
        if line.is_empty() {
            // Skip over blank lines
            continue;
        }
        let end_block = line.starts_with(|c: char| !c.is_whitespace());
        // Apply special end conditions
        if end_block {
            match state {
                ParserState::Trn2(axis, _skip_n, entries) => {
                    let trns_base = match axis {
                        Axis::X => &mut pending_file.trnx,
                        Axis::Y => &mut pending_file.trny,
                        Axis::Z => &mut pending_file.trnz,
                    };
                    trns_base.push(entries);
                    state = ParserState::None;
                }
                ParserState::ObstBlock2(0, _) => {
                    pending_file.obsts.push(vec![]);
                    state = ParserState::None;
                }
                ParserState::Vent2(0, _, 0, _) => {
                    pending_file.vents.push(vec![]);
                    state = ParserState::None;
                }
                ParserState::Surface3(_, _, _) => (),
                // These blocks don't have spaces at the start
                ParserState::FdsVersion1 | ParserState::FdsVersion2 | ParserState::Revision => (),
                _ => state = ParserState::None,
            }
        }
        match state {
            ParserState::None => {
                // We are not currently in a block. Therefore this line should
                // contain the name of a block.
                if line.starts_with(|c: char| c.is_whitespace()) {
                    continue;
                }
                let (name, remainder) = if let Some(n) = (&line).find(|c: char| c.is_whitespace()) {
                    line.split_at(n)
                } else {
                    (line.deref(), "")
                };
                match name {
                    "TITLE" => {
                        state = ParserState::TitleBlock;
                    }
                    "FDSVERSION" => {
                        state = ParserState::FdsVersion1;
                    }
                    "REVISION" => {
                        state = ParserState::Revision;
                    }
                    "CHID" => {
                        state = ParserState::ChidBlock;
                    }
                    "NMESHES" => {
                        state = ParserState::NMeshes;
                    }
                    "VIEWTIMES" => {
                        state = ParserState::ViewTimes;
                    }
                    "ALBEDO" => {
                        state = ParserState::Albedo;
                    }
                    "IBLANK" => {
                        state = ParserState::IBlank;
                    }
                    "GVEC" => {
                        state = ParserState::GVec;
                    }
                    "MATERIAL" => {
                        state = ParserState::Material;
                    }
                    "CLASS_OF_PARTICLES" => {
                        state = ParserState::ClassOfParticles;
                    }
                    "OUTLINE" => {
                        state = ParserState::Outline;
                    }
                    "TOFFSET" => {
                        state = ParserState::TOffset;
                    }
                    "HRRPUVCUT" => {
                        state = ParserState::HrrPuvCut;
                    }
                    "RAMP" => {
                        state = ParserState::Ramp;
                    }
                    "ENDF" => {
                        state = ParserState::Endf;
                    }
                    "SURFDEF" => {
                        state = ParserState::SurfDef;
                    }
                    "PROP" => {
                        state = ParserState::Prop;
                    }
                    "DEVICE" => {
                        state = ParserState::Device1;
                    }
                    "OFFSET" => {
                        state = ParserState::Offset;
                    }
                    "GRID" => {
                        remainder.strip_prefix(' ').unwrap();
                        state = ParserState::Grid(remainder.to_string());
                    }
                    "PDIM" => {
                        state = ParserState::Pdim;
                    }
                    "VENT" => {
                        state = ParserState::Vent1;
                    }
                    "CVENT" => {
                        state = ParserState::CVent;
                    }
                    "SMOKF3D" => {
                        let n = remainder.trim().parse().unwrap();
                        state = ParserState::Smoke3d1(Smoke3dType::F, n);
                    }
                    "SMOKG3D" => {
                        let n = remainder.trim().parse().unwrap();
                        state = ParserState::Smoke3d1(Smoke3dType::G, n);
                    }
                    "SLCC" => {
                        state = ParserState::Slcf1(true, remainder.to_string());
                    }
                    "SLCF" => {
                        state = ParserState::Slcf1(false, remainder.to_string());
                    }
                    "BNDF" => {
                        let mut values = remainder.split_whitespace();
                        let a = values.next().unwrap().parse().unwrap();
                        let b = values.next().unwrap().parse().unwrap();
                        state = ParserState::Bndf1(a, b);
                    }
                    "PRT5" => {
                        let n = remainder.trim().parse().unwrap();
                        state = ParserState::Prt51(n);
                    }
                    "DEVICE_ACT" => {
                        state = ParserState::DeviceAct(remainder.to_string());
                    }
                    "CSVF" => {
                        state = ParserState::CsvfBlock1;
                    }
                    "INPF" => {
                        state = ParserState::InpfBlock;
                    }
                    "OBST" => {
                        state = ParserState::ObstBlock1;
                    }
                    "TRNX" => {
                        state = ParserState::Trn1(Axis::X);
                    }
                    "TRNY" => {
                        state = ParserState::Trn1(Axis::Y);
                    }
                    "TRNZ" => {
                        state = ParserState::Trn1(Axis::Z);
                    }
                    "SURFACE" => {
                        state = ParserState::Surface1;
                    }
                    "SOLID_HT3D" => {
                        state = ParserState::SolidHt3d;
                    }
                    "PL3D" => {
                        state = ParserState::Pl3d;
                    }
                    "XYZ" => {
                        state = ParserState::Xyz;
                    }
                    "CLOSE_VENT" => {
                        let n = remainder.trim().parse().unwrap();
                        state = ParserState::CloseVent(n);
                    }
                    "OPEN_VENT" => {
                        let n = remainder.trim().parse().unwrap();
                        state = ParserState::OpenVent(n);
                    }
                    "HIDE_OBST" => {
                        let n = remainder.trim().parse().unwrap();
                        state = ParserState::HideObst(n);
                    }
                    "SHOW_OBST" => {
                        let n = remainder.trim().parse().unwrap();
                        state = ParserState::ShowObst(n);
                    }
                    name => {
                        eprintln!("Unrecognized block: \"{}\"", name);
                        state = ParserState::None;
                    }
                }
            }
            ParserState::TitleBlock => {
                // This line is the title
                let line = line.strip_prefix(' ').unwrap();
                pending_file.title = Some(line.parse()?);
                state = ParserState::None;
            }
            ParserState::FdsVersion1 => {
                // This line is the title
                pending_file.fds_version = Some(line.parse()?);
                state = ParserState::FdsVersion2;
            }
            ParserState::FdsVersion2 => {
                // This line is the title
                pending_file.fds_version = Some(line.parse()?);
                state = ParserState::None;
            }
            ParserState::Revision => {
                // This line is the title
                pending_file.revision = Some(line.parse()?);
                state = ParserState::None;
            }
            ParserState::NMeshes => {
                let line = line.strip_prefix(' ').unwrap();
                pending_file.n_meshes = Some(line.trim().parse()?);
                state = ParserState::None;
            }
            ParserState::ViewTimes => {
                let line = line.strip_prefix(' ').unwrap();
                let line = line.trim();
                let mut values = line.split_whitespace();
                let tour_tstart: f64 = values.next().unwrap().parse().unwrap();
                let tour_tstop: f64 = values.next().unwrap().parse().unwrap();
                let tour_ntimes: usize = values.next().unwrap().parse().unwrap();
                pending_file.view_times = Some(ViewTimes {
                    tour_tstart,
                    tour_tstop,
                    tour_ntimes,
                });
                state = ParserState::None;
            }
            ParserState::Albedo => {
                let line = line.strip_prefix(' ').unwrap();
                let albedo: f64 = line.trim().parse()?;
                pending_file.albedo = Some(albedo);
                state = ParserState::None;
            }
            ParserState::IBlank => {
                let line = line.strip_prefix(' ').unwrap();
                let i_blank: u64 = line.trim().parse()?;
                pending_file.i_blank = Some(i_blank);
                state = ParserState::None;
            }
            ParserState::GVec => {
                let line = line.strip_prefix(' ').unwrap();
                let mut values = line.split_whitespace();
                let x: f64 = values.next().unwrap().parse().unwrap();
                let y: f64 = values.next().unwrap().parse().unwrap();
                let z: f64 = values.next().unwrap().parse().unwrap();
                pending_file.gvec = Some(Xyz { x, y, z });
                state = ParserState::None;
            }
            ParserState::Material => {
                // TODO: Parse material
                state = ParserState::None;
            }
            ParserState::ClassOfParticles => {
                // TODO: Parse material
                state = ParserState::None;
            }
            ParserState::Outline => {
                // TODO: Parse outline
                state = ParserState::None;
            }
            ParserState::Pl3d => {
                // TODO: Parse
                state = ParserState::None;
            }
            ParserState::CloseVent(n) => {
                let mut values = line.trim().split_whitespace();
                let i = values.next().unwrap().parse().unwrap();
                let time = values.next().unwrap().parse().unwrap();
                pending_file
                    .events
                    .push(SmvEvent::CloseVent { n, i, t: time });
                state = ParserState::None;
            }
            ParserState::OpenVent(n) => {
                let mut values = line.trim().split_whitespace();
                let i = values.next().unwrap().parse().unwrap();
                let time: f64 = values.next().unwrap().parse().unwrap();
                pending_file
                    .events
                    .push(SmvEvent::OpenVent { n, i, t: time });
                state = ParserState::None;
            }
            ParserState::HideObst(n) => {
                let mut values = line.trim().split_whitespace();
                let i = values.next().unwrap().parse().unwrap();
                let time: f64 = values.next().unwrap().parse().unwrap();
                pending_file
                    .events
                    .push(SmvEvent::HideObst { n, i, t: time });
                state = ParserState::None;
            }
            ParserState::ShowObst(n) => {
                let mut values = line.trim().split_whitespace();
                let i = values.next().unwrap().parse().unwrap();
                let time: f64 = values.next().unwrap().parse().unwrap();
                pending_file
                    .events
                    .push(SmvEvent::ShowObst { n, i, t: time });
                state = ParserState::None;
            }
            ParserState::TOffset => {
                let line = line.trim();
                let mut values = line.split_whitespace();
                let x = values.next().unwrap().parse().unwrap();
                let y = values.next().unwrap().parse().unwrap();
                let z = values.next().unwrap().parse().unwrap();
                pending_file.texture_origin = Some(Xyz { x, y, z });
                state = ParserState::None;
            }
            ParserState::HrrPuvCut => {
                // TODO: Parse
                state = ParserState::None;
            }
            ParserState::Ramp => {
                // TODO: Parse
                state = ParserState::None;
            }
            ParserState::Prop => {
                // TODO: Parse
                state = ParserState::None;
            }
            ParserState::Device1 => {
                let line = line.trim();
                let mut values = line.split('%');
                let name = values.next().unwrap().parse().unwrap();
                let quantity = values.next().unwrap().parse().unwrap();
                state = ParserState::Device2(name, quantity);
            }
            ParserState::Device2(name, quantity) => {
                let mut values = line.split_whitespace();
                let x1 = values.next().unwrap().parse().unwrap();
                let y1 = values.next().unwrap().parse().unwrap();
                let z1 = values.next().unwrap().parse().unwrap();
                let x2 = values.next().unwrap().parse().unwrap();
                let y2 = values.next().unwrap().parse().unwrap();
                let z2 = values.next().unwrap().parse().unwrap();
                let state0: i32 = values.next().unwrap().parse().unwrap();
                let nparams: i32 = values.next().unwrap().parse().unwrap();
                let separator = values.next().unwrap();
                let ps = if separator == "#" {
                    let x1n = values.next().unwrap().parse().unwrap();
                    let y1n = values.next().unwrap().parse().unwrap();
                    let z1n = values.next().unwrap().parse().unwrap();
                    let x2n = values.next().unwrap().parse().unwrap();
                    let y2n = values.next().unwrap().parse().unwrap();
                    let z2n = values.next().unwrap().parse().unwrap();
                    let _extra_separator = values.next().unwrap();
                    Some((Xyz::new(x1n, y1n, z1n), Xyz::new(x2n, y2n, z2n)))
                } else {
                    None
                };
                let beam_type = values.next().unwrap().parse().unwrap();

                let device = SmvDevice {
                    name,
                    quantity,
                    p1: Xyz::new(x1, y1, z1),
                    p2: Xyz::new(x2, y2, z2),
                    ps,
                    beam_type,
                    state0,
                    nparams,
                };
                pending_file.devcs.push(device);
                state = ParserState::None;
            }

            //             DEVICE
            //  AOVVFlow % VOLUME FLOW
            //     31.80000    13.00000    13.80000     0.00000     0.00000    -1.00000  0  0 #     31.30000    12.50000    13.80000    32.30000    13.50000    13.80000 % null
            ParserState::Offset => {
                let line = line.trim();
                let mut values = line.split_whitespace();
                let x = values.next().unwrap().parse().unwrap();
                let y = values.next().unwrap().parse().unwrap();
                let z = values.next().unwrap().parse().unwrap();
                pending_file.offsets.push(Xyz { x, y, z });
                state = ParserState::None;
            }
            ParserState::Pdim => {
                let line = line.trim();
                let mut values = line.split_whitespace();
                let xbar0 = values.next().unwrap().parse().unwrap();
                let xbar = values.next().unwrap().parse().unwrap();
                let ybar0 = values.next().unwrap().parse().unwrap();
                let ybar = values.next().unwrap().parse().unwrap();
                let zbar0 = values.next().unwrap().parse().unwrap();
                let zbar = values.next().unwrap().parse().unwrap();
                let r = values.next().unwrap().parse().unwrap();
                let g = values.next().unwrap().parse().unwrap();
                let b = values.next().unwrap().parse().unwrap();
                pending_file.pdims.push(PdimBlock {
                    xbar0,
                    xbar,
                    ybar0,
                    ybar,
                    zbar0,
                    zbar,
                    color: Rgbf { r, g, b },
                });
                state = ParserState::None;
            }
            ParserState::Vent1 => {
                let mut values = line.split_whitespace();
                let total_vents: usize = values.next().unwrap().parse().unwrap();
                let n_dummy_vents: usize = values.next().unwrap().parse().unwrap();
                let n_vents = total_vents - n_dummy_vents;
                let first_vents = Vec::with_capacity(n_vents);
                let first_dummy_vents = Vec::with_capacity(n_dummy_vents);
                state = ParserState::Vent2(n_vents, first_vents, n_dummy_vents, first_dummy_vents);
            }
            ParserState::Vent2(n_vents, mut first_vents, n_dummy_vents, mut first_dummy_vents) => {
                let f = line.trim().parse().unwrap();
                if first_vents.len() < n_vents {
                    first_vents.push(f);
                } else if first_dummy_vents.len() < n_dummy_vents {
                    first_dummy_vents.push(f);
                }
                if (first_vents.len() < n_vents) || first_dummy_vents.len() < n_dummy_vents {
                    state =
                        ParserState::Vent2(n_vents, first_vents, n_dummy_vents, first_dummy_vents);
                } else {
                    let second_vents = Vec::with_capacity(n_vents);
                    let second_dummy_vents = Vec::with_capacity(n_dummy_vents);
                    state = ParserState::Vent3(
                        n_vents,
                        first_vents,
                        second_vents,
                        n_dummy_vents,
                        first_dummy_vents,
                        second_dummy_vents,
                    );
                }
            }
            ParserState::Vent3(
                n_vents,
                mut first_vents,
                mut second_vents,
                n_dummy_vents,
                mut first_dummy_vents,
                mut second_dummy_vents,
            ) => {
                let f = line.trim().parse().unwrap();
                if second_vents.len() < n_vents {
                    second_vents.push(f);
                } else if second_dummy_vents.len() < n_dummy_vents {
                    second_dummy_vents.push(f);
                }
                if (second_vents.len() < n_vents) || second_dummy_vents.len() < n_dummy_vents {
                    state = ParserState::Vent3(
                        n_vents,
                        first_vents,
                        second_vents,
                        n_dummy_vents,
                        first_dummy_vents,
                        second_dummy_vents,
                    );
                } else {
                    // TODO: should normal and dummy be saved together? Currently they are.
                    first_vents.append(&mut first_dummy_vents);
                    second_vents.append(&mut second_dummy_vents);
                    let mut vents = Vec::with_capacity(n_vents + n_dummy_vents);
                    for (first, second) in first_vents.into_iter().zip(second_vents.into_iter()) {
                        vents.push(SmvVent::new(first, second));
                    }
                    pending_file.vents.push(vents);
                    state = ParserState::None;
                }
            }
            ParserState::CVent => {
                // TODO: Parse
                state = ParserState::None;
            }
            ParserState::Grid(name) => {
                let line = line.trim();
                let mut values = line.split_whitespace();
                let i_bar = values.next().unwrap().parse().unwrap();
                let j_bar = values.next().unwrap().parse().unwrap();
                let k_bar = values.next().unwrap().parse().unwrap();
                let mesh_type = values.next().unwrap().parse().unwrap();
                pending_file.grids.push(GridBlock {
                    name,
                    i_bar,
                    j_bar,
                    k_bar,
                    mesh_type,
                });
                state = ParserState::None;
            }
            ParserState::Smoke3d1(smoke_type, mesh) => {
                let file_name = line.strip_prefix(' ').unwrap().trim().to_string();
                state = ParserState::Smoke3d2(smoke_type, mesh, file_name);
            }
            ParserState::Smoke3d2(smoke_type, mesh, file_name) => {
                let long_name = line.strip_prefix(' ').unwrap().trim().to_string();
                state = ParserState::Smoke3d3(smoke_type, mesh, file_name, long_name);
            }
            ParserState::Smoke3d3(smoke_type, mesh, file_name, long_name) => {
                let short_name = line.strip_prefix(' ').unwrap().trim().to_string();
                state = ParserState::Smoke3d4(smoke_type, mesh, file_name, long_name, short_name);
            }
            ParserState::Smoke3d4(smoke_type, mesh, file_name, long_name, short_name) => {
                let units = line.strip_prefix(' ').unwrap().trim().to_string();
                pending_file.smoke_3d.push(Smoke3d {
                    smoke_type,
                    file_name,
                    mesh,
                    long_name,
                    short_name,
                    units,
                });
                state = ParserState::None;
            }
            ParserState::Slcf1(cell_centred, vs) => {
                let line = line.strip_prefix(' ').unwrap();
                let filename = line.trim().to_string();
                state = ParserState::Slcf2(cell_centred, vs, filename);
            }
            ParserState::Slcf2(cell_centred, vs, filename) => {
                let line = line.strip_prefix(' ').unwrap();
                let long_name = line.trim().to_string();
                state = ParserState::Slcf3(cell_centred, vs, filename, long_name);
            }
            ParserState::Slcf3(cell_centred, vs, filename, long_name) => {
                let line = line.strip_prefix(' ').unwrap();
                let short_name = line.trim().to_string();
                state = ParserState::Slcf4(cell_centred, vs, filename, long_name, short_name);
            }
            ParserState::Slcf4(cell_centred, vs, filename, long_name, short_name) => {
                let line = line.strip_prefix(' ').unwrap();
                let units = line.trim().to_string();
                state =
                    ParserState::Slcf5(cell_centred, vs, filename, long_name, short_name, units);
            }
            ParserState::Slcf5(cell_centred, vs, filename, long_name, short_name, units) => {
                pending_file.slcfs.push(Slcf {
                    cell_centred,
                    vs,
                    filename,
                    long_name,
                    short_name,
                    units,
                });
                state = ParserState::None;
            }
            ParserState::Bndf1(a, b) => {
                let line = line.strip_prefix(' ').unwrap();
                let filename = line.trim().to_string();
                state = ParserState::Bndf2(a, b, filename);
            }
            ParserState::Bndf2(a, b, filename) => {
                let line = line.strip_prefix(' ').unwrap();
                let long_name = line.trim().to_string();
                state = ParserState::Bndf3(a, b, filename, long_name);
            }
            ParserState::Bndf3(a, b, filename, long_name) => {
                let line = line.strip_prefix(' ').unwrap();
                let short_name = line.trim().to_string();
                state = ParserState::Bndf4(a, b, filename, long_name, short_name);
            }
            ParserState::Bndf4(cell_centred, vs, filename, long_name, short_name) => {
                let line = line.strip_prefix(' ').unwrap();
                let units = line.trim().to_string();
                state =
                    ParserState::Bndf5(cell_centred, vs, filename, long_name, short_name, units);
            }
            ParserState::Bndf5(a, b, filename, long_name, short_name, units) => {
                pending_file.bndfs.push(Bndf {
                    a,
                    b,
                    filename,
                    long_name,
                    short_name,
                    units,
                });
                state = ParserState::None;
            }
            ParserState::Prt51(n) => {
                let line = line.strip_prefix(' ').unwrap();
                let filename = line.trim().to_string();
                state = ParserState::Prt52(n, filename);
            }
            ParserState::Prt52(n, filename) => {
                let line = line.strip_prefix(' ').unwrap();
                let a: i64 = line.trim().parse().unwrap();
                state = ParserState::Prt53(n, filename, a);
            }
            ParserState::Prt53(n, filename, a) => {
                let line = line.strip_prefix(' ').unwrap();
                let b: i64 = line.trim().parse().unwrap();
                pending_file.prt5s.push(Prt5 { n, filename, a, b });
                state = ParserState::None;
            }
            ParserState::DeviceAct(name) => {
                let mut values = line.trim().split_whitespace();
                let i = values.next().unwrap().parse().unwrap();
                let v: f64 = values.next().unwrap().parse().unwrap();
                let n = values.next().unwrap().parse().unwrap();
                pending_file
                    .device_acts
                    .push(SmvDeviceAct { name, n, v, i });
                state = ParserState::None;
            }
            ParserState::Endf => {
                let line = line.strip_prefix(' ').unwrap();
                pending_file.endf_filename = Some(line.trim().to_string());
                state = ParserState::None;
            }
            ParserState::SurfDef => {
                let line = line.strip_prefix(' ').unwrap();
                pending_file.surf_def = Some(line.trim().to_string());
                state = ParserState::None;
            }
            ParserState::Xyz => {
                let line = line.strip_prefix(' ').unwrap();
                pending_file.xyzs.push(line.parse()?);
                state = ParserState::None;
            }
            ParserState::ChidBlock => {
                // This line is the CHID
                let line = line.strip_prefix(' ').unwrap();
                pending_file.chid = Some(line.parse()?);
                state = ParserState::None;
            }
            ParserState::SolidHt3d => {
                let line = line.strip_prefix(' ').unwrap();
                pending_file.solid_ht3d = Some(line.trim().parse()?);
                state = ParserState::None;
            }
            ParserState::CsvfBlock1 => {
                state = ParserState::CsvfBlock2(line.trim().to_string());
            }
            ParserState::CsvfBlock2(ref csv_type) => {
                pending_file.csvfs.push(CSVEntry {
                    type_: csv_type.clone(),
                    filename: line.trim().to_string(),
                });
                state = ParserState::None;
            }
            ParserState::InpfBlock => {
                // This line is the input filename
                pending_file.input_filename = Some(line.trim().to_string());
                state = ParserState::None;
            }
            ParserState::ObstBlock1 => {
                // This the number of obsts
                let n: usize = line.trim().parse().unwrap();
                let first_obsts = Vec::with_capacity(n);
                state = ParserState::ObstBlock2(n, first_obsts);
            }
            ParserState::ObstBlock2(n, mut first_obsts) => {
                let f = line.trim().parse().unwrap();
                first_obsts.push(f);
                if first_obsts.len() >= n {
                    let second_obsts = Vec::with_capacity(n);
                    state = ParserState::ObstBlock3(n, first_obsts, second_obsts);
                } else {
                    state = ParserState::ObstBlock2(n, first_obsts);
                }
            }
            ParserState::ObstBlock3(n, first_obsts, mut second_obsts) => {
                let f = line.trim().parse().unwrap();
                second_obsts.push(f);
                if second_obsts.len() >= n {
                    let mut obsts = Vec::with_capacity(n);
                    for (half1, half2) in first_obsts.into_iter().zip(second_obsts.into_iter()) {
                        obsts.push(SmvObst::new(half1, half2));
                    }
                    pending_file.obsts.push(obsts);
                    state = ParserState::None
                } else {
                    state = ParserState::ObstBlock3(n, first_obsts, second_obsts);
                }
            }
            ParserState::Surface1 => {
                let line = line.strip_prefix(' ').unwrap();
                let name = line.trim().parse().unwrap();
                state = ParserState::Surface2(name);
            }
            ParserState::Surface2(name) => {
                let line = line.trim();
                let mut values = line.split_whitespace();
                let ignition_temperature = values.next().unwrap().parse().unwrap();
                let emissivity = values.next().unwrap().parse().unwrap();
                state = ParserState::Surface3(name, ignition_temperature, emissivity);
            }
            ParserState::Surface3(name, ignition_temperature, emissivity) => {
                let line = line.trim();
                let mut values = line.split_whitespace();
                let s_type = values.next().unwrap().parse().unwrap();
                let t_width = values.next().unwrap().parse().unwrap();
                let t_height = values.next().unwrap().parse().unwrap();
                let r = values.next().unwrap().parse().unwrap();
                let g = values.next().unwrap().parse().unwrap();
                let b = values.next().unwrap().parse().unwrap();
                let a = values.next().unwrap().parse().unwrap();
                let color = Rgbaf::new(r, g, b, a);
                state = ParserState::Surface4(
                    name,
                    ignition_temperature,
                    emissivity,
                    s_type,
                    t_width,
                    t_height,
                    color,
                );
            }
            ParserState::Surface4(
                name,
                ignition_temperature,
                emissivity,
                surface_type,
                t_width,
                t_height,
                color,
            ) => {
                let line = line.strip_prefix(' ').unwrap();
                let line = line.trim();
                let texture_file = line.to_string();
                let surface = SmvSurface {
                    name,
                    ignition_temperature,
                    emissivity,
                    surface_type,
                    t_width,
                    t_height,
                    color,
                    texture_file,
                };
                pending_file.surfs.push(surface);
                state = ParserState::None;
            }
            ParserState::Trn1(axis) => {
                let skip_n: usize = line.trim().parse().unwrap();
                let entries = Vec::new();
                state = ParserState::Trn2(axis, skip_n, entries);
            }
            ParserState::Trn2(_axis, ref mut skip_n, ref mut entries) => {
                if *skip_n > 0 {
                    // TODO: this is mimicking smv source code, but not sure why
                    *skip_n -= 1;
                    continue;
                }
                let f = line.trim().parse().unwrap();
                entries.push(f);
            }
        }
    }
    Ok(pending_file.try_into()?)
}

#[derive(Clone, Debug, PartialEq)]
pub struct CSVEntry {
    pub type_: String,
    pub filename: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_smv_simple() {
        let result = parse_smv_file(std::io::Cursor::new(include_str!("room_fire.smv")))
            .expect("smv parsing failed");
        assert_eq!(result.title.as_str(), "Single Couch Test Case");
        assert_eq!(result.chid.as_str(), "room_fire");
        assert_eq!(result.meshes.len(), 1);
        assert_eq!(result.meshes[0].obsts.len(), 737);
        assert_eq!(result.surfs.len(), 15);
        assert_eq!(result.meshes[0].vents.len(), 6);
        assert_eq!(result.csvfs.len(), 3);
        assert_eq!(result.meshes[0].trnx.len(), 25);
        assert_eq!(result.meshes[0].trny.len(), 11);
        assert_eq!(result.meshes[0].trnz.len(), 25);
    }
    #[test]
    fn parse_smv_multimesh() {
        let result = parse_smv_file(std::io::Cursor::new(include_str!("test1.smv")))
            .expect("smv parsing failed");
        assert_eq!(result.title.as_str(), "");
        assert_eq!(result.chid.as_str(), "abcde");
        assert_eq!(result.meshes.len(), 6);
        assert_eq!(result.surfs.len(), 17);
        assert_eq!(result.csvfs.len(), 4);
        assert_eq!(result.meshes[0].trnx.len(), 424);
        assert_eq!(result.meshes[0].trny.len(), 19);
        assert_eq!(result.meshes[0].trnz.len(), 26);
    }
}
