use super::super::Aabb;
use super::{FLEX_NO_MESH, FlexMeshAssembly, FlexMeshChannel, FlexSceneError};
use crate::model::flex::FlexTopologyFields;

/// 2 worlds × 3 flex（1D/2D/3D 各一）：1D 一条边、2D 一个三角元素 + 一条壳、3D 一个四面体元素
/// + 一个壳三角。world1 的顶点整体 +10（用于验证 world 隔离）。
fn topology() -> FlexTopologyFields {
    FlexTopologyFields {
        nflex: 3,
        nflexelem: 2,
        nflexelemdata: 7,
        nflexshelldata: 5,
        flex_elemadr: vec![0, 0, 1],
        flex_elemnum: vec![0, 1, 1],
        flex_elemdataadr: vec![0, 0, 3],
        flex_elem: vec![0, 1, 2, 0, 1, 2, 3],
        flex_shellnum: vec![0, 1, 1],
        flex_shelldataadr: vec![0, 0, 2],
        flex_shell: vec![0, 1, 0, 1, 2],
    }
}

fn positions() -> Vec<f32> {
    let world0 = [
        0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, // flex0: 2 verts (edge)
        0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, // flex1: 3 verts
        0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, // flex2: 4 verts
    ];
    let mut values = Vec::new();
    for world in 0..2 {
        for value in world0.iter() {
            values.push(if world == 0 { *value } else { *value + 10.0 });
        }
    }
    values
}

const DIMS: [i32; 3] = [1, 2, 3];
const RADII: [f32; 3] = [0.1, 0.2, 0.3];

fn channel<'a>(
    topology: &'a FlexTopologyFields,
    dims: &'a [i32],
    radii: &'a [f32],
    positions: &'a [f32],
) -> FlexMeshChannel<'a> {
    FlexMeshChannel {
        topology,
        flex_dim: dims,
        flex_vertadr: &[0, 2, 5],
        flex_radius: radii,
        flexvert_xpos: positions,
        flex_edge: &[0, 1],
        geom_flexid: &[0, 1, 2],
        geom_edgeid: &[0, -1, -1],
        worlds: 2,
        nflexvert: 9,
    }
}

fn build() -> FlexMeshAssembly {
    let topology = topology();
    let dims = DIMS;
    let radii = RADII;
    let positions = positions();
    let channel = channel(&topology, &dims, &radii, &positions);
    FlexMeshAssembly::build(&channel).expect("混合 1D/2D/3D 装配应成功")
}

#[test]
fn builds_scene_leaves_and_flex_mesh_entries_per_world() {
    let assembly = build();
    // 叶：world-major，每个 world 3 个图元（1D 边胶囊 + 2D 盒 + 3D 盒）。
    assert_eq!(assembly.leaves.bounds.len(), 6);
    assert_eq!(assembly.leaves.groups, [0, 0, 0, 1, 1, 1]);
    // 1D 叶 = 边端点 min/max ∓ 半径。
    assert_eq!(
        assembly.leaves.bounds[0],
        Aabb {
            lower: [-0.1, -0.1, -0.1],
            upper: [1.1, 0.1, 0.1],
        }
    );
    // 2D 叶 = 三角顶点整体包围盒 ± 半径。
    assert_eq!(assembly.leaves.bounds[1].lower, [-0.2, -0.2, -0.2]);
    assert_eq!(assembly.leaves.bounds[1].upper, [1.2, 1.2, 0.2]);
    // 3D 叶同口径。
    assert_eq!(assembly.leaves.bounds[2].lower, [-0.3, -0.3, -0.3]);
    assert_eq!(assembly.leaves.bounds[2].upper, [1.3, 1.3, 1.3]);
    // world1 整体 +10。
    assert_eq!(assembly.leaves.bounds[3].lower[0], 9.9);
    // 设备条目：flex 主序、world 次序；1D 无 mesh。
    assert_eq!(assembly.mesh_count, 4);
    assert_eq!(assembly.bvh_ids, [FLEX_NO_MESH, 0, 2]);
    assert_eq!(assembly.device.entries.len(), 4);
    for window in assembly.device.entries.windows(2) {
        assert!(window[0][0] <= window[1][0] && window[0][1] <= window[1][1]);
    }
}

#[test]
fn packs_mesh_bvh_into_mesh_device_layout() {
    let assembly = build();
    for (slot, entry) in assembly.device.entries.iter().enumerate() {
        let meta_end = assembly
            .device
            .entries
            .get(slot + 1)
            .map(|next| next[0] as usize)
            .unwrap_or(assembly.device.meta.len());
        let meta = &assembly.device.meta[entry[0] as usize..meta_end];
        let (n_nodes, n_tri, n_vert, root) = (
            meta[0] as usize,
            meta[1] as usize,
            meta[2] as usize,
            meta[3] as usize,
        );
        assert!(n_nodes >= 1 && root < n_nodes);
        let links = 4;
        let order = links + 2 * n_nodes;
        let indices = order + n_tri;
        assert_eq!(indices + 3 * n_tri, meta.len());
        assert_eq!(n_vert, 3 * n_tri, "三角面汤：每三角 3 个独立顶点");
        let params_end = assembly
            .device
            .entries
            .get(slot + 1)
            .map(|next| next[1] as usize)
            .unwrap_or(assembly.device.params.len());
        let params = &assembly.device.params[entry[1] as usize..params_end];
        assert_eq!(params.len(), 6 * n_nodes + 3 * n_vert);
        // 每个叶 link 的 begin/end 落在 order 范围内。
        for node in 0..n_nodes {
            let link = meta[links + 2 * node];
            if link < 0 {
                let begin = (-link - 1) as usize;
                let end = meta[links + 2 * node + 1] as usize;
                assert!(begin < end && end <= n_tri);
            }
        }
    }
    // world 隔离：同一 flex 的两个 world 条目顶点整体差 +10（x 轴）。
    let first = assembly.device.entries[0];
    let second = assembly.device.entries[1];
    let first_meta = &assembly.device.meta[first[0] as usize..];
    let second_meta = &assembly.device.meta[second[0] as usize..];
    let first_verts = first[1] as usize + 6 * first_meta[0] as usize;
    let second_verts = second[1] as usize + 6 * second_meta[0] as usize;
    let delta = assembly.device.params[second_verts] - assembly.device.params[first_verts];
    assert_eq!(delta, 10.0);
}

