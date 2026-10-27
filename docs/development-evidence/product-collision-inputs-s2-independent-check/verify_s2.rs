// QA independent 96/96 reconciliation for S2 (does not reuse the product test support).
// Reads the three frozen 32-bin case directories itself, builds the adapter view with its
// own field table, and compares every adapter output array against the bin bytes and sha256.
use mjwarp_rs::runtime::collision_gpu::{
    CollisionGpuError, CollisionGpuMeta, CollisionInputsSnapshot, inputs_from_snapshot,
    mesh_poly_sidecar_present,
};
use sha2::{Digest, Sha256};
use std::path::Path;

const ROOT: &str = r"D:\Rust\mjwarp-rs";

// (name, case dir, single-world frame?) - the third case lives in the M3 capture window.
const CASES: [(&str, &str); 3] = [
    (
        "mesh-box",
        "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-box",
    ),
    (
        "mesh-cylinder",
        "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-cylinder",
    ),
    (
        "mesh-mesh",
        "docs/development-evidence/mesh-multicontact-m3/cases/mesh-mesh",
    ),
];

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    F,
    I,
}

// id -> (bin file name, host scalar kind); independent of the product spec table.
const FIELDS: [(u32, &str, Kind); 32] = [
    (0, "geom_xpos", Kind::F),
    (1, "geom_xmat", Kind::F),
    (2, "geom_size", Kind::F),
    (3, "geom_margin", Kind::F),
    (4, "geom_gap", Kind::F),
    (5, "geom_rbound", Kind::F),
    (6, "geom_aabb", Kind::F),
    (7, "geom_type", Kind::I),
    (8, "geom_dataid", Kind::I),
    (9, "mesh_vertadr", Kind::I),
    (10, "mesh_vertnum", Kind::I),
    (11, "mesh_graphadr", Kind::I),
    (12, "mesh_octadr", Kind::I),
    (13, "mesh_vert", Kind::F),
    (14, "mesh_graph", Kind::I),
    (15, "mesh_polynum", Kind::I),
    (16, "mesh_polyadr", Kind::I),
    (17, "mesh_polynormal", Kind::F),
    (18, "mesh_polyvertadr", Kind::I),
    (19, "mesh_polyvertnum", Kind::I),
    (20, "mesh_polyvert", Kind::I),
    (21, "mesh_polymapadr", Kind::I),
    (22, "mesh_polymapnum", Kind::I),
    (23, "mesh_polymap", Kind::I),
    (24, "hfield_size", Kind::F),
    (25, "hfield_nrow", Kind::I),
    (26, "hfield_ncol", Kind::I),
    (27, "hfield_adr", Kind::I),
    (28, "hfield_data", Kind::F),
    (29, "oct_aabb", Kind::F),
    (30, "oct_child", Kind::I),
    (31, "oct_coeff", Kind::F),
];

const MESH_POLY_IDS: [u32; 9] = [15, 16, 17, 18, 19, 20, 21, 22, 23];

struct Case {
    f32s: Vec<Vec<f32>>,
    i32s: Vec<Vec<i32>>,
    raw: Vec<Vec<u8>>,
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn read_case(dir: &str) -> Case {
    let dir = Path::new(ROOT).join(dir);
    let mut f32s = Vec::new();
    let mut i32s = Vec::new();
    let mut raw = Vec::new();
    for (_id, name, kind) in FIELDS {
        let path = dir.join(format!("{name}.bin"));
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("missing bin {}: {error}", path.display()));
        assert_eq!(bytes.len() % 4, 0, "{} not 4-byte aligned", name);
        let mut fs = Vec::new();
        let mut is = Vec::new();
        for chunk in bytes.chunks_exact(4) {
            match kind {
                Kind::F => fs.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])),
                Kind::I => is.push(i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])),
            }
        }
        f32s.push(fs);
        i32s.push(is);
        raw.push(bytes);
    }
    Case { f32s, i32s, raw }
}

