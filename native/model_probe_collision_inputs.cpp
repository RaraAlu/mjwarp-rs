// Collision-inputs snapshot probe (S1): read-only model fields plus the
// geom_xpos/geom_xmat frame of an owned mjData after mj_kinematics.
//
// Included by native/model_probe_sparse.cpp after native/model_probe_collision.cpp,
// which itself includes the pinned native/model_probe.cpp. This file must not
// include model_probe.cpp again: that would duplicate same-TU definitions. It
// reuses the owner struct, the dynamic-library helpers and the pinned MuJoCo
// declarations, and it keeps the G01 / collision / flex / sparse / render
// bridges unchanged. Field IDs are shared with
// src/model/collision_inputs.rs::COLLISION_INPUTS_FIELDS.

#include "mjwarp_collision_inputs_probe.h"

#include <stddef.h>
#include <string.h>
#include <new>

static_assert(sizeof(mjwarp_collision_inputs_field_info) == 16);
static_assert(alignof(mjwarp_collision_inputs_field_info) == 8);
static_assert(offsetof(mjwarp_collision_inputs_field_info, count) == 8);
static_assert(sizeof(mjwarp_collision_inputs_info) == 96);
static_assert(alignof(mjwarp_collision_inputs_info) == 8);
static_assert(offsetof(mjwarp_collision_inputs_info, nq) == 8);
static_assert(offsetof(mjwarp_collision_inputs_info, noct) == 88);

// Compile-time native scalar-width guards for every copied source.
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->geom_type) == sizeof(int32_t));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->geom_size) == sizeof(double));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->geom_rbound) == sizeof(double));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->geom_aabb) == sizeof(double));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->mesh_vertadr) == sizeof(int32_t));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->mesh_vert) == sizeof(float));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->mesh_graph) == sizeof(int32_t));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->mesh_polynormal) == sizeof(double));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->hfield_size) == sizeof(double));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->hfield_data) == sizeof(float));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->oct_child) == sizeof(int32_t));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->oct_aabb) == sizeof(double));
static_assert(sizeof(*static_cast<const mjModel*>(nullptr)->oct_coeff) == sizeof(double));

static uint64_t collision_inputs_scalar_bytes(uint32_t kind) {
  switch (kind) {
    case 1: return sizeof(int32_t);
    case 2: return sizeof(double);
    case 4: return sizeof(float);
    default: return 0;
  }
}

struct collision_inputs_field {
  const void* source;
  int64_t count;
  uint32_t kind;
  bool data_sourced;
};