#[test]
fn one_dimensional_only_channel_has_no_mesh_entries() {
    let topology = FlexTopologyFields {
        nflex: 1,
        nflexelem: 0,
        nflexelemdata: 0,
        nflexshelldata: 0,
        flex_elemadr: vec![0],
        flex_elemnum: vec![0],
        flex_elemdataadr: vec![0],
        flex_elem: Vec::new(),
        flex_shellnum: vec![0],
        flex_shelldataadr: vec![0],
        flex_shell: Vec::new(),
    };
    let dims = [1i32];
    let radii = [0.1f32];
    let positions = [
        0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
    ];
    let mut channel = channel(&topology, &dims, &radii, &positions);
    channel.flex_vertadr = &[0];
    channel.geom_flexid = &[0];
    channel.geom_edgeid = &[0];
    channel.nflexvert = 2;
    let assembly = FlexMeshAssembly::build(&channel).expect("1D 通道应成功");
    assert_eq!(assembly.mesh_count, 0);
    assert!(assembly.device.meta.is_empty() && assembly.device.params.is_empty());
    assert_eq!(assembly.bvh_ids, [FLEX_NO_MESH]);
    assert_eq!(assembly.leaves.bounds.len(), 2);
}

#[test]
fn rejects_inconsistent_or_panicking_inputs() {
    let topology = topology();
    let positions = positions();
    let dims = DIMS;
    let radii = RADII;
    // 拓扑 nflex 不一致。
    let mut broken_topology = topology.clone();
    broken_topology.nflex = 2;
    let bad = channel(&broken_topology, &dims, &radii, &positions);
    assert!(matches!(
        FlexMeshAssembly::build(&bad),
        Err(FlexSceneError::Length {
            field: "flex_topology_nflex",
            ..
        })
    ));
    // dims 短向量。
    let bad = channel(&topology, &dims[..2], &radii, &positions);
    assert!(FlexMeshAssembly::build(&bad).is_err());
    // 非有限半径。
    let bad_radii = [0.1f32, f32::NAN, 0.3];
    let bad = channel(&topology, &dims, &bad_radii, &positions);
    assert!(matches!(
        FlexMeshAssembly::build(&bad),
        Err(FlexSceneError::NonFinite {
            field: "flex_radius",
            ..
        })
    ));
    // 2D 元素宽度与 dim 不符（elemnum 改成 2，宽度仍 3）。
    let mut bad_topology = topology.clone();
    bad_topology.flex_elemnum = vec![0, 2, 1];
    let bad = channel(&bad_topology, &dims, &radii, &positions);
    assert!(matches!(
        FlexMeshAssembly::build(&bad),
        Err(FlexSceneError::TopologyWidth { flex: 1, .. })
    ));
    // 顶点地址链断裂。
    let broken_adr = [0i32, 3, 5];
    let mut bad = channel(&topology, &dims, &radii, &positions);
    bad.flex_vertadr = &broken_adr;
    assert!(FlexMeshAssembly::build(&bad).is_err());
    // 位置数组长度不符。
    let bad = channel(&topology, &dims, &radii, &positions[..positions.len() - 3]);
    assert!(matches!(
        FlexMeshAssembly::build(&bad),
        Err(FlexSceneError::Length {
            field: "flexvert_xpos",
            ..
        })
    ));
    // 世界数为 0。
    let mut bad = channel(&topology, &dims, &radii, &positions);
    bad.worlds = 0;
    assert!(matches!(
        FlexMeshAssembly::build(&bad),
        Err(FlexSceneError::Count { field: "worlds" })
    ));
    // 非法维度。
    let bad_dims = [1i32, 4, 3];
    let bad = channel(&topology, &bad_dims, &radii, &positions);
    assert!(matches!(
        FlexMeshAssembly::build(&bad),
        Err(FlexSceneError::UnsupportedDimension { flex: 1, dim: 4 })
    ));
    // 短拓扑向量（S2c 字段短于 nflex）：调用点先自校验，不 panic（QA S2c 观察的硬化要求）。
    let mut short_topology = topology.clone();
    short_topology.flex_elemnum.pop();
    let bad = channel(&short_topology, &dims, &radii, &positions);
    assert!(matches!(
        FlexMeshAssembly::build(&bad),
        Err(FlexSceneError::Topology(_))
    ));
}
