// S7b2 全签名渲染内核的公共部分（渲染/BVH S7b2）。
//
// 本文件被 `src/render/kernel/mod.rs::MEGAKERNEL_SOURCE` 用 `include_str!` 接在
// S2b `scene.cu` 与 `cuda/render_math.cu` 之后、`cuda/render_shade.cu` /
// `cuda/render_megakernel.cu` 之前。因此可以直接复用：
//   - `scene.cu`：`V3` 辅助、`ray_quad`/primitive 求交、`mat_vec`/`mat_t_vec`/`ray_map`；
//   - `render_math.cu`：mesh woop 数学、`RAY_TOL_REL`/`RAY_TOL_ABS`、球面运算顺序对齐实现
//     （`render_sphere_hit` / `render_intersect_primitive`）。
//
// 与 S7b1 `render_depth_seg` 的差异只在“数据怎么来”：S7b2 用 81 槽上游签名，自有 BVH
// 打包通过 `bvh_id`（u64 标量）以设备指针传入（见 `MegakernelBlob`），不消费 Warp BVH
// 句柄；其余语义与 S7b1 一致（S7b1 的 depth/seg 位等是 S7b2 的回归门）。

#define WP_NO_CRT
#define WP_NO_BFLOAT16
#include "builtin.h"

#define RENDER_DEG_TO_RAD 0.017453292519943295f
#define MJ_MAXVAL 1.0e10f

// warp vec.h normalize(vec3)：组件平方和按 fma(z,z,fma(y,y,x*x)) 收缩（S5 已验证位等）。
__device__ __forceinline__ V3 normalize3_rays(V3 a) {
    float length = sqrtf(fmaf(a.z, a.z, fmaf(a.y, a.y, a.x * a.x)));
    if (length > 0.0f) {
        return v3(a.x / length, a.y / length, a.z / length);
    }
    return v3(0.0f, 0.0f, 0.0f);
}

// `render_util.py::compute_ray` 的 f32 翻译（与 S5 `build_rays.cu` 逐字一致）。
__device__ __forceinline__ void compute_ray(int projection, float fovy, float sensor_size_x,
                                            float sensor_size_y, const float* intrinsic, int img_w,
                                            int img_h, int px, int py, float znear, V3* direction,
                                            V3* offset) {
    float inv_img_h = 1.0f / (float)img_h;
    if (projection == 1) {
        *direction = v3(0.0f, 0.0f, -1.0f);
        float aspect = (float)img_w * inv_img_h;
        float sensor_h = fovy;
        float sensor_w = sensor_h * aspect;
        float left = -0.5f * sensor_w;
        float top = 0.5f * sensor_h;
        float bottom = -top;
        float u = ((float)px + 0.5f) / (float)img_w;
        float v = ((float)py + 0.5f) * inv_img_h;
        float x = fmaf(sensor_w, u, left);
        float y = fmaf(bottom - top, v, top);
        *offset = v3(x, y, 0.0f);
        return;
    }
    float aspect = (float)img_w * inv_img_h;
    float sensor_h = sensor_size_y;
    float left;
    float right;
    float top;
    float bottom;
    if (sensor_h != 0.0f) {
        float fx = intrinsic[0];
        float fy = intrinsic[1];
        float cx = intrinsic[2];
        float cy = intrinsic[3];
        float sensor_w = sensor_size_x;
        float target_aspect = aspect;
        float sensor_aspect = sensor_w / sensor_h;
        if (target_aspect > sensor_aspect) {
            sensor_h = sensor_w / target_aspect;
        } else if (target_aspect < sensor_aspect) {
            sensor_w = sensor_h * target_aspect;
        }
        float inv_fx_znear = znear / fx;
        float inv_fy_znear = znear / fy;
        float half_sensor_w = 0.5f * sensor_w;
        float half_sensor_h = 0.5f * sensor_h;
        left = -inv_fx_znear * (half_sensor_w - cx);
        right = inv_fx_znear * (half_sensor_w + cx);
        top = inv_fy_znear * (half_sensor_h - cy);
        bottom = -inv_fy_znear * (half_sensor_h + cy);
    } else {
        float fovy_rad = fovy * RENDER_DEG_TO_RAD;
        float half_height = znear * tanf(0.5f * fovy_rad);
        float half_width = half_height * aspect;
        left = -half_width;
        right = half_width;
        top = half_height;
        bottom = -half_height;
    }
    float u = ((float)px + 0.5f) / (float)img_w;
    float v = ((float)py + 0.5f) * inv_img_h;
    float x = fmaf(right - left, u, left);
    float y = fmaf(bottom - top, v, top);
    *direction = normalize3_rays(v3(x, y, -znear));
    *offset = v3(0.0f, 0.0f, 0.0f);
}

