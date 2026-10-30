// G10@P5 M2：convex-vs-mesh multicontact 的 mesh 几何助手设备版。
// Adapted from frozen MJWarp collision_gjk.py `_intersect1`（L1591-1602）、`_intersect2`
// （L1605-1616）、`_mesh_normals`（L1620-1686）、`_mesh_edge_normals`（L1690-1734）、
// `_mesh_face`（L1926-1948），revision 71da24d. Apache-2.0.
//
// 数值纪律：全程 f32、与上游相同操作数顺序、不做代数化简（照写 `a*b+c`，收缩交给
// `--fmad=true`）；矩阵乘 `mat @ v` 用行优先 `mul_mat3_vec`（与上游 `wp.mat33` 行约定一致），
// 再按上游顺序加 `pos`。
//
// 容量边界（本实现写死；超出时返回 0 → 由 `g5_multicontact` 回退单接触）：
// - 单顶点相邻 polygon 数（= `polymapnum[vertadr+v]`，上游 `nmeshdegmax`）≤
//   `G5_MC_MESH_DEG_MAX`(10)，与 `contact.cu` 的 `n1`/`n2`/`endvert` scratch 一致；
// - 单个 polygon 顶点数（`polyvertnum`，上游 `npolygonmax`）≤ `G5_MC_MESH_FACE_MAX`(20，
//   A1 2026-10-31 由 16 提升)，与 `face1`/`face2` scratch 一致。
// 本片 fixture（cube）的 degree=3、polygon 顶点=4，远低于上界；一般 mesh 超出上界的组合
// 记未覆盖，不改变判据。
#pragma once
#include "multicontact_geom.cu"

// 上游 `_intersect1`（L1591-1602）：求两个整数区间的最多 2 个公共值，返回个数。
// `res` 长度为 2（对应上游 `wp.vec2i`）；按区间顺序扫描，最多到 2 即返回。
__device__ __forceinline__ int g5_mc_intersect1(const int* a1, const int* a2, int start1,
                                                 int start2, int len1, int len2, int* res) {
    int count = 0;
    for (int i = start1; i < start1 + len1; ++i) {
        for (int j = start2; j < start2 + len2; ++j) {
            if (a1[i] == a2[j]) {
                res[count] = a1[i];
                count += 1;
                if (count == 2) {
                    return 2;
                }
            }
        }
    }
    return count;
}

// 上游 `_intersect2`（L1605-1616）：把已收集的最多 2 个值与第三个整数区间求交。
__device__ __forceinline__ int g5_mc_intersect2(const int* a1, const int* a2, int start2, int len1,
                                                 int len2, int* res) {
    int count = 0;
    for (int i = 0; i < len1; ++i) {
        for (int j = start2; j < start2 + len2; ++j) {
            if (a1[i] == a2[j]) {
                res[count] = a1[i];
                count += 1;
                if (count == 2) {
                    return 2;
                }
            }
        }
    }
    return count;
}