// Every field is `unit x width` scalars of one kind. Negative units, non-positive
// widths, overflowing products and missing model sources fail before any output.
static bool collision_inputs_field_of(const mjModel* m, uint32_t id,
                                      collision_inputs_field* out) {
  if (!m || !out) return false;
  const void* source = nullptr;
  int64_t unit = 0;
  int64_t width = 0;
  uint32_t kind = 0;
  bool data_sourced = false;
  switch (id) {
    // mjData fields: metadata only; the data snapshot serves the values.
    case 0: unit = m->ngeom; width = 3; kind = 4; data_sourced = true; break;  // geom_xpos
    case 1: unit = m->ngeom; width = 9; kind = 4; data_sourced = true; break;  // geom_xmat
    case 2: source = m->geom_size; unit = m->ngeom; width = 3; kind = 2; break;
    case 3: source = m->geom_margin; unit = m->ngeom; width = 1; kind = 2; break;
    case 4: source = m->geom_gap; unit = m->ngeom; width = 1; kind = 2; break;
    case 5: source = m->geom_rbound; unit = m->ngeom; width = 1; kind = 2; break;
    case 6: source = m->geom_aabb; unit = m->ngeom; width = 6; kind = 2; break;
    case 7: source = m->geom_type; unit = m->ngeom; width = 1; kind = 1; break;
    case 8: source = m->geom_dataid; unit = m->ngeom; width = 1; kind = 1; break;
    case 9: source = m->mesh_vertadr; unit = m->nmesh; width = 1; kind = 1; break;
    case 10: source = m->mesh_vertnum; unit = m->nmesh; width = 1; kind = 1; break;
    case 11: source = m->mesh_graphadr; unit = m->nmesh; width = 1; kind = 1; break;
    case 12: source = m->mesh_octadr; unit = m->nmesh; width = 1; kind = 1; break;
    case 13: source = m->mesh_vert; unit = m->nmeshvert; width = 3; kind = 4; break;
    case 14: source = m->mesh_graph; unit = m->nmeshgraph; width = 1; kind = 1; break;
    case 15: source = m->mesh_polynum; unit = m->nmesh; width = 1; kind = 1; break;
    case 16: source = m->mesh_polyadr; unit = m->nmesh; width = 1; kind = 1; break;
    case 17: source = m->mesh_polynormal; unit = m->nmeshpoly; width = 3; kind = 2; break;
    case 18: source = m->mesh_polyvertadr; unit = m->nmeshpoly; width = 1; kind = 1; break;
    case 19: source = m->mesh_polyvertnum; unit = m->nmeshpoly; width = 1; kind = 1; break;
    case 20: source = m->mesh_polyvert; unit = m->nmeshpolyvert; width = 1; kind = 1; break;
    case 21: source = m->mesh_polymapadr; unit = m->nmeshvert; width = 1; kind = 1; break;
    case 22: source = m->mesh_polymapnum; unit = m->nmeshvert; width = 1; kind = 1; break;
    case 23: source = m->mesh_polymap; unit = m->nmeshpolymap; width = 1; kind = 1; break;
    case 24: source = m->hfield_size; unit = m->nhfield; width = 4; kind = 2; break;
    case 25: source = m->hfield_nrow; unit = m->nhfield; width = 1; kind = 1; break;
    case 26: source = m->hfield_ncol; unit = m->nhfield; width = 1; kind = 1; break;
    case 27: source = m->hfield_adr; unit = m->nhfield; width = 1; kind = 1; break;
    case 28: source = m->hfield_data; unit = m->nhfielddata; width = 1; kind = 4; break;
    case 29: source = m->oct_aabb; unit = m->noct; width = 6; kind = 2; break;
    case 30: source = m->oct_child; unit = m->noct; width = 8; kind = 1; break;
    case 31: source = m->oct_coeff; unit = m->noct; width = 8; kind = 2; break;
    default: return false;
  }
  if (collision_inputs_scalar_bytes(kind) == 0) return false;
  if (unit < 0 || width <= 0 || unit > INT64_MAX / width) return false;
  const int64_t count = unit * width;
  if (!data_sourced && count > 0 && !source) return false;
  if (static_cast<uint64_t>(count) > SIZE_MAX / collision_inputs_scalar_bytes(kind)) return false;
  *out = {source, count, kind, data_sourced};
  return true;
}

static bool collision_inputs_counts(const mjModel* m,
                                    mjwarp_collision_inputs_info& out) {
  if (!m) return false;
  const mjtSize values[] = {m->nq, m->ngeom, m->nmesh, m->nmeshvert, m->nmeshgraph,
                            m->nmeshpoly, m->nmeshpolyvert, m->nmeshpolymap,
                            m->nhfield, m->nhfielddata, m->noct};
  for (mjtSize value : values) {
    if (value < 0) return false;
  }
  out = {1, 0, m->nq, m->ngeom, m->nmesh, m->nmeshvert, m->nmeshgraph,
         m->nmeshpoly, m->nmeshpolyvert, m->nmeshpolymap, m->nhfield,
         m->nhfielddata, m->noct};
  return true;
}

extern "C" int32_t mjwarp_native_collision_inputs_info(
    const mjwarp_native_owner* owner, mjwarp_collision_inputs_info* info) {
  if (!owner || !owner->model || !info) return 8;
  mjwarp_collision_inputs_info staged{};
  if (!collision_inputs_counts(owner->model, staged)) return 8;
  *info = staged;
  return 0;
}

extern "C" int32_t mjwarp_native_collision_inputs_field_info(
    const mjwarp_native_owner* owner, uint32_t id,
    mjwarp_collision_inputs_field_info* info) {
  if (!owner || !owner->model || !info) return 8;
  collision_inputs_field field{};
  if (!collision_inputs_field_of(owner->model, id, &field)) return 8;
  *info = {field.kind, 0, field.count};
  return 0;
}

