//! G10@P5 M0a：产品 ABI 布局单测（64 位；**不创建 `CudaContext`、不跑 GPU**）。
//!
//! 覆盖：
//! - `CollisionArgs` 496→504：24 个指针（原 23 + `mesh_poly_args`）、标量块 +8、`out`、
//!   尾 3 指针；`mesh_poly_args` 位于指针块末尾 / i32 块之前（偏移 184）；
//! - `MjwarpMeshPolyArgs`（mesh polygon sidecar）：9 指针、72 字节、align 8，顺序与 pinned
//!   `collision_convex.py` L197-205 一致；
//! - 6 个产品入口统一以 `mjwarp_collision_args a` by-value 接收参数且各只有一处声明
//!   （entries 6→6，0 新增入口）。

use std::mem::{align_of, offset_of, size_of};

use cudarc::driver::DeviceRepr;

use super::abi::{CollisionArgs, CollisionContactOut, MjwarpMeshPolyArgs};
use super::source::PRODUCT_SOURCE;
use super::{
    BOXBOX_ENTRY, CONTACT_ENTRY, CONTACT_MULTI_ENTRY, HFIELD_ENTRY, PRIMITIVE_ENTRY, SDF_ENTRY,
};

macro_rules! assert_offsets {
    ($structure:ty, $(($field:ident, $offset:expr)),* $(,)?) => {
        $(
            assert_eq!(
                offset_of!($structure, $field),
                $offset,
                "{}::{} 偏移不符",
                stringify!($structure),
                stringify!($field)
            );
        )*
    };
}

#[cfg(target_pointer_width = "64")]
#[test]
fn abi_collision_args_matches_device_layout_504() {
    fn assert_repr<T: DeviceRepr>() {}
    assert_repr::<CollisionArgs>();
    assert_eq!(align_of::<CollisionArgs>(), 8);
    assert_eq!(size_of::<CollisionArgs>(), 504);
    assert_offsets!(
        CollisionArgs,
        (geom_xpos, 0),
        (geom_xmat, 8),
        (geom_size, 16),
        (geom_margin, 24),
        (geom_gap, 32),
        (geom_rbound, 40),
        (geom_aabb, 48),
        (geom_type, 56),
        (geom_dataid, 64),
        (mesh_vertadr, 72),
        (mesh_vertnum, 80),
        (mesh_graphadr, 88),
        (mesh_octadr, 96),
        (mesh_vert, 104),
        (mesh_graph, 112),
        (hfield_size, 120),
        (hfield_nrow, 128),
        (hfield_ncol, 136),
        (hfield_adr, 144),
        (hfield_data, 152),
        (oct_aabb, 160),
        (oct_child, 168),
        (oct_coeff, 176),
        // M0a：sidecar 指针在指针块末尾、i32 块之前。
        (mesh_poly_args, 184),
        (nworld, 192),
        (ngeom, 196),
        (g1, 200),
        (g2, 204),
        (type1, 208),
        (type2, 212),
        (geomgeomid, 216),
        (naccdmax, 220),
        (hfield_id, 224),
        (oct_root, 228),
        (noct, 232),
        (tolerance, 236),
        (margin, 240),
        (gap, 244),
        (gjk_iterations, 248),
        (epa_iterations, 252),
        (condim, 256),
        (pairid0, 260),
        (pairid1, 264),
        (friction, 268),
        (solref, 288),
        (solreffriction, 296),
        (solimp, 304),
        (adhesion, 324),
        (sdf_initpoints, 328),
        (sdf_iterations, 332),
        (naconmax, 336),
        (efc_width, 340),
        (out, 344),
        (nacon, 480),
        (nccd, 488),
        (overflow, 496),
    );
    assert_eq!(
        offset_of!(CollisionArgs, out) + offset_of!(CollisionContactOut, dist),
        344
    );
    assert_eq!(
        offset_of!(CollisionArgs, out) + offset_of!(CollisionContactOut, efc_address),
        424
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn abi_mesh_poly_args_matches_device_sidecar_layout() {
    fn assert_repr<T: DeviceRepr>() {}
    assert_repr::<MjwarpMeshPolyArgs>();
    assert_eq!(align_of::<MjwarpMeshPolyArgs>(), 8);
    assert_eq!(size_of::<MjwarpMeshPolyArgs>(), 72);
    assert_offsets!(
        MjwarpMeshPolyArgs,
        (mesh_polynum, 0),
        (mesh_polyadr, 8),
        (mesh_polynormal, 16),
        (mesh_polyvertadr, 24),
        (mesh_polyvertnum, 32),
        (mesh_polyvert, 40),
        (mesh_polymapadr, 48),
        (mesh_polymapnum, 56),
        (mesh_polymap, 64),
    );
}

#[test]
fn all_six_entries_take_args_by_value() {
    let entries = [
        CONTACT_ENTRY,
        HFIELD_ENTRY,
        SDF_ENTRY,
        PRIMITIVE_ENTRY,
        CONTACT_MULTI_ENTRY,
        BOXBOX_ENTRY,
    ];
    assert_eq!(entries.len(), 6);
    for entry in entries {
        let declaration = format!("extern \"C\" __global__ void {entry}(mjwarp_collision_args a)");
        assert_eq!(
            PRODUCT_SOURCE.matches(&declaration).count(),
            1,
            "{entry} 须有且仅有一处 `mjwarp_collision_args a` by-value 声明"
        );
    }
    assert_eq!(
        PRODUCT_SOURCE
            .matches("struct mjwarp_collision_args")
            .count(),
        1
    );
    assert_eq!(
        PRODUCT_SOURCE.matches("struct MjwarpMeshPolyArgs").count(),
        1
    );
    assert_eq!(
        PRODUCT_SOURCE
            .matches("const MjwarpMeshPolyArgs* mesh_poly_args;")
            .count(),
        1,
        "设备侧 args 须有且仅有一个 mesh_poly_args sidecar 指针字段"
    );
}
