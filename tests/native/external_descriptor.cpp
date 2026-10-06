#include "mjwarp_probe.h"
#include <cstddef>
#include <cstdio>
#include <type_traits>

using Descriptor = mjwarp_external_buffer_descriptor;
static_assert(std::is_standard_layout_v<Descriptor>);
static_assert(sizeof(Descriptor) == 96);
static_assert(alignof(Descriptor) == 8);
static_assert(offsetof(Descriptor, abi_version) == 0);
static_assert(offsetof(Descriptor, struct_size) == 4);
static_assert(offsetof(Descriptor, device_ordinal) == 8);
static_assert(offsetof(Descriptor, flags) == 12);
static_assert(offsetof(Descriptor, context) == 16);
static_assert(offsetof(Descriptor, allocation) == 24);
static_assert(offsetof(Descriptor, allocation_bytes) == 32);
static_assert(offsetof(Descriptor, offset_bytes) == 40);
static_assert(offsetof(Descriptor, elements) == 48);
static_assert(offsetof(Descriptor, stride_bytes) == 56);
static_assert(offsetof(Descriptor, layout_version) == 64);
static_assert(offsetof(Descriptor, scalar_type) == 72);
static_assert(offsetof(Descriptor, rank) == 76);
static_assert(offsetof(Descriptor, reserved) == 80);
static_assert(MJWARP_EXTERNAL_PROBE_ABI_VERSION == 1);
static_assert(MJWARP_EXTERNAL_PROBE_F32 == 1);
static_assert(MJWARP_EXTERNAL_PROBE_WRITABLE == 1);

int main() {
    std::puts("descriptor_size=96; alignment=8; offsets=pass; constants=pass");
    return 0;
}
