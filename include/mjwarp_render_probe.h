#ifndef MJWARP_RENDER_PROBE_H
#define MJWARP_RENDER_PROBE_H
#include "mjwarp_native_probe.h"
#ifdef __cplusplus
extern "C" {
#endif

// Render model probe ABI (S1). Independent schema and field IDs: the existing
// G01 / collision / flex / sparse ABIs and their semantics are unchanged.
//
// Field kinds: 1 = int32, 2 = double (mjtNum), 3 = uint8 (mjtByte/mjtBool),
// 4 = int64 (mjtSize), 5 = float32.
typedef struct mjwarp_render_field_info {
  uint32_t kind, reserved;
  int64_t count;
} mjwarp_render_field_info;

// Non-negative model counts used by the render field table. The C++ bridge
// rejects negative counts before returning any field description.
typedef struct mjwarp_render_info {
  uint32_t schema, reserved;
  int64_t nbody, njnt, ngeom, nsite, ncam, nlight, nmat, ntex, ntexdata;
  int64_t nmesh, nmeshvert, nmeshnormal, nmeshtexcoord, nmeshface, nmeshgraph;
  int64_t nmeshpoly, nmeshpolyvert, nmeshpolymap, nnames;
} mjwarp_render_info;

int32_t mjwarp_native_render_info(const mjwarp_native_owner* owner,
                                  mjwarp_render_info* info);
// Field description for one render field ID (no allocation, read-only).
int32_t mjwarp_native_render_field_info(const mjwarp_native_owner* owner,
                                        uint32_t id, mjwarp_render_field_info* info);
// Target must own count * scalar-width bytes for the kind returned above and
// cannot overlap native storage. All counts, kinds and source pointers are
// preflighted before the first byte is written.
int32_t mjwarp_native_copy_render_field(const mjwarp_native_owner* owner,
                                        uint32_t id, const mjwarp_render_field_info* info,
                                        void* target);

// Oracle for name-table semantics: resolves mj_id2name from the loaded MuJoCo
// library and compares against the copied name tables on the Rust side.
// objtype uses the mjOBJ_* enum values of the loaded library.
// Returns 0 when a NUL-terminated name was copied (length without NUL),
// 1 when mj_id2name returned null, 2 when capacity is too small (length is still
// reported), 8 on invalid arguments.
int32_t mjwarp_native_render_name(const mjwarp_native_owner* owner, int32_t objtype, int64_t id,
                                  char* buffer, int64_t capacity, int64_t* length);
#ifdef __cplusplus
}
#endif
#endif