extern "C" int32_t mjwarp_native_copy_collision_inputs_field(
    const mjwarp_native_owner* owner, uint32_t id,
    const mjwarp_collision_inputs_field_info* info, void* target) {
  if (!owner || !owner->model || !info) return 8;
  collision_inputs_field field{};
  if (!collision_inputs_field_of(owner->model, id, &field) || field.data_sourced) return 8;
  if (info->reserved != 0 || info->kind != field.kind || info->count != field.count) return 8;
  if (field.count > 0 && !target) return 8;
  if (field.count > 0) {
    const size_t bytes = static_cast<size_t>(field.count) *
                         static_cast<size_t>(collision_inputs_scalar_bytes(field.kind));
    memcpy(target, field.source, bytes);
  }
  return 0;
}

struct mjwarp_collision_inputs_data {
  const mjwarp_native_owner* owner;
  mjData* data;
  decltype(&mj_kinematics) kinematics;
  decltype(&mj_deleteData) delete_data;
};

extern "C" int32_t mjwarp_native_collision_inputs_create(
    const mjwarp_native_owner* owner, mjwarp_collision_inputs_data** output) {
  if (!output) return 8;
  *output = nullptr;
  if (!owner || !owner->library || !owner->model) return 8;
  auto make_data = library_symbol<decltype(&mj_makeData)>(owner->library, "mj_makeData");
  auto delete_data = library_symbol<decltype(&mj_deleteData)>(owner->library, "mj_deleteData");
  auto kinematics = library_symbol<decltype(&mj_kinematics)>(owner->library, "mj_kinematics");
  if (!make_data || !delete_data || !kinematics) return 8;
  auto* handle = new (std::nothrow)
      mjwarp_collision_inputs_data{owner, nullptr, kinematics, delete_data};
  if (!handle) return 8;
  handle->data = make_data(owner->model);
  if (!handle->data) {
    delete handle;
    return 8;
  }
  *output = handle;
  return 0;
}

extern "C" int32_t mjwarp_native_collision_inputs_set_qpos(
    const mjwarp_native_owner* owner, mjwarp_collision_inputs_data* data,
    const double* qpos, uint64_t nq) {
  if (!owner || !owner->model || !data || data->owner != owner || !data->data) return 8;
  if (owner->model->nq < 0) return 8;
  const uint64_t model_nq = static_cast<uint64_t>(owner->model->nq);
  if (nq != model_nq || !data->data->qpos) return 8;
  if (nq > SIZE_MAX / sizeof(double) || (nq > 0 && !qpos)) return 8;
  if (nq > 0) memcpy(data->data->qpos, qpos, static_cast<size_t>(nq) * sizeof(double));
  data->kinematics(owner->model, data->data);
  return 0;
}

extern "C" int32_t mjwarp_native_collision_inputs_copy_xpos_xmat(
    const mjwarp_native_owner* owner, const mjwarp_collision_inputs_data* data,
    float* geom_xpos, float* geom_xmat, uint64_t ngeom) {
  if (!owner || !owner->model || !data || data->owner != owner || !data->data) return 8;
  const mjModel* m = owner->model;
  const mjData* d = data->data;
  if (m->ngeom < 0) return 8;
  const uint64_t model_ngeom = static_cast<uint64_t>(m->ngeom);
  if (ngeom != model_ngeom) return 8;
  if (ngeom > SIZE_MAX / (3 * sizeof(double)) ||
      ngeom > SIZE_MAX / (9 * sizeof(double))) return 8;
  if (ngeom > 0 && (!geom_xpos || !geom_xmat)) return 8;
  if (ngeom > 0 && (!d->geom_xpos || !d->geom_xmat)) return 8;
  const size_t positions = static_cast<size_t>(ngeom) * 3;
  const size_t matrices = static_cast<size_t>(ngeom) * 9;
  for (size_t index = 0; index < positions; ++index) {
    geom_xpos[index] = static_cast<float>(d->geom_xpos[index]);
  }
  for (size_t index = 0; index < matrices; ++index) {
    geom_xmat[index] = static_cast<float>(d->geom_xmat[index]);
  }
  return 0;
}

extern "C" int32_t mjwarp_native_collision_inputs_destroy(
    const mjwarp_native_owner* owner, mjwarp_collision_inputs_data* data) {
  if (!data) return 0;
  // Defensive lifetime check: never delete mjData through a foreign owner.
  if (owner && data->owner != owner) return 8;
  if (data->data && data->delete_data) data->delete_data(data->data);
  data->data = nullptr;
  delete data;
  return 0;
}
