// S7b2 全签名渲染内核（渲染/BVH S7b2）：`_render_megakernel` 的 81 槽上游签名。
//
// 本文件由 `mod.rs::MEGAKERNEL_SOURCE` 接在 `render_common.cu` / `render_shade.cu` 之后，
// 提供唯一入口 `render_megakernel`（1 bounds + 80 形参，顺序与 S7a 冻结的
// `MEGAKERNEL_SLOTS` 逐槽一致；证据 README 给逐槽「填值/real|placeholder」表）。
//
// 语义边界（任务 JSON 已批）：
//   - 内部 BVH 仍是自有打包：场景/网格五段数据经 `bvh_id`（u64 标量）以设备指针传入，
//     不消费 Warp BVH 句柄；`mesh_bvh_id`/`hfield_bvh_id`/`flex_bvh_id` 等槽为 ABI 占位。
//   - 着色子集：headlight 三组、背景色、base color、纹理采样、顶点/面法线、自发光、高光、
//     背面剔除；阴影/天空盒/splat/flex/hfield/SDF 不在本片（`use_shadows=false` 由 host 拒绝）。
//   - 复用 S7b2 自有参数槽（均为上游本片未用的占位槽，证据表标注）：
//       `splat_position` = [headlight_ambient, headlight_diffuse, headlight_specular,
//                           background_color_vec3]（4 × vec3 f32）
//       `splat_scale`    = [znear, zfar, unused]
//       `splat_bvh_id`   = (packed_background_u32 << 32) | flags
//     flags 位见 `MK_FLAG_*`。
//   - 光线固定读 S5 预计算缓冲（`use_precomputed_rays=true`；动态路径由 host 拒绝）。
//   - 深度/分割语义与 S7b1 `render_depth_seg` 完全一致（S7b1 的 120,000/120,000 位等是回归门）。

#define MK_FLAG_TEXTURES 0x1u
#define MK_FLAG_VERTEX_NORMALS 0x2u
#define MK_FLAG_SPECULAR 0x4u
#define MK_FLAG_EMISSION 0x8u
#define MK_FLAG_PER_LIGHT_AMBIENT 0x10u
#define MK_FLAG_AMBIENT_LIGHTING 0x20u
#define MK_FLAG_HEADLIGHT 0x40u
#define MK_FLAG_CULL_BACKFACES 0x80u

#define MK_MAX_SHININESS 128.0f
#define MK_DEFAULT_MAT_SPECULAR 0.5f
#define MK_DEFAULT_MAT_EMISSION 0.0f
#define MK_NO_LIGHT_AMBIENT_FALLBACK 0.3f

