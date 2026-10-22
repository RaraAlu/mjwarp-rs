//! 逐表达式探针：从产品/上游源码逐字提取表达式函数，套用最小内核。
//!
//! 只用于编译期 PTX 对照，不进入产品源码。

use crate::extract::function_block;

const UPSTREAM_PREAMBLE: &str = "#define WP_NO_BFLOAT16\n#define WP_TILE_BLOCK_DIM 256\n#define WP_NO_CRT\n#include \"builtin.h\"\n#include \"deterministic.h\"\n\n";

const IO_HELPERS: &str = r#"
static CUDA_CALLABLE wp::vec_t<3, wp::float32> load_v3(const float* p) { return wp::vec_t<3, wp::float32>(p[0], p[1], p[2]); }
static CUDA_CALLABLE wp::quat_t<wp::float32> load_q4(const float* p) { return wp::quat_t<wp::float32>(p[0], p[1], p[2], p[3]); }
static CUDA_CALLABLE void store_v3(float* p, wp::vec_t<3, wp::float32> v) {
  const wp::int32 i0 = 0; const wp::int32 i1 = 1; const wp::int32 i2 = 2;
  p[0] = wp::extract(v, i0); p[1] = wp::extract(v, i1); p[2] = wp::extract(v, i2);
}
static CUDA_CALLABLE void store_q4(float* p, wp::quat_t<wp::float32> q) {
  const wp::int32 i0 = 0; const wp::int32 i1 = 1; const wp::int32 i2 = 2; const wp::int32 i3 = 3;
  p[0] = wp::extract(q, i0); p[1] = wp::extract(q, i1); p[2] = wp::extract(q, i2); p[3] = wp::extract(q, i3);
}
"#;

/// 产品侧共有前缀：struct V/Q 与表达式基元逐字取自 KINEMATICS_CUDA。
fn product_prelude(kinematics_cuda: &str) -> Result<&str, String> {
    let start = kinematics_cuda
        .find("struct V { float x,y,z; };")
        .ok_or_else(|| "产品 CUDA 缺少 struct V".to_string())?;
    let rotate = function_block(kinematics_cuda, "__device__ V rotate(Q q,V v)")?;
    let rotate_offset = rotate.as_ptr() as usize - kinematics_cuda.as_ptr() as usize;
    Ok(&kinematics_cuda[start..rotate_offset + rotate.len()])
}

/// 产品侧探针：rotate / multiply / normalize（默认包装：先加载 q）。
pub fn product_probes(kinematics_cuda: &str) -> Result<Vec<(String, String)>, String> {
    let prelude = product_prelude(kinematics_cuda)?;
    Ok(vec![
        (
            "rotate".to_string(),
            format!(
                "{prelude}\nextern \"C\" __global__ void probe_rotate(const float* in,float* out) {{\n  Q q=load_q(in); V v=load_v(in+4);\n  store_v(out,rotate(q,v));\n}}\n"
            ),
        ),
        (
            "multiply".to_string(),
            format!(
                "{prelude}\nextern \"C\" __global__ void probe_multiply(const float* in,float* out) {{\n  Q a=load_q(in); Q b=load_q(in+4);\n  store_q(out,multiply(a,b));\n}}\n"
            ),
        ),
        (
            "normalize".to_string(),
            format!(
                "{prelude}\nextern \"C\" __global__ void probe_normalize(const float* in,float* out) {{\n  Q q=load_q(in);\n  store_q(out,normalize(q));\n}}\n"
            ),
        ),
    ])
}

/// 上游侧共用的生成 CUDA helper（逐字提取）。
fn upstream_helpers(smooth_cuda: &str) -> Result<(&str, &str), String> {
    let rotate = function_block(
        smooth_cuda,
        "static CUDA_CALLABLE wp::vec_t<3, wp::float32> rot_vec_quat_0(",
    )?;
    let multiply = function_block(
        smooth_cuda,
        "static CUDA_CALLABLE wp::quat_t<wp::float32> mul_quat_0(",
    )?;
    Ok((rotate, multiply))
}

