// Include the immutable original bridge in the same translation unit.
// The extension reuses its private owner without exporting or guessing layout.
#include "model_probe.cpp"
#include "mjwarp_collision_probe.h"

static_assert(sizeof(mjwarp_primitive_geometry_targets) == 120);
static_assert(alignof(mjwarp_primitive_geometry_targets) == 8);
static_assert(offsetof(mjwarp_primitive_geometry_targets, ngeom) == 8);
static_assert(offsetof(mjwarp_primitive_geometry_targets, geom_type) == 16);
static_assert(offsetof(mjwarp_primitive_geometry_targets, geom_solmix) == 64);
static_assert(offsetof(mjwarp_primitive_geometry_targets, geom_gap) == 112);

extern "C" int32_t mjwarp_native_copy_primitive_geometry(
    const mjwarp_native_owner* owner, const mjwarp_primitive_geometry_targets* t) {
  if (!owner || !owner->model || !t || t->schema != 1 || t->reserved != 0) return 8;
  const mjModel* m = owner->model;
  if (m->ngeom < 0 || t->ngeom != static_cast<uint64_t>(m->ngeom) ||
      t->ngeom > SIZE_MAX / (5 * sizeof(double)) ||
      t->ngeom > static_cast<uint64_t>(INT32_MAX) / 5) return 8;
  if (!t->ngeom) return 0;
#define REQUIRED(field) if (!t->field || !m->field) return 8
  REQUIRED(geom_type); REQUIRED(geom_bodyid); REQUIRED(geom_contype);
  REQUIRED(geom_conaffinity); REQUIRED(geom_condim); REQUIRED(geom_priority);
  REQUIRED(geom_solmix); REQUIRED(geom_solref); REQUIRED(geom_solimp);
  REQUIRED(geom_size); REQUIRED(geom_friction); REQUIRED(geom_margin); REQUIRED(geom_gap);
#undef REQUIRED
  // No output writes occur until every field has passed preflight.
#define COPY(field, width) memcpy(t->field, m->field, t->ngeom * (width) * sizeof(*t->field))
  COPY(geom_type, 1); COPY(geom_bodyid, 1); COPY(geom_contype, 1);
  COPY(geom_conaffinity, 1); COPY(geom_condim, 1); COPY(geom_priority, 1);
  COPY(geom_solmix, 1); COPY(geom_solref, 2); COPY(geom_solimp, 5);
  COPY(geom_size, 3); COPY(geom_friction, 3); COPY(geom_margin, 1); COPY(geom_gap, 1);
#undef COPY
  return 0;
}
