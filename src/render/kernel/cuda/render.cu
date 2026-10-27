// 渲染/BVH S7b1 渲染内核：主射线阶段（平面深度 + 分割）。
//
// 源码由 `src/render/kernel/cast.rs::DEPTH_SEG_SOURCE` 用 Rust `concat!` 拼成三段：
//   1) S2b `src/render/bvh/cuda/scene.cu`：`V3` 辅助、`_ray_map`/`_ray_quad` 与六个
//      primitive 求交逐字复用（该文件只读、未修改）；其中同名内核 `scene_bvh_ray`
//      出现在本编译单元但不会被发射；
//   2) `cuda/render_math.cu`：S3 mesh woop 数学逐式复制；
//   3) 本文件：相机系→世界系、去畸变射线重建（dynamic）、场景+网格组合、最近命中
//      tie-break、深度/分割写入。
//
// ABI（`RenderSignature` 槽位见 `kernel/mod.rs::DEPTH_SEG_SLOTS`，按值布局由
// `RenderArrayValue`/`RenderScalar` 校验；首槽位 `launch_bounds_t<2>` 形状
// `[nworld, total_rays]`，`index = worldid * total_rays + rayid`）：
//
//   extern "C" __global__ void render_depth_seg(
//       wp::launch_bounds_t<2> dim,
//       wp::array_t<wp::int32> scene_meta,
//       wp::array_t<float> scene_params,
//       wp::array_t<wp::int32> mesh_meta,
//       wp::array_t<float> mesh_params,
//       wp::array_t<wp::vec2i> mesh_offsets,
//       wp::array_t<wp::int32> geom_type,
//       wp::array_t<wp::vec4> geom_rgba,
//       wp::array_t<wp::int32> geom_matid,
//       wp::array_t<wp::vec4> mat_rgba,
//       wp::array_t<wp::int32> cam_projection,
//       wp::array_t<float> cam_fovy,
//       wp::array_t<wp::vec2> cam_sensorsize,
//       wp::array_t<wp::vec4> cam_intrinsic,
//       wp::array_t<wp::vec3> cam_xpos,
//       wp::array_t<wp::mat33> cam_xmat,
//       wp::array_t<wp::vec2i> cam_res,
//       wp::array_t<wp::int32> depth_adr,
//       wp::array_t<wp::int32> seg_adr,
//       wp::array_t<bool> render_depth,
//       wp::array_t<bool> render_seg,
//       wp::array_t<wp::vec3> ray,
//       wp::array_t<wp::vec3> ray_offset,
//       wp::array_t<float> depth_out,
//       wp::array_t<wp::vec2i> seg_out,
//       wp::int32 nrender, wp::int32 znear_bits, wp::int32 zfar_bits,
//       wp::int32 depth_stride, wp::int32 seg_stride,
//       bool use_precomputed_rays, bool cull_backfaces)
//
// 语义口径（冻结上游 `render.py::_render_megakernel` 71da24d，本片只覆盖主射线/
// 深度/分割）：
//   - 射线：`use_precomputed_rays=true` 读 S5 缓冲；`false` 用 `compute_ray` 逐像素重算
//     （与 S5 `build_rays.cu` 同一份 f32 表达式，spp=1 时逐位等价）。
//   - 世界系：`ray_dir_world = cam_xmat @ ray_dir_local_cam`（行点积、不重归一化）；
//     `ray_origin_world = cam_xpos + cam_xmat @ ray_offset`（透视 offset 恒 0，加法位等；
//     正交相机由 S5 给出 offset，与上游 `has_orthographic_camera` 分支同式）。
//   - `max_cam_dist = zfar / max(-ray_dir_local_cam.z, 1e-6)`（`zfar > 0`），否则 MJ_MAXVAL。
//   - 命中：primitive 复用 S2b 数学（**球面例外**：`render_math.cu::render_sphere_hit` 用
//     显式 fma 序列对齐冻结上游 megakernel 的 d0/det 运算顺序，见该节注释）；mesh 复用 S3
//     数学（局部系 woop + 8 次背面剔除步进）；
//     查询上界 `dist*(1+RAY_TOL_REL)+RAY_TOL_ABS`、透明 geom（`rgba[3]==0`）跳过、
//     背面剔除 `dot(dir, n) > 0` 置 miss、近平局取更大 geom id。
//   - 深度：平面深度 `dist * -ray_dir_local_cam.z`；无命中/`zfar` 外写 0。
//   - 分割：命中写 `(geom_id, 5)`（mjOBJ_GEOM）；无命中原样写 `(-1, -1)`，等价上游
//     `render()` 先 `seg_data.fill_(-1,-1)` 再只写命中（S7b1 每次渲染都显式写背景）。
//   - 本片不含：RGB/着色、AA（nsamples=1、ray_base=0）、阴影/天空盒/splat/flex/hfield。
//   - znear/zfar 以 f32 位型经 `int32` 标量传入：渲染 ABI 的按值标量只有
//     int32/uint64/bool（上游这两个量是 `wp.static` 编译期常量，没有浮点标量位）。
#define WP_NO_CRT
#define WP_NO_BFLOAT16
#include "builtin.h"

