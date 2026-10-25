#pragma once
// G10@P5 G5 碰撞 GPU 内核公共数学：与 host `gjk/math.rs` 逐式对应。
// Adapted from frozen MJWarp math.py, revision 71da24d. Apache-2.0.
// 编译时用 NVRTC `--fmad=false` 可与 host f32 源码顺序逐位对照；
// GPU 判据（对上游 f32 GPU）另按 pinned PTX 的显式收缩模式编译，两模式分开记录。
#pragma once

struct v3 {
  float x, y, z;
};

__device__ __forceinline__ v3 v3_make(float x, float y, float z) {
  v3 out = {x, y, z};
  return out;
}

__device__ __forceinline__ v3 v3_add(v3 a, v3 b) {
  return v3_make(a.x + b.x, a.y + b.y, a.z + b.z);
}

__device__ __forceinline__ v3 v3_sub(v3 a, v3 b) {
  return v3_make(a.x - b.x, a.y - b.y, a.z - b.z);
}

__device__ __forceinline__ v3 v3_neg(v3 a) {
  return v3_make(-a.x, -a.y, -a.z);
}

__device__ __forceinline__ v3 v3_scale(v3 a, float s) {
  return v3_make(a.x * s, a.y * s, a.z * s);
}

__device__ __forceinline__ v3 v3_cw_mul(v3 a, v3 b) {
  return v3_make(a.x * b.x, a.y * b.y, a.z * b.z);
}

// 上游 `warp/native/builtin.h`：`sign(x) = x < 0 ? -1 : 1`（**零/-0.0 返回 +1，不返回 0**）。
// 设备侧旧实现把零映射为 0，导致 BOX 支撑在 local_dir 分量恰为 0 时取面中心而非角点
// （ellipsoid-box：GJK 首轮支撑点即分叉 → dim 4→3 → EPA 路径/witness 切向偏；见证据 §11.6）。
__device__ __forceinline__ v3 v3_sign(v3 a) {
  v3 out = {a.x < 0.0f ? -1.0f : 1.0f, a.y < 0.0f ? -1.0f : 1.0f,
            a.z < 0.0f ? -1.0f : 1.0f};
  return out;
}

__device__ __forceinline__ float v3_dot(v3 a, v3 b) {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}

__device__ __forceinline__ v3 v3_cross(v3 a, v3 b) {
  return v3_make(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z,
                 a.x * b.y - a.y * b.x);
}

__device__ __forceinline__ float v3_length(v3 a) {
  return sqrtf(v3_dot(a, a));
}

__device__ __forceinline__ v3 v3_normalize(v3 a) {
  float norm = v3_length(a);
  if (norm == 0.0f) return v3_make(0.0f, 0.0f, 0.0f);
  return v3_make(a.x / norm, a.y / norm, a.z / norm);
}

// 行优先 mat33 @ vec3（与 host `mul_mat3_vec` 一致：m[0]*v0+m[1]*v1+m[2]*v2）。
__device__ __forceinline__ v3 mul_mat3_vec(const float* m, v3 v) {
  return v3_make(m[0] * v.x + m[1] * v.y + m[2] * v.z,
                 m[3] * v.x + m[4] * v.y + m[5] * v.z,
                 m[6] * v.x + m[7] * v.y + m[8] * v.z);
}

// transpose(mat33) @ vec3（与 host `transpose_mul_mat3_vec` 一致）。
__device__ __forceinline__ v3 transpose_mul_mat3_vec(const float* m, v3 v) {
  return v3_make(m[0] * v.x + m[3] * v.y + m[6] * v.z,
                 m[1] * v.x + m[4] * v.y + m[7] * v.z,
                 m[2] * v.x + m[5] * v.y + m[8] * v.z);
}

// 上游 `relative_matrix`：`transpose(a) * b`，行优先存储（rel[row][col] = Σ_k a[k][row]*b[k][col]）。
// 累加按上游 `warp/native/mat.h::mul(mat,mat)` 的 `muladd`（fmaf）语义用显式 `__fmaf_rn` 链
// （k 升序：`sum = a0*b0; sum = fma(a1,b1,sum); sum = fma(a2,b2,sum)`）；不交给编译器按
// `--fmad` 自动收缩，避免判据位型随编译开关漂移。注意 `transpose_mul_mat3_vec`（matvec）
// 上游是普通乘加，保持原样。
// 判据是上游语义；host `sdf/descent.rs` / `hfield/driver.rs` 的旧 helper 在行优先约定下
// 返回的是转置（`bᵀa`）且用普通乘加，设备侧不复制该约定（详见 `src/physics/collision/relmat.rs`）。
__device__ __forceinline__ void relative_matrix(const float* a, const float* b,
                                                float* out) {
  for (int row = 0; row < 3; ++row) {
    for (int column = 0; column < 3; ++column) {
      float sum = a[row] * b[column];
      sum = __fmaf_rn(a[3 + row], b[3 + column], sum);
      out[row * 3 + column] = __fmaf_rn(a[6 + row], b[6 + column], sum);
    }
  }
}

// 上游 `math.make_frame`：以法线为第一行的接触坐标系（行优先）。
__device__ __forceinline__ void make_frame(v3 normal, float* frame) {
  v3 a = v3_normalize(normal);
  v3 selector = v3_make(0.0f, 1.0f, 0.0f);
  if (!(-0.5f < a.y && a.y < 0.5f)) selector = v3_make(0.0f, 0.0f, 1.0f);
  v3 b = v3_normalize(v3_sub(selector, v3_scale(a, v3_dot(a, selector))));
  if (v3_length(a) == 0.0f) b = v3_make(0.0f, 0.0f, 0.0f);
  v3 c = v3_cross(a, b);
  frame[0] = a.x; frame[1] = a.y; frame[2] = a.z;
  frame[3] = b.x; frame[4] = b.y; frame[5] = b.z;
  frame[6] = c.x; frame[7] = c.y; frame[8] = c.z;
}