// warp `wp::vec3` → 本编译单元的 `V3`（81 槽签名里的 vec3 数组是 warp 类型）。
__device__ __forceinline__ V3 to_v3(wp::vec3 v) { return v3(v[0], v[1], v[2]); }
// 自有 BVH/网格打包 blob（host `MegakernelArrays::pack_blob` 按同一布局写入）：
//   header(u32 × 10)：五段的字节偏移/字节长度对（scene_meta、scene_params、
//                     mesh_meta、mesh_params、mesh_offsets）
//   随后是各段数据（4 字节对齐）。`bvh_id` 标量即该 blob 的设备地址。
// 布局与 S7b1 `SceneDevice`/`MeshDevice` 的 meta/params 完全一致（见 `cast.rs`）。
struct MegakernelBlob {
    const int* scene_meta;
    const float* scene_params;
    const int* mesh_meta;
    const float* mesh_params;
    const wp::vec2i* mesh_offsets;
};

__device__ __forceinline__ MegakernelBlob megakernel_blob(wp::uint64 bvh_id) {
    const unsigned int* header = (const unsigned int*)(size_t)bvh_id;
    const unsigned char* base = (const unsigned char*)bvh_id;
    MegakernelBlob blob;
    blob.scene_meta = (const int*)(base + header[0]);
    blob.scene_params = (const float*)(base + header[2]);
    blob.mesh_meta = (const int*)(base + header[4]);
    blob.mesh_params = (const float*)(base + header[6]);
    blob.mesh_offsets = (const wp::vec2i*)(base + header[8]);
    return blob;
}

// 场景 BVH 设备视图：与 S7b1 `render.cu::SceneView` 同布局。
//   meta   = [n_nodes, n_leaf, n_world, reserved,
//             roots(n_world), links(2*n_nodes), leaf_geom(n_leaf), leaf_mesh(n_leaf)]
//   params = [node bounds 6*n_nodes, leaf size 3*n_leaf, leaf xpos 3*n_leaf,
//             leaf xmat(row-major) 9*n_leaf]
struct SceneView {
    int n_nodes;
    int n_leaf;
    int n_world;
    const int* roots;
    const int* links;
    const int* leaf_geom;
    const int* leaf_mesh;
    const float* node_bounds;
    const float* leaf_size;
    const float* leaf_xpos;
    const float* leaf_xmat;
};

__device__ __forceinline__ SceneView scene_view(const int* meta, const float* params) {
    SceneView view;
    view.n_nodes = meta[0];
    view.n_leaf = meta[1];
    view.n_world = meta[2];
    view.roots = meta + 4;
    view.links = view.roots + view.n_world;
    view.leaf_geom = view.links + 2 * view.n_nodes;
    view.leaf_mesh = view.leaf_geom + view.n_leaf;
    view.node_bounds = params;
    view.leaf_size = view.node_bounds + 6 * view.n_nodes;
    view.leaf_xpos = view.leaf_size + 3 * view.n_leaf;
    view.leaf_xmat = view.leaf_xpos + 3 * view.n_leaf;
    return view;
}

