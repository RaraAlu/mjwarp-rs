// S7b2 着色阶段（渲染/BVH S7b2）：逐式翻译冻结上游 `render.py` 的着色/纹理/光照子集。
//
// 上游对应代码：`_geom_rgba` / `_texture_*` / `_make_sample_texture` / `vertex_normal`
// / `_make_compute_lighting`（7 个 primitive 分支里本片实际用到的 5 类）+ `_render_megakernel`
// 的 shading 段（headlight 三组、环境光、逐灯漫反射/高光、自发光、min/max、pack_rgba）。
//
// 精度对齐方法沿用 S7b1：参考 megakernel 由 warp 1.15 自带 NVRTC（CUDA 12.9、sm_89）编译，
// 产品是 CUDA 12.8.1 / compute_70，同一 C++ 表达式的 fma 收缩顺序可能不同。因此涉及
// dot/normalize/atan2/asin/pow 的关键表达式用显式 `__fmul_rn`/`__fmaf_rn`/`__fsub_rn`
// 固定上游顺序（点积见 `dot3_ba`/`dot3_ch0`，normalize 见 `normalize3_rays`），其余交给
// 8 位量化吸收；对比口径是「解包后每通道 ≤1 LSB」，不做任何容差放宽。
//
// 本文件被 `mod.rs::MEGAKERNEL_SOURCE` 接在 `render_common.cu` 之后、`render_megakernel.cu`
// 之前，因此可用 `V3` 辅助、`normalize3_rays`、`dot3_ba`/`dot3_ch0`、`geom_rgba`。

__device__ __forceinline__ float safe_div_render(float x, float y) {
    return x / (y != 0.0f ? y : 1.0e-15f);
}

