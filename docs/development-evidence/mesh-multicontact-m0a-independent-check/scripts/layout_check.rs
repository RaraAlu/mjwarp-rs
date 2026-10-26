// Independent layout check for M0a ABI (copies the struct definitions from
// src/runtime/collision_gpu/abi.rs; no crate-private access needed).
use std::mem::{align_of, offset_of, size_of};

#[repr(C)]
pub struct CollisionContactOut {
    pub dist: *mut f32,
    pub pos: *mut f32,
    pub frame: *mut f32,
    pub includemargin: *mut f32,
    pub friction: *mut f32,
    pub solref: *mut f32,
    pub solreffriction: *mut f32,
    pub solimp: *mut f32,
    pub dim: *mut i32,
    pub geom: *mut i32,
    pub efc_address: *mut i32,
    pub worldid: *mut i32,
    pub contact_type: *mut i32,
    pub geomcollisionid: *mut i32,
    pub adhesion: *mut f32,
    pub nacon: *mut i32,
    pub efc_address_width: i32,
}

#[repr(C)]
pub struct MjwarpMeshPolyArgs {
    pub mesh_polynum: *const i32,
    pub mesh_polyadr: *const i32,
    pub mesh_polynormal: *const f32,
    pub mesh_polyvertadr: *const i32,
    pub mesh_polyvertnum: *const i32,
    pub mesh_polyvert: *const i32,
    pub mesh_polymapadr: *const i32,
    pub mesh_polymapnum: *const i32,
    pub mesh_polymap: *const i32,
}

#[repr(C)]
pub struct CollisionArgs {
    pub geom_xpos: *const f32,
    pub geom_xmat: *const f32,
    pub geom_size: *const f32,
    pub geom_margin: *const f32,
    pub geom_gap: *const f32,
    pub geom_rbound: *const f32,
    pub geom_aabb: *const f32,
    pub geom_type: *const i32,
    pub geom_dataid: *const i32,
    pub mesh_vertadr: *const i32,
    pub mesh_vertnum: *const i32,
    pub mesh_graphadr: *const i32,
    pub mesh_octadr: *const i32,
    pub mesh_vert: *const f32,
    pub mesh_graph: *const i32,
    pub hfield_size: *const f32,
    pub hfield_nrow: *const i32,
    pub hfield_ncol: *const i32,
    pub hfield_adr: *const i32,
    pub hfield_data: *const f32,
    pub oct_aabb: *const f32,
    pub oct_child: *const i32,
    pub oct_coeff: *const f32,
    pub mesh_poly_args: *const MjwarpMeshPolyArgs,
    pub nworld: i32,
    pub ngeom: i32,
    pub g1: i32,
    pub g2: i32,
    pub type1: i32,
    pub type2: i32,
    pub geomgeomid: i32,
    pub naccdmax: i32,
    pub hfield_id: i32,
    pub oct_root: i32,
    pub noct: i32,
    pub tolerance: f32,
    pub margin: f32,
    pub gap: f32,
    pub gjk_iterations: i32,
    pub epa_iterations: i32,
    pub condim: i32,
    pub pairid0: i32,
    pub pairid1: i32,
    pub friction: [f32; 5],
    pub solref: [f32; 2],
    pub solreffriction: [f32; 2],
    pub solimp: [f32; 5],
    pub adhesion: f32,
    pub sdf_initpoints: i32,
    pub sdf_iterations: i32,
    pub naconmax: i32,
    pub efc_width: i32,
    pub out: CollisionContactOut,
    pub nacon: *mut i32,
    pub nccd: *mut i32,
    pub overflow: *mut u32,
}

fn main() {
    println!(
        "CollisionArgs size={} align={}",
        size_of::<CollisionArgs>(),
        align_of::<CollisionArgs>()
    );
    println!("mesh_poly_args offset={}", offset_of!(CollisionArgs, mesh_poly_args));
    println!("nworld offset={}", offset_of!(CollisionArgs, nworld));
    println!("efc_width offset={}", offset_of!(CollisionArgs, efc_width));
    println!("out offset={}", offset_of!(CollisionArgs, out));
    println!("nacon offset={}", offset_of!(CollisionArgs, nacon));
    println!("nccd offset={}", offset_of!(CollisionArgs, nccd));
    println!("overflow offset={}", offset_of!(CollisionArgs, overflow));
    println!(
        "CollisionContactOut size={} align={}",
        size_of::<CollisionContactOut>(),
        align_of::<CollisionContactOut>()
    );
    println!(
        "MjwarpMeshPolyArgs size={} align={}",
        size_of::<MjwarpMeshPolyArgs>(),
        align_of::<MjwarpMeshPolyArgs>()
    );
    for (name, off) in [
        ("mesh_polynum", offset_of!(MjwarpMeshPolyArgs, mesh_polynum)),
        ("mesh_polyadr", offset_of!(MjwarpMeshPolyArgs, mesh_polyadr)),
        ("mesh_polynormal", offset_of!(MjwarpMeshPolyArgs, mesh_polynormal)),
        ("mesh_polyvertadr", offset_of!(MjwarpMeshPolyArgs, mesh_polyvertadr)),
        ("mesh_polyvertnum", offset_of!(MjwarpMeshPolyArgs, mesh_polyvertnum)),
        ("mesh_polyvert", offset_of!(MjwarpMeshPolyArgs, mesh_polyvert)),
        ("mesh_polymapadr", offset_of!(MjwarpMeshPolyArgs, mesh_polymapadr)),
        ("mesh_polymapnum", offset_of!(MjwarpMeshPolyArgs, mesh_polymapnum)),
        ("mesh_polymap", offset_of!(MjwarpMeshPolyArgs, mesh_polymap)),
    ] {
        println!("sidecar {name} offset={off}");
    }
}
