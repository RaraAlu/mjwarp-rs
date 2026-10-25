//! GJK/EPA 单元测试：支撑点、单纯形子算法、分支语义与接触装配。

use super::geom_type::{BOX, CAPSULE, CYLINDER, ELLIPSOID, MESH, SPHERE};
use super::math::{dot3, length3, normalize3};
use super::support::support;
use super::{CcdOptions, ContactOptions, Geom, ccd, ccd_pair};

fn identity() -> [f32; 9] {
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
}

fn sphere(pos: [f32; 3], radius: f32) -> Geom<'static> {
    Geom::simple(pos, identity(), [radius, 0.0, 0.0], 0.0)
}

fn box_geom(pos: [f32; 3], size: [f32; 3]) -> Geom<'static> {
    Geom::simple(pos, identity(), size, 0.0)
}

#[test]
fn sphere_support_matches_center_plus_radius() {
    let geom = sphere([0.1, -0.2, 0.3], 0.25);
    let dir = normalize3([1.0, 2.0, -2.0]);
    let point = support(&geom, SPHERE, dir).point;
    let expected = [
        0.1 + 0.25 * dir[0],
        -0.2 + 0.25 * dir[1],
        0.3 + 0.25 * dir[2],
    ];
    assert_eq!(point, expected);
}

#[test]
fn box_support_picks_signed_corner() {
    let geom = box_geom([0.0; 3], [0.1, 0.2, 0.3]);
    let point = support(&geom, BOX, [1.0, -1.0, 1.0]);
    assert_eq!(point.point, [0.1, -0.2, 0.3]);
    assert_eq!(point.vertex_index, 1 + 0 + 4);
}

#[test]
fn capsule_support_uses_cylinder_half_length() {
    let geom = Geom::simple([0.0; 3], identity(), [0.05, 0.12, 0.0], 0.0);
    // 胶囊 = 半径 0.05 的半球帽 + 半长 0.12 的圆柱：±z 支撑为 ±(r + h)。
    let top = support(&geom, CAPSULE, [0.0, 0.0, 1.0]).point;
    let bottom = support(&geom, CAPSULE, [0.0, 0.0, -1.0]).point;
    assert_eq!(top, [0.0, 0.0, 0.17]);
    assert_eq!(bottom, [0.0, 0.0, -0.17]);
}

#[test]
fn cylinder_support_circle_and_cap() {
    let geom = Geom::simple([0.0; 3], identity(), [0.08, 0.1, 0.0], 0.0);
    let point = support(&geom, CYLINDER, [1.0, 0.0, 0.0]).point;
    assert!((point[0] - 0.08).abs() < 1e-7);
    assert_eq!(point[2], 0.1);
    assert_eq!(support(&geom, CYLINDER, [0.0, 0.0, -1.0]).point[2], -0.1);
}

#[test]
fn ellipsoid_support_lies_on_surface() {
    let geom = Geom::simple([0.0; 3], identity(), [0.1, 0.06, 0.05], 0.0);
    let point = support(&geom, ELLIPSOID, normalize3([1.0, 1.0, 1.0])).point;
    let scaled = [point[0] / 0.1, point[1] / 0.06, point[2] / 0.05];
    assert!((length3(scaled) - 1.0).abs() < 1e-6);
}

#[test]
fn margin_inflates_support_along_direction() {
    let geom = Geom::simple([0.0; 3], identity(), [0.1, 0.1, 0.1], 0.02);
    let point = support(&geom, BOX, [1.0, 0.0, 0.0]).point;
    assert_eq!(point[0], 0.11);
}

#[test]
fn disjoint_boxes_are_separated() {
    let left = box_geom([0.0; 3], [0.1, 0.1, 0.1]);
    let right = box_geom([1.0, 0.0, 0.0], [0.1, 0.1, 0.1]);
    let result = ccd_pair(&left, &right, BOX, BOX, ContactOptions::default());
    assert!(result.contacts.is_empty());
    assert_eq!(result.ncontact, 1);
}

