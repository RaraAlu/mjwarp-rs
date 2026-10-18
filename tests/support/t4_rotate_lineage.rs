// 精确逆变换旋转分组修复（8a5bac0）；不放行其他源码漂移。
pub const DOT: &str = "__device__ float dot_v(V a,V b) { return a.x*b.x+a.y*b.y+a.z*b.z; }\n";
pub const CROSS: &str =
    "__device__ V cross_v(V a,V b) { return {a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x}; }\n";
pub const BEFORE: &str = r#"__device__ V rotate(Q q,V v) {
  float d=q.x*v.x+q.y*v.y+q.z*v.z;
  float s=q.w*q.w-q.x*q.x-q.y*q.y-q.z*q.z;
  return {s*v.x+2*d*q.x+2*q.w*(q.y*v.z-q.z*v.y),
          s*v.y+2*d*q.y+2*q.w*(q.z*v.x-q.x*v.z),
          s*v.z+2*d*q.z+2*q.w*(q.x*v.y-q.y*v.x)};
}"#;
pub const AFTER: &str = r#"__device__ V rotate(Q q,V v) {
  V u={q.x,q.y,q.z};
  V lhs=scale(scale(u,dot_v(u,v)),2.0f);
  float factor=q.w*q.w-dot_v(u,u);
  V r=add(lhs,scale(v,factor));
  return add(r,scale(cross_v(u,v),2.0f*q.w));
}"#;

/// 还原修复前的完整源码；修复体与辅助函数须各出现一次。
pub fn before(text: &str) -> Result<String, String> {
    let text = text.replace("\r\n", "\n");
    for part in [DOT, CROSS, AFTER] {
        if text.matches(part).count() != 1 {
            return Err("rotation lineage drift".into());
        }
    }
    if text.contains(BEFORE) {
        return Err("rotation lineage drift".into());
    }
    Ok(text
        .replace(DOT, "")
        .replace(CROSS, "")
        .replace(AFTER, BEFORE))
}
