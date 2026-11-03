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

// kind: 1 = int32, 2 = double, 3 = uint8, 4 = float32, 5 = int64. Count means
// scalar elements. The struct layout is fixed; only kind values were added.
// Field IDs 0..44 are the G01 input set; IDs 45..58 are the volume/SDF model
// fields (hfield, mesh octree, opt.sdf_*) consumed by src/io/volume.rs.
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

// R06-S2b: flex render data source (additive ABI; the flex_position structs above are unchanged).
// info is 32 B / targets is 72 B. Only five new fields plus three counts are exported:
// flex_dim (1/2/3), flex_radius (native mjtNum; the host converts once to f32),
// flex_rgba (native float x 4), flex_edgeadr, flex_edgenum. flex_edge stays in the G01
// field set and is not exported here (single source of truth).
typedef struct mjwarp_flex_render_info {
  uint32_t schema, reserved;
  int64_t nflex, nflexvert, nflexedge;
} mjwarp_flex_render_info;
typedef struct mjwarp_flex_render_targets {
  uint32_t schema, reserved;
  uint64_t nflex, nflexvert, nflexedge;
  int32_t* flex_dim;
  double* flex_radius;
  float* flex_rgba;
  int32_t* flex_edgeadr;
  int32_t* flex_edgenum;
} mjwarp_flex_render_targets;
// Independent count DTO; zero counts allow null targets.
int32_t mjwarp_native_flex_render_info(const mjwarp_native_owner* owner,
                                       mjwarp_flex_render_info* info);
// Checks all five sources/counts and required pointers before writing; targets
// must be disjoint and sized for their declared counts. Error code 8 = reject.
int32_t mjwarp_native_copy_flex_render(const mjwarp_native_owner* owner,
                                       const mjwarp_flex_render_targets* targets);

// R06-S2c: flex shell/elem topology (additive ABI; the flex_position / flex_render
// structs above are unchanged). info is 40 B / targets is 96 B. Seven new fields plus
// four counts are exported: flex_elemadr, flex_elemnum, flex_elemdataadr,
// flex_elem (nflexelemdata flat ids; dim+1 per element), flex_shellnum,
// flex_shelldataadr, flex_shell (nflexshelldata flat ids; dim per fragment).
// flex_vertadr / flex_vertnum stay in the flex_position bucket and are not duplicated.
typedef struct mjwarp_flex_topology_info {
  uint32_t schema, reserved;
  int64_t nflex, nflexelem, nflexelemdata, nflexshelldata;
} mjwarp_flex_topology_info;
typedef struct mjwarp_flex_topology_targets {
  uint32_t schema, reserved;
  uint64_t nflex, nflexelem, nflexelemdata, nflexshelldata;
  int32_t* flex_elemadr;
  int32_t* flex_elemnum;
  int32_t* flex_elemdataadr;
  int32_t* flex_elem;
  int32_t* flex_shellnum;
  int32_t* flex_shelldataadr;
  int32_t* flex_shell;
} mjwarp_flex_topology_targets;
// Independent count DTO; zero counts allow null targets.
int32_t mjwarp_native_flex_topology_info(const mjwarp_native_owner* owner,
                                         mjwarp_flex_topology_info* info);
// Checks all seven sources/counts and required pointers before writing; targets
// must be disjoint and sized for their declared counts. Error code 8 = reject.
int32_t mjwarp_native_copy_flex_topology(const mjwarp_native_owner* owner,
                                         const mjwarp_flex_topology_targets* targets);

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
// Render-model probe (S1) keeps its own schema and field IDs; the G01 ABI above
// is unchanged. Including it here keeps one native include root for probes.
#include "mjwarp_render_probe.h"
#endif
