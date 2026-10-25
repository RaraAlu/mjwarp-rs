// Read-only G01 extraction. No CPU physics or model compilation.
static_assert(sizeof(mjwarp_g01_field_info) == 16);
static_assert(alignof(mjwarp_g01_field_info) == 8);
static_assert(offsetof(mjwarp_g01_field_info, count) == 8);
// Scalar kind codes (shared with src/io/g01.rs and src/model/volume.rs):
// 1 = int32, 2 = double, 3 = uint8, 4 = float32 (hfield_data), 5 = int64 (mjtSize).
static_assert(sizeof(float) == 4 && sizeof(int64_t) == 8 && sizeof(mjtNum) == 8);
static uint64_t g01_scalar_bytes(uint32_t kind) {
  switch (kind) {
    case 2: return sizeof(double);
    case 3: return sizeof(uint8_t);
    case 4: return sizeof(float);
    case 5: return sizeof(int64_t);
    default: return sizeof(int32_t);
  }
}
struct g01_field {
  const void* source;
  int64_t count;
  uint32_t kind;
};
static bool g01_select(const mjModel* m, uint32_t id, g01_field* out) {
  if (!m || !out) return false;
  int64_t count = 0; int64_t width = 1;
  const void* source = nullptr; uint32_t kind = 0;
  switch (id) {
    case 0: source = m->body_mocapid; count = m->nbody; width = 1; kind = 1; break;
    case 1: source = m->geom_bodyid; count = m->ngeom; width = 1; kind = 1; break;
    case 2: source = m->site_bodyid; count = m->nsite; width = 1; kind = 1; break;
    case 3: source = m->cam_mode; count = m->ncam; width = 1; kind = 1; break;
    case 4: source = m->cam_bodyid; count = m->ncam; width = 1; kind = 1; break;
    case 5: source = m->cam_targetbodyid; count = m->ncam; width = 1; kind = 1; break;
    case 6: source = m->light_mode; count = m->nlight; width = 1; kind = 1; break;
    case 7: source = m->light_bodyid; count = m->nlight; width = 1; kind = 1; break;
    case 8: source = m->light_targetbodyid; count = m->nlight; width = 1; kind = 1; break;
    case 9: source = m->geom_type; count = m->ngeom; width = 1; kind = 1; break;
    case 10: source = m->tendon_adr; count = m->ntendon; width = 1; kind = 1; break;
    case 11: source = m->tendon_num; count = m->ntendon; width = 1; kind = 1; break;
    case 12: source = m->wrap_type; count = m->nwrap; width = 1; kind = 1; break;
    case 13: source = m->wrap_objid; count = m->nwrap; width = 1; kind = 1; break;
    case 14: source = m->ten_J_rowadr; count = m->ntendon; width = 1; kind = 1; break;
    case 15: source = m->ten_J_rownnz; count = m->ntendon; width = 1; kind = 1; break;
    case 16: source = m->ten_J_colind; count = m->nJten; width = 1; kind = 1; break;
    case 17: source = m->body_treeid; count = m->nbody; width = 1; kind = 1; break;
    case 18: source = m->flex_edgeadr; count = m->nflex; width = 1; kind = 1; break;
    case 19: source = m->flex_edgenum; count = m->nflex; width = 1; kind = 1; break;
    case 20: source = m->flex_edge; count = m->nflexedge; width = 2; kind = 1; break;
    case 21: source = m->flexedge_J_rowadr; count = m->nflexedge; width = 1; kind = 1; break;
    case 22: source = m->flexedge_J_rownnz; count = m->nflexedge; width = 1; kind = 1; break;
    case 23: source = m->flexedge_J_colind; count = m->nJfe; width = 1; kind = 1; break;
    case 24: source = m->geom_pos; count = m->ngeom; width = 3; kind = 2; break;
    case 25: source = m->geom_quat; count = m->ngeom; width = 4; kind = 2; break;
    case 26: source = m->site_pos; count = m->nsite; width = 3; kind = 2; break;
    case 27: source = m->site_quat; count = m->nsite; width = 4; kind = 2; break;
    case 28: source = m->cam_pos; count = m->ncam; width = 3; kind = 2; break;
    case 29: source = m->cam_quat; count = m->ncam; width = 4; kind = 2; break;
    case 30: source = m->cam_poscom0; count = m->ncam; width = 3; kind = 2; break;
    case 31: source = m->cam_pos0; count = m->ncam; width = 3; kind = 2; break;
    case 32: source = m->cam_mat0; count = m->ncam; width = 9; kind = 2; break;
    case 33: source = m->light_pos; count = m->nlight; width = 3; kind = 2; break;
    case 34: source = m->light_dir; count = m->nlight; width = 3; kind = 2; break;
    case 35: source = m->light_poscom0; count = m->nlight; width = 3; kind = 2; break;
    case 36: source = m->light_pos0; count = m->nlight; width = 3; kind = 2; break;
    case 37: source = m->light_dir0; count = m->nlight; width = 3; kind = 2; break;
    case 38: source = m->geom_size; count = m->ngeom; width = 3; kind = 2; break;
    case 39: source = m->wrap_prm; count = m->nwrap; width = 1; kind = 2; break;
    case 40: source = m->tendon_range; count = m->ntendon; width = 2; kind = 2; break;
    case 41: source = m->tendon_margin; count = m->ntendon; width = 1; kind = 2; break;
    case 42: source = m->tendon_limited; count = m->ntendon; width = 1; kind = 3; break;
    case 43: source = &m->opt.enableflags; count = 1; width = 1; kind = 1; break;
    case 44: source = &m->opt.disableflags; count = 1; width = 1; kind = 1; break;
    // Volume/SDF model fields (G1). IDs 45..58 stay in sync with
    // src/model/volume.rs::VOLUME_FIELDS; src/io/volume.rs queries them.
    case 45: source = m->hfield_nrow; count = m->nhfield; width = 1; kind = 1; break;
    case 46: source = m->hfield_ncol; count = m->nhfield; width = 1; kind = 1; break;
    case 47: source = m->hfield_size; count = m->nhfield; width = 4; kind = 2; break;
    case 48: source = m->hfield_adr; count = m->nhfield; width = 1; kind = 1; break;
    case 49: source = m->hfield_data; count = m->nhfielddata; width = 1; kind = 4; break;
    case 50: source = &m->noct; count = 1; width = 1; kind = 5; break;
    case 51: source = m->oct_depth; count = m->noct; width = 1; kind = 1; break;
    case 52: source = m->oct_child; count = m->noct; width = 8; kind = 1; break;
    case 53: source = m->oct_aabb; count = m->noct; width = 6; kind = 2; break;
    case 54: source = m->oct_coeff; count = m->noct; width = 8; kind = 2; break;
    case 55: source = m->mesh_octadr; count = m->nmesh; width = 1; kind = 1; break;
    case 56: source = m->mesh_octnum; count = m->nmesh; width = 1; kind = 1; break;
    case 57: source = &m->opt.sdf_iterations; count = 1; width = 1; kind = 1; break;
    case 58: source = &m->opt.sdf_initpoints; count = 1; width = 1; kind = 1; break;
    default: return false;
  }
  if (count < 0 || count > INT64_MAX / width) return false;
  count *= width;
  const uint64_t bytes = g01_scalar_bytes(kind);
  if (static_cast<uint64_t>(count) > SIZE_MAX / bytes) return false;
  *out = {source, count, kind};
  return true;
}
extern "C" int32_t mjwarp_native_g01_field_info(
    const mjwarp_native_owner* owner, uint32_t id, mjwarp_g01_field_info* info) {
  g01_field f{};
  if (!owner || !info || !g01_select(owner->model, id, &f)) return 8;
  *info = {f.kind, 0, f.count};
  return 0;
}
extern "C" int32_t mjwarp_native_copy_g01_field(
    const mjwarp_native_owner* owner, uint32_t id,
    const mjwarp_g01_field_info* info, void* target) {
  g01_field f{};
  if (!owner || !info || !g01_select(owner->model, id, &f) ||
      info->reserved || info->kind != f.kind || info->count != f.count ||
      (f.count && (!f.source || !target))) return 8;
  const size_t width = g01_scalar_bytes(f.kind);
  if (f.count) memcpy(target, f.source, static_cast<size_t>(f.count) * width);
  return 0;
}
