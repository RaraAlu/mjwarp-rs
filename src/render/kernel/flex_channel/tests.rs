//! `flex_channel` 单元测试（R06-S2；拆出以守「代码文件 ≤ 500 行」棘轮，沿 `src/model/flex/tests.rs` 先例）。
//!
//! 覆盖卡内 acceptance 的宿主校验面：空 flex、单 flex、多 flex、长度 / 世界数 / geom 映射不一致
//! 显式拒绝，以及两处 2-D 槽的 `max(1)` 形状口径与 u64 BVH 句柄的小端字对承载。

use super::{
    FlexCounts, FlexHostInputs, bvh_id_words, flex_edge_shape, flex_geom_shape,
    flex_group_root_shape, flex_model_shape, flex_vertadr_shape, flexvert_xpos_shape,
};
use crate::runtime::launcher::LauncherError;

const EDGE: [i32; 2] = [0, 1];

fn empty_channel<'a>() -> FlexHostInputs<'a> {
    FlexHostInputs {
        nworld: 1,
        nflex: 0,
        nflexvert: 0,
        flex_vertadr: &[],
        flex_edge: &[],
        flex_radius: &[],
        flexvert_xpos: &[],
        flex_rgba: &[],
        flex_bvh_id: &[],
        flex_group_root: &[],
        flex_geom_flexid: &[],
        flex_geom_edgeid: &[],
    }
}

/// 单 flex（1D 边图元）通道：nworld=1、nflex=1、nflexvert=2、1 条边。
fn single_flex_channel<'a>(
    radius: &'a [f32],
    rgba: &'a [f32],
    xpos: &'a [f32],
    bvh: &'a [u64],
    root: &'a [i32],
    flexid: &'a [i32],
    edgeid: &'a [i32],
) -> FlexHostInputs<'a> {
    FlexHostInputs {
        nworld: 1,
        nflex: 1,
        nflexvert: 2,
        flex_vertadr: &[0],
        flex_edge: &EDGE,
        flex_radius: radius,
        flexvert_xpos: xpos,
        flex_rgba: rgba,
        flex_bvh_id: bvh,
        flex_group_root: root,
        flex_geom_flexid: flexid,
        flex_geom_edgeid: edgeid,
    }
}

/// 空 flex：与 `None` 等价（计数全 0、不报错）。
#[test]
fn empty_flex_channel_validates_as_zero_flex() {
    let channel = empty_channel();
    assert!(channel.is_empty());
    assert_eq!(channel.validate_counts(1), Ok(FlexCounts::default()));
    assert_eq!(channel.validate_counts(4), Ok(FlexCounts::default()));
    assert!(channel.validate(4).is_ok());
}

/// 单 flex：1 边、1 个 1D flex 图元（flexid=0、edgeid=0）。
#[test]
fn single_flex_channel_validates() {
    let radius = [0.025f32];
    let rgba = [1.0f32, 0.0, 0.0, 1.0];
    let xpos = [0.0f32; 6];
    let bvh = [7u64];
    let root = [-1i32];
    let flexid = [0i32];
    let edgeid = [0i32];
    let channel = single_flex_channel(&radius, &rgba, &xpos, &bvh, &root, &flexid, &edgeid);
    assert!(!channel.is_empty());
    assert_eq!(
        channel.validate_counts(1),
        Ok(FlexCounts {
            nflex: 1,
            nflexvert: 2,
            nflexedge: 1,
            bvh_nflexgeom: 1,
        })
    );
}

/// 多 flex + 多 world：混合 1D 边图元与 2D/3D `edgeid = -1` 图元。
#[test]
fn multi_flex_multi_world_channel_validates() {
    let radius = [0.02f32, 0.03];
    let rgba = [1.0f32; 8];
    let xpos = [0.0f32; 3 * 2 * 4];
    let bvh = [3u64, 0];
    let root = [-1i32; 4];
    let flexid = [0i32, 1];
    let edgeid = [0i32, -1];
    let channel = FlexHostInputs {
        nworld: 2,
        nflex: 2,
        nflexvert: 4,
        flex_vertadr: &[0, 2],
        flex_edge: &EDGE,
        flex_radius: &radius,
        flexvert_xpos: &xpos,
        flex_rgba: &rgba,
        flex_bvh_id: &bvh,
        flex_group_root: &root,
        flex_geom_flexid: &flexid,
        flex_geom_edgeid: &edgeid,
    };
    assert_eq!(
        channel.validate_counts(2),
        Ok(FlexCounts {
            nflex: 2,
            nflexvert: 4,
            nflexedge: 1,
            bvh_nflexgeom: 2,
        })
    );
}

/// 世界数不一致：显式拒绝（不按通道 nworld 静默接受）。
#[test]
fn world_count_mismatch_is_rejected() {
    let radius = [0.02f32];
    let rgba = [1.0f32; 4];
    let xpos = [0.0f32; 6];
    let bvh = [0u64];
    let root = [-1i32];
    let flexid = [0i32];
    let edgeid = [0i32];
    let channel = single_flex_channel(&radius, &rgba, &xpos, &bvh, &root, &flexid, &edgeid);
    assert_eq!(
        channel.validate(2),
        Err(LauncherError::InvalidArray("flex_nworld"))
    );
}

