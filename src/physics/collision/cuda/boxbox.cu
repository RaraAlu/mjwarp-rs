#pragma once
// G10@P5 G5：上游 MJWarp 多点 box-box 碰撞核心 `box_box` 设备版。
// Adapted from frozen MJWarp collision_primitive_core.py L563-591 (`_compute_rotmore`)
// and L595-1049 (`box_box`), revision 71da24d. Apache-2.0.
//
// 逐式 f32 转写：8 槽候选（dist 初值 MJ_MAXVAL=1e10）、3 面法线轴 + 9 边叉积轴、
// 面恢复与边恢复两条路径。mat3×mat3 用上游 `mat.h::mul` 的 fmaf 累加链
// （sum=0; sum=fmaf(...)，同 common.cu `relative_matrix` 口径）；mat3×vec3 用
// common.cu `mul_mat3_vec`；其余为直写 f32 表达式（不加代数化简、不用 double）。
#include "geom.cu"

// 上游 `types.MJ_MAXVAL`（mujoco.mjMAXVAL = 1e10）与 `types.MJ_MINVAL`（1e-15）。
#define G5_BB_MAXVAL 1.0e10f
#define G5_BB_MINVAL 1.0e-15f

struct g5_box_box_result {
    float dist[8];  // 未填充槽 = 1.0e10f (MJ_MAXVAL)
    v3 pos[8];
    v3 normal[8];
};

// 上游 `wp.vec3` 分量读（q=0/1/2）与分量写（用于 `points[n, q] += ...`）。
__device__ __forceinline__ float g5_bb_comp(v3 v, int q) {
    return q == 0 ? v.x : (q == 1 ? v.y : v.z);
}

__device__ __forceinline__ v3 g5_bb_set_comp(v3 v, int q, float x) {
    if (q == 0) { v.x = x; } else if (q == 1) { v.y = x; } else { v.z = x; }
    return v;
}

__device__ __forceinline__ void g5_bb_add_comp(v3* v, int q, float x) {
    if (q == 0) { v->x += x; } else if (q == 1) { v->y += x; } else { v->z += x; }
}

__device__ __forceinline__ v3 g5_bb_abs(v3 v) {
    return v3_make(fabsf(v.x), fabsf(v.y), fabsf(v.z));
}

// 上游 `wp.mat33` 的第 i 行（行优先）与 `wp.transpose(mat33)`。
__device__ __forceinline__ v3 g5_bb_row(const float* m, int i) {
    return v3_make(m[i * 3], m[i * 3 + 1], m[i * 3 + 2]);
}

__device__ __forceinline__ void g5_bb_transpose(const float* m, float* out) {
    for (int i = 0; i < 3; ++i) {
        for (int j = 0; j < 3; ++j) {
            out[i * 3 + j] = m[j * 3 + i];
        }
    }
}

// 上游 `mat.h::mul(mat33, mat33)`：`sum = 0; for k: sum = muladd(a[i][k], b[k][j], sum)`，
// `muladd(float)=fmaf`。显式 `__fmaf_rn` 固定 k 升序累加，避免依赖 `--fmad` 收缩顺序；
// rotmore 为 0/±1 置换阵时每个输出只有一个非零乘积，结果与上游逐位相同。
__device__ __forceinline__ void g5_bb_mat3_mul(const float* a, const float* b, float* out) {
    for (int i = 0; i < 3; ++i) {
        for (int j = 0; j < 3; ++j) {
            float sum = 0.0f;
            sum = __fmaf_rn(a[i * 3 + 0], b[0 * 3 + j], sum);
            sum = __fmaf_rn(a[i * 3 + 1], b[1 * 3 + j], sum);
            sum = __fmaf_rn(a[i * 3 + 2], b[2 * 3 + j], sum);
            out[i * 3 + j] = sum;
        }
    }
}

// 上游 `math.safe_div`：`x / where(y != 0.0, y, MJ_MINVAL)`。
__device__ __forceinline__ float g5_bb_safe_div(float x, float y) {
    return x / (y != 0.0f ? y : G5_BB_MINVAL);
}

