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
#ifdef __cplusplus
}
#endif
#endif
