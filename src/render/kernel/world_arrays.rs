//! R10-S2 world 维设备数组形状规则（`render_megakernel` 实参）。
//!
//! 冻结内核的索引方式决定 host 侧形状，逐类核对（`cuda/render_megakernel.cu`）：
//!
//! - `cam_xpos_in` / `cam_xmat_in`：内核按 `worldid * shape.dims[1] + mujoco_cam_id` 读取
//!   （行 148-150），故必须上传 `[nworld, ncam]`（world-major 展平）；
//! - `light_type`：内核用 `shape.dims[1]` 取 `nlight`（行 274），数据本身按单索引读取
//!   （行 294），故形状为 `[1, nlight]`（首维固定 1，避免 `[nworld, nlight]` 的缓冲翻倍，
//!   同时让 `dims[1]` 仍等于 `nlight`）；
//! - 其余模型级/静态 data 数组（geom_type/dataid/matid/size/rgba、cam_projection/fovy/
//!   sensorsize/intrinsic、light 的 castshadow/active/attenuation/cutoff/exponent/ambient/
//!   diffuse/specular、mat_texid/texuniform/texrepeat/emission/specular/shininess/rgba、
//!   geom_xpos/geom_xmat、light_xpos/light_xdir）：内核只按单索引读取（`geom_id` /
//!   `mat_id` / `light` / `camid`），故为 1D 单份。
//!
//! 语义边界（R10-S2 证据口径）：多 world 下着色使用**单份**几何/灯位姿，本片只覆盖
//! 「两 world 几何相同（同模型多 state、world_dx=0）」的情形；逐 world 几何/灯位姿数据
//! 缺口见 backlog `2026-10-28-render-per-world-geom-light-gap`，不宣称上游 `nworld>1` 等价。

/// 模型级共享数组（内核只按单索引读取）的设备形状：1D 单份，与 `nworld` 无关。
pub(crate) fn shared_shape(len: usize) -> [usize; 1] {
    [len]
}

/// 模型级共享矩阵（如 `mat_texid`，内核按 `mat_id * 10 + slot` 单索引读取）：保留列语义。
pub(crate) fn shared_matrix_shape(rows: usize, columns: usize) -> [usize; 2] {
    [rows, columns]
}

/// 相机位姿数组形状：`[nworld, ncam]`（内核按 `worldid * dims[1] + cam_id` 读取）。
pub(crate) fn camera_pose_shape(nworld: usize, ncam: usize) -> [usize; 2] {
    [nworld, ncam]
}

/// `light_type` 形状：`[1, nlight]`（内核用 `shape.dims[1]` 取 `nlight`，数据单份）。
pub(crate) fn light_type_shape(nlight: usize) -> [usize; 2] {
    [1, nlight]
}

#[cfg(test)]
mod tests {
    use super::{camera_pose_shape, light_type_shape, shared_matrix_shape, shared_shape};

    #[test]
    fn shared_arrays_are_single_world_regardless_of_nworld() {
        // nworld=1 与 nworld>1 的形状逐元素相同（nworld=1 回归结构不变）。
        for len in [0usize, 1, 7, 28] {
            assert_eq!(shared_shape(len), [len]);
            assert_eq!(shared_shape(len).len(), 1);
        }
        assert_eq!(shared_shape(7), [7]);
        assert_eq!(shared_shape(28), [28]);
        // 共享矩阵：首维是模型元素数（mat 数），第二维保留列语义（texid 每 mat 10 个）。
        assert_eq!(shared_matrix_shape(4, 10), [4, 10]);
        assert_eq!(shared_matrix_shape(4, 10)[1], 10);
    }

    #[test]
    fn camera_poses_keep_world_major_shape() {
        assert_eq!(camera_pose_shape(1, 2), [1, 2]);
        assert_eq!(camera_pose_shape(2, 2), [2, 2]);
        // dims[1] 恒为 ncam（内核查表用）。
        assert_eq!(camera_pose_shape(3, 2)[1], 2);
    }

    #[test]
    fn light_type_exposes_nlight_in_second_dimension_with_single_world_data() {
        // 内核 `int nlight = light_type.shape.dims[1];`：首维固定 1，不随 nworld 翻倍。
        assert_eq!(light_type_shape(0), [1, 0]);
        assert_eq!(light_type_shape(3), [1, 3]);
        assert_eq!(light_type_shape(3)[1], 3);
        assert_eq!(light_type_shape(usize::MAX), [1, usize::MAX]);
    }
}
