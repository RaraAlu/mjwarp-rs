// 独立因子实验；不调用满额面诊断。
use super::*;
use std::fs;
#[cfg(windows)]
use std::{
    ffi::{CString, c_char, c_void},
    path::Path,
};

use super::super::super::face_lineage as lineage;
use lineage::{NEW_N, NEW_R, OLD_M, OLD_N, OLD_R};

const NEW_M: &str = "    // Preserve Warp's left-associated quaternion product.\n    float next[4];\n    next[0]=rot[3]*q[0]+q[3]*rot[0]+rot[1]*q[2]-q[1]*rot[2];\n    next[1]=rot[3]*q[1]+q[3]*rot[1]+rot[2]*q[0]-q[2]*rot[0];\n    next[2]=rot[3]*q[2]+q[3]*rot[2]+rot[0]*q[1]-q[0]*rot[1];\n    next[3]=rot[3]*q[3]-rot[0]*q[0]-rot[1]*q[1]-rot[2]*q[2];";

fn variant(base: &str, mask: usize) -> String {
    if mask == 10 {
        // 掩码10跟随当前产品的三轴末项融合版本。
        return variant(base, 7)
            .replace(NEW_M, lineage::FUSED_M)
            .replace("isolated_7", "isolated_10");
    }
    let mut source = base.to_owned();
    for (bit, old, new) in [(1, OLD_R, NEW_R), (2, OLD_M, NEW_M), (4, OLD_N, NEW_N)] {
        assert_eq!(source.matches(old).count(), 1);
        if mask & bit != 0 {
            source = source.replace(old, new);
        }
    }
    if mask == 8 {
        source = source.replace("face_polar(quats+4*f,F);", "wp::mat_t<3,3,float> mat(F[0][0],F[1][0],F[2][0],F[0][1],F[1][1],F[2][1],F[0][2],F[1][2],F[2][2]);\n    wp::quat_t<float> q=mat33_to_quat_polar_0(mat);\n    quats[4*f]=q.x;quats[4*f+1]=q.y;quats[4*f+2]=q.z;quats[4*f+3]=q.w;");
    }
    source.replace("void flex_faces(", &format!("void isolated_{mask}("))
}


#[path = "t4_face_operators/compile.rs"]
mod compile;
#[path = "t4_face_operators/factorial.rs"]
mod factorial;
#[path = "t4_face_operators/stats.rs"]
mod stats;
#[path = "t4_face_operators/trace.rs"]
mod trace;
