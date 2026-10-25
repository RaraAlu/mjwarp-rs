// G10@P5 G5：multicontact 的 BOX/CYLINDER 几何助手设备版，与 host
// `gjk/multicontact/{feature,normals}.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py L1533-1587（`_feature_dim` / `_aligned_faces` /
// `_aligned_face_edge`）、L1739-1922（`_box_normals2` / `_box_normals` / `_box_edge_normals` /
// `_box_face`）、L1953-2023（`_cylinder_normals` / `_cylinder_edge_normals` / `_cylinder_face`）、
// L2199-2211（`_set_edge`）, revision 71da24d. Apache-2.0.
//
// 本片只覆盖 BOX/CYLINDER。mesh 多接触不实现：上游 `_mesh_normals`（L1621）、
// `_mesh_edge_normals`（L1691）、`_mesh_face`（L1927）及其助手 `_intersect1`/`_intersect2`
// （L1592/L1606）整体省略；mesh 类型在 `multicontact.cu` 直接回退单接触。
//
// 数值纪律：全程 f32、与上游相同操作数顺序、不做代数化简（照写 `a*b+c`，收缩交给
// `--fmad=true`）。常量按上游 f64 计算值落到 f32：容差宏在行尾给出位型，`_CYLINDER_*_16`
// 两个表的十进制均为 f32 最短回读字面量（已逐项回读校验 == host `gjk/mod.rs` 的 from_bits）。
#pragma once
#include "geom.cu"

// 上游 L46-47：`FACE_TOL = cos(0.0889)`、`EDGE_TOL = sin(0.0889)`。
#define G5_MC_FACE_TOL 0.996051f      // 0x3F7EFD33
#define G5_MC_EDGE_TOL 0.0887829438f  // 0x3DB5D3D5
// 上游 L33-34：`MINVAL` 用于 `_aligned_face_edge` 的朝向门；`MINVAL2` 在 L1533-2451
// 的调用链里未被引用，仅为常量完整性保留。
#define G5_MC_MINVAL 1.0e-15f
#define G5_MC_MINVAL2 1.0e-30f

// 上游 `float(bool)`：整数掩码非零的真值。
__device__ __forceinline__ bool g5_mc_bit(int value, int mask) {
    return (value & mask) != 0;
}

// 取行优先 mat33 的第 col 列（上游 `mat[:, col]`）。
// 上游 `mat33` 的 `mat[:, j]` 是列向量：行优先存储下为 (m[j], m[3+j], m[6+j])。
__device__ __forceinline__ v3 g5_mc_mat_col(const float* mat, int col) {
    return v3_make(mat[col], mat[3 + col], mat[6 + col]);
}

// 上游 `mat @ vec3 + pos`（行优先 matvec 后再平移）。
__device__ __forceinline__ v3 g5_mc_rot_pos(const float* mat, v3 v, v3 pos) {
    return v3_add(mul_mat3_vec(mat, v), pos);
}

// 上游 `_feature_dim`（L1533）：返回 feature 维度（1/2/3），必要时重排 index/vertex 的第 2 行。
// `face` 为 EPA 面解包出的三个 polytope 顶点号；`offset` 0 = geom1，1 = geom2。
__device__ __forceinline__ int g5_mc_feature_dim(const int* face, const int* vert_index,
                                                 const v3* vert, int offset, int* feature_index,
                                                 v3* feature_vertex) {
    int v1i = vert_index[2 * face[0] + offset];
    int v2i = vert_index[2 * face[1] + offset];
    int v3i = vert_index[2 * face[2] + offset];

    feature_index[0] = v1i;
    feature_index[1] = v2i;
    feature_index[2] = v3i;
    feature_vertex[0] = vert[2 * face[0] + offset];
    feature_vertex[1] = vert[2 * face[1] + offset];
    feature_vertex[2] = vert[2 * face[2] + offset];

    if (v1i != v2i) {
        return (v3i == v1i || v3i == v2i) ? 2 : 3;
    }

    feature_index[1] = v3i;
    feature_vertex[1] = vert[2 * face[2] + offset];

    return (v1i != v3i) ? 2 : 1;
}

// 上游 `_aligned_faces`（L1559）：找一对相互朝向（dot < -FACE_TOL）的法线，返回 1 表示找到。
// 命中时 `res = {i, j}`（两个法线数组的下标）；未命中保持 {0, 0}。
__device__ __forceinline__ int g5_mc_aligned_faces(const v3* vert1, int len1, const v3* vert2,
                                                   int len2, int* res) {
    res[0] = 0;
    res[1] = 0;
    for (int i = 0; i < len1; ++i) {
        for (int j = 0; j < len2; ++j) {
            if (v3_dot(vert1[i], vert2[j]) < -G5_MC_FACE_TOL) {
                res[0] = i;
                res[1] = j;
                return 1;
            }
        }
    }
    return 0;
}

