// R06-S3a：flex 渲染设备侧（渲染/BVH R06）。
//
// 拼接位置：`src/render/kernel/mod.rs::MEGAKERNEL_SOURCE` 中 `scene.cu` + `render_math.cu`
// 之后、`render_common.cu` 之前——因此可直接复用：
//   - `scene.cu`：`V3` 辅助（`v3`/`add`/`sub`/`scale`/`dot`/`mat_vec`/`mat_t_vec`/`ray_map`）
//     与 `ray_capsule`（1D flex 胶囊求交；上游 `ray.py::ray_capsule` 的逐式翻译）；
//   - `render_math.cu`：`RAY_TOL_REL`/`RAY_TOL_ABS` 等共享常量。
//
// 语义锚点（冻结上游 `71da24d`，逐项对表见证据 README §pre-check）：
//   - `render_util.py` L490-512：1D flex 每条边一个胶囊图元（`edgeid = flex_edgeadr[f] + e`）；
//     2D/3D 每 flex 一个 mesh 图元（`edgeid = -1`）。R06-S3c 起两条路径都在设备侧接通：
//     1D 走本文件的胶囊求交；2D/3D 走 `bvh/flex_scene` 打包的三角 BVH（世界空间、identity
//     位姿、无内部背面剔除），设备条目索引 = `flex_bvh_id[flexid] + worldid`。
//   - `render.py` L762-800：flex 叶命中分派与胶囊几何（`pos = 0.5*(v0+v1)`、
//     `vec = v1-v0`、`quat_z2vec`、`size = (radius, 0.5*|vec|, 0)`、`ray_capsule`）；
//     命中后 `hit_geom_id = -2`、`hit_mesh_id = flexid`；`flex_rgba[flexid][3] == 0` 跳过。
//   - `ray.py` L759-802：flex 已在世界空间，**无 `_ray_map`、无多段 march、无内部背面剔除**；
//     背面剔除由调用方 `cast_ray` 的统一后判据处理（`render.py` L802-806）。
//   - `types.py` L532：`GeomType.FLEX = mjGEOM_FLEX`；`mjtype.h` L292-301：`mjOBJ_FLEX = 9`
//     （seg 编码 `(flexid, 9)` 用）。
//
// 精度：只做 f32 算术；`sinf`/`cosf` 的 f64 归约属渲染门禁精确放行项。`warp_length3`
// 按 NVRTC `fmad=true` 的收缩顺序显式用 `fmaf` 固定（与 `render_common.cu::normalize3_rays`
// 同一口径），避免依赖推断收缩。

// 上游 `types.MJ_MINVAL`（mujoco `mjMINVAL`，`mjtype.h` L27）。
#define FLEX_MINVAL 1.0e-15f

// `warp vec.h length(vec3)`：`sqrt(dot(v,v))`，收缩序显式固定为 fma(z,z,fma(y,y,x*x))。
__device__ __forceinline__ float warp_length3(V3 v) {
    return sqrtf(fmaf(v.z, v.z, fmaf(v.y, v.y, v.x * v.x)));
}

// 上游 `math.py::quat_z2vec`：从 z 轴旋到给定向量的四元数（分量序 x,y,z,w）。
__device__ __forceinline__ void flex_quat_z2vec(V3 vec, float* quat) {
    quat[0] = 0.0f;
    quat[1] = 0.0f;
    quat[2] = 0.0f;
    quat[3] = 1.0f;
    float norm = warp_length3(vec);
    if (norm < FLEX_MINVAL) {
        return;
    }
    V3 unit = v3(vec.x / norm, vec.y / norm, vec.z / norm);
    V3 axis = v3(-unit.y, unit.x, 0.0f);
    float a = warp_length3(axis);
    if (a < FLEX_MINVAL) {
        // 近平行：上游 `math.py::quat_z2vec` 是**整体替换**为单位四元数；反向时 `quat = (1,0,0,0)`，
        // 同向保持初始单位四元数 `(0,0,0,1)`。z2vec 修复（2026-11-11）：必须显式写全四位——
        // 旧版只写 `quat[0] = 1.0f`，遗留初始化的 `quat[3] = 1.0f`，得到 `(1,0,0,1)`（矩阵差 2.0）。
        if (unit.z < 0.0f) {
            quat[0] = 1.0f;
            quat[1] = 0.0f;
            quat[2] = 0.0f;
            quat[3] = 0.0f;
        }
        return;
    }
    axis = v3(axis.x / a, axis.y / a, axis.z / a);
    float angle = atan2f(a, unit.z);
    float half = angle * 0.5f;
    float s = sinf(half);
    float c = cosf(half);
    // 上游 `axis_angle_to_quat`：`quat(c, axis*s)`。
    quat[0] = c;
    quat[1] = axis.x * s;
    quat[2] = axis.y * s;
    quat[3] = axis.z * s;
}

