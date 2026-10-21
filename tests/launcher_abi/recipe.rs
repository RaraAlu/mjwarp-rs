//! 配方、Warp数组与启动边界校验测试。

use mjwarp_rs::runtime::launcher::{
    CompileRecipe, LaunchBounds2, LaunchBounds3, LauncherError, RecipePurpose, WarpArray,
};

#[test]
fn recipes_admit_only_their_own_purpose() {
    let recipes = [
        (CompileRecipe::WARP_1_15_PHYSICS, RecipePurpose::Physics),
        (
            CompileRecipe::WARP_1_15_DIAGNOSTIC,
            RecipePurpose::Diagnostic,
        ),
        (CompileRecipe::P1_PROBE, RecipePurpose::Probe),
    ];
    for (recipe, own) in &recipes {
        for (_, required) in &recipes {
            let result = recipe.admit(*required);
            if own == required {
                assert!(result.is_ok());
            } else {
                assert_eq!(
                    result,
                    Err(LauncherError::RecipePurpose {
                        expected: *required,
                        actual: *own,
                    })
                );
            }
        }
    }
    let fingerprints: Vec<String> = recipes.iter().map(|(r, _)| r.fingerprint()).collect();
    assert_ne!(fingerprints[0], fingerprints[1]);
    assert_ne!(fingerprints[0], fingerprints[2]);
    assert_ne!(fingerprints[1], fingerprints[2]);
    assert_eq!(
        CompileRecipe::WARP_1_15_PHYSICS.fingerprint(),
        "recipe-v1;name=warp-1.15-physics;purpose=physics;fmad=1;fast-math=0;std=--std=c++17;arch=--gpu-architecture=compute_70;gate=f32-physics;defines="
    );
}

#[test]
fn warp_array_rejects_null_empty_deep_and_overflowing_shapes() {
    assert!(WarpArray::<f32>::contiguous(0, &[4]).is_err());
    assert!(WarpArray::<f32>::contiguous(0x1000, &[]).is_err());
    assert!(WarpArray::<f32>::contiguous(0x1000, &[4, 0]).is_err());
    assert!(WarpArray::<f32>::contiguous(0x1000, &[1, 1, 1, 1, 1]).is_err());
    assert_eq!(
        WarpArray::<f32>::contiguous(0x1000, &[1usize << 31]).unwrap_err(),
        LauncherError::Overflow("warp_array_shape")
    );
    // 内层字节跨度超出i32。
    assert_eq!(
        WarpArray::<f64>::contiguous(0x1000, &[2, 1 << 28]).unwrap_err(),
        LauncherError::Overflow("warp_array_stride")
    );
    let array = WarpArray::<f32>::contiguous(0x1000, &[1 << 20, 64]).unwrap();
    assert_eq!(array.strides(), [256, 4]);
    assert_eq!(array.byte_len(), 1 << 28);
}

#[test]
fn launch_bounds_reject_zero_and_overflowing_extents() {
    let bounds = LaunchBounds3::new([4, 5, 6]).unwrap();
    assert_eq!(bounds.size(), 120);
    assert_eq!(bounds.shape(), [4, 5, 6]);
    assert!(LaunchBounds2::new([0, 8]).is_err());
    assert_eq!(
        LaunchBounds2::new([1 << 31, 1]).unwrap_err(),
        LauncherError::Overflow("launch_bounds_shape")
    );
    assert_eq!(
        LaunchBounds2::new([1 << 16, 1 << 16]).unwrap_err(),
        LauncherError::Overflow("launch_bounds_size")
    );
    assert_eq!(
        LaunchBounds2::new([1 << 16, (1 << 16) - 1]).unwrap().size(),
        u64::from(u32::MAX) - 65535
    );
}