// 上游 `_aligned_face_edge`（L1573）：找一条与面垂直（|dot| < EDGE_TOL）的边。
// 命中时 `res = {边下标, 面下标}`。
__device__ __forceinline__ int g5_mc_aligned_face_edge(const v3* edge, int nedge, const v3* face,
                                                       int nface, v3 dir, int* res) {
    res[0] = 0;
    res[1] = 0;
    for (int i = 0; i < nface; ++i) {
        // 忽略背向碰撞方向的面（负点积）。
        if (v3_dot(face[i], dir) <= G5_MC_MINVAL) continue;

        for (int j = 0; j < nedge; ++j) {
            if (fabsf(v3_dot(edge[j], face[i])) < G5_MC_EDGE_TOL) {
                res[0] = j;
                res[1] = i;
                return 1;
            }
        }
    }
    return 0;
}

// 上游 `_set_edge`（L2199）：用 EPA 顶点与 endvert 拼出 2 点“面”。
__device__ __forceinline__ int g5_mc_set_edge(const v3* vert1, const v3* vert2, int start, int end,
                                              int offset, v3* face_out) {
    face_out[0] = vert1[2 * start + offset];
    face_out[1] = vert2[end];
    return 2;
}

// 上游 `_box_normals2`（L1739）：由碰撞法线反推最近的 box 面法线。
__device__ __forceinline__ int g5_mc_box_normals2(const float* mat, v3 n, v3* normal_out,
                                                  int* index_out) {
    // list of box face normals
    const v3 face_normals[6] = {v3_make(1.0f, 0.0f, 0.0f),  v3_make(-1.0f, 0.0f, 0.0f),
                                v3_make(0.0f, 1.0f, 0.0f),  v3_make(0.0f, -1.0f, 0.0f),
                                v3_make(0.0f, 0.0f, 1.0f),  v3_make(0.0f, 0.0f, -1.0f)};

    // get local coordinates of the normal（上游逐项写 transpose(mat) @ n）
    v3 local_n = v3_normalize(transpose_mul_mat3_vec(mat, n));

    // determine if there is a side close to the normal
    for (int i = 0; i < 6; ++i) {
        if (v3_dot(local_n, face_normals[i]) > G5_MC_FACE_TOL) {
            normal_out[0] = mul_mat3_vec(mat, face_normals[i]);
            index_out[0] = i;
            return 1;
        }
    }

    return 0;
}

