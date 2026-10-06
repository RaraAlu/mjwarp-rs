#ifndef MJWARP_PROBE_H
#define MJWARP_PROBE_H

#include <stdint.h>

/* P1 x64 data layout only. This header exports no engine functions.
 * Addresses and context handles are process-local CUDA values, not host pointers.
 * Byte ranges must be contiguous f32 and belong to a live legacy cuMemAlloc.
 * Ownership, exclusive access and a recorded producer event require a separate
 * unsafe handoff contract. Validating this descriptor cannot establish them.
 * This is not a serialized format or a stable production tensor ABI. */
#define MJWARP_EXTERNAL_PROBE_ABI_VERSION 1u
#define MJWARP_EXTERNAL_PROBE_F32 1u
#define MJWARP_EXTERNAL_PROBE_WRITABLE 1u

typedef struct mjwarp_external_buffer_descriptor {
    uint32_t abi_version;
    uint32_t struct_size;
    uint32_t device_ordinal;
    uint32_t flags;
    uint64_t context;
    uint64_t allocation;
    uint64_t allocation_bytes;
    uint64_t offset_bytes;
    uint64_t elements;
    uint64_t stride_bytes;
    uint64_t layout_version;
    uint32_t scalar_type;
    uint32_t rank;
    uint64_t reserved[2];
} mjwarp_external_buffer_descriptor;

#endif