// 上游 `math.py::quat_to_mat`：四元数 → 行主序 3x3（与 `geom_xmat` / `ray_map` 同口径）。
__device__ __forceinline__ void flex_quat_to_mat(const float* quat, float* mat) {
    float x = quat[0];
    float y = quat[1];
    float z = quat[2];
    float w = quat[3];
    float q00 = x * x;
    float q01 = x * y;
    float q02 = x * z;
    float q03 = x * w;
    float q11 = y * y;
    float q12 = y * z;
    float q13 = y * w;
    float q22 = z * z;
    float q23 = z * w;
    float q33 = w * w;
    mat[0] = q00 + q11 - q22 - q33;
    mat[1] = 2.0f * (q12 - q03);
    mat[2] = 2.0f * (q13 + q02);
    mat[3] = 2.0f * (q12 + q03);
    mat[4] = q00 - q11 + q22 - q33;
    mat[5] = 2.0f * (q23 - q01);
    mat[6] = 2.0f * (q13 - q02);
    mat[7] = 2.0f * (q23 + q01);
    mat[8] = q00 - q11 - q22 + q33;
}

// flex 通道的设备视图（81 槽中 9 个 flex 形参的裸指针视图；`xpos` 是 2-D 展平）。
//
// 注意：本文件在 `MEGAKERNEL_SOURCE` 里位于 `render_common.cu`（其 `#include "builtin.h"`
// 引入 `wp::` 命名空间）**之前**，所以这里只用 `scene.cu` 的 `V3` 与裸指针，不用 `wp::` 类型；
// `render_megakernel.cu` 负责把 `wp::array_t` 的 `.data` 转成这些裸指针。
struct FlexView {
    const int* vertadr;
    const int* edge;
    const float* radius;
    const float* xpos;
    const float* rgba;
    const int* geom_flexid;
    const int* geom_edgeid;
    // R06-S3c：2D/3D mesh 条目的 blob 绝对条目基址（`u64::MAX` = 该 flex 走 1D 胶囊）。
    const unsigned long long* bvh_id;
    int nflexvert;
};

// 单个 flex 图元叶的求交（上游 `render.py` L762-800 的 flex 分支）。
//
// `flex_local` 是该叶在 `bvh_nflexgeom` 里的局部号（scene 叶的 `leaf_geom = -1 - flex_local`）。
// 命中返回 true 并填 `out_dist`/`out_normal`/`out_mesh_id`（= flexid，着色与 seg 用）；
// miss / 透明（`rgba[3] == 0`）返回 false。2D/3D mesh 图元（`edgeid < 0`）由
// `render_common.cu::flex_mesh_leaf_hit` 走三角 BVH 路径（本函数只服务 1D 胶囊叶）。
__device__ __forceinline__ bool flex_leaf_hit(const FlexView& flex, int worldid, int flex_local,
                                              V3 origin, V3 direction, float* out_dist,
                                              V3* out_normal, int* out_mesh_id) {
    int flexid = flex.geom_flexid[flex_local];
    if (flexid < 0) {
        return false;
    }
    // 上游 L766-767：透明 flex（alpha == 0）跳过（胶囊与 BVH 两路都跳）。
    if (flex.rgba[4 * flexid + 3] == 0.0f) {
        return false;
    }
    int edge_id = flex.geom_edgeid[flex_local];
    if (edge_id < 0) {
        return false;
    }
    int vert_adr = flex.vertadr[flexid];
    int local0 = flex.edge[2 * edge_id];
    int local1 = flex.edge[2 * edge_id + 1];
    const float* verts = flex.xpos + (long)worldid * (long)flex.nflexvert * 3;
    const float* first = verts + 3 * (long)(vert_adr + local0);
    const float* second = verts + 3 * (long)(vert_adr + local1);
    V3 v0 = v3(first[0], first[1], first[2]);
    V3 v1 = v3(second[0], second[1], second[2]);
    // 上游 L775-781：胶囊中心/轴向/长度/半径（`0.5 * (v0 + v1)` 与 `0.5 * length`）。
    V3 pos = scale(add(v0, v1), 0.5f);
    V3 vec = sub(v1, v0);
    float length = warp_length3(vec);
    float quat[4];
    flex_quat_z2vec(vec, quat);
    float mat[9];
    flex_quat_to_mat(quat, mat);
    float size[3] = {flex.radius[flexid], 0.5f * length, 0.0f};
    float dist = -1.0f;
    V3 normal = v3(0.0f, 0.0f, 0.0f);
    ray_capsule(pos, mat, size, origin, direction, &dist, &normal);
    if (dist < 0.0f) {
        return false;
    }
    *out_dist = dist;
    *out_normal = normal;
    *out_mesh_id = flexid;
    return true;
}
