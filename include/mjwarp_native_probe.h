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

// kind: 1 = int32, 2 = double, 3 = uint8. Count means scalar elements.
typedef struct mjwarp_g01_field_info {
  uint32_t kind, reserved;
  int64_t count;
} mjwarp_g01_field_info;
int32_t mjwarp_native_g01_field_info(const mjwarp_native_owner* owner,
                                   uint32_t id, mjwarp_g01_field_info* info);
// Target must own count * scalar-width bytes and cannot overlap native storage.
// Preflight rejects wrong kinds, counts, IDs and required null pointers.
int32_t mjwarp_native_copy_g01_field(const mjwarp_native_owner* owner,
                                   uint32_t id, const mjwarp_g01_field_info* info,
                                   void* target);

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
typedef struct mjwarp_inertial_targets {
  uint32_t schema, reserved;
  uint64_t nbody, nv;
  double* body_ipos;
  double* body_iquat;
  double* body_mass;
  double* body_inertia;
  int32_t* dof_bodyid;
  int32_t* dof_jntid;
  int32_t* dof_parentid;
  double* dof_armature;
  double* dof_damping;
} mjwarp_inertial_targets;

typedef struct mjwarp_flex_position_info {
  uint32_t schema, reserved;
  int64_t nflex, nflexnode, nflexvert;
} mjwarp_flex_position_info;
typedef struct mjwarp_flex_position_targets {
  uint32_t schema, reserved;
  uint64_t nflex, nflexnode, nflexvert;
  int32_t* flex_interp;
  int32_t* flex_cellnum;
  int32_t* flex_nodeadr;
  int32_t* flex_nodenum;
  int32_t* flex_vertadr;
  int32_t* flex_vertnum;
  uint8_t* flex_centered;
  int32_t* flex_nodebodyid;
  int32_t* flex_vertbodyid;
  double* flex_node;
  double* flex_vert;
  double* flex_vert0;
} mjwarp_flex_position_targets;

// Windows uses UTF-16; Linux uses native path bytes. Both are NUL-terminated.
#ifdef _WIN32
typedef uint16_t mjwarp_native_path_char;
#else
typedef char mjwarp_native_path_char;
#endif
// Trusted shared library and MJB only. Never accepts a borrowed mjModel pointer.
int32_t mjwarp_native_open(const mjwarp_native_path_char* library_path, const void* mjb, int32_t bytes,
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
// Same disjoint-buffer contract; preflights all nine sources and targets.
int32_t mjwarp_native_copy_inertial(const mjwarp_native_owner* owner,
                                   const mjwarp_inertial_targets* targets);
// Independent count DTO preserves the existing core-info ABI.
int32_t mjwarp_native_flex_position_info(const mjwarp_native_owner* owner,
                                        mjwarp_flex_position_info* info);
// Checks all twelve sources and targets before writing; zero counts allow null.
// Targets must be disjoint and sized for their declared counts.
int32_t mjwarp_native_copy_flex_position(const mjwarp_native_owner* owner,
                                        const mjwarp_flex_position_targets* targets);
#ifdef __cplusplus
}
#endif
#endif