// counts derived only from bin lengths (nq is not used by the 32-field widths)
struct Counts {
    ngeom: i64,
    nmesh: i64,
    nmeshvert: i64,
    nmeshgraph: i64,
    nmeshpoly: i64,
    nmeshpolyvert: i64,
    nmeshpolymap: i64,
    nhfield: i64,
    nhfielddata: i64,
    noct: i64,
}

impl Counts {
    fn from_case(case: &Case) -> Self {
        Self {
            ngeom: case.i32s[7].len() as i64,
            nmesh: case.i32s[9].len() as i64,
            nmeshvert: case.f32s[13].len() as i64 / 3,
            nmeshgraph: case.i32s[14].len() as i64,
            nmeshpoly: case.f32s[17].len() as i64 / 3,
            nmeshpolyvert: case.i32s[20].len() as i64,
            nmeshpolymap: case.i32s[23].len() as i64,
            nhfield: case.i32s[25].len() as i64,
            nhfielddata: case.f32s[28].len() as i64,
            noct: case.f32s[29].len() as i64 / 6,
        }
    }

    fn width(&self, id: u32) -> i64 {
        match id {
            0 => self.ngeom * 3,
            1 => self.ngeom * 9,
            2 => self.ngeom * 3,
            3 => self.ngeom,
            4 => self.ngeom,
            5 => self.ngeom,
            6 => self.ngeom * 6,
            7 => self.ngeom,
            8 => self.ngeom,
            9 => self.nmesh,
            10 => self.nmesh,
            11 => self.nmesh,
            12 => self.nmesh,
            13 => self.nmeshvert * 3,
            14 => self.nmeshgraph,
            15 => self.nmesh,
            16 => self.nmesh,
            17 => self.nmeshpoly * 3,
            18 => self.nmeshpoly,
            19 => self.nmeshpoly,
            20 => self.nmeshpolyvert,
            21 => self.nmeshvert,
            22 => self.nmeshvert,
            23 => self.nmeshpolymap,
            24 => self.nhfield * 4,
            25 => self.nhfield,
            26 => self.nhfield,
            27 => self.nhfield,
            28 => self.nhfielddata,
            29 => self.noct * 6,
            30 => self.noct * 8,
            31 => self.noct * 8,
            other => panic!("bad id {other}"),
        }
    }
}

fn meta_for(counts: &Counts) -> CollisionGpuMeta {
    CollisionGpuMeta {
        nworld: 1,
        ngeom: counts.ngeom as i32,
        g1: 0,
        g2: 1,
        type1: 0,
        type2: 0,
        geomgeomid: 0,
        naccdmax: 0,
        hfield_id: -1,
        oct_root: -1,
        noct: counts.noct as i32,
        tolerance: 0.0,
        margin: 0.0,
        gap: 0.0,
        gjk_iterations: 0,
        epa_iterations: 0,
        condim: 3,
        pairid0: -1,
        pairid1: -1,
        friction: [0.0; 5],
        solref: [0.0; 2],
        solreffriction: [0.0; 2],
        solimp: [0.0; 5],
        adhesion: 0.0,
        sdf_initpoints: 0,
        sdf_iterations: 0,
        naconmax: 64,
        efc_width: 4,
    }
}