// 网格三角 BVH 设备布局（`render_math.cu::mesh_query_ray` 消费）：
//   meta   = [n_nodes, n_tri, n_vert, root, links(2*n_nodes), order(n_tri), indices(3*n_tri)]
//   params = [node bounds 6*n_nodes, vertices 3*n_vert]
__device__ __forceinline__ float ray_mesh_leaf(const int* mesh_meta, const float* mesh_params,
                                               const wp::vec2i* mesh_offsets, int mesh_index, V3 pos,
                                               const float* mat, V3 pnt, V3 vec, float max_t,
                                               bool cull_backfaces, V3* normal, float* bary_u,
                                               float* bary_v, int* face_out) {
    *normal = v3(0.0f, 0.0f, 0.0f);
    if (mesh_index < 0) {
        return -1.0f;
    }
    const int* meta = mesh_meta + mesh_offsets[mesh_index][0];
    const float* params = mesh_params + mesh_offsets[mesh_index][1];
    const int n_nodes = meta[0];
    const int n_tri = meta[1];
    const int n_vert = meta[2];
    const int root = meta[3];
    const int* links = meta + 4;
    const int* order = links + 2 * n_nodes;
    const int* indices = order + n_tri;
    const float* node_bounds = params;
    const float* vertices = params + 6 * n_nodes;

    V3 lpnt;
    V3 lvec;
    ray_map(pos, mat, pnt, vec, &lpnt, &lvec);
    float distance = -1.0f;
    float offset = 0.0f;
    for (int march = 0; march < CULL_MARCHES; march++) {
        V3 start = add(lpnt, scale(lvec, offset));
        float t = 0.0f;
        float tu = 0.0f;
        float tv = 0.0f;
        float sign = 0.0f;
        V3 n_local = v3(0.0f, 0.0f, 0.0f);
        int face = -1;
        if (!mesh_query_ray(start, lvec, max_t - offset, n_nodes, root, n_vert, links, order, indices,
                            node_bounds, vertices, &t, &tu, &tv, &sign, &n_local, &face)) {
            break;
        }
        if (!cull_backfaces || dot(lvec, n_local) < 0.0f) {
            distance = offset + t;
            *normal = normalize3_mesh(mat_vec(mat, n_local));
            *bary_u = tu;
            *bary_v = tv;
            *face_out = face;
            break;
        }
        offset += t * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
    }
    return distance;
}

// 上游 `_geom_rgba`：geom_rgba 为默认 (0.5,0.5,0.5,1) 且 matid>=0 时改用 mat_rgba。
__device__ __forceinline__ void render_geom_rgba(const int* geom_matid, const float* geom_rgba,
                                          const float* mat_rgba, int geom_id, float* rgba,
                                          int* mat_id) {
    float r = geom_rgba[4 * geom_id + 0];
    float g = geom_rgba[4 * geom_id + 1];
    float b = geom_rgba[4 * geom_id + 2];
    float a = geom_rgba[4 * geom_id + 3];
    int id = geom_matid[geom_id];
    *mat_id = id;
    if (id >= 0 && r == 0.5f && g == 0.5f && b == 0.5f && a == 1.0f) {
        rgba[0] = mat_rgba[4 * id + 0];
        rgba[1] = mat_rgba[4 * id + 1];
        rgba[2] = mat_rgba[4 * id + 2];
        rgba[3] = mat_rgba[4 * id + 3];
        return;
    }
    rgba[0] = r;
    rgba[1] = g;
    rgba[2] = b;
    rgba[3] = a;
}

__device__ __forceinline__ float geom_alpha(const int* geom_matid, const float* geom_rgba,
                                            const float* mat_rgba, int geom_id) {
    float rgba[4];
    int mat_id = -1;
    render_geom_rgba(geom_matid, geom_rgba, mat_rgba, geom_id, rgba, &mat_id);
    return rgba[3];
}

// 候选节点最小堆：按 AABB 进入距离升序给出候选叶子（与 S7b1 `render.cu` 同语义）。
struct NodeCandidate {
    int node;
    float entry;
};

