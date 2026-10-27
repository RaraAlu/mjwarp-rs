#ifndef MJWARP_COLLISION_INPUTS_PROBE_H
#define MJWARP_COLLISION_INPUTS_PROBE_H
#include "mjwarp_native_probe.h"
#ifdef __cplusplus
extern "C" {
#endif

// Collision-inputs snapshot probe (S1). Independent schema and field IDs: the
// existing G01 / collision / flex / sparse / render ABIs and their semantics are
// unchanged, and no product wiring is added here.
//
// Field IDs are 1:1 with the 32 host slices of
// src/runtime/collision_gpu/types.rs::CollisionGpuInputs (23 existing + the 9
// optional mesh_poly inputs). Field order is shared with
// src/model/collision_inputs.rs::COLLISION_INPUTS_FIELDS.
//
// Field kinds in this schema: 1 = int32, 2 = double (mjtNum), 4 = float32.
typedef struct mjwarp_collision_inputs_field_info {
  uint32_t kind, reserved;
  int64_t count;
} mjwarp_collision_inputs_field_info;

// Non-negative model counts used by the 32-field table. The bridge rejects
// negative counts before returning any field description.
typedef struct mjwarp_collision_inputs_info {
  uint32_t schema, reserved;
  int64_t nq, ngeom, nmesh, nmeshvert, nmeshgraph, nmeshpoly, nmeshpolyvert,
      nmeshpolymap, nhfield, nhfielddata, noct;
} mjwarp_collision_inputs_info;

int32_t mjwarp_native_collision_inputs_info(const mjwarp_native_owner* owner,
                                            mjwarp_collision_inputs_info* info);
// Field description for one collision-inputs field ID (no allocation, read-only).
int32_t mjwarp_native_collision_inputs_field_info(
    const mjwarp_native_owner* owner, uint32_t id,
    mjwarp_collision_inputs_field_info* info);
// Copies one model-sourced field (IDs 2..31). IDs 0 and 1 are mjData fields
// (geom_xpos/geom_xmat) and are served by the data-snapshot entry below; the
// model copy rejects them with 8. The target must own count * scalar-width bytes
// and must not overlap native storage. Kind, count, ID, source and target are
// all preflighted before the first byte is written.
int32_t mjwarp_native_copy_collision_inputs_field(
    const mjwarp_native_owner* owner, uint32_t id,
    const mjwarp_collision_inputs_field_info* info, void* target);

// Independent mjData handle for geom_xpos/geom_xmat. The caller passes qpos
// (nq doubles), and the bridge runs mj_kinematics before copying the Cartesian
// geom frames as float32. The mjData object is created once and reused: only
// kinematics is re-run per set_qpos call. The handle must be destroyed before
// the owner is closed; destroy checks the owner and refuses foreign handles.
typedef struct mjwarp_collision_inputs_data mjwarp_collision_inputs_data;
int32_t mjwarp_native_collision_inputs_create(const mjwarp_native_owner* owner,
                                              mjwarp_collision_inputs_data** output);
int32_t mjwarp_native_collision_inputs_set_qpos(const mjwarp_native_owner* owner,
                                                mjwarp_collision_inputs_data* data,
                                                const double* qpos, uint64_t nq);
// Copies d->geom_xpos (ngeom x 3) and d->geom_xmat (ngeom x 9) as float32.
// Both targets are preflighted before any conversion or write.
int32_t mjwarp_native_collision_inputs_copy_xpos_xmat(
    const mjwarp_native_owner* owner, const mjwarp_collision_inputs_data* data,
    float* geom_xpos, float* geom_xmat, uint64_t ngeom);
// Returns 8 and deletes nothing when the owner does not own the handle.
int32_t mjwarp_native_collision_inputs_destroy(const mjwarp_native_owner* owner,
                                               mjwarp_collision_inputs_data* data);

#ifdef __cplusplus
}
#endif
#endif