// 上游 `_compute_rotmore(face_idx)`（L563-591）：6 个面各一个 0/±1 正交阵。
__device__ __forceinline__ void g5_bb_compute_rotmore(int face_idx, float* rotmore) {
    for (int i = 0; i < 9; ++i) rotmore[i] = 0.0f;
    if (face_idx == 0) {
        rotmore[2] = -1.0f; rotmore[4] = 1.0f; rotmore[6] = 1.0f;
    } else if (face_idx == 1) {
        rotmore[0] = 1.0f; rotmore[5] = -1.0f; rotmore[7] = 1.0f;
    } else if (face_idx == 2) {
        rotmore[0] = 1.0f; rotmore[4] = 1.0f; rotmore[8] = 1.0f;
    } else if (face_idx == 3) {
        rotmore[2] = 1.0f; rotmore[4] = 1.0f; rotmore[6] = -1.0f;
    } else if (face_idx == 4) {
        rotmore[0] = 1.0f; rotmore[5] = 1.0f; rotmore[7] = -1.0f;
    } else if (face_idx == 5) {
        rotmore[0] = -1.0f; rotmore[4] = 1.0f; rotmore[8] = -1.0f;
    }
}

// 上游 `box_box`：返回 8 个候选接触（未填充槽 dist=MJ_MAXVAL、pos/normal 全零）。
__device__ __forceinline__ g5_box_box_result g5_box_box(
    v3 pos1, const float* rot1, v3 size1,
    v3 pos2, const float* rot2, v3 size2, float margin) {
    g5_box_box_result result;
    for (int i = 0; i < 8; ++i) {
        result.dist[i] = G5_BB_MAXVAL;
        result.pos[i] = v3_make(0.0f, 0.0f, 0.0f);
        result.normal[i] = v3_make(0.0f, 0.0f, 0.0f);
    }

    // 盒间坐标变换（L630-640）。
    const v3 pos21 = transpose_mul_mat3_vec(rot1, v3_sub(pos2, pos1));
    const v3 pos12 = transpose_mul_mat3_vec(rot2, v3_sub(pos1, pos2));
    float rot1t[9], rot21[9], rot12[9], rot21abs[9], rot12abs[9];
    g5_bb_transpose(rot1, rot1t);
    g5_bb_mat3_mul(rot1t, rot2, rot21);
    g5_bb_transpose(rot21, rot12);
    for (int i = 0; i < 9; ++i) rot21abs[i] = fabsf(rot21[i]);
    g5_bb_transpose(rot21abs, rot12abs);
    const v3 plen2 = mul_mat3_vec(rot21abs, size2);
    const v3 plen1 = mul_mat3_vec(rot12abs, size1);

    const float s1[3] = {size1.x, size1.y, size1.z}, s2[3] = {size2.x, size2.y, size2.z};
    const float p21[3] = {pos21.x, pos21.y, pos21.z}, pl2[3] = {plen2.x, plen2.y, plen2.z};
    const float p12[3] = {pos12.x, pos12.y, pos12.z}, pl1[3] = {plen1.x, plen1.y, plen1.z};

    // 最大分离轴初值（L642-645）。
    const v3 s_sum_3 = v3_scale(v3_add(size1, size2), 3.0f);
    float separation = margin + s_sum_3.x + s_sum_3.y + s_sum_3.z;
    int axis_code = -1;

    // 第一段：盒面法线轴（L647-661）。
    for (int i = 0; i < 3; ++i) {
        const float c1 = -fabsf(p21[i]) + s1[i] + pl2[i];
        const float c2 = -fabsf(p12[i]) + s2[i] + pl1[i];
        if (c1 < -margin || c2 < -margin) return result;
        if (c1 < separation) {
            separation = c1;
            axis_code = i + 3 * (p21[i] < 0.0f ? 1 : 0) + 0;
        }
        if (c2 < separation) {
            separation = c2;
            axis_code = i + 3 * (p12[i] < 0.0f ? 1 : 0) + 6;
        }
    }

    v3 clnorm = v3_make(0.0f, 0.0f, 0.0f);
    bool inv = false;
    int cle1 = 0, cle2 = 0;

    // 第二段：盒边叉积轴（L668-717）。
    for (int i = 0; i < 3; ++i) {
        for (int j = 0; j < 3; ++j) {
            v3 cross_axis;
            if (i == 0) {
                cross_axis = v3_make(0.0f, -rot12[j * 3 + 2], rot12[j * 3 + 1]);
            } else if (i == 1) {
                cross_axis = v3_make(rot12[j * 3 + 2], 0.0f, -rot12[j * 3 + 0]);
            } else {
                cross_axis = v3_make(-rot12[j * 3 + 1], rot12[j * 3 + 0], 0.0f);
            }
            const float cross_length = v3_length(cross_axis);
            if (cross_length < G5_BB_MINVAL) continue;
            cross_axis = v3_make(cross_axis.x / cross_length, cross_axis.y / cross_length,
                                 cross_axis.z / cross_length);

            const float box_dist = v3_dot(pos21, cross_axis);
            float c3 = 0.0f;
            for (int k = 0; k < 3; ++k) {
                if (k != i) c3 += s1[k] * fabsf(g5_bb_comp(cross_axis, k));
                if (k != j) c3 += s2[k] * rot21abs[i * 3 + 3 - k - j] / cross_length;
            }
            c3 -= fabsf(box_dist);
            if (c3 < -margin) return result;

            if (c3 < separation * (1.0f - 1e-12f)) {
                separation = c3;
                cle1 = 0;
                cle2 = 0;
                for (int k = 0; k < 3; ++k) {
                    if (k != i &&
                        (((g5_bb_comp(cross_axis, k) > 0.0f) ? 1 : 0) ^
                         ((box_dist < 0.0f) ? 1 : 0))) {
                        cle1 += 1 << k;
                    }
                    if (k != j) {
                        const int t = (((rot21[i * 3 + 3 - k - j] > 0.0f) ? 1 : 0) ^
                                       ((box_dist < 0.0f) ? 1 : 0) ^
                                       ((((k - j + 3) % 3 == 1)) ? 1 : 0));
                        if (t) cle2 += 1 << k;
                    }
                }
                axis_code = 12 + i * 3 + j;
                clnorm = cross_axis;
                inv = box_dist < 0.0f;
            }
        }
    }
    if (axis_code == -1) return result;

    v3 points[8];
    float depth[8];
    for (int i = 0; i < 8; ++i) {
        points[i] = v3_make(0.0f, 0.0f, 0.0f);
        depth[i] = 0.0f;
    }

    int n = 0;
    float rw[9], hz = 0.0f;
    v3 pw = v3_make(0.0f, 0.0f, 0.0f), normal = v3_make(0.0f, 0.0f, 0.0f);

    if (axis_code < 12) {
        // 面-顶点路径（L728-837）。
        const int face_idx = axis_code % 6, box_idx = axis_code / 6;
        const bool box2 = box_idx != 0;
        float rotmore[9], r[9], rt[9], rwt[9];
        g5_bb_compute_rotmore(face_idx, rotmore);
        g5_bb_mat3_mul(rotmore, box2 ? rot12 : rot21, r);
        const v3 p = mul_mat3_vec(rotmore, box2 ? pos12 : pos21);
        const v3 s = box2 ? size1 : size2;
        const v3 ss = g5_bb_abs(mul_mat3_vec(rotmore, box2 ? size2 : size1));
        v3 lp = v3_make(p.x, p.y, p.z - ss.z);
        const float s_axis[3] = {s.x, s.y, s.z};
        g5_bb_transpose(r, rt);
        const float lx = ss.x, ly = ss.y;

        int clcorner = 0;
        for (int i = 0; i < 3; ++i) {
            if (r[6 + i] < 0.0f) clcorner += 1 << i;
        }
        for (int i = 0; i < 3; ++i) {
            const float corner_sign = (clcorner & (1 << i)) ? 1.0f : -1.0f;
            lp = v3_add(lp, v3_scale(v3_scale(g5_bb_row(rt, i), s_axis[i]), corner_sign));
        }

        v3 cn1 = v3_make(0.0f, 0.0f, 0.0f), cn2 = cn1;
        int dirs = 0;
        for (int i = 0; i < 3; ++i) {
            if (!(fabsf(r[6 + i]) < 0.5f)) continue;
            const float cn_sign = (clcorner & (1 << i)) ? -2.0f : 2.0f;
            const v3 cn = v3_scale(v3_scale(g5_bb_row(rt, i), s_axis[i]), cn_sign);
            if (dirs == 0) cn1 = cn; else cn2 = cn;
            ++dirs;
        }

        const int kk = dirs * dirs;
        n = 0;
        const v3 zero = v3_make(0.0f, 0.0f, 0.0f);

        // 盒边与盒面线段的交点（L772-792）。
        for (int i = 0; i < kk; ++i) {
            for (int q = 0; q < 2; ++q) {
                const v3 lav = v3_add(lp, i < 2 ? zero : (i == 2 ? cn1 : cn2));
                const v3 lbv = (i == 0 || i == 3) ? cn1 : cn2;
                const float lbv_q = g5_bb_comp(lbv, q);
                if (!(fabsf(lbv_q) > G5_BB_MINVAL)) continue;
                const float br = 1.0f / lbv_q;
                for (int jj = -1; jj <= 1; jj += 2) {
                    const float l = g5_bb_comp(ss, q) * (float)jj;
                    const float c1 = (l - g5_bb_comp(lav, q)) * br;
                    if (c1 < 0.0f || c1 > 1.0f) continue;
                    const float c2 = g5_bb_comp(lav, 1 - q) + g5_bb_comp(lbv, 1 - q) * c1;
                    if (fabsf(c2) > g5_bb_comp(ss, 1 - q)) continue;
                    points[n] = v3_add(lav, v3_scale(lbv, c1));
                    ++n;
                }
            }
        }

        // 两面相交时的网格点（L794-819）。
        if (dirs == 2) {
            const float ax = cn1.x, bx = cn2.x, ay = cn1.y, by = cn2.y;
            const float cdet = g5_bb_safe_div(1.0f, ax * by - bx * ay);
            for (int i = 0; i < 4; ++i) {
                const float llx = (i / 2) ? lx : -lx, lly = (i % 2) ? ly : -ly;
                const float x = llx - lp.x, y = lly - lp.y;
                const float u = (x * by - y * bx) * cdet, v = (y * ax - x * ay) * cdet;
                if (u > 0.0f && v > 0.0f && u < 1.0f && v < 1.0f) {
                    points[n] = v3_make(llx, lly, lp.z + u * cn1.z + v * cn2.z);
                    ++n;
                }
            }
        }

        for (int i = 0; i < (1 << dirs); ++i) {
            const v3 tmpv = v3_add(v3_add(lp, v3_scale(cn1, (float)(i & 1))),
                                   v3_scale(cn2, (i & 2) ? 1.0f : 0.0f));
            if (tmpv.x > -lx && tmpv.x < lx && tmpv.y > -ly && tmpv.y < ly) {
                points[n] = tmpv;
                ++n;
            }
        }

        // 折算深度并取位置中点（L821-833）。
        const int m = n;
        n = 0;
        for (int i = 0; i < m; ++i) {
            if (points[i].z > margin) continue;
            if (i != n) points[n] = points[i];
            depth[n] = points[n].z;
            points[n].z = points[n].z * 0.5f;
            ++n;
        }

        g5_bb_transpose(rotmore, rwt);
        g5_bb_mat3_mul(box2 ? rot2 : rot1, rwt, rw);
        pw = box2 ? pos2 : pos1;
        const float nsign = box2 ? -1.0f : 1.0f;
        normal = v3_make(nsign * rw[2], nsign * rw[5], nsign * rw[8]);
        hz = ss.z;
    } else {
        // 边-边路径（L840-1035）。
        const int edge1 = (axis_code - 12) / 3, edge2 = (axis_code - 12) % 3;
        int ax1 = 1 - (edge2 & 1), ax2 = 2 - (edge2 & 2);
        int pax1 = 1 - (edge1 & 1), pax2 = 2 - (edge1 & 2);
        if (rot21abs[edge1 * 3 + ax1] < rot21abs[edge1 * 3 + ax2]) {
            const int t = ax1; ax1 = ax2; ax2 = t;
        }
        if (rot12abs[edge2 * 3 + pax1] < rot12abs[edge2 * 3 + pax2]) {
            const int t = pax1; pax1 = pax2; pax2 = t;
        }

        float rotmore[9], r[9], rt[9], rwt[9], rw_scaled[9];
        g5_bb_compute_rotmore((cle1 & (1 << pax2)) ? pax2 : pax2 + 3, rotmore);
        v3 p = mul_mat3_vec(rotmore, pos21);
        const v3 rnorm = mul_mat3_vec(rotmore, clnorm);
        g5_bb_mat3_mul(rotmore, rot21, r);
        g5_bb_transpose(r, rt);
        const v3 sv = g5_bb_abs(transpose_mul_mat3_vec(rotmore, size1));
        const float s_axis[3] = {sv.x, sv.y, sv.z};
        const float lx = sv.x, ly = sv.y;
        hz = sv.z;
        p.z = p.z - hz;

        // 盒2最近面的 4 个候选点（L871-888）。
        const v3 rtax1 = g5_bb_row(rt, ax1), rtax2 = g5_bb_row(rt, ax2);
        const v3 rt_edge = g5_bb_row(rt, edge2);
        const float sign_ax1 = (cle2 & (1 << ax1)) ? 1.0f : -1.0f;
        const float sign_ax2 = (cle2 & (1 << ax2)) ? 1.0f : -1.0f;
        const v3 term_ax1 = v3_scale(v3_scale(rtax1, s2[ax1]), sign_ax1);
        const v3 term_ax2 = v3_scale(v3_scale(rtax2, s2[ax2]), sign_ax2);
        const v3 term_ax1n = v3_scale(v3_scale(rtax1, s2[ax1]), -sign_ax1);
        const v3 edge_term = v3_scale(rt_edge, s2[edge2]);
        points[0] = v3_add(v3_add(p, term_ax1), term_ax2);
        points[1] = v3_sub(points[0], edge_term);
        points[0] = v3_add(points[0], edge_term);
        points[2] = v3_add(v3_add(p, term_ax1n), term_ax2);
        points[3] = v3_sub(points[2], edge_term);
        points[2] = v3_add(points[2], edge_term);
        n = 4;

        const v3 axi_lp = points[0];
        const v3 axi_cn1 = v3_sub(points[1], points[0]);
        const v3 axi_cn2 = v3_sub(points[2], points[0]);
        if (fabsf(rnorm.z) < G5_BB_MINVAL) return result;
        const float inv_sign = inv ? -1.0f : 1.0f;
        const float innorm = inv_sign / rnorm.z;

        v3 pu[4];
        for (int i = 0; i < 4; ++i) {
            pu[i] = points[i];
            const float c_scl = points[i].z * inv_sign * innorm;
            points[i] = v3_sub(points[i], v3_scale(rnorm, c_scl));
        }
        const v3 pts_lp = points[0];
        const v3 pts_cn1 = v3_sub(points[1], points[0]);
        const v3 pts_cn2 = v3_sub(points[2], points[0]);
        n = 0;

        // 盒1边的投影线段与盒2投影面求交（L916-947）。
        for (int i = 0; i < 4; ++i) {
            for (int q = 0; q < 2; ++q) {
                const float la = g5_bb_comp(pts_lp, q) +
                                 (i < 2 ? 0.0f
                                        : (i == 2 ? g5_bb_comp(pts_cn1, q)
                                                  : g5_bb_comp(pts_cn2, q)));
                const float lb = (i == 0 || i == 3) ? g5_bb_comp(pts_cn1, q)
                                                    : g5_bb_comp(pts_cn2, q);
                const float lc = g5_bb_comp(pts_lp, 1 - q) +
                                 (i < 2 ? 0.0f
                                        : (i == 2 ? g5_bb_comp(pts_cn1, 1 - q)
                                                  : g5_bb_comp(pts_cn2, 1 - q)));
                const float ld = (i == 0 || i == 3) ? g5_bb_comp(pts_cn1, 1 - q)
                                                    : g5_bb_comp(pts_cn2, 1 - q);
                const v3 lua = v3_add(axi_lp, i < 2 ? v3_make(0.0f, 0.0f, 0.0f)
                                                    : (i == 2 ? axi_cn1 : axi_cn2));
                const v3 lub = (i == 0 || i == 3) ? axi_cn1 : axi_cn2;
                if (!(fabsf(lb) > G5_BB_MINVAL)) continue;
                const float br = 1.0f / lb;
                for (int jj = -1; jj <= 1; jj += 2) {
                    if (n == 8) break;
                    const float l = g5_bb_comp(sv, q) * (float)jj;
                    const float c1 = (l - la) * br;
                    if (c1 < 0.0f || c1 > 1.0f) continue;
                    const float c2 = lc + ld * c1;
                    if (fabsf(c2) > g5_bb_comp(sv, 1 - q)) continue;
                    if ((lua.z + lub.z * c1) * innorm > margin) continue;
                    points[n] = v3_add(v3_scale(lua, 0.5f),
                                       v3_scale(v3_scale(lub, c1), 0.5f));
                    g5_bb_add_comp(&points[n], q, 0.5f * l);
                    g5_bb_add_comp(&points[n], 1 - q, 0.5f * c2);
                    depth[n] = points[n].z * innorm * 2.0f;
                    ++n;
                }
            }
        }

        // 投影面角点/网格点（L950-990）。
        const int nl = n;
        const float ax = pts_cn1.x, bx = pts_cn2.x;
        const float ay = pts_cn1.y, by = pts_cn2.y;
        const float cdet = g5_bb_safe_div(1.0f, ax * by - bx * ay);
        for (int i = 0; i < 4; ++i) {
            if (n == 8) break;
            const float llx = (i / 2) ? lx : -lx, lly = (i % 2) ? ly : -ly;
            const float x = llx - pts_lp.x, y = lly - pts_lp.y;
            float u = (x * by - y * bx) * cdet, v = (y * ax - x * ay) * cdet;
            if (nl == 0) {
                if ((u < 0.0f || u > 1.0f) && (v < 0.0f || v > 1.0f)) continue;
            } else if (u < 0.0f || v < 0.0f || u > 1.0f || v > 1.0f) continue;
            u = fminf(fmaxf(0.0f, u), 1.0f);
            v = fminf(fmaxf(0.0f, v), 1.0f);
            const float w = 1.0f - u - v;
            const v3 vtmp = v3_add(v3_add(v3_scale(pu[0], w), v3_scale(pu[1], u)),
                                   v3_scale(pu[2], v));
            points[n] = v3_make(llx, lly, 0.0f);
            const v3 vtmp2 = v3_sub(points[n], vtmp);
            const float tc1 = v3_dot(vtmp2, vtmp2);
            if (vtmp.z > 0.0f && tc1 > margin * margin) continue;
            points[n] = v3_scale(v3_add(points[n], vtmp), 0.5f);
            depth[n] = sqrtf(tc1) * (vtmp.z < 0.0f ? -1.0f : 1.0f);
            ++n;
        }

        // 盒2投影面的顶点（L993-1029）。
        const int nf = n;
        for (int i = 0; i < 4; ++i) {
            if (n >= 8) break;
            const float x = g5_bb_comp(pu[i], 0), y = g5_bb_comp(pu[i], 1);
            if (nl == 0 && nf != 0) {
                if ((x < -lx || x > lx) && (y < -ly || y > ly)) continue;
            } else if (x < -lx || x > lx || y < -ly || y > ly) continue;

            float c1 = 0.0f;
            for (int j = 0; j < 2; ++j) {
                const float puj = g5_bb_comp(pu[i], j);
                if (puj < -s_axis[j]) c1 += (puj + s_axis[j]) * (puj + s_axis[j]);
                else if (puj > s_axis[j]) c1 += (puj - s_axis[j]) * (puj - s_axis[j]);
            }
            const float pu2 = g5_bb_comp(pu[i], 2);
            c1 += pu2 * innorm * pu2 * innorm;
            if (pu2 > 0.0f && c1 > margin * margin) continue;

            v3 tmp_p = v3_make(x, y, 0.0f);
            for (int j = 0; j < 2; ++j) {
                const float puj = g5_bb_comp(pu[i], j);
                if (puj < -s_axis[j]) tmp_p = g5_bb_set_comp(tmp_p, j, -s_axis[j] * 0.5f);
                else if (puj > s_axis[j]) tmp_p = g5_bb_set_comp(tmp_p, j, s_axis[j] * 0.5f);
            }
            tmp_p = v3_add(tmp_p, pu[i]);
            points[n] = v3_scale(tmp_p, 0.5f);
            depth[n] = sqrtf(c1) * (pu2 < 0.0f ? -1.0f : 1.0f);
            ++n;
        }

        // 接触面基（L1031-1034）：`(scalar * rw) @ rnorm`，先缩放矩阵再乘向量。
        g5_bb_transpose(rotmore, rwt);
        g5_bb_mat3_mul(rot1, rwt, rw);
        pw = pos1;
        const float nsign = inv ? -1.0f : 1.0f;
        for (int i = 0; i < 9; ++i) rw_scaled[i] = nsign * rw[i];
        normal = mul_mat3_vec(rw_scaled, rnorm);
    }

    // 输出（L1036-1046）：先还原 z，再变换到世界系。
    for (int i = 0; i < n; ++i) {
        points[i].z = points[i].z + hz;
        result.dist[i] = depth[i];
        result.pos[i] = v3_add(mul_mat3_vec(rw, points[i]), pw);
        result.normal[i] = normal;
    }
    return result;
}