#define MAX_CANDIDATES 128

__device__ __forceinline__ void push_candidate(NodeCandidate* heap, int* count, int node, float entry) {
    if (*count >= MAX_CANDIDATES) {
        return;
    }
    int index = (*count)++;
    while (index > 0) {
        int parent = (index - 1) / 2;
        if (heap[parent].entry <= entry) {
            break;
        }
        heap[index] = heap[parent];
        index = parent;
    }
    heap[index].node = node;
    heap[index].entry = entry;
}

__device__ __forceinline__ NodeCandidate pop_candidate(NodeCandidate* heap, int* count) {
    NodeCandidate top = heap[0];
    (*count)--;
    if (*count > 0) {
        NodeCandidate last = heap[*count];
        int index = 0;
        while (true) {
            int left = 2 * index + 1;
            if (left >= *count) {
                break;
            }
            int right = left + 1;
            int child = (right < *count && heap[right].entry < heap[left].entry) ? right : left;
            if (heap[child].entry >= last.entry) {
                break;
            }
            heap[index] = heap[child];
            index = child;
        }
        heap[index] = last;
    }
    return top;
}

// 上游 `cast_ray`（closest-hit）的 S7b2 子集：候选 leaf 按进入距离升序；透明 geom 跳过；
// primitive 与 mesh 同池；背面剔除后按 `RAY_TOL_REL` 近局取更大 geom id。
// 返回 geom_id/dist/normal/face/bary（mesh 贴图需要）。
struct CastHit {
    int geom_id;
    float dist;
    V3 normal;
    int face;
    float bary_u;
    float bary_v;
    int mesh_id;
};

