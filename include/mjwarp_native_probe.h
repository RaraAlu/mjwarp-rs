#ifndef MJWARP_NATIVE_PROBE_H
#define MJWARP_NATIVE_PROBE_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct mjwarp_native_info {
  uint32_t schema;
  int32_t native_version;
  uint32_t pointer_bytes, num_bytes, index_bytes, size_bytes;
  uint64_t native_model_bytes;
  int64_t nq, nv, nu, na, nbody, njnt, ngeom, nsensordata;
} mjwarp_native_info;
typedef struct mjwarp_native_owner mjwarp_native_owner;

typedef struct mjwarp_kinematic_targets {
  uint32_t schema, reserved;
  uint64_t nq, nbody, njnt;
  double* qpos0;
  int32_t* body_parentid;
  int32_t* body_jntadr;
  int32_t* body_jntnum;
  double* body_pos;
  double* body_quat;
  int32_t* jnt_type;
  int32_t* jnt_bodyid;
  int32_t* jnt_qposadr;
  int32_t* jnt_dofadr;
  double* jnt_pos;
  double* jnt_axis;
} mjwarp_kinematic_targets;

// Trusted DLL and MJB only. Never accepts a borrowed mjModel pointer.
int32_t mjwarp_native_open(const uint16_t* dll_path, const void* mjb, int32_t bytes,
                          mjwarp_native_owner** owner, mjwarp_native_info* info,
                          uint32_t* detail);
void mjwarp_native_close(mjwarp_native_owner* owner);
// Checks all lengths and pointers before writing any output.
int32_t mjwarp_native_copy(const mjwarp_native_owner* owner,
                          double* qpos0, uint64_t nq,
                          double* body_mass, int32_t* body_parentid, uint64_t nbody,
                          int32_t* jnt_type, uint64_t njnt);
// All targets must be disjoint and large enough for their declared counts.
// Checks every count and required pointer before writing any output.
int32_t mjwarp_native_copy_kinematic(const mjwarp_native_owner* owner,
                                    const mjwarp_kinematic_targets* targets);
#ifdef __cplusplus
}
#endif
#endif
