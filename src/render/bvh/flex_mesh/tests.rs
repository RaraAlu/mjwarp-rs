use super::{FlexMeshBvh, FlexMeshError, FlexMeshInput};

/// 单个三角元素 + 一条边界边的 2D 网格：面数 = 2*1 + 2*1 = 4（上游 L1017-1018）。
fn two_dimensional() -> (FlexMeshInput<'static>, Vec<[f32; 3]>) {
    let input = FlexMeshInput {
        dim: 2,
        radius: 0.5,
        vert_adr: 0,
        vert_num: 3,
        elem_data_adr: 0,
        elem_num: 1,
        elem: &[0, 1, 2],
        shell_data_adr: 0,
        shell_num: 1,
        shell: &[0, 1],
    };
    let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    (input, vertices)
}

/// 2D 顶/底/侧面的点与绕序（法线 = +z，偏移 ±radius）。
#[test]
fn two_dimensional_faces_follow_upstream_offset_and_winding() {
    let (input, vertices) = two_dimensional();
    let mesh = FlexMeshBvh::build(&input, &vertices).expect("2D 网格应构建");
    assert_eq!(mesh.triangle_count(), 4);
    let points = mesh.points();
    // top face：(v0,v1,v2) + r*z
    assert_eq!(points[0], [0.0, 0.0, 0.5]);
    assert_eq!(points[1], [1.0, 0.0, 0.5]);
    assert_eq!(points[2], [0.0, 1.0, 0.5]);
    // bottom face：绕序 (v0,v2,v1)，z = -r
    assert_eq!(points[3], [0.0, 0.0, -0.5]);
    assert_eq!(points[4], [0.0, 1.0, -0.5]);
    assert_eq!(points[5], [1.0, 0.0, -0.5]);
    // side face 0：(v0+r*n0, v1-r*n1, v1+r*n1)
    assert_eq!(points[6], [0.0, 0.0, 0.5]);
    assert_eq!(points[7], [1.0, 0.0, -0.5]);
    assert_eq!(points[8], [1.0, 0.0, 0.5]);
    // side face 1：(v1-r*n1, v0-r*n0, v0+r*n0)
    assert_eq!(points[9], [1.0, 0.0, -0.5]);
    assert_eq!(points[10], [0.0, 0.0, -0.5]);
    assert_eq!(points[11], [0.0, 0.0, 0.5]);
    // group_root = 该 (flex, world) 子树根（MeshBvh 最后一个节点）。
    let root = mesh.group_root() as usize;
    assert_eq!(root, mesh.bvh().nodes().len() - 1);
}

/// 3D 只由 shell 片段构成：面点就是世界顶点（上游 `_build_flex_3d_shells`）。
#[test]
fn three_dimensional_faces_are_shell_triangles() {
    let input = FlexMeshInput {
        dim: 3,
        radius: 0.25,
        vert_adr: 0,
        vert_num: 4,
        elem_data_adr: 0,
        elem_num: 0,
        elem: &[],
        shell_data_adr: 0,
        shell_num: 1,
        shell: &[0, 1, 2],
    };
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let mesh = FlexMeshBvh::build(&input, &vertices).expect("3D 网格应构建");
    assert_eq!(mesh.triangle_count(), 1);
    assert_eq!(mesh.points(), &vertices[..3]);
}

/// 重拟合：顶点平移后点与 BVH 包围盒跟随更新，语义命中仍成立。
#[test]
fn refit_follows_world_space_vertices() {
    let (input, mut vertices) = two_dimensional();
    let mut mesh = FlexMeshBvh::build(&input, &vertices).expect("2D 网格应构建");
    for vertex in &mut vertices {
        vertex[2] += 2.0;
    }
    mesh.refit(&input, &vertices)
        .expect("重拟合应接受同拓扑顶点");
    assert_eq!(mesh.points()[0][2], 2.5);
    assert!(
        !mesh
            .bvh()
            .ray_candidates([0.1, 0.1, 5.0], [0.0, 0.0, -1.0])
            .is_empty(),
        "重拟合后射线应进入三角形包围盒"
    );
}

/// 不支持维度 / 拓扑越界在分配前显式拒绝。
#[test]
fn rejects_unsupported_dimensions_and_bad_topology() {
    let (mut input, vertices) = two_dimensional();
    input.dim = 1;
    assert_eq!(
        FlexMeshBvh::build(&input, &vertices).err(),
        Some(FlexMeshError::UnsupportedDimension { dim: 1 })
    );
    let (mut input, vertices) = two_dimensional();
    input.shell = &[0, 9];
    assert!(matches!(
        FlexMeshBvh::build(&input, &vertices),
        Err(FlexMeshError::IndexOutOfRange { .. })
    ));
    let (mut input, vertices) = two_dimensional();
    input.elem = &[0, 1];
    assert!(matches!(
        FlexMeshBvh::build(&input, &vertices),
        Err(FlexMeshError::IndexOutOfRange { .. })
    ));
}
