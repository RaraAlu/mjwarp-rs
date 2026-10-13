// 精确逆变换保护历史映射；不放行其他源码漂移。
pub const OLD_R: &str = "    float x=q[0],y=q[1],z=q[2],s=q[3];\n    // Column-major rotation columns, matching Warp's xyzw convention.\n    float R[3][3]={{1-2*(y*y+z*z),2*(x*y+s*z),2*(x*z-s*y)},\n      {2*(x*y-s*z),1-2*(x*x+z*z),2*(y*z+s*x)},\n      {2*(x*z+s*y),2*(y*z-s*x),1-2*(x*x+y*y)}};";
pub const NEW_R: &str = "    // Warp quat_to_matrix rotates each basis vector without renormalizing.\n    float R[3][3];\n    for(int col=0;col<3;++col) {\n      float v[3]={0,0,0};v[col]=1.0f;\n      float c=2.0f*q[3]*q[3]-1.0f;\n      float d=2.0f*(q[0]*v[0]+q[1]*v[1]+q[2]*v[2]);\n      R[col][0]=v[0]*c+q[0]*d+(q[1]*v[2]-q[2]*v[1])*q[3]*2.0f;\n      R[col][1]=v[1]*c+q[1]*d+(q[2]*v[0]-q[0]*v[2])*q[3]*2.0f;\n      R[col][2]=v[2]*c+q[2]*d+(q[0]*v[1]-q[1]*v[0])*q[3]*2.0f;\n    }";
pub const OLD_M: &str = "    float next[4];float cross[3];face_cross(cross,rot,q);\n    for(int k=0;k<3;++k) next[k]=(rot[3]*q[k]+q[3]*rot[k])+cross[k];\n    next[3]=rot[3]*q[3]-face_dot(rot,q);";
pub const FIX_M: &str = "    // Frozen Warp PTX rounds q.xyz*rot.w before the two fused additions.\n    float next[4];\n    next[0]=__fsub_rn(fmaf(q[2],rot[1],fmaf(q[3],rot[0],__fmul_rn(q[0],rot[3]))),__fmul_rn(q[1],rot[2]));\n    next[1]=__fsub_rn(fmaf(q[0],rot[2],fmaf(q[3],rot[1],__fmul_rn(q[1],rot[3]))),__fmul_rn(q[2],rot[0]));\n    next[2]=__fsub_rn(fmaf(q[1],rot[0],fmaf(q[3],rot[2],__fmul_rn(q[2],rot[3]))),__fmul_rn(q[0],rot[1]));\n    next[3]=rot[3]*q[3]-rot[0]*q[0]-rot[1]*q[1]-rot[2]*q[2];";
pub const OLD_N: &str = "    for(int k=0;k<4;++k) q[k]=next[k]/n;";
pub const NEW_N: &str = "    // Warp kEps is zero; nonpositive lengths return the identity.\n    if(n>0.0f) {float inv_n=1.0f/n;for(int k=0;k<4;++k) q[k]=next[k]*inv_n;}\n    else {q[0]=q[1]=q[2]=0.0f;q[3]=1.0f;}";

pub fn historical(text: &str) -> Result<String, String> {
    let mut text = text.replace("\r\n", "\n");
    for (old, new) in [(OLD_R, NEW_R), (OLD_M, FIX_M), (OLD_N, NEW_N)] {
        if text.matches(new).count() != 1 || text.contains(old) {
            return Err("face repair lineage drift".into());
        }
        text = text.replace(new, old);
    }
    Ok(text)
}