// 上游 `_mesh_normals`（L1620-1686）：由最多 3 个 mesh 顶点（局部顶点号）恢复可能的 polygon
// 法线。`vertadr` = 该 geom 的 mesh_vert 起点；`polyadr` = `mesh_polyadr[dataid]`
// （逐 mesh 的首个 polygon 地址）。返回写出的法线数；0 表示无法恢复。
__device__ __forceinline__ int g5_mc_mesh_normals(int feature_dim, const int* feature_index,
                                                   const float* mat, int vertadr, int polyadr,
                                                   const g5_mc_mesh_view& mesh, v3* normal_out,
                                                   int* index_out) {
    int v1 = feature_index[0];
    int v2 = feature_index[1];
    int v3i = feature_index[2];

    if (feature_dim == 3) {
        int v1_adr = mesh.polymapadr[vertadr + v1];
        int v1_num = mesh.polymapnum[vertadr + v1];

        int v2_adr = mesh.polymapadr[vertadr + v2];
        int v2_num = mesh.polymapnum[vertadr + v2];

        int v3_adr = mesh.polymapadr[vertadr + v3i];
        int v3_num = mesh.polymapnum[vertadr + v3i];

        int edgeset[2];
        int n = g5_mc_intersect1(mesh.polymap, mesh.polymap, v1_adr, v2_adr, v1_num, v2_num,
                                 edgeset);
        if (n == 0) {
            return 0;
        }
        int faceset[2];
        n = g5_mc_intersect2(edgeset, mesh.polymap, v3_adr, n, v3_num, faceset);
        if (n == 0) {
            return 0;
        }

        // three vertices on mesh define a unique face
        normal_out[0] = mul_mat3_vec(mat, g5_mc_mesh_poly_normal(mesh, polyadr + faceset[0]));
        index_out[0] = faceset[0];
        return 1;
    }

    if (feature_dim == 2) {
        int v1_adr = mesh.polymapadr[vertadr + v1];
        int v1_num = mesh.polymapnum[vertadr + v1];

        int v2_adr = mesh.polymapadr[vertadr + v2];
        int v2_num = mesh.polymapnum[vertadr + v2];

        // up to two faces as two vertices define an edge
        int edgeset[2];
        int n = g5_mc_intersect1(mesh.polymap, mesh.polymap, v1_adr, v2_adr, v1_num, v2_num,
                                 edgeset);
        if (n == 0) {
            return 0;
        }
        for (int i = 0; i < n; ++i) {
            normal_out[i] = mul_mat3_vec(mat, g5_mc_mesh_poly_normal(mesh, polyadr + edgeset[i]));
            index_out[i] = edgeset[i];
        }
        return n;
    }

    if (feature_dim == 1) {
        int v1_adr = mesh.polymapadr[vertadr + v1];
        int v1_num = mesh.polymapnum[vertadr + v1];
        if (v1_num > G5_MC_MESH_DEG_MAX) {
            return 0;
        }
        for (int i = 0; i < v1_num; ++i) {
            int index = mesh.polymap[v1_adr + i];
            normal_out[i] = mul_mat3_vec(mat, g5_mc_mesh_poly_normal(mesh, polyadr + index));
            index_out[i] = index;
        }
        return v1_num;
    }
    return 0;
}

// 上游 `_mesh_edge_normals`（L1690-1734）：由最多 2 个顶点恢复可能的边方向与端点。
__device__ __forceinline__ int g5_mc_mesh_edge_normals(int dim, const float* mat, v3 pos,
                                                       int vertadr, int polyadr,
                                                       const g5_mc_mesh_view& mesh, v3 v1, v3 v2,
                                                       int v1i, v3* normal_out, v3* endvert_out) {
    // only one edge
    if (dim == 2) {
        endvert_out[0] = v2;
        normal_out[0] = v3_normalize(v3_sub(v2, v1));
        return 1;
    }

    if (dim == 1) {
        int v1_adr = mesh.polymapadr[vertadr + v1i];
        int v1_num = mesh.polymapnum[vertadr + v1i];
        if (v1_num > G5_MC_MESH_DEG_MAX) {
            return 0;
        }

        // loop through all faces with vertex v1
        for (int i = 0; i < v1_num; ++i) {
            int idx = mesh.polymap[v1_adr + i];
            int adr = mesh.polyvertadr[polyadr + idx];
            int nvert = mesh.polyvertnum[polyadr + idx];
            // find previous vertex in polygon to form edge
            for (int j = 0; j < nvert; ++j) {
                if (mesh.polyvert[adr + j] == v1i) {
                    int k = (j == 0) ? nvert - 1 : j - 1;
                    endvert_out[i] = v3_add(
                        mul_mat3_vec(mat,
                                     g5_mesh_vertex(mesh.vert, vertadr + mesh.polyvert[adr + k])),
                        pos);
                    normal_out[i] = v3_normalize(v3_sub(endvert_out[i], v1));
                }
            }
        }
        return v1_num;
    }
    return 0;
}

// 上游 `_mesh_face`（L1926-1948）：由 polygon 索引恢复 mesh 面的世界顶点（逆序）。
__device__ __forceinline__ int g5_mc_mesh_face(const float* mat, v3 pos, int vertadr, int polyadr,
                                                const g5_mc_mesh_view& mesh, int idx,
                                                v3* face_out) {
    int adr = mesh.polyvertadr[polyadr + idx];
    int nvert = mesh.polyvertnum[polyadr + idx];
    if (nvert > G5_MC_MESH_FACE_MAX) {
        return 0;
    }
    int j = 0;
    for (int i = nvert - 1; i >= 0; --i) {
        face_out[j] = v3_add(mul_mat3_vec(mat, g5_mesh_vertex(mesh.vert, vertadr +
                                                                          mesh.polyvert[adr + i])),
                             pos);
        j += 1;
    }
    return nvert;
}
