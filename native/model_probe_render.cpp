// Render model probe (S1): read-only render fields, no GPU and no BVH.
// Included by native/model_probe.cpp inside the same translation unit, after
// the owner struct, the dynamic-library helpers and the G01 field table.
// The G01 / collision / flex / sparse bridges are not modified by this file.
// Field order is shared with src/model/render.rs::RENDER_FIELDS.

// ---- Render model probe (S1): read-only render fields; no GPU, no BVH. ----
// Independent schema/IDs from G01. Field order is shared with
// src/model/render.rs::RENDER_FIELDS; the Rust test compares every field
// against fixtures/render-bvh/scene-a/model-312.json.
static_assert(sizeof(mjwarp_render_field_info) == 16);
static_assert(alignof(mjwarp_render_field_info) == 8);
static_assert(offsetof(mjwarp_render_field_info, count) == 8);
static_assert(sizeof(mjwarp_render_info) == 160);
static_assert(alignof(mjwarp_render_info) == 8);
static_assert(offsetof(mjwarp_render_info, nbody) == 8);
static_assert(offsetof(mjwarp_render_info, nnames) == 152);

static bool render_counts(const mjModel* m, mjwarp_render_info& out) {
  if (m->nbody < 0 || m->njnt < 0 || m->ngeom < 0 || m->nsite < 0 ||
      m->ncam < 0 || m->nlight < 0 || m->nmat < 0 || m->ntex < 0 ||
      m->ntexdata < 0 || m->nmesh < 0 || m->nmeshvert < 0 ||
      m->nmeshnormal < 0 || m->nmeshtexcoord < 0 || m->nmeshface < 0 ||
      m->nmeshgraph < 0 || m->nmeshpoly < 0 || m->nmeshpolyvert < 0 ||
      m->nmeshpolymap < 0 || m->nnames < 0) return false;
  out = {1, 0, m->nbody, m->njnt, m->ngeom, m->nsite, m->ncam, m->nlight,
         m->nmat, m->ntex, m->ntexdata, m->nmesh, m->nmeshvert,
         m->nmeshnormal, m->nmeshtexcoord, m->nmeshface, m->nmeshgraph,
         m->nmeshpoly, m->nmeshpolyvert, m->nmeshpolymap, m->nnames};
  return true;
}

struct render_field {
  const void* source;
  int64_t count;
  uint32_t kind;
};

static uint64_t render_scalar_bytes(uint32_t kind) {
  switch (kind) {
    case 5: return sizeof(float);
    case 4: return sizeof(int64_t);
    case 2: return sizeof(double);
    case 3: return sizeof(uint8_t);
    default: return sizeof(int32_t);
  }
}