fn view<'a>(case: &'a Case, mesh_poly: bool) -> CollisionInputsSnapshot<'a> {
    let keep = |id: u32| mesh_poly || !MESH_POLY_IDS.contains(&id);
    let f = |id: u32| -> &'a [f32] {
        if keep(id) {
            &case.f32s[id as usize]
        } else {
            &[]
        }
    };
    let i = |id: u32| -> &'a [i32] {
        if keep(id) {
            &case.i32s[id as usize]
        } else {
            &[]
        }
    };
    CollisionInputsSnapshot {
        geom_xpos: f(0),
        geom_xmat: f(1),
        geom_size: f(2),
        geom_margin: f(3),
        geom_gap: f(4),
        geom_rbound: f(5),
        geom_aabb: f(6),
        geom_type: i(7),
        geom_dataid: i(8),
        mesh_vertadr: i(9),
        mesh_vertnum: i(10),
        mesh_graphadr: i(11),
        mesh_octadr: i(12),
        mesh_vert: f(13),
        mesh_graph: i(14),
        mesh_polynum: i(15),
        mesh_polyadr: i(16),
        mesh_polynormal: f(17),
        mesh_polyvertadr: i(18),
        mesh_polyvertnum: i(19),
        mesh_polyvert: i(20),
        mesh_polymapadr: i(21),
        mesh_polymapnum: i(22),
        mesh_polymap: i(23),
        hfield_size: f(24),
        hfield_nrow: i(25),
        hfield_ncol: i(26),
        hfield_adr: i(27),
        hfield_data: f(28),
        oct_aabb: f(29),
        oct_child: i(30),
        oct_coeff: f(31),
    }
}

fn output_bytes(inputs: &mjwarp_rs::runtime::collision_gpu::CollisionGpuInputs<'_>, id: u32) -> Vec<u8> {
    let f = |values: &[f32]| -> Vec<u8> { values.iter().flat_map(|v| v.to_le_bytes()).collect() };
    let i = |values: &[i32]| -> Vec<u8> { values.iter().flat_map(|v| v.to_le_bytes()).collect() };
    match id {
        0 => f(inputs.geom_xpos),
        1 => f(inputs.geom_xmat),
        2 => f(inputs.geom_size),
        3 => f(inputs.geom_margin),
        4 => f(inputs.geom_gap),
        5 => f(inputs.geom_rbound),
        6 => f(inputs.geom_aabb),
        7 => i(inputs.geom_type),
        8 => i(inputs.geom_dataid),
        9 => i(inputs.mesh_vertadr),
        10 => i(inputs.mesh_vertnum),
        11 => i(inputs.mesh_graphadr),
        12 => i(inputs.mesh_octadr),
        13 => f(inputs.mesh_vert),
        14 => i(inputs.mesh_graph),
        15 => i(inputs.mesh_polynum),
        16 => i(inputs.mesh_polyadr),
        17 => f(inputs.mesh_polynormal),
        18 => i(inputs.mesh_polyvertadr),
        19 => i(inputs.mesh_polyvertnum),
        20 => i(inputs.mesh_polyvert),
        21 => i(inputs.mesh_polymapadr),
        22 => i(inputs.mesh_polymapnum),
        23 => i(inputs.mesh_polymap),
        24 => f(inputs.hfield_size),
        25 => i(inputs.hfield_nrow),
        26 => i(inputs.hfield_ncol),
        27 => i(inputs.hfield_adr),
        28 => f(inputs.hfield_data),
        29 => f(inputs.oct_aabb),
        30 => i(inputs.oct_child),
        31 => f(inputs.oct_coeff),
        other => panic!("bad id {other}"),
    }
}

fn read_meta(dir: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    let path = Path::new(ROOT).join(dir).join("meta.txt");
    let text = std::fs::read_to_string(&path).expect("meta.txt");
    let mut map = std::collections::BTreeMap::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else { continue };
        map.insert(key.to_string(), parts.map(str::to_string).collect());
    }
    map
}

