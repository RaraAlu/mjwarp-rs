//! CUDA源码、签名与启动辅助校验。

use super::support::artifact;
use super::support::compact;
use super::support::signature_lines;
use super::*;

pub(super) fn validate_cuda(bytes: &[u8]) -> Result<(), ProbeError> {
    if bytes.contains(&0) {
        return Err(artifact("cu", "embedded nul"));
    }
    let source = std::str::from_utf8(bytes).map_err(|error| artifact("cu", error.to_string()))?;
    for (entry, arrays) in cuda_signatures() {
        let signature = signature_lines(source, &format!("extern \"C\" __global__ void {entry}("))?;
        let expected = format!(
            "extern\"C\"__global__void{entry}(wp::launch_bounds_t<2>dim,{})",
            arrays.join(",")
        );
        if compact(&signature) != expected {
            return Err(artifact("cu", format!("{entry} ABI mismatch")));
        }
    }
    Ok(())
}

pub(super) fn cuda_signatures() -> [(&'static str, &'static [&'static str]); 7] {
    [
        (
            KIN_ENTRY,
            &[
                "wp::array_t<wp::float32>var_qpos0",
                "wp::array_t<wp::int32>var_body_parentid",
                "wp::array_t<wp::int32>var_body_mocapid",
                "wp::array_t<wp::int32>var_body_jntnum",
                "wp::array_t<wp::int32>var_body_jntadr",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_body_pos",
                "wp::array_t<wp::quat_t<wp::float32>>var_body_quat",
                "wp::array_t<wp::int32>var_jnt_type",
                "wp::array_t<wp::int32>var_jnt_qposadr",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_jnt_pos",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_jnt_axis",
                "wp::array_t<wp::int32>var_body_branches",
                "wp::array_t<wp::int32>var_body_branch_start",
                "wp::array_t<wp::float32>var_qpos_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_mocap_pos_in",
                "wp::array_t<wp::quat_t<wp::float32>>var_mocap_quat_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xpos_out",
                "wp::array_t<wp::quat_t<wp::float32>>var_xquat_out",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xanchor_out",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xaxis_out",
            ],
        ),
        (
            MATRIX_ENTRY,
            &[
                "wp::array_t<wp::quat_t<wp::float32>>var_xquat_in",
                "wp::array_t<wp::mat_t<3,3,wp::float32>>var_xmat_out",
            ],
        ),
        (
            INERTIAL_ENTRY,
            &[
                "wp::array_t<wp::vec_t<3,wp::float32>>var_body_ipos",
                "wp::array_t<wp::quat_t<wp::float32>>var_body_iquat",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xpos_in",
                "wp::array_t<wp::quat_t<wp::float32>>var_xquat_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xipos_out",
                "wp::array_t<wp::mat_t<3,3,wp::float32>>var_ximat_out",
            ],
        ),
        (
            INIT_ENTRY,
            &[
                "wp::array_t<wp::float32>var_body_mass",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xipos_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_out",
            ],
        ),
        (
            ACC_ENTRY,
            &[
                "wp::array_t<wp::int32>var_body_parentid",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
                "wp::array_t<wp::int32>var_body_tree_",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_out",
            ],
        ),
        (
            DIV_ENTRY,
            &[
                "wp::array_t<wp::float32>var_body_subtreemass",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_out",
            ],
        ),
        (
            CDOF_ENTRY,
            &[
                "wp::array_t<wp::int32>var_body_rootid",
                "wp::array_t<wp::int32>var_jnt_type",
                "wp::array_t<wp::int32>var_jnt_dofadr",
                "wp::array_t<wp::int32>var_jnt_bodyid",
                "wp::array_t<wp::mat_t<3,3,wp::float32>>var_xmat_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xanchor_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xaxis_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
                "wp::array_t<wp::vec_t<6,wp::float32>>var_cdof_out",
            ],
        ),
    ]
}
