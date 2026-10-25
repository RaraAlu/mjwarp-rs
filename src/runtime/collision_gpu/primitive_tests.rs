//! G10@P5：primitive narrowphase 设备入口的纯 CPU 单测（不创建 `CudaContext`、不跑 GPU）。
//!
//! 覆盖：`CollisionKernel::Primitive` 的类型组合校验、16 个设备源片段的拼装顺序、
//! `product.cu` 的 primitive 入口唯一性、以及设备源里没有重复定义 `make_frame`。

use super::source::{DEVICE_SOURCES, PRODUCT_SOURCE, assemble_device_source};
use super::{CollisionGpuError, PRIMITIVE_ENTRY, validate_primitive_types};

/// 上游 `mjtGeom`：PLANE=0、SPHERE=2、CAPSULE=3、BOX=6、MESH=7。
const SPHERE: i32 = 2;
const CAPSULE: i32 = 3;
const BOX: i32 = 6;

#[test]
fn primitive_accepts_sphere_box_and_capsule_box_in_either_order() {
    for (type1, type2) in [(SPHERE, BOX), (BOX, SPHERE), (CAPSULE, BOX), (BOX, CAPSULE)] {
        assert_eq!(
            validate_primitive_types(type1, type2),
            Ok(()),
            "({type1},{type2}) 应被 primitive 接受"
        );
    }
}

#[test]
fn primitive_rejects_other_combinations() {
    for (type1, type2) in [
        (SPHERE, SPHERE),
        (CAPSULE, CAPSULE),
        (BOX, BOX),
        (SPHERE, CAPSULE),
        (CAPSULE, SPHERE),
        (0, BOX),
        (BOX, 0),
        (2, 7),
        (7, 3),
        (105, BOX),
    ] {
        assert_eq!(
            validate_primitive_types(type1, type2),
            Err(CollisionGpuError::InvalidArgument(
                "primitive 只支持 SPHERE-BOX / CAPSULE-BOX 组合"
            )),
            "({type1},{type2}) 应被 primitive 拒绝"
        );
    }
}

#[test]
fn device_sources_place_primitive_after_contact() {
    assert_eq!(DEVICE_SOURCES.len(), 21, "设备源片段数应为 21");
    let position = |wanted: &str| {
        DEVICE_SOURCES
            .iter()
            .position(|(name, _)| *name == wanted)
            .unwrap_or_else(|| panic!("DEVICE_SOURCES 缺少 {wanted}"))
    };
    let contact = position("contact.cu");
    let primitive = position("primitive.cu");
    let wrapper = position("primitive_wrapper.cu");
    assert_eq!(primitive, contact + 1, "primitive.cu 应紧随 contact.cu");
    assert_eq!(
        wrapper,
        primitive + 1,
        "primitive_wrapper.cu 应紧随 primitive.cu"
    );
    for name in ["primitive.cu", "primitive_wrapper.cu"] {
        assert!(
            DEVICE_SOURCES
                .iter()
                .any(|(candidate, _)| *candidate == name),
            "DEVICE_SOURCES 缺少 {name}"
        );
    }
}

#[test]
fn primitive_device_sources_define_kernels_and_entry_once() {
    let assembled = assemble_device_source();
    for symbol in [
        "g5_sphere_box",
        "g5_capsule_box",
        "g5_sphere_box_wrapper",
        "g5_capsule_box_wrapper",
    ] {
        assert!(assembled.contains(symbol), "拼装源缺少设备函数 {symbol}");
    }
    let declaration = format!("extern \"C\" __global__ void {PRIMITIVE_ENTRY}(");
    assert_eq!(
        PRODUCT_SOURCE.matches(&declaration).count(),
        1,
        "{PRIMITIVE_ENTRY} 须在 product.cu 有且仅有一处定义"
    );
    assert_eq!(PRIMITIVE_ENTRY, "mjwarp_collision_primitive");
    assert_eq!(
        assembled.matches("void make_frame(").count(),
        1,
        "make_frame 只应由 common.cu 定义一次（primitive.cu 不得重名重定义）"
    );
    assert!(
        assembled.ends_with(PRODUCT_SOURCE),
        "product.cu 必须追加在合并源最后"
    );
}