// `shadow_ray`：阴影射线按上游 `cast_ray_first_hit`（any_hit 分支）语义处理背面剔除——
// mesh 命中在 any-hit 下 `n` 为零向量（上游不填 surface data）→ cull 判据不生效；
// primitive 命中仍按 cull 剔除。primary ray 传 false，行为与 S7b1/S7b2 完全一致。
__device__ __forceinline__ CastHit cast_ray(const SceneView& scene, const MegakernelBlob& blob,
                                            const int* geom_type, const int* geom_dataid,
                                            const int* geom_matid, const float* geom_rgba,
                                            const float* mat_rgba, const wp::uint64* hfield_bvh_id,
                                            int worldid, V3 origin, V3 direction, float max_dist,
                                            bool cull_backfaces, bool shadow_ray) {
    CastHit hit;
    hit.geom_id = -1;
    hit.dist = max_dist;
    hit.normal = v3(0.0f, 0.0f, 0.0f);
    hit.face = -1;
    hit.bary_u = 0.0f;
    hit.bary_v = 0.0f;
    hit.mesh_id = -1;
    float initial_bound = hit.dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
    int root = scene.roots[worldid];
    NodeCandidate heap[MAX_CANDIDATES];
    int count = 0;
    float entry = 0.0f;
    if (!ray_aabb(origin, direction, scene.node_bounds + 6 * root, initial_bound, &entry)) {
        return hit;
    }
    push_candidate(heap, &count, root, entry);
    while (count > 0) {
        NodeCandidate candidate = pop_candidate(heap, &count);
        float bound = hit.dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
        if (candidate.entry > bound) {
            break;
        }
        int node = candidate.node;
        int left = scene.links[2 * node];
        if (left >= 0) {
            int right = scene.links[2 * node + 1];
            float child_entry = 0.0f;
            if (ray_aabb(origin, direction, scene.node_bounds + 6 * left, bound, &child_entry)) {
                push_candidate(heap, &count, left, child_entry);
            }
            if (ray_aabb(origin, direction, scene.node_bounds + 6 * right, bound, &child_entry)) {
                push_candidate(heap, &count, right, child_entry);
            }
            continue;
        }
        int leaf = -left - 1;
        int geom = scene.leaf_geom[leaf];
        int gtype = geom_type[geom];
        if (geom_alpha(geom_matid, geom_rgba, mat_rgba, geom) == 0.0f) {
            continue;
        }
        const float* leaf_size = scene.leaf_size + 3 * leaf;
        const float* leaf_pos = scene.leaf_xpos + 3 * leaf;
        const float* leaf_mat = scene.leaf_xmat + 9 * leaf;
        float query_dist = hit.dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
        float d = -1.0f;
        V3 n = v3(0.0f, 0.0f, 0.0f);
        V3 pos = v3(leaf_pos[0], leaf_pos[1], leaf_pos[2]);
        float pass_bary_u = 0.0f;
        float pass_bary_v = 0.0f;
        int pass_face = -1;
        int mesh_index = scene.leaf_mesh[leaf];
        if (gtype == 7 || gtype == 1) {
            // R05-S2：hfield（gtype==1）与 mesh 同走 `ray_mesh_leaf`；hfield 的 mesh 索引
            // 从 `hfield_bvh_id[geom_dataid[geom]]` 取（blob 内索引，追加在既有 mesh 末尾；
            // `u64::MAX` 哨兵=未建/未用 → 跳过）。上游 `render.py` L682-693 同义。
            int query_mesh = mesh_index;
            if (gtype == 1) {
                int hfield_index = geom_dataid[geom];
                query_mesh = -1;
                if (hfield_index >= 0) {
                    wp::uint64 id = hfield_bvh_id[hfield_index];
                    if (id != 0xFFFFFFFFFFFFFFFFull) {
                        query_mesh = (int)id;
                    }
                }
            }
            if (query_mesh >= 0) {
                d = ray_mesh_leaf(blob.mesh_meta, blob.mesh_params, blob.mesh_offsets, query_mesh,
                                  pos, leaf_mat, origin, direction, query_dist,
                                  cull_backfaces && !shadow_ray, &n,
                                  &pass_bary_u, &pass_bary_v, &pass_face);
            }
        } else if (gtype != 8) {
            // gtype==8（SDF）仍由 host 拒绝（`megakernel.rs` 的 `reject_sdf_geometry`）；设备侧保守不判交。
            // 解锁契约（gap 卡 `2026-10-28-render-shadow-sdf-gtype-cull-gap`）：SDF 求交必须走上游
            // any-hit 语义——命中时 d=0、未命中 d=-1、**不填 normal**、且不参与背面剔除（见下方 cull_this）。
            render_intersect_primitive(gtype, pos, leaf_size, leaf_mat, origin, direction, &d, &n);
        }
        // 上游阴影射线走 any-hit 分支（mesh/hfield/SDF 的 n 为零向量 → cull 不生效）；
        // primitive 仍 cull。SDF（gtype==8）按上游 any-hit 语义也不填 normal ⇒ 排除
        // （latent 对齐，见 gap 卡 `2026-10-28-render-shadow-sdf-gtype-cull-gap`；
        // 设备侧 SDF 求交尚未解锁，host 仍拒绝，见 `megakernel.rs` 的 `reject_sdf_geometry`）。
        bool cull_this = cull_backfaces && !(shadow_ray && (gtype == 7 || gtype == 1 || gtype == 8));
        if (cull_this && d >= 0.0f && dot(direction, n) > 0.0f) {
            d = -1.0f;
        }
        float tol = RAY_TOL_REL * (hit.dist > 1.0f ? hit.dist : 1.0f);
        bool closer = (d < hit.dist - tol) || (fabsf(d - hit.dist) <= tol && (hit.geom_id < 0 || geom > hit.geom_id));
        if (d >= 0.0f && closer) {
            hit.dist = d;
            hit.geom_id = geom;
            hit.normal = n;
            hit.face = pass_face;
            hit.bary_u = pass_bary_u;
            hit.bary_v = pass_bary_v;
            hit.mesh_id = gtype == 7 ? mesh_index : -1;
        }
    }
    return hit;
}