/// 上游侧探针（默认包装：先加载 q）。
pub fn upstream_probes(smooth_cuda: &str) -> Result<Vec<(String, String)>, String> {
    let (rotate, multiply) = upstream_helpers(smooth_cuda)?;
    Ok(vec![
        (
            "rotate".to_string(),
            format!(
                "{UPSTREAM_PREAMBLE}{rotate}\n{IO_HELPERS}\nextern \"C\" __global__ void probe_rotate(const float* in,float* out) {{\n  wp::quat_t<wp::float32> q=load_q4(in); wp::vec_t<3, wp::float32> v=load_v3(in+4);\n  store_v3(out,rot_vec_quat_0(v,q));\n}}\n"
            ),
        ),
        (
            "multiply".to_string(),
            format!(
                "{UPSTREAM_PREAMBLE}{multiply}\n{IO_HELPERS}\nextern \"C\" __global__ void probe_multiply(const float* in,float* out) {{\n  wp::quat_t<wp::float32> a=load_q4(in); wp::quat_t<wp::float32> b=load_q4(in+4);\n  store_q4(out,mul_quat_0(a,b));\n}}\n"
            ),
        ),
        (
            "normalize".to_string(),
            format!(
                "{UPSTREAM_PREAMBLE}{IO_HELPERS}\nextern \"C\" __global__ void probe_normalize(const float* in,float* out) {{\n  wp::quat_t<wp::float32> q=load_q4(in);\n  store_q4(out,wp::normalize(q));\n}}\n"
            ),
        ),
    ])
}

/// 产品侧 rotate“先加载 v”包装变体（测量包装上下文对收缩的影响）。
pub fn product_rotate_v_first(kinematics_cuda: &str) -> Result<String, String> {
    Ok(format!(
        "{}\nextern \"C\" __global__ void probe_rotate_v_first(const float* in,float* out) {{\n  V v=load_v(in+4); Q q=load_q(in);\n  store_v(out,rotate(q,v));\n}}\n",
        product_prelude(kinematics_cuda)?
    ))
}

/// 上游侧 rotate“先加载 v”包装变体。
pub fn upstream_rotate_v_first(smooth_cuda: &str) -> Result<String, String> {
    let (rotate, _) = upstream_helpers(smooth_cuda)?;
    Ok(format!(
        "{UPSTREAM_PREAMBLE}{rotate}\n{IO_HELPERS}\nextern \"C\" __global__ void probe_rotate_v_first(const float* in,float* out) {{\n  wp::vec_t<3, wp::float32> v=load_v3(in+4); wp::quat_t<wp::float32> q=load_q4(in);\n  store_v3(out,rot_vec_quat_0(v,q));\n}}\n"
    ))
}

/// 产品侧 rotate 的候选转录：按上游生成代码的语句/操作数顺序重写表达式，
/// 用于验证“只改源形态能否复现上游收缩”。不进入产品源码。
pub fn product_rotate_candidate(kinematics_cuda: &str) -> Result<String, String> {
    Ok(format!(
        "{}\n{IO_ROTATE_CANDIDATE}\nextern \"C\" __global__ void probe_rotate_candidate(const float* in,float* out) {{\n  Q q=load_q(in); V v=load_v(in+4);\n  store_v(out,rotate_candidate(q,v));\n}}\n",
        product_prelude(kinematics_cuda)?
    ))
}

/// 候选转录的“先加载 v”包装变体。
pub fn product_rotate_candidate_v_first(kinematics_cuda: &str) -> Result<String, String> {
    Ok(format!(
        "{}\n{IO_ROTATE_CANDIDATE}\nextern \"C\" __global__ void probe_rotate_candidate(const float* in,float* out) {{\n  V v=load_v(in+4); Q q=load_q(in);\n  store_v(out,rotate_candidate(q,v));\n}}\n",
        product_prelude(kinematics_cuda)?
    ))
}

const IO_ROTATE_CANDIDATE: &str = r#"
__device__ V smul(float s,V a) { return {s*a.x,s*a.y,s*a.z}; }
__device__ V rotate_candidate(Q q,V v) {
  V u={q.x,q.y,q.z};
  float d1=dot_v(u,v);
  V lhs=smul(2.0f,smul(d1,u));
  float factor=q.w*q.w-dot_v(u,u);
  V r=add(lhs,smul(factor,v));
  V c=cross_v(u,v);
  float two_s=2.0f*q.w;
  return add(r,smul(two_s,c));
}
"#;

/// 产品探针入口名。
pub fn product_entry(id: &str) -> String {
    format!("probe_{id}")
}