// Every render field is `<unit> x <width>` scalars of one kind. Negative units,
// non-positive widths and overflowing products fail before any allocation.
static bool render_stage(const mjModel* m, uint32_t id, render_field* out) {
  if (!m || !out) return false;
  int64_t unit = 0;
  int64_t width = 0;
  const void* source = nullptr;
  uint32_t kind = 0;
  switch (id) {
    case 0: source = m->geom_type; unit = m->ngeom; width = 1; kind = 1; break;  // geom_type
    case 1: source = m->geom_bodyid; unit = m->ngeom; width = 1; kind = 1; break;  // geom_bodyid
    case 2: source = m->geom_dataid; unit = m->ngeom; width = 1; kind = 1; break;  // geom_dataid
    case 3: source = m->geom_matid; unit = m->ngeom; width = 1; kind = 1; break;  // geom_matid
    case 4: source = m->geom_group; unit = m->ngeom; width = 1; kind = 1; break;  // geom_group
    case 5: source = m->geom_size; unit = m->ngeom; width = 3; kind = 2; break;  // geom_size
    case 6: source = m->geom_pos; unit = m->ngeom; width = 3; kind = 2; break;  // geom_pos
    case 7: source = m->geom_quat; unit = m->ngeom; width = 4; kind = 2; break;  // geom_quat
    case 8: source = m->geom_rgba; unit = m->ngeom; width = 4; kind = 5; break;  // geom_rgba
    case 9: source = m->cam_mode; unit = m->ncam; width = 1; kind = 1; break;  // cam_mode
    case 10: source = m->cam_bodyid; unit = m->ncam; width = 1; kind = 1; break;  // cam_bodyid
    case 11: source = m->cam_targetbodyid; unit = m->ncam; width = 1; kind = 1; break;  // cam_targetbodyid
    case 12: source = m->cam_projection; unit = m->ncam; width = 1; kind = 1; break;  // cam_projection
    case 13: source = m->cam_resolution; unit = m->ncam; width = 2; kind = 1; break;  // cam_resolution
    case 14: source = m->cam_output; unit = m->ncam; width = 1; kind = 1; break;  // cam_output
    case 15: source = m->cam_fovy; unit = m->ncam; width = 1; kind = 2; break;  // cam_fovy
    case 16: source = m->cam_ipd; unit = m->ncam; width = 1; kind = 2; break;  // cam_ipd
    case 17: source = m->cam_sensorsize; unit = m->ncam; width = 2; kind = 5; break;  // cam_sensorsize
    case 18: source = m->cam_intrinsic; unit = m->ncam; width = 4; kind = 5; break;  // cam_intrinsic
    case 19: source = m->cam_pos; unit = m->ncam; width = 3; kind = 2; break;  // cam_pos
    case 20: source = m->cam_quat; unit = m->ncam; width = 4; kind = 2; break;  // cam_quat
    case 21: source = m->cam_poscom0; unit = m->ncam; width = 3; kind = 2; break;  // cam_poscom0
    case 22: source = m->cam_pos0; unit = m->ncam; width = 3; kind = 2; break;  // cam_pos0
    case 23: source = m->cam_mat0; unit = m->ncam; width = 9; kind = 2; break;  // cam_mat0
    case 24: source = m->light_mode; unit = m->nlight; width = 1; kind = 1; break;  // light_mode
    case 25: source = m->light_bodyid; unit = m->nlight; width = 1; kind = 1; break;  // light_bodyid
    case 26: source = m->light_targetbodyid; unit = m->nlight; width = 1; kind = 1; break;  // light_targetbodyid
    case 27: source = m->light_type; unit = m->nlight; width = 1; kind = 1; break;  // light_type
    case 28: source = m->light_texid; unit = m->nlight; width = 1; kind = 1; break;  // light_texid
    case 29: source = m->light_castshadow; unit = m->nlight; width = 1; kind = 3; break;  // light_castshadow
    case 30: source = m->light_active; unit = m->nlight; width = 1; kind = 3; break;  // light_active
    case 31: source = m->light_bulbradius; unit = m->nlight; width = 1; kind = 5; break;  // light_bulbradius
    case 32: source = m->light_intensity; unit = m->nlight; width = 1; kind = 5; break;  // light_intensity
    case 33: source = m->light_range; unit = m->nlight; width = 1; kind = 5; break;  // light_range
    case 34: source = m->light_attenuation; unit = m->nlight; width = 3; kind = 5; break;  // light_attenuation
    case 35: source = m->light_cutoff; unit = m->nlight; width = 1; kind = 5; break;  // light_cutoff
    case 36: source = m->light_softness; unit = m->nlight; width = 1; kind = 5; break;  // light_softness
    case 37: source = m->light_exponent; unit = m->nlight; width = 1; kind = 5; break;  // light_exponent
    case 38: source = m->light_ambient; unit = m->nlight; width = 3; kind = 5; break;  // light_ambient
    case 39: source = m->light_diffuse; unit = m->nlight; width = 3; kind = 5; break;  // light_diffuse
    case 40: source = m->light_specular; unit = m->nlight; width = 3; kind = 5; break;  // light_specular
    case 41: source = m->light_pos; unit = m->nlight; width = 3; kind = 2; break;  // light_pos
    case 42: source = m->light_dir; unit = m->nlight; width = 3; kind = 2; break;  // light_dir
    case 43: source = m->light_poscom0; unit = m->nlight; width = 3; kind = 2; break;  // light_poscom0
    case 44: source = m->light_pos0; unit = m->nlight; width = 3; kind = 2; break;  // light_pos0
    case 45: source = m->light_dir0; unit = m->nlight; width = 3; kind = 2; break;  // light_dir0
    case 46: source = m->mat_texid; unit = m->nmat; width = 10; kind = 1; break;  // mat_texid
    case 47: source = m->mat_texuniform; unit = m->nmat; width = 1; kind = 3; break;  // mat_texuniform
    case 48: source = m->mat_texrepeat; unit = m->nmat; width = 2; kind = 5; break;  // mat_texrepeat
    case 49: source = m->mat_emission; unit = m->nmat; width = 1; kind = 5; break;  // mat_emission
    case 50: source = m->mat_specular; unit = m->nmat; width = 1; kind = 5; break;  // mat_specular
    case 51: source = m->mat_shininess; unit = m->nmat; width = 1; kind = 5; break;  // mat_shininess
    case 52: source = m->mat_reflectance; unit = m->nmat; width = 1; kind = 5; break;  // mat_reflectance
    case 53: source = m->mat_metallic; unit = m->nmat; width = 1; kind = 5; break;  // mat_metallic
    case 54: source = m->mat_roughness; unit = m->nmat; width = 1; kind = 5; break;  // mat_roughness
    case 55: source = m->mat_rgba; unit = m->nmat; width = 4; kind = 5; break;  // mat_rgba
    case 56: source = m->tex_type; unit = m->ntex; width = 1; kind = 1; break;  // tex_type
    case 57: source = m->tex_colorspace; unit = m->ntex; width = 1; kind = 1; break;  // tex_colorspace
    case 58: source = m->tex_height; unit = m->ntex; width = 1; kind = 1; break;  // tex_height
    case 59: source = m->tex_width; unit = m->ntex; width = 1; kind = 1; break;  // tex_width
    case 60: source = m->tex_nchannel; unit = m->ntex; width = 1; kind = 1; break;  // tex_nchannel
    case 61: source = m->tex_adr; unit = m->ntex; width = 1; kind = 4; break;  // tex_adr
    case 62: source = m->tex_data; unit = m->ntexdata; width = 1; kind = 3; break;  // tex_data
    case 63: source = m->tex_pathadr; unit = m->ntex; width = 1; kind = 1; break;  // tex_pathadr
    case 64: source = m->mesh_vertadr; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_vertadr
    case 65: source = m->mesh_vertnum; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_vertnum
    case 66: source = m->mesh_faceadr; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_faceadr
    case 67: source = m->mesh_facenum; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_facenum
    case 68: source = m->mesh_bvhadr; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_bvhadr
    case 69: source = m->mesh_bvhnum; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_bvhnum
    case 70: source = m->mesh_octadr; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_octadr
    case 71: source = m->mesh_octnum; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_octnum
    case 72: source = m->mesh_normaladr; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_normaladr
    case 73: source = m->mesh_normalnum; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_normalnum
    case 74: source = m->mesh_texcoordadr; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_texcoordadr
    case 75: source = m->mesh_texcoordnum; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_texcoordnum
    case 76: source = m->mesh_graphadr; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_graphadr
    case 77: source = m->mesh_extrema; unit = m->nmesh; width = 27; kind = 1; break;  // mesh_extrema
    case 78: source = m->mesh_vert; unit = m->nmeshvert; width = 3; kind = 5; break;  // mesh_vert
    case 79: source = m->mesh_normal; unit = m->nmeshnormal; width = 3; kind = 5; break;  // mesh_normal
    case 80: source = m->mesh_texcoord; unit = m->nmeshtexcoord; width = 2; kind = 5; break;  // mesh_texcoord
    case 81: source = m->mesh_face; unit = m->nmeshface; width = 3; kind = 1; break;  // mesh_face
    case 82: source = m->mesh_facenormal; unit = m->nmeshface; width = 3; kind = 1; break;  // mesh_facenormal
    case 83: source = m->mesh_facetexcoord; unit = m->nmeshface; width = 3; kind = 1; break;  // mesh_facetexcoord
    case 84: source = m->mesh_graph; unit = m->nmeshgraph; width = 1; kind = 1; break;  // mesh_graph
    case 85: source = m->mesh_scale; unit = m->nmesh; width = 3; kind = 2; break;  // mesh_scale
    case 86: source = m->mesh_pos; unit = m->nmesh; width = 3; kind = 2; break;  // mesh_pos
    case 87: source = m->mesh_quat; unit = m->nmesh; width = 4; kind = 2; break;  // mesh_quat
    case 88: source = m->mesh_pathadr; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_pathadr
    case 89: source = m->mesh_polynum; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_polynum
    case 90: source = m->mesh_polyadr; unit = m->nmesh; width = 1; kind = 1; break;  // mesh_polyadr
    case 91: source = m->mesh_polynormal; unit = m->nmeshpoly; width = 3; kind = 2; break;  // mesh_polynormal
    case 92: source = m->mesh_polyvertadr; unit = m->nmeshpoly; width = 1; kind = 1; break;  // mesh_polyvertadr
    case 93: source = m->mesh_polyvertnum; unit = m->nmeshpoly; width = 1; kind = 1; break;  // mesh_polyvertnum
    case 94: source = m->mesh_polyvert; unit = m->nmeshpolyvert; width = 1; kind = 1; break;  // mesh_polyvert
    case 95: source = m->mesh_polymapadr; unit = m->nmeshvert; width = 1; kind = 1; break;  // mesh_polymapadr
    case 96: source = m->mesh_polymapnum; unit = m->nmeshvert; width = 1; kind = 1; break;  // mesh_polymapnum
    case 97: source = m->mesh_polymap; unit = m->nmeshpolymap; width = 1; kind = 1; break;  // mesh_polymap
    case 98: source = m->vis.headlight.ambient; unit = 1; width = 3; kind = 5; break;  // vis.headlight.ambient
    case 99: source = m->vis.headlight.diffuse; unit = 1; width = 3; kind = 5; break;  // vis.headlight.diffuse
    case 100: source = m->vis.headlight.specular; unit = 1; width = 3; kind = 5; break;  // vis.headlight.specular
    case 101: source = &m->vis.headlight.active; unit = 1; width = 1; kind = 1; break;  // vis.headlight.active
    case 102: source = &m->vis.map.znear; unit = 1; width = 1; kind = 5; break;  // vis.map.znear
    case 103: source = &m->vis.map.zfar; unit = 1; width = 1; kind = 5; break;  // vis.map.zfar
    case 104: source = &m->stat.extent; unit = 1; width = 1; kind = 2; break;  // stat.extent
    case 105: source = m->stat.center; unit = 1; width = 3; kind = 2; break;  // stat.center
    case 106: source = m->name_bodyadr; unit = m->nbody; width = 1; kind = 1; break;  // name_bodyadr
    case 107: source = m->name_jntadr; unit = m->njnt; width = 1; kind = 1; break;  // name_jntadr
    case 108: source = m->name_geomadr; unit = m->ngeom; width = 1; kind = 1; break;  // name_geomadr
    case 109: source = m->name_siteadr; unit = m->nsite; width = 1; kind = 1; break;  // name_siteadr
    case 110: source = m->name_camadr; unit = m->ncam; width = 1; kind = 1; break;  // name_camadr
    case 111: source = m->name_lightadr; unit = m->nlight; width = 1; kind = 1; break;  // name_lightadr
    case 112: source = m->name_meshadr; unit = m->nmesh; width = 1; kind = 1; break;  // name_meshadr
    case 113: source = m->name_texadr; unit = m->ntex; width = 1; kind = 1; break;  // name_texadr
    case 114: source = m->name_matadr; unit = m->nmat; width = 1; kind = 1; break;  // name_matadr
    case 115: source = m->names; unit = m->nnames; width = 1; kind = 3; break;  // names
    default: return false;
  }
  if (unit < 0 || width <= 0 || unit > INT64_MAX / width) return false;
  const int64_t count = unit * width;
  if (count > 0 && !source) return false;
  if (static_cast<uint64_t>(count) > SIZE_MAX / render_scalar_bytes(kind)) return false;
  *out = {source, count, kind};
  return true;
}