fn main() {
    let mut mismatches = 0usize;
    let mut compared = 0usize;
    for (name, dir) in CASES {
        let case = read_case(dir);
        let counts = Counts::from_case(&case);
        let meta = read_meta(dir);
        assert_eq!(meta["nworld"], vec!["1".to_string()], "{name} nworld");
        assert_eq!(
            meta["ngeom"][0].parse::<i64>().unwrap(),
            counts.ngeom,
            "{name} meta ngeom"
        );
        assert!(meta["naconmax"][0].parse::<i64>().unwrap() > 0, "{name} naconmax");
        assert!(meta["efc_width"][0].parse::<i64>().unwrap() > 0, "{name} efc_width");
        for (id, field, _kind) in FIELDS {
            let expected = counts.width(id);
            assert_eq!(
                case.raw[id as usize].len() as i64,
                expected * 4,
                "{name} {field} bin width"
            );
        }
        let inputs = inputs_from_snapshot(&view(&case, true), meta_for(&counts))
            .unwrap_or_else(|error| panic!("{name} adapter failed: {error:?}"));
        assert!(mesh_poly_sidecar_present(&inputs), "{name} sidecar");
        for (id, field, _kind) in FIELDS {
            let actual = output_bytes(&inputs, id);
            let expected = &case.raw[id as usize];
            compared += 1;
            if actual != *expected {
                mismatches += 1;
                println!("MISMATCH {name} {field} id={id}");
            }
            println!(
                "{name} id={id:02} {field} bytes={} bin_sha={} adapter_sha={} equal={}",
                expected.len(),
                sha256(expected),
                sha256(&actual),
                actual == *expected
            );
        }
        // legacy 23-item view: 9 mesh_poly ids empty -> validate Ok, sidecar null, 23 fields pass through
        let legacy = view(&case, false);
        let legacy_inputs = inputs_from_snapshot(&legacy, meta_for(&counts))
            .unwrap_or_else(|error| panic!("{name} legacy adapter failed: {error:?}"));
        assert!(!mesh_poly_sidecar_present(&legacy_inputs), "{name} legacy sidecar");
        for id in MESH_POLY_IDS {
            assert!(output_bytes(&legacy_inputs, id).is_empty(), "{name} legacy id {id}");
        }
        for (id, field, _kind) in FIELDS {
            if MESH_POLY_IDS.contains(&id) {
                continue;
            }
            assert_eq!(
                output_bytes(&legacy_inputs, id),
                case.raw[id as usize],
                "{name} legacy pass-through {field}"
            );
        }
        println!("{name} legacy23: empty9+sidecar_null+pass-through23 OK");
    }

    // negative cases on the first case
    let case = read_case(CASES[0].1);
    let counts = Counts::from_case(&case);
    let mut partial = view(&case, false);
    partial.mesh_polynum = &case.i32s[15];
    match inputs_from_snapshot(&partial, meta_for(&counts)) {
        Err(CollisionGpuError::InvalidArgument(message)) if message.contains("不完整") => {
            println!("negative partial mesh_poly: InvalidArgument({message}) OK")
        }
        other => panic!("partial mesh_poly: {other:?}"),
    }
    let mut bad_xpos = view(&case, true);
    bad_xpos.geom_xpos = &[];
    match inputs_from_snapshot(&bad_xpos, meta_for(&counts)) {
        Err(CollisionGpuError::InvalidArgument(message)) if message.contains("geom_xpos") => {
            println!("negative empty geom_xpos: InvalidArgument OK")
        }
        other => panic!("empty geom_xpos: {other:?}"),
    }
    let mut bad_type = view(&case, true);
    bad_type.geom_type = &case.i32s[7][..case.i32s[7].len() - 1];
    match inputs_from_snapshot(&bad_type, meta_for(&counts)) {
        Err(CollisionGpuError::InvalidArgument(message)) if message.contains("geom_type") => {
            println!("negative short geom_type: InvalidArgument OK")
        }
        other => panic!("short geom_type: {other:?}"),
    }
    for (label, mutate) in [
        ("nworld", 0usize),
        ("naconmax", 1),
        ("efc_width", 2),
    ] {
        let mut meta = meta_for(&counts);
        match mutate {
            0 => meta.nworld = 0,
            1 => meta.naconmax = 0,
            _ => meta.efc_width = 0,
        }
        match inputs_from_snapshot(&view(&case, true), meta) {
            Err(CollisionGpuError::InvalidArgument(message)) if message.contains(label) => {
                println!("negative meta {label}=0: InvalidArgument OK")
            }
            other => panic!("meta {label}: {other:?}"),
        }
    }
    println!("SUMMARY compared={compared} mismatches={mismatches}");
    assert_eq!(mismatches, 0);
    assert_eq!(compared, 96);
}
