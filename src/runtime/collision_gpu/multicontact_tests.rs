//! G10@P5 multicontact 片：纯 CPU 单测（不创建 `CudaContext`、不跑 GPU）。
//!
//! 覆盖：21 个设备源片段的拼装顺序与新增片段存在性、两个新入口在 `product.cu` 的唯一性、
//! `boxbox` 入口的 CPU 侧类型校验、以及拼装源里 `make_frame` 仍只有一个定义。

use super::source::{DEVICE_SOURCES, PRODUCT_SOURCE, assemble_device_source};
use super::{BOXBOX_ENTRY, CONTACT_MULTI_ENTRY, validate_boxbox_types};

/// 上游 `mjtGeom`：BOX=6。
const BOX: i32 = 6;

#[test]
fn device_sources_place_multicontact_and_boxbox_in_order() {
    assert_eq!(DEVICE_SOURCES.len(), 21, "设备源片段数应为 21");
    let position = |wanted: &str| {
        DEVICE_SOURCES
            .iter()
            .position(|(name, _)| *name == wanted)
            .unwrap_or_else(|| panic!("DEVICE_SOURCES 缺少 {wanted}"))
    };
    let mc_geom = position("multicontact_geom.cu");
    let mc_clip = position("multicontact_clip.cu");
    let mc = position("multicontact.cu");
    let contact = position("contact.cu");
    let primitive = position("primitive.cu");
    let primitive_wrapper = position("primitive_wrapper.cu");
    let boxbox = position("boxbox.cu");
    let boxbox_wrapper = position("boxbox_wrapper.cu");
    assert_eq!(mc_clip, mc_geom + 1, "multicontact_clip 应紧随 geom");
    assert_eq!(mc, mc_clip + 1, "multicontact.cu 应紧随 clip");
    assert_eq!(
        contact,
        mc + 1,
        "contact.cu 应紧随 multicontact（调用方在后）"
    );
    assert_eq!(primitive, contact + 1, "primitive.cu 应紧随 contact.cu");
    assert_eq!(primitive_wrapper, primitive + 1);
    assert_eq!(
        boxbox,
        primitive_wrapper + 1,
        "boxbox.cu 应紧随 primitive_wrapper"
    );
    assert_eq!(boxbox_wrapper, boxbox + 1);
    assert_eq!(
        boxbox_wrapper,
        DEVICE_SOURCES.len() - 1,
        "boxbox_wrapper 应为最后一片段"
    );
}

#[test]
fn assembled_source_contains_new_kernels_and_entries_once() {
    let assembled = assemble_device_source();
    for symbol in [
        "g5_multicontact",
        "g5_mc_result",
        "g5_box_box",
        "g5_box_box_wrapper",
    ] {
        assert!(assembled.contains(symbol), "拼装源缺少设备函数 {symbol}");
    }
    for entry in [CONTACT_MULTI_ENTRY, BOXBOX_ENTRY] {
        let declaration = format!("extern \"C\" __global__ void {entry}(");
        assert_eq!(
            PRODUCT_SOURCE.matches(&declaration).count(),
            1,
            "{entry} 须在 product.cu 有且仅有一处定义"
        );
    }
    assert_eq!(CONTACT_MULTI_ENTRY, "mjwarp_collision_contact_multi");
    assert_eq!(BOXBOX_ENTRY, "mjwarp_collision_boxbox");
    assert_eq!(
        assembled.matches("void make_frame(").count(),
        1,
        "make_frame 只应由 common.cu 定义一次"
    );
    assert!(
        assembled.ends_with(PRODUCT_SOURCE),
        "product.cu 必须追加在合并源最后"
    );
}

#[test]
fn boxbox_accepts_only_box_box() {
    assert_eq!(validate_boxbox_types(BOX, BOX), Ok(()));
    for (type1, type2) in [(BOX, 2), (2, BOX), (2, 2), (6, 7), (7, 6)] {
        assert!(
            validate_boxbox_types(type1, type2).is_err(),
            "({type1},{type2}) 不应被 boxbox 接受"
        );
    }
}
