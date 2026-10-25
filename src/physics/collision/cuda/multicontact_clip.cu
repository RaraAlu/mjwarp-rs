// G10@P5 G5：multicontact 的多边形裁剪设备版（上游 `_polygon_clip` 及其平面/取点助手）。
// Adapted from frozen MJWarp collision_gjk.py L1487-1528（`_area4` / `_polygon_quad`）、
// L2027-2195（`_plane_normal` / `_halfspace` / `_plane_intersect` / `_witness_on_face` /
// `_polygon_clip`）, revision 71da24d. Apache-2.0.
//
// `_set_edge`（L2199）属几何恢复助手，放在 `multicontact_geom.cu`；本文件只做裁剪。
// 数值纪律同 geom 片：全 f32、上游操作数顺序、不做代数化简。
//
// 容量契约（调用方 `multicontact.cu` 的本地 scratch）：`plane_normal` 16 个 v3、`plane_dist`
// 16 个 float（cylinder 端面 16 边形时的最大 nface1）、`polygon_out`/`clipped_out` 各 32 个 v3
// （上游 npolygonmax=16 的两倍上界，见 L2094 注释）。
#pragma once
#include "multicontact_geom.cu"

// 上游 L50：`INTERSECT_TOL`（平面与线段相交的参数容差，逐字 0.0000003）。
#define G5_MC_INTERSECT_TOL 0.0000003f  // 0x34A10FB0
// 上游 L30：`FLOAT_MAX`（`_plane_intersect` 的“平行/无交点”哨兵）。
#define G5_MC_FLOAT_MAX 1.0e30f  // 0x7149F2CA

// 上游 `_area4`（L1487）：空间中四点组成的四边形面积。
__device__ __forceinline__ float g5_mc_area4(v3 a, v3 b, v3 c, v3 d) {
    return 0.5f * v3_length(v3_cross(v3_sub(a, c), v3_sub(b, d)));
}

// 上游 `_polygon_quad`（L1493）：凸多边形（npolygon > 4）中面积最大的四边形索引。
// 上游用 `res` 记住最近一次改进；未改进时不回写，故四个下标只在 m 提升处更新。
__device__ __forceinline__ void g5_mc_polygon_quad(const v3* polygon, int npolygon, int* out) {
    int b = 1;
    int c = 2;
    int d = 3;
    out[0] = 0;
    out[1] = b;
    out[2] = c;
    out[3] = d;
    float m = g5_mc_area4(polygon[0], polygon[b], polygon[c], polygon[d]);
    for (int a = 0; a < npolygon; ++a) {
        while (true) {
            float m_next =
                g5_mc_area4(polygon[a], polygon[b], polygon[c], polygon[(d + 1) % npolygon]);
            if (m_next <= m) break;
            m = m_next;
            d = (d + 1) % npolygon;
            out[0] = a;
            out[1] = b;
            out[2] = c;
            out[3] = d;
            while (true) {
                float m_next =
                    g5_mc_area4(polygon[a], polygon[b], polygon[(c + 1) % npolygon], polygon[d]);
                if (m_next <= m) break;
                m = m_next;
                c = (c + 1) % npolygon;
                out[0] = a;
                out[1] = b;
                out[2] = c;
                out[3] = d;
                while (true) {
                    float m_next =
                        g5_mc_area4(polygon[a], polygon[(b + 1) % npolygon], polygon[c], polygon[d]);
                    if (m_next <= m) break;
                    m = m_next;
                    b = (b + 1) % npolygon;
                    out[0] = a;
                    out[1] = b;
                    out[2] = c;
                    out[3] = d;
                }
            }
        }
        if (b == a) {
            b = (b + 1) % npolygon;
            if (c == b) {
                c = (c + 1) % npolygon;
                if (d == c) {
                    d = (d + 1) % npolygon;
                }
            }
        }
    }
}

// 上游 `_plane_normal`（L2027）：由边 v1->v2 与方向 n 张成的裁剪平面。
// 返回平面常数 `dot(res, v1)`，平面法线写入 `normal_out`。
__device__ __forceinline__ float g5_mc_plane_normal(v3 v1, v3 v2, v3 n, v3* normal_out) {
    v3 v3v = v3_add(v1, n);
    v3 res = v3_cross(v3_sub(v2, v1), v3_sub(v3v, v1));
    *normal_out = res;
    return v3_dot(res, v1);
}

// 上游 `_halfspace`（L2034）：点 p 是否在过 a、法线 n 的半空间内。
__device__ __forceinline__ bool g5_mc_halfspace(v3 a, v3 n, v3 p) {
    return v3_dot(v3_sub(p, a), n) > -1.0e-10f;
}

// 上游 `_plane_intersect`（L2039）：直线 a + t(b - a) 与平面的交点参数。
__device__ __forceinline__ float g5_mc_plane_intersect(v3 pn, float pd, v3 a, v3 b) {
    float dot = v3_dot(pn, v3_sub(b, a));

    // parallel; no intersection
    if (fabsf(dot) < 1.0e-10f) {
        return G5_MC_FLOAT_MAX;
    }

    return (pd - v3_dot(pn, a)) / dot;
}

// 上游 `_witness_on_face`（L2051）：把裁剪多边形顶点投影成一对 witness 与距离。
__device__ __forceinline__ void g5_mc_witness_on_face(v3 v, v3 p, v3 n, v3 dir, v3* w1, v3* w2,
                                                      float* dist) {
    v3 d = v3_sub(v, p);
    *dist = v3_dot(d, n);
    float abs_dist = fabsf(*dist);
    *w1 = v3_sub(v, v3_scale(dir, abs_dist));
    *w2 = v;
}