extern "C" __global__ void render_megakernel(
    wp::launch_bounds_t<2> dim, wp::array_t<wp::int32> geom_type,
    wp::array_t<wp::int32> geom_dataid, wp::array_t<wp::int32> geom_matid,
    wp::array_t<wp::vec3> geom_size, wp::array_t<wp::vec4> geom_rgba,
    wp::array_t<wp::int32> cam_projection, wp::array_t<float> cam_fovy,
    wp::array_t<wp::vec2> cam_sensorsize, wp::array_t<wp::vec4> cam_intrinsic,
    wp::array_t<wp::int32> light_type, wp::array_t<bool> light_castshadow,
    wp::array_t<bool> light_active, wp::array_t<wp::vec3> light_attenuation,
    wp::array_t<float> light_cutoff, wp::array_t<float> light_exponent,
    wp::array_t<wp::vec3> light_ambient, wp::array_t<wp::vec3> light_diffuse,
    wp::array_t<wp::vec3> light_specular, wp::array_t<wp::int32> flex_vertadr,
    wp::array_t<wp::vec2i> flex_edge, wp::array_t<float> flex_radius,
    wp::array_t<wp::int32> mesh_faceadr, wp::array_t<wp::int32> mesh_normaladr,
    wp::array_t<wp::vec3> mesh_normal, wp::array_t<wp::int32> mat_texid,
    wp::array_t<bool> mat_texuniform, wp::array_t<wp::vec2> mat_texrepeat,
    wp::array_t<float> mat_emission, wp::array_t<float> mat_specular,
    wp::array_t<float> mat_shininess, wp::array_t<wp::vec4> mat_rgba,
    wp::array_t<wp::vec3> geom_xpos_in, wp::array_t<wp::mat33> geom_xmat_in,
    wp::array_t<wp::vec3> cam_xpos_in, wp::array_t<wp::mat33> cam_xmat_in,
    wp::array_t<wp::vec3> light_xpos_in, wp::array_t<wp::vec3> light_xdir_in,
    wp::array_t<wp::vec3> flexvert_xpos_in, wp::int32 nrender, bool use_shadows,
    wp::int32 bvh_ngeom, wp::int32 bvh_nflexgeom, wp::array_t<wp::vec2i> cam_res,
    wp::array_t<wp::int32> cam_id_map, wp::array_t<wp::vec3> ray,
    wp::array_t<wp::vec3> ray_offset, wp::int32 ray_base, wp::array_t<wp::int32> rgb_adr,
    wp::array_t<wp::int32> depth_adr, wp::array_t<wp::int32> seg_adr,
    wp::array_t<bool> render_rgb, wp::array_t<bool> render_depth,
    wp::array_t<bool> render_seg, wp::uint64 bvh_id, wp::array_t<wp::int32> group_root,
    wp::array_t<wp::uint64> flex_bvh_id, wp::array_t<wp::int32> flex_group_root,
    wp::array_t<wp::int32> enabled_geom_ids, wp::array_t<wp::uint64> mesh_bvh_id,
    wp::array_t<wp::vec3i> mesh_facetexcoord, wp::array_t<wp::vec3i> mesh_facenormal,
    wp::array_t<wp::vec2> mesh_texcoord, wp::array_t<wp::int32> mesh_texcoord_offsets,
    wp::array_t<wp::uint64> hfield_bvh_id, wp::array_t<wp::vec4> flex_rgba,
    wp::array_t<wp::int32> flex_geom_flexid, wp::array_t<wp::int32> flex_geom_edgeid,
    wp::array_t<wp::int32> skybox_tex_id, wp::array_t<wp::int32> skybox_face_width,
    wp::array_t<wp::texture2d_t> textures, wp::array_t<wp::vec3> splat_position,
    wp::array_t<wp::quat> splat_rotation, wp::array_t<wp::vec3> splat_scale,
    wp::array_t<wp::vec4> splat_rgba, wp::uint64 splat_bvh_id,
    wp::array_t<wp::int32> splat_group_root, wp::array_t<wp::uint32> rgb_out,
    wp::array_t<wp::vec3> aa_accum_out, wp::array_t<float> depth_out,
    wp::array_t<wp::vec2i> seg_out) {
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
    // 本片：阴影/splat 未实现；host 侧已拒绝（这里只做防御性判断）。
    if (use_shadows) {
        return;
    }

    const int ncam = nrender;
    int camid = -1;
    int rayid_local = -1;
    int accum = 0;
    for (int camera = 0; camera < ncam; camera++) {
        int width = cam_res.data[camera][0];
        int height = cam_res.data[camera][1];
        int num_i = width * height;
        if (rayid < accum + num_i) {
            camid = camera;
            rayid_local = rayid - accum;
            break;
        }
        accum += num_i;
    }
    if (camid < 0 || rayid_local < 0) {
        return;
    }
    bool do_depth = render_depth.data[camid];
    bool do_seg = render_seg.data[camid];
    bool do_rgb = render_rgb.data[camid];
    if (!do_depth && !do_seg && !do_rgb) {
        return;
    }

    unsigned long long flags = splat_bvh_id;
    unsigned int flag_bits = (unsigned int)(flags & 0xFFFFFFFFull);
    unsigned int packed_background = (unsigned int)(flags >> 32);
    V3 headlight_ambient = to_v3(splat_position.data[0]);
    V3 headlight_diffuse = to_v3(splat_position.data[1]);
    V3 headlight_specular = to_v3(splat_position.data[2]);
    V3 background = to_v3(splat_position.data[3]);
    float zfar = splat_scale.data[0][1];
    (void)background;

    int mujoco_cam_id = cam_id_map.data[camid];
    V3 ray_dir_local_cam = to_v3(ray.data[ray_base + rayid]);
    V3 ray_offset_local_cam = to_v3(ray_offset.data[ray_base + rayid]);
    const float* cam_mat = (const float*)(cam_xmat_in.data + mujoco_cam_id);
    V3 cam_pos = to_v3(cam_xpos_in.data[mujoco_cam_id]);
    V3 ray_origin_world = add(cam_pos, mat_vec(cam_mat, ray_offset_local_cam));
    V3 ray_dir_world = mat_vec(cam_mat, ray_dir_local_cam);

    float max_cam_dist = MJ_MAXVAL;
    if (zfar > 0.0f) {
        max_cam_dist = zfar / fmaxf(-ray_dir_local_cam.z, 1.0e-6f);
    }

    MegakernelBlob blob = megakernel_blob(bvh_id);
    SceneView scene = scene_view(blob.scene_meta, blob.scene_params);
    bool cull_backfaces = (flag_bits & MK_FLAG_CULL_BACKFACES) != 0u;
    CastHit hit = cast_ray(scene, blob, geom_type.data, geom_matid.data, (const float*)geom_rgba.data,
                           (const float*)mat_rgba.data, worldid, ray_origin_world, ray_dir_world,
                           max_cam_dist, cull_backfaces);

    if (hit.geom_id != -1 && zfar > 0.0f && (hit.dist * -ray_dir_local_cam.z) > zfar) {
        hit.geom_id = -1;
    }

    // 输出 pitch（array2d 的第二维）：上游 `rgb_out[worldid, adr + rayid_local]`。
    int rgb_pitch = rgb_out.shape.dims[1];
    int depth_pitch = depth_out.shape.dims[1];
    int seg_pitch = seg_out.shape.dims[1];

    if (do_seg) {
        wp::vec2i value;
        value[0] = hit.geom_id;
        value[1] = hit.geom_id == -1 ? -1 : 5;
        seg_out.data[(long)worldid * seg_pitch + seg_adr.data[camid] + rayid_local] = value;
    }

    if (hit.geom_id == -1) {
        if (do_depth) {
            depth_out.data[(long)worldid * depth_pitch + depth_adr.data[camid] + rayid_local] = 0.0f;
        }
        if (do_rgb) {
            rgb_out.data[(long)worldid * rgb_pitch + rgb_adr.data[camid] + rayid_local] =
                packed_background;
        }
        return;
    }

    if (do_depth) {
        depth_out.data[(long)worldid * depth_pitch + depth_adr.data[camid] + rayid_local] =
            hit.dist * -ray_dir_local_cam.z;
    }
    if (!do_rgb) {
        return;
    }

    int geom_id = hit.geom_id;
    V3 normal = hit.normal;
    int face = hit.face;
    int mesh_id = hit.mesh_id;

    // 顶点法线（上游 `enable_vertex_normals` 分支；mesh 命中且 face>=0 时）。
    if ((flag_bits & MK_FLAG_VERTEX_NORMALS) != 0u && mesh_id >= 0 && face >= 0) {
        const float* geom_mat = (const float*)(geom_xmat_in.data + geom_id);
        V3 geom_face = mat_t_vec(geom_mat, normal);
        const int* face_normal = (const int*)(mesh_facenormal.data + (mesh_faceadr.data[mesh_id] + face));
        int adr = mesh_normaladr.data[mesh_id];
        V3 vn = add(
            scale(vertex_normal(to_v3(mesh_normal.data[adr + face_normal[0]]), geom_face), hit.bary_u),
            scale(vertex_normal(to_v3(mesh_normal.data[adr + face_normal[1]]), geom_face), hit.bary_v));
        vn = add(vn, scale(vertex_normal(to_v3(mesh_normal.data[adr + face_normal[2]]), geom_face),
                           1.0f - hit.bary_u - hit.bary_v));
        normal = normalize3_rays(mat_vec(geom_mat, vn));
    }
    if ((flag_bits & MK_FLAG_CULL_BACKFACES) == 0u && dot3_ba(normal.x, normal.y, normal.z,
                                                               ray_dir_world.x, ray_dir_world.y,
                                                               ray_dir_world.z) > 0.0f) {
        normal = scale(normal, -1.0f);
    }

    // 命中点/材质。
    V3 hit_point = add(ray_origin_world, scale(ray_dir_world, hit.dist));
    float rgba[4];
    int mat_id = -1;
    render_geom_rgba(geom_matid.data, (const float*)geom_rgba.data, (const float*)mat_rgba.data, geom_id,
              rgba, &mat_id);
    V3 base_color = v3(rgba[0], rgba[1], rgba[2]);

    if ((flag_bits & MK_FLAG_TEXTURES) != 0u && mat_id >= 0) {
        int tex_id = mat_texid.data[mat_id * 10 + 1];
        if (tex_id >= 0) {
            const float* geom_pos = (const float*)(geom_xpos_in.data + geom_id);
            const float* geom_mat = (const float*)(geom_xmat_in.data + geom_id);
            bool tex_uniform = mat_texuniform.data[mat_id];
            V3 tex_color = sample_texture(
                geom_type.data, mesh_faceadr.data, geom_id, (const float*)(geom_size.data + geom_id),
                (const float*)(mat_texrepeat.data + mat_id), tex_uniform, textures.data[tex_id],
                v3(geom_pos[0], geom_pos[1], geom_pos[2]), geom_mat,
                (const int*)mesh_facetexcoord.data, (const float*)mesh_texcoord.data,
                mesh_texcoord_offsets.data, hit_point, normal, hit.bary_u, hit.bary_v, face, mesh_id);
            base_color = cw_mul(base_color, tex_color);
        }
    }

    float mat_spec = MK_DEFAULT_MAT_SPECULAR;
    float mat_shin_exp = 0.5f * MK_MAX_SHININESS;
    float mat_emis = MK_DEFAULT_MAT_EMISSION;
    if (mat_id >= 0) {
        if ((flag_bits & MK_FLAG_SPECULAR) != 0u) {
            mat_spec = mat_specular.data[mat_id];
            mat_shin_exp = mat_shininess.data[mat_id] * MK_MAX_SHININESS;
        }
        if ((flag_bits & MK_FLAG_EMISSION) != 0u) {
            mat_emis = mat_emission.data[mat_id];
        }
    }

    V3 result = v3(0.0f, 0.0f, 0.0f);
    if ((flag_bits & MK_FLAG_EMISSION) != 0u) {
        result = scale(base_color, mat_emis);
    }

    int nlight = light_type.shape.dims[1];
    if ((flag_bits & MK_FLAG_AMBIENT_LIGHTING) != 0u) {
        if ((flag_bits & MK_FLAG_HEADLIGHT) != 0u) {
            result = add(result, cw_mul(base_color, headlight_ambient));
        } else if (nlight == 0) {
            result = add(result, scale(base_color, MK_NO_LIGHT_AMBIENT_FALLBACK));
        }
        if ((flag_bits & MK_FLAG_PER_LIGHT_AMBIENT) != 0u) {
            for (int light = 0; light < nlight; light++) {
                if (light_active.data[light]) {
                    result = add(result, cw_mul(base_color, to_v3(light_ambient.data[light])));
                }
            }
        }
    }

    V3 view_dir = scale(ray_dir_world, -1.0f);
    bool enable_specular = (flag_bits & MK_FLAG_SPECULAR) != 0u;
    for (int light = 0; light < nlight; light++) {
        bool active = light_active.data[light];
        int type = light_type.data[light];
        V3 lightpos = to_v3(light_xpos_in.data[light]);
        V3 lightdir = to_v3(light_xdir_in.data[light]);
        V3 attenuation = to_v3(light_attenuation.data[light]);
        float cutoff = light_cutoff.data[light] * RENDER_DEG_TO_RAD;
        float exponent = light_exponent.data[light];
        V3 diff_rgb;
        V3 spec_rgb;
        compute_lighting(active, type, lightpos, lightdir, attenuation, cutoff, exponent,
                         to_v3(light_diffuse.data[light]), to_v3(light_specular.data[light]), normal, hit_point,
                         view_dir, mat_spec, mat_shin_exp, enable_specular, &diff_rgb, &spec_rgb);
        result = add(add(result, cw_mul(base_color, diff_rgb)), spec_rgb);
    }

    // Headlight：camera 位置 + 光轴方向（上游 `cam_fwd = -cam_mat_world[:, 2]`）。
    // 上游 headlight 调用传的是**材质** `mat_spec`/`mat_shin_exp`（megakernel 行 9419-9432
    // 的 `var_352`/`var_353`），不是固定的 headlight 高光指数。
    if ((flag_bits & MK_FLAG_HEADLIGHT) != 0u) {
        V3 cam_fwd = v3(-cam_mat[2], -cam_mat[5], -cam_mat[8]);
        V3 diff_rgb;
        V3 spec_rgb;
        compute_lighting(true, 1, ray_origin_world, cam_fwd, v3(1.0f, 0.0f, 0.0f), 0.0f, 0.0f,
                         headlight_diffuse, headlight_specular, normal, hit_point, view_dir, mat_spec,
                         mat_shin_exp, enable_specular, &diff_rgb, &spec_rgb);
        result = add(add(result, cw_mul(base_color, diff_rgb)), spec_rgb);
    }

    V3 hit_color = v3(fminf(result.x, 1.0f), fminf(result.y, 1.0f), fminf(result.z, 1.0f));
    hit_color = v3(fmaxf(hit_color.x, 0.0f), fmaxf(hit_color.y, 0.0f), fmaxf(hit_color.z, 0.0f));
    float r = hit_color.x * 255.0f;
    float g = hit_color.y * 255.0f;
    float b = hit_color.z * 255.0f;
    unsigned int pixel = pack_rgba_to_uint32(r, g, b, 255.0f);
    rgb_out.data[(long)worldid * rgb_pitch + rgb_adr.data[camid] + rayid_local] = pixel;
}
