//! stage-2 等价性窗口的**宿主侧复算**（被 `tests/stage2_equivalence_window.rs` 以 `#[path]` 引入）。
//!
//! 输入与设备侧逐位相同（`crate::cases` 装配）；这里只做纯 CPU 复算：
//! - 接触 case：`ccd_pair`（`use_multiccd` 与设备内核同口径）；
//! - `sdf-box`：`sdf_pair_contacts`（体积 octree 取自 G5 冻结 bin）。

#![allow(dead_code)]

use mjwarp_rs::physics::{
    CcdOptions, ContactOptions, Geom, MeshView, SdfPair, SdfShape, SdfType, VolumeData, ccd_pair,
    sdf_pair_contacts,
};

use crate::cases::{
    bins_dir, build_meta, g2_pair_inputs, host_mesh_topology, host_pair_inputs_from_bins, is_synth,
    read_f32, read_i32, read_meta,
};

/// 每个 world 的接触行（宿主复算结果）。
#[derive(Clone, Copy, Debug)]
pub struct HostRow {
    pub worldid: i32,
    pub geomcollisionid: i32,
    pub dist: f32,
    pub pos: [f32; 3],
    pub frame: [f32; 9],
}

/// 宿主 `ccd_pair` 复算（每个 world 一次）。`use_multiccd` 必须与设备内核同口径。
pub fn host_contact_rows(case: &str, use_multiccd: bool) -> (Vec<HostRow>, Vec<u32>) {
    let (input, _) = if is_synth(case) {
        g2_pair_inputs(case)
    } else {
        (
            host_pair_inputs_from_bins(case),
            host_mesh_topology(case)
                .map(|mesh| mesh.vert)
                .unwrap_or_default(),
        )
    };
    let meta = input.meta;
    let topology = host_mesh_topology(case);
    let mesh_view = topology.as_ref().map(|mesh| MeshView {
        vert: &mesh.vert,
        graph: &[],
        polyadr: 0,
        polynormal: &mesh.polynormal,
        polyvertadr: &mesh.polyvertadr,
        polyvertnum: &mesh.polyvertnum,
        polyvert: &mesh.polyvert,
        polymapadr: &mesh.polymapadr,
        polymapnum: &mesh.polymapnum,
        polymap: &mesh.polymap,
    });
    let options = ContactOptions {
        ccd: CcdOptions {
            tolerance: meta.tolerance,
            cutoff: meta.gap,
            gjk_iterations: meta.gjk_iterations as usize,
            epa_iterations: meta.epa_iterations as usize,
        },
        margin: meta.margin,
        gap: meta.gap,
        collision_sensor: false,
        use_multiccd,
    };
    let mut rows = Vec::new();
    let mut overflow = Vec::with_capacity(meta.nworld as usize);
    for world in 0..meta.nworld as usize {
        let mut geoms = [
            Geom::simple(
                input.pos[world * 2],
                input.rot[world * 2],
                input.size[0],
                0.0,
            ),
            Geom::simple(
                input.pos[world * 2 + 1],
                input.rot[world * 2 + 1],
                input.size[1],
                0.0,
            ),
        ];
        if let Some(view) = mesh_view.as_ref() {
            for geom in &mut geoms {
                geom.vertadr = 0;
                geom.vertnum = view.vert.len() as i32;
                geom.graphadr = -1;
                geom.mesh = Some(view);
            }
        }
        let output = ccd_pair(&geoms[0], &geoms[1], meta.type1, meta.type2, options);
        overflow.push(output.overflow);
        for (index, contact) in output.contacts.iter().enumerate() {
            rows.push(HostRow {
                worldid: world as i32,
                geomcollisionid: index as i32,
                dist: contact.dist,
                pos: contact.pos,
                frame: contact.frame,
            });
        }
    }
    (rows, overflow)
}