#[test]
fn overlapping_boxes_report_single_contact_for_moderate_penetration() {
    // 两 box 同轴心轻微重叠：EPA 返回 1 个接触（multicontact 可能生成 ≤4）。
    let left = box_geom([0.0; 3], [0.1, 0.1, 0.1]);
    let right = box_geom([0.15, 0.0, 0.0], [0.1, 0.1, 0.1]);
    let result = ccd_pair(&left, &right, BOX, BOX, ContactOptions::default());
    assert!(!result.contacts.is_empty());
    assert!(result.contacts.len() <= 4);
    for contact in &result.contacts {
        assert!(contact.dist < 0.0);
    }
}

#[test]
fn ccd_reports_multiccd_idx_only_for_supported_types() {
    let left = box_geom([0.0; 3], [0.1, 0.1, 0.1]);
    let right = box_geom([0.15, 0.0, 0.0], [0.1, 0.1, 0.1]);
    let options = CcdOptions::default();
    // 原点平移到 geom1 位置由 `ccd_pair` 负责；这里直接给相对位置。
    let result = ccd(&left, &right, BOX, BOX, [0.0; 3], right.pos, options);
    assert!(result.multiccd_idx > -1);
    assert_eq!(result.overflow & super::overflow_type::EPA_HORIZON, 0);

    // 椭球不支持 multicontact。
    let ellipsoid = Geom::simple([0.15, 0.0, 0.0], identity(), [0.1, 0.06, 0.05], 0.0);
    let result = ccd(
        &left,
        &ellipsoid,
        BOX,
        ELLIPSOID,
        [0.0; 3],
        ellipsoid.pos,
        options,
    );
    assert_eq!(result.multiccd_idx, -1);
}

#[test]
fn mesh_support_exhaustive_search_returns_extreme_vertex() {
    let verts = vec![
        [0.0_f32, 0.0, 0.0],
        [0.2, 0.0, 0.0],
        [0.0, 0.2, 0.0],
        [0.0, 0.0, 0.2],
    ];
    let mesh = super::MeshView {
        vert: &verts,
        graph: &[],
        polyadr: 0,
        polynormal: &[],
        polyvertadr: &[],
        polyvertnum: &[],
        polyvert: &[],
        polymapadr: &[],
        polymapnum: &[],
        polymap: &[],
    };
    let mut geom = Geom::simple([0.0; 3], identity(), [0.1, 0.1, 0.1], 0.0);
    geom.vertadr = 0;
    geom.vertnum = 4;
    geom.graphadr = -1;
    geom.mesh = Some(&mesh);
    let point = support(&geom, MESH, [0.0, 1.0, 0.0]).point;
    assert_eq!(point, [0.0, 0.2, 0.0]);
}

#[test]
fn discrete_branch_disables_tolerance() {
    // is_discrete：box-box 且 margin=0 → epsilon=0、min_norm=MINVAL。
    assert!(super::core::is_discrete(BOX, BOX, 0.0, 0.0));
    assert!(!super::core::is_discrete(BOX, BOX, 0.01, 0.0));
    assert!(!super::core::is_discrete(BOX, SPHERE, 0.0, 0.0));
    assert!(super::core::is_discrete(MESH, BOX, 0.0, 0.0));
}

#[test]
fn sphere_box_penetration_matches_analytic_depth() {
    // 球心 (0.05,0,0.1) r=0.1 vs box 半尺寸 (0.1,0.08,0.06)：解析穿透 -0.06。
    let sphere_geom = sphere([0.05, 0.0, 0.1], 0.1);
    let box_geom_value = box_geom([0.0; 3], [0.1, 0.08, 0.06]);
    let result = ccd_pair(
        &box_geom_value,
        &sphere_geom,
        BOX,
        SPHERE,
        ContactOptions::default(),
    );
    assert_eq!(result.contacts.len(), 1);
    let contact = result.contacts[0];
    assert!((contact.dist - -0.06).abs() < 1e-6);
    assert!((contact.pos[2] - 0.03).abs() < 1e-6);
}

#[test]
fn simplex_s1d_midpoint_projection() {
    let value = super::simplex::s1d([1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]);
    assert!((value[0] - 0.5).abs() < 1e-7);
    assert!((value[1] - 0.5).abs() < 1e-7);
}

#[test]
fn simplex_s2d_origin_inside_triangle() {
    let value = super::simplex::s2d([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, -1.0, 0.0]);
    let sum = value[0] + value[1] + value[2];
    assert!((sum - 1.0).abs() < 1e-6);
}