// f32(pi / 180.0) exactly as Warp folds `wp.static(wp.pi / 180.0)` (S5 build_rays.cu)。
// 名字加 RENDER_ 前缀：W15 `builtin.h` 已定义同名 `DEG_TO_RAD` 宏。
#define RENDER_DEG_TO_RAD 0.017453292519943295f
// MuJoCo `mjMAXVAL`（上游 `float(MJ_MAXVAL)`）。
#define MJ_MAXVAL 1.0e10f

// warp vec.h normalize(vec3) as contracted by the frozen reference compiler：组件平方和
// 发射为 fma(z,z,fma(y,y,x*x))；显式 fmaf 链让舍入与局部 NVRTC 收缩选择无关
// （逐字取自 S5 `build_rays.cu`，保证 dynamic 与 precomputed 射线逐位一致）。
__device__ __forceinline__ V3 normalize3_rays(V3 a) {
    float length = sqrtf(fmaf(a.z, a.z, fmaf(a.y, a.y, a.x * a.x)));
    if (length > 0.0f) {
        return v3(a.x / length, a.y / length, a.z / length);
    }
    return v3(0.0f, 0.0f, 0.0f);
}

// compute_ray(projection, fovy, sensorsize, intrinsic, img_w, img_h, px, py, znear).
// 逐字复制 S5 `camera/cuda/build_rays.cu`（上游 `render_util.py::compute_ray` 的 f32 翻译）。
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
// 场景 BVH 设备布局（`cast.rs` 打包；与 S2b `scene.cu` 的 meta/params 同式，另加
// world 根与 leaf→网格绑定；不再单独存 leaf geom 类型，按上游从 `geom_type[geom_id]` 取）：
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

// 网格三角 BVH 设备布局（`render_math.cu::mesh_query_ray` 消费；`mesh_offsets` 指向
// 每个网格的 (meta 基址, params 基址)）：
//   meta   = [n_nodes, n_tri, n_vert, root, links(2*n_nodes), order(n_tri), indices(3*n_tri)]
//   params = [node bounds 6*n_nodes, vertices 3*n_vert]
// 世界位姿来自场景 leaf（同 S3 `MeshRayGeometry` 的 geom 位姿）。
__device__ __forceinline__ float ray_mesh_leaf(const int* mesh_meta, const float* mesh_params,
                                               const wp::vec2i* mesh_offsets, int mesh_index, V3 pos,
                                               const float* mat, V3 pnt, V3 vec, float max_t,
                                               bool cull_backfaces, V3* normal) {
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
            break;
        }
        offset += t * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
    }
    return distance;
}

// 上游 `_geom_rgba`：geom_rgba 为默认 (0.5,0.5,0.5,1) 且 matid>=0 时改用 mat_rgba。
__device__ __forceinline__ float geom_alpha(const int* geom_matid, const float* geom_rgba,
                                            const float* mat_rgba, int geom_id) {
    float r = geom_rgba[4 * geom_id + 0];
    float g = geom_rgba[4 * geom_id + 1];
    float b = geom_rgba[4 * geom_id + 2];
    float a = geom_rgba[4 * geom_id + 3];
    int mat_id = geom_matid[geom_id];
    if (mat_id >= 0 && r == 0.5f && g == 0.5f && b == 0.5f && a == 1.0f) {
        a = mat_rgba[4 * mat_id + 3];
    }
    return a;
}

// 候选节点最小堆：Warp `wp.bvh_query_ray` 按 AABB 进入距离升序给出候选叶子，上游
// tie-break（`RAY_TOL_REL` 近平局取更大 geom id）依赖该顺序，故这里按 entry 升序
// 出堆复刻候选顺序；S2b 的栈序不满足该语义。
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