// 上游 `_polygon_clip`（L2061）：把 face2 多边形裁剪到 face1 的逐个半空间上，
// 输出 <=4 对 witness 与距离，返回接触数（0 表示退化/无交叠）。
__device__ __forceinline__ int g5_mc_polygon_clip(v3* plane_normal, float* plane_dist,
                                                  const v3* face1, int nface1, const v3* face2,
                                                  int nface2, v3 n, v3 dir, v3* polygon_out,
                                                  v3* clipped_out, v3* witness1, v3* witness2,
                                                  float* dists) {
    for (int i = 0; i < 4; ++i) {
        witness1[i] = v3_make(0.0f, 0.0f, 0.0f);
        witness2[i] = v3_make(0.0f, 0.0f, 0.0f);
        dists[i] = 0.0f;
    }

    // clipping face needs to be at least a triangle
    if (nface1 < 3) {
        return 0;
    }

    // compute plane normal and distance to plane for each vertex
    v3* pn = plane_normal;
    float* pd = plane_dist;
    for (int i = 0; i < nface1 - 1; ++i) {
        pd[i] = g5_mc_plane_normal(face1[i], face1[i + 1], n, &pn[i]);
    }
    pd[nface1 - 1] = g5_mc_plane_normal(face1[nface1 - 1], face1[0], n, &pn[nface1 - 1]);

    // reserve 2 * max_sides as max sides for a clipped polygon
    int npolygon = nface2;
    int nclipped = 0;

    for (int i = 0; i < nface2; ++i) {
        polygon_out[i] = face2[i];
    }

    // clip the polygon by one edge e at a time
    v3* polygon = polygon_out;
    v3* clipped = clipped_out;
    for (int e = 0; e < nface1; ++e) {
        for (int i = 0; i < npolygon; ++i) {
            // get edge PQ of the polygon
            v3 P = polygon[i];
            v3 Q = polygon[(i + 1) % npolygon];

            // determine if P and Q are in the halfspace of the clipping edge
            bool inside1 = g5_mc_halfspace(face1[e], pn[e], P);
            bool inside2 = g5_mc_halfspace(face1[e], pn[e], Q);

            // PQ entirely outside the clipping edge, skip
            if (!inside1 && !inside2) {
                continue;
            }

            // edge PQ is inside the clipping edge, add Q
            if (inside1 && inside2) {
                clipped[nclipped] = Q;
                nclipped += 1;
                continue;
            }

            // add new vertex to clipped polygon where PQ intersects the clipping edge
            float t = g5_mc_plane_intersect(pn[e], pd[e], P, Q);
            if (t > -G5_MC_INTERSECT_TOL && t < 1.0f + G5_MC_INTERSECT_TOL) {
                t = fminf(fmaxf(t, 0.0f), 1.0f);  // 上游 wp.clamp(t, 0.0, 1.0)
                clipped[nclipped] = v3_add(P, v3_scale(v3_sub(Q, P), t));
                nclipped += 1;
            }

            // add Q as PQ is now back inside the clipping edge
            if (inside2) {
                clipped[nclipped] = Q;
                nclipped += 1;
            }
        }

        // swap clipped and polygon
        v3* tmp = polygon;
        polygon = clipped;
        clipped = tmp;
        npolygon = nclipped;
        nclipped = 0;
    }

    // prune out vertices with positive distance from the face
    int m = npolygon;
    npolygon = 0;
    for (int i = 0; i < m; ++i) {
        if (v3_dot(v3_sub(polygon[i], face1[0]), n) <= 0.0f) {
            if (npolygon != i) {
                polygon[npolygon] = polygon[i];
            }
            npolygon += 1;
        }
    }

    if (npolygon < 1) {
        return 0;
    }

    // if the face is an edge, remove potential duplicates
    if (nface2 == 2 && npolygon > 2) {
        int best1 = 0;
        int best2 = 1;
        float max_d = 0.0f;
        for (int i = 0; i < npolygon; ++i) {
            v3 polygon_out_i = polygon[i];
            for (int j = i + 1; j < npolygon; ++j) {
                v3 diff = v3_sub(polygon[j], polygon_out_i);
                float d2 = v3_dot(diff, diff);
                if (d2 > max_d) {
                    max_d = d2;
                    best1 = i;
                    best2 = j;
                }
            }
        }

        g5_mc_witness_on_face(polygon[best1], face1[0], n, dir, &witness1[0], &witness2[0],
                              &dists[0]);
        g5_mc_witness_on_face(polygon[best2], face1[0], n, dir, &witness1[1], &witness2[1],
                              &dists[1]);

        return 2;
    }

    if (npolygon > 4) {
        int quad[4];
        g5_mc_polygon_quad(polygon, npolygon, quad);
        for (int i = 0; i < 4; ++i) {
            g5_mc_witness_on_face(polygon[quad[i]], face1[0], n, dir, &witness1[i], &witness2[i],
                                  &dists[i]);
        }
        return 4;
    }

    // no pruning needed
    for (int i = 0; i < npolygon; ++i) {
        g5_mc_witness_on_face(polygon[i], face1[0], n, dir, &witness1[i], &witness2[i], &dists[i]);
    }
    return npolygon;
}
