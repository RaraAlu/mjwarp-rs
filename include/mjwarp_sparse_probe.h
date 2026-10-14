#ifndef MJWARP_SPARSE_PROBE_H
#define MJWARP_SPARSE_PROBE_H
#include "mjwarp_native_probe.h"
#ifdef __cplusplus
extern "C" {
#endif

// Independent schema, without changing the original native/G01 ABI.
typedef struct mjwarp_sparse_mass_info {
  uint32_t schema, reserved;
  uint64_t nv, nC, ntree;
} mjwarp_sparse_mass_info;
typedef struct mjwarp_sparse_mass_targets {
  mjwarp_sparse_mass_info info;
  int32_t *dof_parentid, *M_rownnz, *M_rowadr, *M_colind;
  int32_t *tree_dofadr, *tree_dofnum;
} mjwarp_sparse_mass_targets;

// Immutable owner must remain alive. No output writes occur on preflight failure.
int32_t mjwarp_native_sparse_mass_info(const mjwarp_native_owner* owner,
                                     mjwarp_sparse_mass_info* info);
// Caller owns disjoint, aligned arrays: nv, nv, nv, nC, ntree, ntree elements.
// Arrays must not overlap native storage. Zero-length pointers are never accessed.
int32_t mjwarp_native_copy_sparse_mass(const mjwarp_native_owner* owner,
                                     const mjwarp_sparse_mass_targets* targets);
#ifdef __cplusplus
}
#endif
#endif
