#ifndef MJWARP_COLLISION_PROBE_H
#define MJWARP_COLLISION_PROBE_H
#include "mjwarp_native_probe.h"
#ifdef __cplusplus
extern "C" {
#endif

// Independent schema; existing G01 field IDs and ABI remain unchanged.
typedef struct mjwarp_primitive_geometry_targets {
  uint32_t schema, reserved;
  uint64_t ngeom;
  int32_t *geom_type, *geom_bodyid, *geom_contype, *geom_conaffinity;
  int32_t *geom_condim, *geom_priority;
  double *geom_solmix, *geom_solref, *geom_solimp, *geom_size;
  double *geom_friction, *geom_margin, *geom_gap;
} mjwarp_primitive_geometry_targets;

// The caller owns disjoint, aligned arrays with ngeom * field-width elements.
// Nonempty arrays must not be null or overlap native storage.
// All counts and required pointers are checked before any output write.
int32_t mjwarp_native_copy_primitive_geometry(
    const mjwarp_native_owner* owner, const mjwarp_primitive_geometry_targets* targets);

#ifdef __cplusplus
}
#endif
#endif
