//! 刚体内部量与七项输出的静态映射。
//! 零分支已有产物实证；完整精度仍不放行。

#[path = "t4_rigid_internal/captures.rs"]
mod captures;
#[path = "t4_rigid_internal/external.rs"]
mod external;
#[path = "t4_rigid_internal/lineage.rs"]
mod lineage;
#[path = "t4_rigid_internal/tests.rs"]
mod tests;
#[path = "t4_rigid_internal/types.rs"]
mod types;
#[path = "t4_rigid_internal/validate.rs"]
mod validate;

pub(super) use lineage::historical_rigid_kernel;

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-rigid-internal.json");
const PHYSICS: &str = include!("../../docs/development-evidence/t4-resident-anchors/physics.rs");
const RESIDENT: &str = include!("../../docs/development-evidence/t4-resident-anchors/resident.rs");
const TRANSFER: &str = concat!(
    include_str!("../../src/runtime/transfer.rs"),
    "\n",
    include_str!("../../src/runtime/transfer/buffer.rs"),
    "\n",
    include_str!("../../src/runtime/transfer/session.rs")
);
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/mod.rs", PHYSICS),
    ("src/physics/resident.rs", RESIDENT),
    ("src/runtime/transfer.rs", TRANSFER),
];
const EXTERNAL: &[&str] = &[
    "upstream/_src/types.py",
    "upstream/_src/smooth.py",
    "upstream/_src/math.py",
    "upstream/_src/io.py",
    "warp/_src/types.py",
    "warp/_src/builtins.py",
    "warp/native/quat.h",
    "warp/native/builtin.h",
];
// 顺序、表达式数量与边界共同构成审查分母。
const STEPS: &[&str] = &[
    "normalize-positive|source-boundary|warp/native/quat.h|normalize|wp.quat|3|5",
    "quaternion-product|same-scalar|upstream/_src/math.py|mul_quat|wp.quat|3|3",
    "rotation|same-scalar|upstream/_src/math.py|rot_vec_quat|wp.vec3|3|3",
    "matrix|same-scalar|upstream/_src/math.py|quat_to_mat|wp.mat33|3|4",
    "free-state|normalization-dependent|upstream/_src/smooth.py|_kinematics_branch|wp.vec3+wp.quat|3|5",
    "mocap|same-scalar|upstream/_src/smooth.py|_kinematics_branch|wp.vec3+wp.quat|2|5",
    "parent-compose|different-order|upstream/_src/smooth.py|_kinematics_branch|wp.vec3+wp.quat|3|2",
    "joint-anchor|different-order|upstream/_src/smooth.py|_kinematics_branch|wp.vec3|1|1",
    "joint-axis|different-order|upstream/_src/smooth.py|_kinematics_branch|wp.vec3|1|2",
    "slide|same-scalar|upstream/_src/smooth.py|_kinematics_branch|wp.vec3|1|2",
    "ball|normalization-dependent|upstream/_src/smooth.py|_kinematics_branch|wp.quat|2|3",
    "hinge-trigonometry|same-scalar|upstream/_src/math.py|axis_angle_to_quat|float+wp.quat|2|4",
    "rotation-accumulate|same-scalar|upstream/_src/smooth.py|_kinematics_branch|wp.quat|1|3",
    "offcenter-correction|different-expression|upstream/_src/smooth.py|_kinematics_branch|wp.vec3|1|1",
    "final-normalize|normalization-dependent|upstream/_src/smooth.py|_kinematics_branch|wp.quat|1|1",
    "inertial-position|fused-stage|upstream/_src/smooth.py|_compute_body_inertial_frames|wp.vec3|1|1",
    "inertial-matrix|fused-stage|upstream/_src/smooth.py|_compute_body_inertial_frames|wp.mat33|1|1",
];
const OUTPUTS: &[&str] = &[
    "xpos|wp.vec3|float* xp=result+4+(unsigned long long)w*(28*nb+6*nj);|store_v(xp+3*b,p);|&v[..3 * nb]|_kinematics_branch",
    "xquat|wp.quat|float* xq=xp+3*nb;|store_q(xq+4*b,r);|&v[3 * nb..7 * nb]|_kinematics_branch",
    "xmat|wp.mat33|float* xm=xq+4*nb;|matrix(xm+9*b,r);|&v[7 * nb..16 * nb]|_compute_body_matrices",
    "xipos|wp.vec3|float* xi=xm+9*nb;|store_v(xi+3*b,add(p,rotate(r,load_v(ip+3*b))));|&v[16 * nb..19 * nb]|_compute_body_inertial_frames",
    "ximat|wp.mat33|float* im=xi+3*nb;|matrix(im+9*b,multiply(r,load_q(iq+4*b)));|&v[19 * nb..28 * nb]|_compute_body_inertial_frames",
    "xanchor|wp.vec3|float* anchor=im+9*nb;|store_v(anchor+3*j,a);|&v[28 * nb..28 * nb + 3 * nj]|_kinematics_branch",
    "xaxis|wp.vec3|float* xa=anchor+3*nj;|store_v(xa+3*j,ax);|&v[28 * nb + 3 * nj..]|_kinematics_branch",
];
const BOUNDARIES: [&str; 3] = [
    "world-pose-preserved",
    "serial-versus-branch-schedule",
    "diagnostic-f64-no-feedback",
];
const REMAINING: [&str; 5] = [
    "zero-quaternion-full-pipeline-not-retested",
    "operator-order-and-generated-semantics",
    "other-kernel-internals-and-outputs",
    "upstream-generated-kernels",
    "measured-performance",
];