/// 长度不一致：逐字段显式拒绝（不截断、不补齐、不静默回退）。
#[test]
fn length_mismatches_are_rejected_explicitly() {
    let radius = [0.025f32];
    let rgba = [1.0f32; 4];
    let xpos = [0.0f32; 6];
    let bvh = [0u64];
    let root = [-1i32];
    let flexid = [0i32];
    let edgeid = [0i32];
    let good = single_flex_channel(&radius, &rgba, &xpos, &bvh, &root, &flexid, &edgeid);
    assert!(good.validate(1).is_ok());
    let cases = [
        (
            "flex_vertadr",
            FlexHostInputs {
                flex_vertadr: &[],
                ..good
            },
        ),
        (
            "flex_radius",
            FlexHostInputs {
                flex_radius: &[],
                ..good
            },
        ),
        (
            "flex_rgba",
            FlexHostInputs {
                flex_rgba: &[0.0f32; 3],
                ..good
            },
        ),
        (
            "flex_bvh_id",
            FlexHostInputs {
                flex_bvh_id: &[],
                ..good
            },
        ),
        (
            "flexvert_xpos",
            FlexHostInputs {
                flexvert_xpos: &[0.0f32; 3],
                ..good
            },
        ),
        (
            "flex_group_root",
            FlexHostInputs {
                flex_group_root: &[],
                ..good
            },
        ),
        (
            "flex_geom_edgeid",
            FlexHostInputs {
                flex_geom_edgeid: &[],
                ..good
            },
        ),
        (
            "flex_edge",
            FlexHostInputs {
                flex_edge: &[0i32],
                ..good
            },
        ),
    ];
    for (field, channel) in cases {
        assert!(
            matches!(
                channel.validate(1),
                Err(LauncherError::InvalidArray(name)) if name == field
            ),
            "字段 {field} 长度不一致必须显式拒绝"
        );
    }
}

/// 取值 / 单调 / 映射越界：显式拒绝。
#[test]
fn range_and_monotonic_violations_are_rejected() {
    let radius = [0.025f32];
    let rgba = [1.0f32; 4];
    let xpos = [0.0f32; 6];
    let bvh = [0u64];
    let root = [-1i32];
    let flexid = [0i32];
    let edgeid = [0i32];
    let good = single_flex_channel(&radius, &rgba, &xpos, &bvh, &root, &flexid, &edgeid);
    let cases = [
        (
            "flex_vertadr",
            FlexHostInputs {
                flex_vertadr: &[1],
                ..good
            },
        ),
        (
            "flex_vertadr",
            FlexHostInputs {
                flex_vertadr: &[2],
                ..good
            },
        ),
        (
            "flex_edge",
            FlexHostInputs {
                flex_edge: &[0, 2],
                ..good
            },
        ),
        (
            "flex_edge",
            FlexHostInputs {
                flex_edge: &[-1, 0],
                ..good
            },
        ),
        (
            "flex_geom_flexid",
            FlexHostInputs {
                flex_geom_flexid: &[1],
                ..good
            },
        ),
        (
            "flex_geom_edgeid",
            FlexHostInputs {
                flex_geom_edgeid: &[3],
                ..good
            },
        ),
    ];
    for (field, channel) in cases {
        assert!(
            matches!(
                channel.validate(1),
                Err(LauncherError::InvalidArray(name)) if name == field
            ),
            "字段 {field} 取值越界必须显式拒绝"
        );
    }
    // 多 flex 下 vertadr 回退（[0, 2, 1]）必须拒绝。
    let monotonous = FlexHostInputs {
        nflex: 3,
        nflexvert: 3,
        flex_vertadr: &[0, 2, 1],
        flex_radius: &[0.0f32; 3],
        flexvert_xpos: &[0.0f32; 9],
        flex_rgba: &[0.0f32; 12],
        flex_bvh_id: &[0u64; 3],
        flex_group_root: &[-1i32; 3],
        ..empty_channel()
    };
    assert!(matches!(
        monotonous.validate(1),
        Err(LauncherError::InvalidArray("flex_vertadr"))
    ));
}

/// 槽形状：空通道 1 元素占位 + `max(1)`，2-D 槽恒定 2 维且首维 = nworld。
#[test]
fn slot_shapes_follow_max_one_and_two_dimensional_upstream() {
    let empty = FlexCounts::default();
    assert_eq!(flex_vertadr_shape(empty), [1]);
    assert_eq!(flex_edge_shape(empty), [1]);
    assert_eq!(flex_model_shape(empty), [1]);
    assert_eq!(flex_geom_shape(empty), [1]);
    assert_eq!(flexvert_xpos_shape(3, 0), [3, 1]);
    assert_eq!(flex_group_root_shape(3, 0), [3, 1]);
    assert_eq!(flexvert_xpos_shape(2, 5), [2, 5]);
    assert_eq!(flex_group_root_shape(2, 1), [2, 1]);
    let counts = FlexCounts {
        nflex: 2,
        nflexvert: 5,
        nflexedge: 3,
        bvh_nflexgeom: 4,
    };
    assert_eq!(flex_vertadr_shape(counts), [2]);
    assert_eq!(flex_edge_shape(counts), [3]);
    assert_eq!(flex_model_shape(counts), [2]);
    assert_eq!(flex_geom_shape(counts), [4]);
    // 2-D 槽首维恒为 nworld（上游 `[nworld, ...]`）。
    assert_eq!(flexvert_xpos_shape(2, 5)[0], 2);
    assert_eq!(flex_group_root_shape(2, 2)[0], 2);
}

/// `u64` BVH 句柄按小端 u32 字数对承载；空数组 = 1 个 u64 占位。
#[test]
fn bvh_ids_are_carried_as_little_endian_word_pairs() {
    assert_eq!(bvh_id_words(&[]), vec![0u32, 0]);
    assert_eq!(bvh_id_words(&[0x1_0000_0002]), vec![2u32, 1]);
    assert_eq!(bvh_id_words(&[u64::MAX, 0]), vec![u32::MAX, u32::MAX, 0, 0]);
}