// 上游 `_texture_plane`。
__device__ __forceinline__ void texture_plane(V3 local, const float* size, const float* repeat,
                                              bool tex_uniform, V3* uv, V3* offset, V3* rep) {
    float u = 0.5f * local.x;
    float rep_u = repeat[0];
    if (size[0] > 0.0f) {
        float s0 = fmaxf(size[0], 1.0e-6f);
        u = (local.x + size[0]) / (2.0f * s0);
        if (tex_uniform) {
            rep_u = repeat[0] * size[0];
        }
    }
    float v = -0.5f * local.y;
    float rep_v = repeat[1];
    if (size[1] > 0.0f) {
        float s1 = fmaxf(size[1], 1.0e-6f);
        v = (size[1] - local.y) / (2.0f * s1);
        if (tex_uniform) {
            rep_v = repeat[1] * size[1];
        }
    }
    float off = (size[0] <= 0.0f || size[1] <= 0.0f) ? -0.5f : 0.0f;
    *uv = v3(u, v, 0.0f);
    *offset = v3(off, off, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_sphere_ellipsoid`。
__device__ __forceinline__ void texture_sphere_ellipsoid(V3 local, const float* size,
                                                          const float* repeat, bool tex_uniform,
                                                          bool is_sphere, V3* uv, V3* offset,
                                                          V3* rep) {
    float rx = fmaxf(is_sphere ? size[0] : size[0], 1.0e-6f);
    float ry = fmaxf(is_sphere ? size[0] : size[1], 1.0e-6f);
    float rz = fmaxf(is_sphere ? size[0] : size[2], 1.0e-6f);
    V3 pt = v3(local.x / rx, local.y / ry, local.z / rz);
    float inv_len = 1.0f / fmaxf(length3(pt), 1.0e-6f);
    V3 d = scale(pt, inv_len);
    float azimuth = atan2f(d.y, d.x);
    if (azimuth < 0.0f) {
        azimuth += 6.283185307179586f;
    }
    float elevation = asinf(fminf(fmaxf(d.z, -1.0f), 1.0f));
    float u = azimuth * (0.5f / 3.141592653589793f);
    float v = 0.5f - elevation * (1.0f / 3.141592653589793f);
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * rx;
        rep_v = repeat[1] * ry;
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_cylinder`。
__device__ __forceinline__ void texture_cylinder(V3 local, V3 local_normal, const float* size,
                                                  const float* repeat, bool tex_uniform, V3* uv,
                                                  V3* offset, V3* rep) {
    float r = fmaxf(size[0], 1.0e-6f);
    float h = fmaxf(size[1], 1.0e-6f);
    float u;
    float v;
    if (fabsf(local_normal.z) > 0.7071f) {
        u = 0.5f + 0.5f * (local.x / r);
        v = 0.5f + 0.5f * (local.y / r);
    } else {
        float azimuth = atan2f(local.y, local.x);
        if (azimuth < 0.0f) {
            azimuth += 6.283185307179586f;
        }
        u = azimuth * (0.5f / 3.141592653589793f);
        v = (h - local.z) / (2.0f * h);
    }
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * size[0];
        rep_v = repeat[1] * size[0];
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_capsule`。
__device__ __forceinline__ void texture_capsule(V3 local, const float* size, const float* repeat,
                                                 bool tex_uniform, V3* uv, V3* offset, V3* rep) {
    float r = fmaxf(size[0], 1.0e-6f);
    float h = fmaxf(size[1], 1.0e-6f);
    float z = local.z;
    float azimuth = atan2f(local.y, local.x);
    if (azimuth < 0.0f) {
        azimuth += 6.283185307179586f;
    }
    float u = azimuth * (0.5f / 3.141592653589793f);
    float v;
    if (z > h) {
        float dz = fminf(fmaxf((z - h) / r, 0.0f), 1.0f);
        v = 1.0f - asinf(dz) * (2.0f / 3.141592653589793f);
    } else if (z < -h) {
        float dz = fminf(fmaxf((-h - z) / r, 0.0f), 1.0f);
        v = asinf(dz) * (2.0f / 3.141592653589793f);
    } else {
        v = (h - z) / (2.0f * h);
    }
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * size[0];
        rep_v = repeat[1] * size[0];
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_box`。
__device__ __forceinline__ void texture_box(V3 local, V3 local_normal, const float* size,
                                             const float* repeat, bool tex_uniform, V3* uv,
                                             V3* offset, V3* rep) {
    float sx = fmaxf(size[0], 1.0e-6f);
    float sy = fmaxf(size[1], 1.0e-6f);
    float sz = fmaxf(size[2], 1.0e-6f);
    float ax = fabsf(local_normal.x);
    float ay = fabsf(local_normal.y);
    float az = fabsf(local_normal.z);
    float u;
    float v;
    if (az >= ax && az >= ay) {
        if (local_normal.z > 0.0f) {
            u = (local.x + sx) / (2.0f * sx);
            v = (sy - local.y) / (2.0f * sy);
        } else {
            u = (local.x + sx) / (2.0f * sx);
            v = (local.y + sy) / (2.0f * sy);
        }
    } else if (ax >= ay && ax >= az) {
        if (local_normal.x > 0.0f) {
            u = (local.y + sy) / (2.0f * sy);
            v = (sz - local.z) / (2.0f * sz);
        } else {
            u = (sy - local.y) / (2.0f * sy);
            v = (sz - local.z) / (2.0f * sz);
        }
    } else if (local_normal.y < 0.0f) {
        u = (local.x + sx) / (2.0f * sx);
        v = (sz - local.z) / (2.0f * sz);
    } else {
        u = (sx - local.x) / (2.0f * sx);
        v = (sz - local.z) / (2.0f * sz);
    }
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * size[0];
        rep_v = repeat[1] * size[1];
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_hfield`（render.py L332-350）：水平范围 `[-size[0], size[0]] x [-size[1], size[1]]`，
// `s0 = max(size[0], 1e-6)`、`s1 = max(size[1], 1e-6)`；UV 不依赖命中面/法线。
__device__ __forceinline__ void texture_hfield(V3 local, const float* size, const float* repeat,
                                                bool tex_uniform, V3* uv, V3* offset, V3* rep) {
    float s0 = fmaxf(size[0], 1.0e-6f);
    float s1 = fmaxf(size[1], 1.0e-6f);
    float u = (local.x + size[0]) / (2.0f * s0);
    float v = (size[1] - local.y) / (2.0f * s1);
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * size[0];
        rep_v = repeat[1] * size[1];
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_mesh`（s7b 场景只有 1 个 mesh 且无 texcoord：走 OBJECT_PLANE fallback）。
__device__ __forceinline__ void texture_mesh(const int* mesh_faceadr, V3 local, const float* size,
                                              const float* repeat, bool tex_uniform, int face,
                                              int mesh_id, float bary_u, float bary_v,
                                              const int* mesh_facetexcoord, const float* mesh_texcoord,
                                              const int* mesh_texcoord_offsets, V3* uv, V3* offset,
                                              V3* rep) {
    bool has_uv = false;
    *uv = v3(0.0f, 0.0f, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(repeat[0], repeat[1], 0.0f);
    if (face >= 0 && mesh_id >= 0) {
        int texcoord_offset = mesh_texcoord_offsets[mesh_id];
        if (texcoord_offset >= 0) {
            int face_adr = mesh_faceadr[mesh_id] + face;
            const int* coords = mesh_facetexcoord + 3 * face_adr;
            V3 uv0 = v3(mesh_texcoord[2 * (texcoord_offset + coords[0]) + 0],
                        mesh_texcoord[2 * (texcoord_offset + coords[0]) + 1], 0.0f);
            V3 uv1 = v3(mesh_texcoord[2 * (texcoord_offset + coords[1]) + 0],
                        mesh_texcoord[2 * (texcoord_offset + coords[1]) + 1], 0.0f);
            V3 uv2 = v3(mesh_texcoord[2 * (texcoord_offset + coords[2]) + 0],
                        mesh_texcoord[2 * (texcoord_offset + coords[2]) + 1], 0.0f);
            float w0 = bary_u;
            float w1 = bary_v;
            float w2 = 1.0f - bary_u - bary_v;
            float u = fmaf(uv0.x, w0, fmaf(uv1.x, w1, uv2.x * w2));
            float v = fmaf(uv0.y, w0, fmaf(uv1.y, w1, uv2.y * w2));
            *uv = v3(u, v, 0.0f);
            has_uv = true;
        }
    }
    if (!has_uv) {
        *uv = v3(0.5f * local.x, -0.5f * local.y, 0.0f);
        *offset = v3(-0.5f, -0.5f, 0.0f);
        float rep_u = repeat[0];
        float rep_v = repeat[1];
        if (size[0] > 0.0f) {
            rep_u = rep_u / size[0];
        }
        if (size[1] > 0.0f) {
            rep_v = rep_v / size[1];
        }
        if (tex_uniform) {
            if (size[0] > 0.0f) {
                rep_u = rep_u * size[0];
            }
            if (size[1] > 0.0f) {
                rep_v = rep_v * size[1];
            }
        }
        *rep = v3(rep_u, rep_v, 0.0f);
    }
}

// 上游 `sample_texture`：按 geom 类型给出 UV，wrap 后采样。
__device__ __forceinline__ V3 sample_texture(const int* geom_type, const int* mesh_faceadr,
                                              int geom_id, const float* size, const float* repeat,
                                              bool tex_uniform, wp::texture2d_t tex,
                                              V3 pos, const float* rot, const int* mesh_facetexcoord,
                                              const float* mesh_texcoord,
                                              const int* mesh_texcoord_offsets, V3 hit_point,
                                              V3 normal, float bary_u, float bary_v, int face,
                                              int mesh_id) {
    V3 uv = v3(0.0f, 0.0f, 0.0f);
    V3 offset = v3(0.0f, 0.0f, 0.0f);
    V3 rep = v3(repeat[0], repeat[1], 0.0f);
    int gtype = geom_type[geom_id];
    V3 local = mat_t_vec(rot, sub(hit_point, pos));
    V3 local_normal = mat_t_vec(rot, normal);
    if (gtype == 0) {
        texture_plane(local, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 2 || gtype == 4) {
        texture_sphere_ellipsoid(local, size, repeat, tex_uniform, gtype == 2, &uv, &offset, &rep);
    } else if (gtype == 5) {
        texture_cylinder(local, local_normal, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 3) {
        texture_capsule(local, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 6) {
        texture_box(local, local_normal, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 1) {
        // R05-S2：hfield 纹理（上游 render.py L462-464 分支）。
        texture_hfield(local, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 7 || gtype == 8) {
        texture_mesh(mesh_faceadr, local, size, repeat, tex_uniform, face, mesh_id, bary_u, bary_v,
                     mesh_facetexcoord, mesh_texcoord, mesh_texcoord_offsets, &uv, &offset, &rep);
    }
    float u = uv.x * rep.x + offset.x;
    float v = uv.y * rep.y + offset.y;
    u = u - floorf(u);
    v = v - floorf(v);
    wp::vec4 tex_color = wp::texture_sample<wp::vec4>(tex, wp::vec2(u, v), 0.0f);
    return v3(tex_color[0], tex_color[1], tex_color[2]);
}

// 上游 `vertex_normal`：顶点法线与面法线夹角超过 ~37° 时用面法线。
__device__ __forceinline__ V3 vertex_normal(V3 n, V3 face) {
    if (dot3_ba(n.x, n.y, n.z, face.x, face.y, face.z) < 0.8f) {
        return face;
    }
    return n;
}

// 上游 `_make_compute_lighting`（本片：R09-S2 起支持 `use_shadows`；逐灯参数与 headlight
// 由调用方给）。`has_spot` 为场景属性：lighttype==0 时走聚光衰减（其余类型不受影响）。
// 阴影：仅 `use_shadows && lightcastshadow` 时经 `shadow_visibility`（R09）发射阴影射线
// （方案 A：复用 first-hit `cast_ray`；上游 L919-961 语义，实现路径不同）。
__device__ __forceinline__ void compute_lighting(const SceneView& scene, const MegakernelBlob& blob,
                                                  const int* geom_type, const int* geom_dataid,
                                                  const int* geom_matid, const float* geom_rgba,
                                                  const float* mat_rgba,
                                                  const wp::uint64* hfield_bvh_id, int worldid,
                                                  bool cull_backfaces, bool lightactive,
                                                  int lighttype, V3 lightpos, V3 lightdir,
                                                  V3 lightattenuation, float lightcutoff_rad,
                                                  float lightexp, V3 lightdiff, V3 lightspec,
                                                  V3 normal, V3 hitpoint, V3 view_dir, float mat_spec,
                                                  float mat_shin_exp, bool enable_specular,
                                                  bool use_shadows, bool cast_shadow,
                                                  float shadow_light_fraction, V3* diff_rgb,
                                                  V3* spec_rgb) {
    *diff_rgb = v3(0.0f, 0.0f, 0.0f);
    *spec_rgb = v3(0.0f, 0.0f, 0.0f);
    if (!lightactive) {
        return;
    }
    V3 L = v3(0.0f, 0.0f, 0.0f);
    float dist_to_light = MJ_MAXVAL;
    float attenuation = 1.0f;
    if (lighttype == 1) {
        L = scale(lightdir, -1.0f);
    } else {
        V3 delta = sub(lightpos, hitpoint);
        float len = length3(delta);
        float norm = len;
        if (norm > 0.0f) {
            L = scale(delta, 1.0f / norm);
        } else {
            L = v3(0.0f, 0.0f, 0.0f);
        }
        dist_to_light = norm;
        V3 factor = v3(1.0f, dist_to_light, dist_to_light * dist_to_light);
        attenuation = safe_div_render(
            1.0f, dot3_ba(factor.x, factor.y, factor.z, lightattenuation.x, lightattenuation.y,
                          lightattenuation.z));
        if (lighttype == 0) {
            float cos_theta = dot3_ba(-L.x, -L.y, -L.z, lightdir.x, lightdir.y, lightdir.z);
            float cos_cutoff = cosf(lightcutoff_rad);
            if (cos_theta < cos_cutoff) {
                return;
            }
            attenuation = attenuation * powf(fmaxf(cos_theta, 0.0f), lightexp);
        }
    }
    float ndotl = fmaxf(0.0f, dot3_ba(normal.x, normal.y, normal.z, L.x, L.y, L.z));
    if (ndotl == 0.0f) {
        return;
    }
    // R09-S2 阴影可见度：仅 `use_shadows && cast_shadow` 时发射阴影射线（上游 L919）；
    // 命中时 `visible = shadow_light_fraction`，否则 1.0。
    float weight = attenuation * shadow_visibility(
                                        scene, blob, geom_type, geom_dataid, geom_matid, geom_rgba,
                                        mat_rgba, hfield_bvh_id, worldid, cull_backfaces, normal,
                                        hitpoint, L, lighttype, dist_to_light, use_shadows,
                                        cast_shadow, shadow_light_fraction);
    *diff_rgb = v3(lightdiff.x * (ndotl * weight), lightdiff.y * (ndotl * weight),
                   lightdiff.z * (ndotl * weight));
    if (enable_specular) {
        if (mat_spec > 0.0f && mat_shin_exp > 0.0f) {
            V3 half = normalize3_rays(add(L, view_dir));
            float ndoth = fmaxf(0.0f, dot3_ba(normal.x, normal.y, normal.z, half.x, half.y, half.z));
            float energy = mat_spec * powf(ndoth, mat_shin_exp) * weight;
            *spec_rgb = v3(lightspec.x * energy, lightspec.y * energy, lightspec.z * energy);
        }
    }
}

// 上游 `pack_rgba_to_uint32`：`int()` 截断（不是四舍五入），通道顺序 ABGR。
__device__ __forceinline__ unsigned int pack_rgba_to_uint32(float r, float g, float b, float a) {
    return ((unsigned int)(int)a << 24) | ((unsigned int)(int)r << 16) | ((unsigned int)(int)g << 8) |
           (unsigned int)(int)b;
}
