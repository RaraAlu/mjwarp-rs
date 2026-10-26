// G10@P5 G5/M2：multicontact 主函数设备版（上游 `multicontact()`）。
// Adapted from frozen MJWarp collision_gjk.py L2216-2451, revision 71da24d. Apache-2.0.
//
// 与上游逐行对应：`witness1[0]=x1`、`witness2[0]=x2` → `_feature_dim` → 面/边法线 →
// `_aligned_faces` / `_aligned_face_edge` → 面恢复（`_box_face`/`_mesh_face`/`_cylinder_face`/
// `_set_edge`）→ `_polygon_clip`。返回 `n, witness1, witness2, dists`。
//
// M2：mesh 分支按上游 L2245-2262（mesh 数组视图）、L2278-2291 / L2297-2310（`_mesh_normals`）、
// L2326-2345 / L2360-2379（`_mesh_edge_normals`）、L2400-2412 / L2422-2434（`_mesh_face`）
// 接入（mesh 助手在 `multicontact_mesh.cu`）。sidecar 缺失（9 数组全空 / `mesh_polyadr < 0`）
// 时回退单接触，与 M0a 的 null sidecar 语义一致。box/cylinder 路径逐字保持不变。
//
// 调用方（`contact.cu`）保证 scratch 容量：`polygon`/`clipped` 各 32 个 v3、`plane_normal` 16、
// `plane_dist` 16、`idx1`/`idx2`/`n1`/`n2`/`endvert` 各 10、`face1`/`face2` 各 16。
// 本函数不做动态分配、不用递归。
#pragma once
#include "multicontact_clip.cu"
#include "multicontact_mesh.cu"
// EPA face 解包（`g5_get_face_verts`）与 polytope 视图同源，直接复用 polytope 片。
#include "polytope.cu"

struct g5_mc_result {
    // 契约上 0..4；本实现对“失败/无多接触”统一回退 n=1（witness 为 x1/x2），故实际返回 1..4。
    // 调用方 `contact.cu` 以 `n >= 1` 判定是否采用本结果。
    int n;
    v3 witness1[4];
    v3 witness2[4];
    float dists[4];
};