/// 宿主 SDF 窄相复算（`sdf-box`）：体积 octree 与几何位姿均来自 G5 冻结 bin。
pub fn host_sdf_rows(case: &str) -> (Vec<HostRow>, Vec<u32>) {
    let directory = bins_dir(case);
    let meta = build_meta(&read_meta(&directory));
    let xpos = read_f32(&directory, "geom_xpos");
    let xmat = read_f32(&directory, "geom_xmat");
    let size_values = read_f32(&directory, "geom_size");
    let aabb_values = read_f32(&directory, "geom_aabb");
    let oct_aabb_values = read_f32(&directory, "oct_aabb");
    let oct_child_values = read_i32(&directory, "oct_child");
    let oct_coeff_values = read_f32(&directory, "oct_coeff");
    let noct = meta.noct as usize;
    assert!(noct > 0, "SDF case 必须有 octree");
    let oct_aabb: Vec<([f32; 3], [f32; 3])> = (0..noct)
        .map(|node| {
            (
                [
                    oct_aabb_values[node * 6],
                    oct_aabb_values[node * 6 + 1],
                    oct_aabb_values[node * 6 + 2],
                ],
                [
                    oct_aabb_values[node * 6 + 3],
                    oct_aabb_values[node * 6 + 4],
                    oct_aabb_values[node * 6 + 5],
                ],
            )
        })
        .collect();
    let oct_child: Vec<[i32; 8]> = (0..noct)
        .map(|node| {
            let mut row = [0_i32; 8];
            row.copy_from_slice(&oct_child_values[node * 8..node * 8 + 8]);
            row
        })
        .collect();
    let oct_coeff: Vec<[f32; 8]> = (0..noct)
        .map(|node| {
            let mut row = [0.0_f32; 8];
            row.copy_from_slice(&oct_coeff_values[node * 8..node * 8 + 8]);
            row
        })
        .collect();
    let root = meta.oct_root as usize;
    let (center, half_size) = oct_aabb[root];
    let volume = VolumeData::new(center, half_size, oct_aabb, oct_child, oct_coeff, root)
        .expect("VolumeData");
    let local_aabb = |geom: i32| -> ([f32; 3], [f32; 3]) {
        let base = (geom as usize) * 6;
        let mut center = [0.0_f32; 3];
        center.copy_from_slice(&aabb_values[base..base + 3]);
        let mut half = [0.0_f32; 3];
        half.copy_from_slice(&aabb_values[base + 3..base + 6]);
        (center, half)
    };
    let size_of = |geom: i32| -> [f32; 3] {
        let base = (geom as usize) * 3;
        [
            size_values[base],
            size_values[base + 1],
            size_values[base + 2],
        ]
    };
    let pose = |world: usize, geom: i32| -> ([f32; 3], [f32; 9]) {
        let index = world * meta.ngeom as usize + geom as usize;
        let base = index * 3;
        let mut rot = [0.0_f32; 9];
        rot.copy_from_slice(&xmat[index * 9..index * 9 + 9]);
        ([xpos[base], xpos[base + 1], xpos[base + 2]], rot)
    };
    let shape1 = SdfShape::new(
        SdfType::from_geom(meta.type1).expect("geom1 类型"),
        size_of(meta.g1),
    );
    let shape2 = SdfShape::new(
        SdfType::from_geom(meta.type2).expect("geom2 类型"),
        size_of(meta.g2),
    )
    .with_volume(&volume);
    let mut rows = Vec::new();
    let mut overflow = Vec::with_capacity(meta.nworld as usize);
    for world in 0..meta.nworld as usize {
        let (pos1, rot1) = pose(world, meta.g1);
        let (pos2, rot2) = pose(world, meta.g2);
        let pair = SdfPair {
            shape1,
            shape2,
            pos1,
            rot1,
            pos2,
            rot2,
            aabb1: local_aabb(meta.g1),
            aabb2: local_aabb(meta.g2),
            sdf_initpoints: meta.sdf_initpoints as usize,
            sdf_iterations: meta.sdf_iterations as usize,
        };
        let contacts = sdf_pair_contacts(&pair);
        overflow.push(0);
        for (index, contact) in contacts.iter().enumerate() {
            rows.push(HostRow {
                worldid: world as i32,
                geomcollisionid: index as i32,
                dist: contact.dist,
                pos: contact.pos,
                frame: contact.frame,
            });
        }
    }
    (rows, overflow)
}