// 上游 `cast_ray` 的最近命中子集：候选 leaf 按进入距离升序；透明 geom 跳过；primitive 与
// mesh 进入同一比较；背面剔除后按 `RAY_TOL_REL` 容差 tie-break（近平局取更大 geom id）。
__device__ __forceinline__ void cast_ray(const SceneView& scene, const int* mesh_meta,
                                         const float* mesh_params, const wp::vec2i* mesh_offsets,
                                         const int* geom_type, const int* geom_matid,
                                         const float* geom_rgba, const float* mat_rgba, int worldid,
                                         V3 origin, V3 direction, float max_dist, bool cull_backfaces,
                                         int* out_geom_id, float* out_dist) {
    float dist = max_dist;
    int geom_id = -1;
    float initial_bound = dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
    int root = scene.roots[worldid];
    NodeCandidate heap[MAX_CANDIDATES];
    int count = 0;
    float entry = 0.0f;
    if (!ray_aabb(origin, direction, scene.node_bounds + 6 * root, initial_bound, &entry)) {
        *out_geom_id = -1;
        *out_dist = dist;
        return;
    }
    push_candidate(heap, &count, root, entry);
    while (count > 0) {
        NodeCandidate candidate = pop_candidate(heap, &count);
        float bound = dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
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
        float query_dist = dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
        float d = -1.0f;
        V3 n = v3(0.0f, 0.0f, 0.0f);
        V3 pos = v3(leaf_pos[0], leaf_pos[1], leaf_pos[2]);
        if (gtype == 7 || gtype == 8) {
            // S7b1 只实现 mesh（type 7）；SDF（type 8）不在本片范围，按 miss 处理
            // （scene-a 无 SDF geom，参考不可观测）。
            if (gtype == 7) {
                d = ray_mesh_leaf(mesh_meta, mesh_params, mesh_offsets, scene.leaf_mesh[leaf], pos,
                                  leaf_mat, origin, direction, query_dist, cull_backfaces, &n);
            }
        } else {
            render_intersect_primitive(gtype, pos, leaf_size, leaf_mat, origin, direction, &d, &n);
        }
        if (cull_backfaces && d >= 0.0f && dot(direction, n) > 0.0f) {
            d = -1.0f;
        }
        float tol = RAY_TOL_REL * (dist > 1.0f ? dist : 1.0f);
        bool closer = (d < dist - tol) || (fabsf(d - dist) <= tol && (geom_id < 0 || geom > geom_id));
        if (d >= 0.0f && closer) {
            dist = d;
            geom_id = geom;
        }
    }
    *out_geom_id = geom_id;
    *out_dist = dist;
}