__device__ __forceinline__ g5_mc_result g5_multicontact(
    v3* polygon, v3* clipped, v3* plane_normal, float* plane_dist, int* idx1, int* idx2, v3* n1,
    v3* n2, v3* endvert, v3* face1, v3* face2, const v3* epa_vert, const int* epa_vert_index,
    unsigned epa_face, v3 x1, v3 x2, const d_geom& geom1, const d_geom& geom2, int geomtype1,
    int geomtype2) {
    g5_mc_result out;
    out.n = 1;
    for (int i = 0; i < 4; ++i) {
        out.witness1[i] = v3_make(0.0f, 0.0f, 0.0f);
        out.witness2[i] = v3_make(0.0f, 0.0f, 0.0f);
        out.dists[i] = 0.0f;
    }
    out.witness1[0] = x1;
    out.witness2[0] = x2;

    // M2：mesh 参与的 pair 先取 mesh polygon 视图（上游 L2245-2262 的共享数组选择；
    // 单 mesh 组合下等价）。sidecar 未提供（9 数组全空 / mesh_polyadr < 0）时回退单接触，
    // 与 M0a 的 null sidecar 语义一致。
    g5_mc_mesh_view mesh;
    mesh.vert = nullptr;
    mesh.polynormal = nullptr;
    mesh.polyvertadr = nullptr;
    mesh.polyvertnum = nullptr;
    mesh.polyvert = nullptr;
    mesh.polymapadr = nullptr;
    mesh.polymapnum = nullptr;
    mesh.polymap = nullptr;
    mesh.has_poly = false;
    if (geomtype1 == G5_MESH || geomtype2 == G5_MESH) {
        const d_geom& mesh_geom = (geomtype1 == G5_MESH) ? geom1 : geom2;
        mesh = g5_mc_mesh_view_of(mesh_geom);
        if (!mesh.has_poly || mesh_geom.mesh_polyadr < 0) {
            return out;
        }
    }

    int face[3];
    g5_get_face_verts(epa_face, face);

    // get dimensions of features of geoms 1 and 2
    int feature_index1[3];
    int feature_index2[3];
    v3 feature_vertex1[3];
    v3 feature_vertex2[3];
    int nface1 =
        g5_mc_feature_dim(face, epa_vert_index, epa_vert, 0, feature_index1, feature_vertex1);
    int nface2 =
        g5_mc_feature_dim(face, epa_vert_index, epa_vert, 1, feature_index2, feature_vertex2);

    v3 dir = v3_sub(x2, x1);
    v3 dir_neg = v3_neg(dir);

    // get all possible face normals for each geom
    int nnorms1 = 0;
    int nnorms2 = 0;
    if (geomtype1 == G5_BOX) {
        nnorms1 = g5_mc_box_normals(nface1, feature_index1, geom1.rot, dir_neg, n1, idx1);
    } else if (geomtype1 == G5_MESH) {
        nnorms1 = g5_mc_mesh_normals(nface1, feature_index1, geom1.rot, geom1.vertadr,
                                     geom1.mesh_polyadr, mesh, n1, idx1);
    } else if (geomtype1 == G5_CYLINDER) {
        nnorms1 = g5_mc_cylinder_normals(nface1, feature_index1, geom1.rot, n1, idx1);
    }

    if (geomtype2 == G5_BOX) {
        nnorms2 = g5_mc_box_normals(nface2, feature_index2, geom2.rot, dir, n2, idx2);
    } else if (geomtype2 == G5_MESH) {
        nnorms2 = g5_mc_mesh_normals(nface2, feature_index2, geom2.rot, geom2.vertadr,
                                     geom2.mesh_polyadr, mesh, n2, idx2);
    } else if (geomtype2 == G5_CYLINDER) {
        nnorms2 = g5_mc_cylinder_normals(nface2, feature_index2, geom2.rot, n2, idx2);
    }

    // determine if any two face normals match
    int is_edge_contact_geom1 = 0;
    int is_edge_contact_geom2 = 0;
    int res[2];
    int nres = g5_mc_aligned_faces(n1, nnorms1, n2, nnorms2, res);
    if (!nres) {
        // check if edge-face collision
        if (nface1 < 3 && nface1 <= nface2) {
            nnorms1 = 0;
            if (geomtype1 == G5_BOX) {
                nnorms1 = g5_mc_box_edge_normals(nface1, geom1.rot, geom1.pos, geom1.size,
                                                feature_vertex1[0], feature_vertex1[1],
                                                feature_index1[0], n1, endvert);
            } else if (geomtype1 == G5_MESH) {
                nnorms1 = g5_mc_mesh_edge_normals(nface1, geom1.rot, geom1.pos, geom1.vertadr,
                                                  geom1.mesh_polyadr, mesh, feature_vertex1[0],
                                                  feature_vertex1[1], feature_index1[0], n1,
                                                  endvert);
            } else if (geomtype1 == G5_CYLINDER) {
                nnorms1 = g5_mc_cylinder_edge_normals(nface1, geom1.rot, geom1.size,
                                                      feature_vertex1[0], feature_index1[0], n1,
                                                      endvert);
            }
            nres = g5_mc_aligned_face_edge(n1, nnorms1, n2, nnorms2, dir, res);
            if (!nres) {
                return out;
            }
            is_edge_contact_geom1 = 1;
        }
        // check if face-edge collision
        else if (nface2 < 3) {
            nnorms2 = 0;
            if (geomtype2 == G5_BOX) {
                nnorms2 = g5_mc_box_edge_normals(nface2, geom2.rot, geom2.pos, geom2.size,
                                                feature_vertex2[0], feature_vertex2[1],
                                                feature_index2[0], n2, endvert);
            } else if (geomtype2 == G5_MESH) {
                nnorms2 = g5_mc_mesh_edge_normals(nface2, geom2.rot, geom2.pos, geom2.vertadr,
                                                  geom2.mesh_polyadr, mesh, feature_vertex2[0],
                                                  feature_vertex2[1], feature_index2[0], n2,
                                                  endvert);
            } else if (geomtype2 == G5_CYLINDER) {
                nnorms2 = g5_mc_cylinder_edge_normals(nface2, geom2.rot, geom2.size,
                                                      feature_vertex2[0], feature_index2[0], n2,
                                                      endvert);
            }
            nres = g5_mc_aligned_face_edge(n2, nnorms2, n1, nnorms1, dir_neg, res);
            if (!nres) {
                return out;
            }
            is_edge_contact_geom2 = 1;
        } else {
            // no multi-contact
            return out;
        }
    }

    int i = res[0];
    int j = res[1];

    // recover geom1 matching edge or face
    if (is_edge_contact_geom1) {
        nface1 = g5_mc_set_edge(epa_vert, endvert, face[0], i, 0, face1);
    } else {
        int ind = is_edge_contact_geom2 ? idx1[j] : idx1[i];
        if (geomtype1 == G5_BOX) {
            nface1 = g5_mc_box_face(geom1.rot, geom1.pos, geom1.size, ind, face1);
        } else if (geomtype1 == G5_MESH) {
            nface1 = g5_mc_mesh_face(geom1.rot, geom1.pos, geom1.vertadr, geom1.mesh_polyadr, mesh,
                                     ind, face1);
        } else if (geomtype1 == G5_CYLINDER) {
            nface1 = g5_mc_cylinder_face(geom1.rot, geom1.pos, geom1.size, ind, face1);
        }
    }

    // recover geom2 matching edge or face
    if (is_edge_contact_geom2) {
        nface2 = g5_mc_set_edge(epa_vert, endvert, face[0], i, 1, face2);
    } else {
        if (geomtype2 == G5_BOX) {
            nface2 = g5_mc_box_face(geom2.rot, geom2.pos, geom2.size, idx2[j], face2);
        } else if (geomtype2 == G5_MESH) {
            nface2 = g5_mc_mesh_face(geom2.rot, geom2.pos, geom2.vertadr, geom2.mesh_polyadr, mesh,
                                     idx2[j], face2);
        } else if (geomtype2 == G5_CYLINDER) {
            nface2 = g5_mc_cylinder_face(geom2.rot, geom2.pos, geom2.size, idx2[j], face2);
        }
    }

    // M2：mesh 面/边超出 scratch 容量时助手返回 0 → 无法恢复 feature，回退单接触
    // （box/cylinder 助手恒返回 >=1，故对既有路径无行为变化）。
    if (nface1 == 0 || nface2 == 0) {
        return out;
    }

    // 裁剪结果的 witness 先落到本地，便于 `_polygon_clip` 失败时保留 x1/x2 单接触回退。
    v3 clip_w1[4];
    v3 clip_w2[4];
    float clip_d[4];
    int nclip = 0;
    int swapped = 0;

    // face1 is an edge; clip face1 against face2
    if (is_edge_contact_geom1) {
        nclip = g5_mc_polygon_clip(plane_normal, plane_dist, face2, nface2, face1, nface1, n2[j],
                                   v3_neg(n2[j]), polygon, clipped, clip_w1, clip_w2, clip_d);
        // the faces were flipped in calling _polygon_clip so we need to flip them back
        swapped = 1;
    } else if (is_edge_contact_geom2) {
        // face2 is an edge; clip face2 against face1
        nclip = g5_mc_polygon_clip(plane_normal, plane_dist, face1, nface1, face2, nface2, n1[j],
                                   v3_neg(n1[j]), polygon, clipped, clip_w1, clip_w2, clip_d);
    } else {
        // face-face collision
        nclip = g5_mc_polygon_clip(plane_normal, plane_dist, face1, nface1, face2, nface2, n1[i],
                                   n2[j], polygon, clipped, clip_w1, clip_w2, clip_d);
    }

    // 上游调用方在 ncollision < 1 时回退 EPA 单接触（witness 用 x1/x2）。
    if (nclip < 1) {
        return out;
    }

    out.n = nclip;
    for (int k = 0; k < nclip; ++k) {
        out.witness1[k] = swapped ? clip_w2[k] : clip_w1[k];
        out.witness2[k] = swapped ? clip_w1[k] : clip_w2[k];
        out.dists[k] = clip_d[k];
    }
    return out;
}