extern "C" int32_t mjwarp_native_render_info(
    const mjwarp_native_owner* owner, mjwarp_render_info* info) {
  if (!owner || !owner->model || !info) return 8;
  mjwarp_render_info staged{};
  if (!render_counts(owner->model, staged)) return 8;
  *info = staged;
  return 0;
}

extern "C" int32_t mjwarp_native_render_field_info(
    const mjwarp_native_owner* owner, uint32_t id, mjwarp_render_field_info* info) {
  render_field field{};
  if (!owner || !info || !render_stage(owner->model, id, &field)) return 8;
  *info = {field.kind, 0, field.count};
  return 0;
}

extern "C" int32_t mjwarp_native_copy_render_field(
    const mjwarp_native_owner* owner, uint32_t id,
    const mjwarp_render_field_info* info, void* target) {
  render_field field{};
  if (!owner || !info || !render_stage(owner->model, id, &field) ||
      info->reserved != 0 || info->kind != field.kind ||
      info->count != field.count || (field.count > 0 && !target)) return 8;
  if (field.count > 0) {
    memcpy(target, field.source,
           static_cast<size_t>(field.count) * render_scalar_bytes(field.kind));
  }
  return 0;
}

// mj_id2name oracle: the semantics under test come from the loaded library, not
// from a re-implementation in this bridge.
extern "C" int32_t mjwarp_native_render_name(
    const mjwarp_native_owner* owner, int32_t objtype, int64_t id,
    char* buffer, int64_t capacity, int64_t* length) {
  if (!owner || !owner->library || !owner->model || !length || capacity < 0 ||
      (capacity > 0 && !buffer)) return 8;
  auto id2name = library_symbol<decltype(&mj_id2name)>(owner->library, "mj_id2name");
  if (!id2name) return 8;
  if (id < static_cast<int64_t>(INT32_MIN) || id > static_cast<int64_t>(INT32_MAX)) return 1;
  const char* name =
      id2name(owner->model, static_cast<mjtObj>(objtype), static_cast<int>(id));
  if (!name) return 1;
  const size_t bytes = strlen(name);
  *length = static_cast<int64_t>(bytes);
  if (static_cast<uint64_t>(capacity) < static_cast<uint64_t>(bytes) + 1) return 2;
  memcpy(buffer, name, bytes + 1);
  return 0;
}