#[test]
fn simplex_s3d_origin_inside_tetrahedron() {
    let value = super::simplex::s3d(
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [-1.0, -1.0, -1.0],
    );
    let sum = value[0] + value[1] + value[2] + value[3];
    assert!((sum - 1.0).abs() < 1e-6);
}

#[test]
fn make_frame_is_orthonormal_for_axis_normal() {
    let frame = super::math::make_frame([0.0, 0.0, -1.0]);
    let row0 = [frame[0], frame[1], frame[2]];
    let row1 = [frame[3], frame[4], frame[5]];
    let row2 = [frame[6], frame[7], frame[8]];
    assert!((length3(row0) - 1.0).abs() < 1e-6);
    assert!((length3(row1) - 1.0).abs() < 1e-6);
    assert!((dot3(row0, row1)).abs() < 1e-6);
    assert!((length3(row2) - 1.0).abs() < 1e-6);
}

#[test]
fn tangent_box_contact_normal_matches_axis() {
    let left = box_geom([0.0; 3], [0.1, 0.1, 0.1]);
    let right = box_geom([0.0, 0.0, 0.15], [0.1, 0.1, 0.1]);
    let result = ccd_pair(&left, &right, BOX, BOX, ContactOptions::default());
    assert!(!result.contacts.is_empty());
    let frame = result.contacts[0].frame;
    // frame 第一行为接触法线，应沿 z 轴反向。
    assert!(frame[2].abs() > 0.99);
}

#[test]
fn epa_buffers_match_upstream_capacity() {
    // 上游 `collision_convex.py`：vert `10+2I`、face `6+5I`、horizon `MJ_MAX_EPAHORIZON`。
    let polytope = super::Polytope::with_capacity(35);
    assert_eq!(polytope.vert.len(), 10 + 2 * 35);
    assert_eq!(polytope.vert_index.len(), 10 + 2 * 35);
    assert_eq!(polytope.face.len(), 6 + super::MJ_MAX_EPAFACES * 35);
    assert_eq!(polytope.horizon.len(), super::MJ_MAX_EPAHORIZON);
    assert_eq!(super::MJ_MAXCONPAIR, 50);
}

#[test]
fn shallow_separation_reports_positive_distance_without_epa() {
    // 球面离开 box 顶面 0.05：GJK shrink → `_inflate` 返回正距离，ncontact=1、不进 EPA。
    let sphere_geom = sphere([0.0, 0.0, 0.21], 0.1);
    let box_value = box_geom([0.0; 3], [0.1, 0.08, 0.06]);
    let result = super::ccd(
        &sphere_geom,
        &box_value,
        SPHERE,
        BOX,
        [0.0; 3],
        [0.0, 0.0, 0.21],
        super::CcdOptions::default(),
    );
    assert_eq!(result.ncontact, 1);
    assert!((result.dist - 0.05).abs() < 1e-6);
    assert_eq!(result.multiccd_idx, -1);
    assert_eq!(result.overflow, 0);
}

#[test]
fn separated_pair_produces_no_contact() {
    let left = box_geom([0.0; 3], [0.1, 0.1, 0.1]);
    let right = box_geom([1.0, 0.0, 0.0], [0.1, 0.1, 0.1]);
    let output = ccd_pair(&left, &right, BOX, BOX, ContactOptions::default());
    assert!(output.contacts.is_empty());
}

#[test]
fn multicontact_is_suppressed_for_unsupported_types() {
    // 椭球不在 multicontact 支持列表：即使接触也只写 1 条。
    let box_value = box_geom([0.0; 3], [0.1, 0.08, 0.06]);
    let ellipsoid = Geom::simple([0.04, 0.0, 0.1], identity(), [0.1, 0.06, 0.05], 0.0);
    let output = ccd_pair(
        &ellipsoid,
        &box_value,
        ELLIPSOID,
        BOX,
        ContactOptions::default(),
    );
    assert_eq!(output.contacts.len(), 1);
}

#[test]
fn multicontact_returns_at_most_four_contacts() {
    // 面-面 box-box（margin=0、MULTICCD 开启）走 multicontact：≤4 条。
    let left = box_geom([0.0; 3], [0.1, 0.08, 0.07]);
    let right = box_geom([0.01, 0.005, 0.12], [0.09, 0.07, 0.08]);
    let output = ccd_pair(&left, &right, BOX, BOX, ContactOptions::default());
    assert!(!output.contacts.is_empty());
    assert!(output.contacts.len() <= 4);
}