// 上游 `_box_normals`（L1772）：由最多三个顶点恢复 box 面法线（含 index 语义）。
__device__ __forceinline__ int g5_mc_box_normals(int feature_dim, const int* feature_index,
                                                 const float* mat, v3 dir, v3* normal_out,
                                                 int* index_out) {
    int v1 = feature_index[0];
    int v2 = feature_index[1];
    int v3 = feature_index[2];

    if (feature_dim == 3) {
        int c = 0;
        float x = (float)(g5_mc_bit(v1, 1) && g5_mc_bit(v2, 1) && g5_mc_bit(v3, 1)) -
                  (float)(!g5_mc_bit(v1, 1) && !g5_mc_bit(v2, 1) && !g5_mc_bit(v3, 1));
        float y = (float)(g5_mc_bit(v1, 2) && g5_mc_bit(v2, 2) && g5_mc_bit(v3, 2)) -
                  (float)(!g5_mc_bit(v1, 2) && !g5_mc_bit(v2, 2) && !g5_mc_bit(v3, 2));
        float z = (float)(g5_mc_bit(v1, 4) && g5_mc_bit(v2, 4) && g5_mc_bit(v3, 4)) -
                  (float)(!g5_mc_bit(v1, 4) && !g5_mc_bit(v2, 4) && !g5_mc_bit(v3, 4));
        normal_out[0] = mul_mat3_vec(mat, v3_make(x, y, z));
        float sgn = x + y + z;
        if (x != 0.0f) {
            index_out[c] = 0;
            c += 1;
        }
        if (y != 0.0f) {
            index_out[c] = 2;
            c += 1;
        }
        if (z != 0.0f) {
            index_out[c] = 4;
            c += 1;
        }
        if (sgn == -1.0f) {
            index_out[0] = index_out[0] + 1;
        }
        if (c == 1) {
            return 1;
        }
        return g5_mc_box_normals2(mat, dir, normal_out, index_out);
    }
    if (feature_dim == 2) {
        int c = 0;
        float x = (float)(g5_mc_bit(v1, 1) && g5_mc_bit(v2, 1)) -
                  (float)(!g5_mc_bit(v1, 1) && !g5_mc_bit(v2, 1));
        float y = (float)(g5_mc_bit(v1, 2) && g5_mc_bit(v2, 2)) -
                  (float)(!g5_mc_bit(v1, 2) && !g5_mc_bit(v2, 2));
        float z = (float)(g5_mc_bit(v1, 4) && g5_mc_bit(v2, 4)) -
                  (float)(!g5_mc_bit(v1, 4) && !g5_mc_bit(v2, 4));
        if (x != 0.0f) {
            normal_out[c] = mul_mat3_vec(mat, v3_make(x, 0.0f, 0.0f));
            index_out[c] = (x > 0.0f) ? 0 : 1;
            c += 1;
        }
        if (y != 0.0f) {
            normal_out[c] = mul_mat3_vec(mat, v3_make(0.0f, y, 0.0f));
            index_out[c] = (y > 0.0f) ? 2 : 3;
            c += 1;
        }
        if (z != 0.0f) {
            normal_out[c] = mul_mat3_vec(mat, v3_make(0.0f, 0.0f, z));
            index_out[c] = (z > 0.0f) ? 4 : 5;
            c += 1;
        }
        // c is 1 if edge is diagonal of a box face
        // c is 2 if edge is an external edge of box
        if (c == 1 || c == 2) {
            return c;
        }
        return g5_mc_box_normals2(mat, dir, normal_out, index_out);
    }

    if (feature_dim == 1) {
        float x = g5_mc_bit(v1, 1) ? 1.0f : -1.0f;
        float y = g5_mc_bit(v1, 2) ? 1.0f : -1.0f;
        float z = g5_mc_bit(v1, 4) ? 1.0f : -1.0f;
        normal_out[0] = mul_mat3_vec(mat, v3_make(x, 0.0f, 0.0f));
        normal_out[1] = mul_mat3_vec(mat, v3_make(0.0f, y, 0.0f));
        normal_out[2] = mul_mat3_vec(mat, v3_make(0.0f, 0.0f, z));
        index_out[0] = (x > 0.0f) ? 0 : 1;
        index_out[1] = (y > 0.0f) ? 2 : 3;
        index_out[2] = (z > 0.0f) ? 4 : 5;
        return 3;
    }
    return 0;
}

// 上游 `_box_edge_normals`（L1846）：box 的边-面碰撞候选方向（dim==1 时 3 条相邻边）。
__device__ __forceinline__ int g5_mc_box_edge_normals(int dim, const float* mat, v3 pos, v3 size,
                                                      v3 v1, v3 v2, int v1i, v3* normal_out,
                                                      v3* endvert_out) {
    if (dim == 2) {
        endvert_out[0] = v2;
        normal_out[0] = v3_normalize(v3_sub(v2, v1));
        return 1;
    }

    // return 3 adjacent vertices
    if (dim == 1) {
        float x = g5_mc_bit(v1i, 1) ? size.x : -size.x;
        float y = g5_mc_bit(v1i, 2) ? size.y : -size.y;
        float z = g5_mc_bit(v1i, 4) ? size.z : -size.z;

        endvert_out[0] = g5_mc_rot_pos(mat, v3_make(-x, y, z), pos);
        normal_out[0] = v3_normalize(v3_sub(endvert_out[0], v1));

        endvert_out[1] = g5_mc_rot_pos(mat, v3_make(x, -y, z), pos);
        normal_out[1] = v3_normalize(v3_sub(endvert_out[1], v1));

        endvert_out[2] = g5_mc_rot_pos(mat, v3_make(x, y, -z), pos);
        normal_out[2] = v3_normalize(v3_sub(endvert_out[2], v1));
        return 3;
    }
    return 0;
}