extern "C" __global__ void render_depth_seg(
    wp::launch_bounds_t<2> dim, wp::array_t<wp::int32> scene_meta, wp::array_t<float> scene_params,
    wp::array_t<wp::int32> mesh_meta, wp::array_t<float> mesh_params, wp::array_t<wp::vec2i> mesh_offsets,
    wp::array_t<wp::int32> geom_type, wp::array_t<wp::vec4> geom_rgba, wp::array_t<wp::int32> geom_matid,
    wp::array_t<wp::vec4> mat_rgba, wp::array_t<wp::int32> cam_projection, wp::array_t<float> cam_fovy,
    wp::array_t<wp::vec2> cam_sensorsize, wp::array_t<wp::vec4> cam_intrinsic,
    wp::array_t<wp::vec3> cam_xpos, wp::array_t<wp::mat33> cam_xmat, wp::array_t<wp::vec2i> cam_res,
    wp::array_t<wp::int32> depth_adr, wp::array_t<wp::int32> seg_adr,
    wp::array_t<bool> render_depth, wp::array_t<bool> render_seg,
    wp::array_t<wp::vec3> ray, wp::array_t<wp::vec3> ray_offset, wp::array_t<float> depth_out,
    wp::array_t<wp::vec2i> seg_out, wp::int32 nrender, wp::int32 znear_bits, wp::int32 zfar_bits,
    wp::int32 depth_stride, wp::int32 seg_stride, bool use_precomputed_rays, bool cull_backfaces) {
    size_t index = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= dim.size) {
        return;
    }
    int worldid = 0;
    int rayid = 0;
    wp::tid(worldid, rayid, index, dim);
    if (worldid >= dim.shape[0]) {
        return;
    }

    const int* modes = cam_projection;
    const float* fovys = cam_fovy;
    // `array_t<vec2/vec4/vec3/mat33>` 只能转成元素指针；几何类数组按原始 f32 视图读取
    // （本机序与 Warp 设备布局一致，S7a ABI 证据）。
    const float* sensors = (const float*)cam_sensorsize.data;
    const float* intrinsics = (const float*)cam_intrinsic.data;
    const wp::vec2i* resolutions = cam_res;

    int camid = -1;
    int rayid_local = -1;
    int accum = 0;
    for (int camera = 0; camera < nrender; camera++) {
        int num_i = resolutions[camera][0] * resolutions[camera][1];
        if (rayid < accum + num_i) {
            camid = camera;
            rayid_local = rayid - accum;
            break;
        }
        accum += num_i;
    }
    if (camid == -1 || rayid_local < 0) {
        return;
    }
    const bool* depth_flags = render_depth;
    const bool* seg_flags = render_seg;
    if (!depth_flags[camid] && !seg_flags[camid]) {
        return;
    }
    const float znear = __int_as_float(znear_bits);
    const float zfar = __int_as_float(zfar_bits);
    V3 ray_dir_local_cam;
    V3 ray_offset_local_cam;
    const float* directions = (const float*)ray.data;
    const float* offsets = (const float*)ray_offset.data;
    if (use_precomputed_rays) {
        ray_dir_local_cam = v3(directions[3 * rayid + 0], directions[3 * rayid + 1],
                               directions[3 * rayid + 2]);
        ray_offset_local_cam =
            v3(offsets[3 * rayid + 0], offsets[3 * rayid + 1], offsets[3 * rayid + 2]);
    } else {
        int img_w = resolutions[camid][0];
        int img_h = resolutions[camid][1];
        int px = rayid_local % img_w;
        int py = rayid_local / img_w;
        compute_ray(modes[camid], fovys[camid], sensors[2 * camid], sensors[2 * camid + 1],
                    intrinsics + 4 * camid, img_w, img_h, px, py, znear, &ray_dir_local_cam,
                    &ray_offset_local_cam);
    }

    const float* cam_positions = (const float*)cam_xpos.data;
    const float* cam_matrices = (const float*)cam_xmat.data; int cam_slot = worldid * (int)cam_xpos.shape.dims[1] + camid;
    V3 cam_pos = v3(cam_positions[3 * cam_slot + 0], cam_positions[3 * cam_slot + 1],
                    cam_positions[3 * cam_slot + 2]);
    const float* cam_mat = cam_matrices + 9 * cam_slot;
    V3 ray_origin_world = add(cam_pos, mat_vec(cam_mat, ray_offset_local_cam));
    V3 ray_dir_world = mat_vec(cam_mat, ray_dir_local_cam);

    float max_cam_dist = MJ_MAXVAL;
    if (zfar > 0.0f) {
        max_cam_dist = zfar / fmaxf(-ray_dir_local_cam.z, 1.0e-6f);
    }

    SceneView scene = scene_view(scene_meta, scene_params);
    int geom_id = -1;
    float dist = max_cam_dist;
    cast_ray(scene, mesh_meta, mesh_params, mesh_offsets, (const int*)geom_type.data,
             (const int*)geom_matid.data, (const float*)geom_rgba.data, (const float*)mat_rgba.data,
             worldid, ray_origin_world, ray_dir_world, max_cam_dist, cull_backfaces, &geom_id, &dist);

    if (geom_id != -1 && zfar > 0.0f && (dist * -ray_dir_local_cam.z) > zfar) {
        geom_id = -1;
    }

    if (depth_flags[camid]) {
        int depth_index = depth_adr[camid] + rayid_local;
        float depth = 0.0f;
        if (geom_id != -1) {
            depth = dist * -ray_dir_local_cam.z;
        }
        float* depth_pixels = depth_out;
        depth_pixels[((long)worldid) * depth_stride + depth_index] = depth;
    }
    if (seg_flags[camid]) {
        int seg_index = seg_adr[camid] + rayid_local;
        wp::vec2i value;
        if (geom_id != -1) {
            value[0] = geom_id;
            value[1] = 5;  // mjOBJ_GEOM
        } else {
            value[0] = -1;
            value[1] = -1;
        }
        wp::vec2i* seg_pixels = seg_out;
        seg_pixels[((long)worldid) * seg_stride + seg_index] = value;
    }
}