#[test]
fn host_ccd_forced_horizon_reports_only_epa_horizon() {
    // 深穿透 box-box（needs_epa），horizon 容量压到 2（等价容量）→ 上游同路径返回
    // `FLOAT_MAX` 且只置 `EPA_HORIZON`；修正前宿主会误加 `CCD`（272 = 256|16）。
    let left = box_geom([0.0; 3], [0.1, 0.1, 0.1]);
    let right = box_geom([0.05, 0.0, 0.0], [0.1, 0.1, 0.1]);
    let options = CcdOptions::default();

    let internal = super::ccd::ccd_internal_with_horizon(
        &left, &right, BOX, BOX, [0.0; 3], right.pos, options, 2,
    );
    assert_eq!(internal.overflow, super::overflow_type::EPA_HORIZON);
    assert_eq!(internal.overflow & super::overflow_type::CCD, 0);
    assert_ne!(
        internal.overflow,
        super::overflow_type::EPA_HORIZON | super::overflow_type::CCD
    );
    assert_eq!(internal.dist, super::FLOAT_MAX);
    assert_eq!(internal.ncontact, 0);
    assert_eq!(internal.multiccd_idx, -1);

    let output = super::contact::ccd_pair_with_capacity(
        &left,
        &right,
        BOX,
        BOX,
        ContactOptions::default(),
        None,
        2,
    );
    assert_eq!(output.overflow, super::overflow_type::EPA_HORIZON);
    assert_eq!(output.overflow & super::overflow_type::CCD, 0);
    assert_eq!(output.dist, super::FLOAT_MAX);
    assert!(output.contacts.is_empty());
}

#[test]
fn host_ccd_capacity_gate_still_sets_ccd_bit() {
    // 防矫枉过正：真实 `nccd >= naccdmax` 容量截断仍必须置 `CCD`（上游语义），
    // 且容量足够时不能误伤 EPA_HORIZON / 正常接触。
    let left = box_geom([0.0; 3], [0.1, 0.1, 0.1]);
    let right = box_geom([0.05, 0.0, 0.0], [0.1, 0.1, 0.1]);
    let options = ContactOptions::default();

    // ① 真实容量截断：ccd_slot=1 >= naccdmax=1 → 只置 CCD、不进入 EPA、不写接触。
    let truncated = super::contact::ccd_pair_with_capacity(
        &left,
        &right,
        BOX,
        BOX,
        options,
        Some(super::contact::CcdCapacity {
            ccd_slot: 1,
            naccdmax: 1,
        }),
        super::MJ_MAX_EPAHORIZON,
    );
    assert_eq!(truncated.overflow, super::overflow_type::CCD);
    assert!(truncated.contacts.is_empty());

    // ② 容量足够 + 强制 horizon：进入 EPA，只置 EPA_HORIZON（不含 CCD）。
    let forced = super::contact::ccd_pair_with_capacity(
        &left,
        &right,
        BOX,
        BOX,
        options,
        Some(super::contact::CcdCapacity {
            ccd_slot: 0,
            naccdmax: 1,
        }),
        2,
    );
    assert_eq!(forced.overflow, super::overflow_type::EPA_HORIZON);
    assert_eq!(forced.overflow & super::overflow_type::CCD, 0);

    // ③ 容量足够 + 正常穿透：无溢出位、接触照常产生（容量门不误伤正常路径）。
    let shallow_left = box_geom([0.0; 3], [0.1, 0.1, 0.1]);
    let shallow_right = box_geom([0.15, 0.0, 0.0], [0.1, 0.1, 0.1]);
    let adequate = super::contact::ccd_pair_with_capacity(
        &shallow_left,
        &shallow_right,
        BOX,
        BOX,
        options,
        Some(super::contact::CcdCapacity {
            ccd_slot: 0,
            naccdmax: 1,
        }),
        super::MJ_MAX_EPAHORIZON,
    );
    assert_eq!(adequate.overflow, 0);
    assert!(!adequate.contacts.is_empty());
}