// 上游 `_box_face`（L1884）：按面索引恢复 box 的 4 个世界顶点。
__device__ __forceinline__ int g5_mc_box_face(const float* mat, v3 pos, v3 size, int idx,
                                              v3* face_out) {
    // compute global coordinates of the box face and face normal
    float sx = size.x;
    float sy = size.y;
    float sz = size.z;
    if (idx == 0) {  // right
        face_out[0] = g5_mc_rot_pos(mat, v3_make(sx, sy, sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(sx, sy, -sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(sx, -sy, -sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(sx, -sy, sz), pos);
        return 4;
    }
    if (idx == 1) {  // left
        face_out[0] = g5_mc_rot_pos(mat, v3_make(-sx, sy, -sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(-sx, sy, sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, -sz), pos);
        return 4;
    }
    if (idx == 2) {  // top
        face_out[0] = g5_mc_rot_pos(mat, v3_make(-sx, sy, -sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(sx, sy, -sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(sx, sy, sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(-sx, sy, sz), pos);
        return 4;
    }
    if (idx == 3) {  // bottom
        face_out[0] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(sx, -sy, sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(sx, -sy, -sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, -sz), pos);
        return 4;
    }
    if (idx == 4) {  // front
        face_out[0] = g5_mc_rot_pos(mat, v3_make(-sx, sy, sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(sx, sy, sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(sx, -sy, sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, sz), pos);
        return 4;
    }
    if (idx == 5) {  // back
        face_out[0] = g5_mc_rot_pos(mat, v3_make(sx, sy, -sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(-sx, sy, -sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, -sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(sx, -sy, -sz), pos);
        return 4;
    }
    return 0;
}

// 上游 `_cylinder_normals`（L1953）：由顶点索引反推圆柱端面法线（旋转矩阵第 3 列）。
__device__ __forceinline__ int g5_mc_cylinder_normals(int feature_dim, const int* feature_index,
                                                      const float* mat, v3* normal_out,
                                                      int* index_out) {
    if (feature_dim == 1) {
        float sgn = (feature_index[0] != 0) ? -1.0f : 1.0f;
        v3 col = g5_mc_mat_col(mat, 2);
        normal_out[0] = v3_make(col.x * sgn, col.y * sgn, col.z * sgn);
        index_out[0] = feature_index[0];
        return 1;
    }
    return 0;
}

// 上游 `_cylinder_edge_normals`（L1972）：由碰撞点恢复圆柱边方向与端点。
__device__ __forceinline__ int g5_mc_cylinder_edge_normals(int dim, const float* mat, v3 size, v3 v,
                                                           int v1i, v3* normal_out,
                                                           v3* endvert_out) {
    if (dim == 1 || dim == 2) {
        float sgn = (v1i != 0) ? 1.0f : -1.0f;
        v3 col = g5_mc_mat_col(mat, 2);
        v3 res = v3_make(col.x * sgn, col.y * sgn, col.z * sgn);
        normal_out[0] = res;
        endvert_out[0] = v3_add(v, v3_scale(res, 2.0f * size.y));
        return 1;
    }
    return 0;
}

// 上游 L59-61：`_CYLINDER_COS_16 = cos(i*pi/8)`、`_CYLINDER_SIN_16 = sin(i*pi/8)`（16 边形
// 端面近似）。十进制为 f32 最短回读字面量，逐项校验 == host `gjk/mod.rs` 的 from_bits 位型；
// 近零项 0x248D3132 / 0x250D3132 / 0xA553C9CA 即 f64 `math.cos(pi/2)` 等的 f32 舍入。
static __device__ const float g5_mc_cylinder_cos16[16] = {
    1.0f,           0.9238795f,     0.707106769f,   0.382683426f,  6.123234e-17f,
    -0.382683426f,  -0.707106769f,  -0.9238795f,    -1.0f,         -0.9238795f,
    -0.707106769f,  -0.382683426f,  -1.83697015e-16f, 0.382683426f, 0.707106769f,
    0.9238795f};

static __device__ const float g5_mc_cylinder_sin16[16] = {
    0.0f,           0.382683426f,   0.707106769f,   0.9238795f,    1.0f,
    0.9238795f,     0.707106769f,   0.382683426f,   1.22464685e-16f, -0.382683426f,
    -0.707106769f,  -0.9238795f,    -1.0f,          -0.9238795f,   -0.707106769f,
    -0.382683426f};

// 上游 `_cylinder_face`（L1994）：把圆柱端面近似成 16 边形并返回 16 个世界顶点。
__device__ __forceinline__ int g5_mc_cylinder_face(const float* mat, v3 pos, v3 size, int idx,
                                                   v3* face_out) {
    float sgn = (idx != 0) ? -1.0f : 1.0f;
    v3 center = v3_add(v3_scale(g5_mc_mat_col(mat, 2), sgn * size.y), pos);
    v3 col0 = v3_scale(g5_mc_mat_col(mat, 0), size.x);
    v3 col1 = v3_scale(g5_mc_mat_col(mat, 1), size.x * sgn);
    for (int i = 0; i < 16; ++i) {
        // 上游逐行写 `col0*cos[i] - col1*sin[i] + center`。
        face_out[i] = v3_add(v3_sub(v3_scale(col0, g5_mc_cylinder_cos16[i]),
                                    v3_scale(col1, g5_mc_cylinder_sin16[i])),
                             center);
    }
    return 16;
}
