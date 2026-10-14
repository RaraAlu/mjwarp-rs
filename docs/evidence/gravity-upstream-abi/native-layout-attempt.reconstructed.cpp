// 该文件只重建两次失败探针的输入意图。
// 原临时源码未保留，不能宣称逐字一致。
#include "array.h"

#include <cstddef>

static_assert(sizeof(wp::array_t<wp::float32>) == 56);
static_assert(alignof(wp::array_t<wp::float32>) == 8);
static_assert(offsetof(wp::array_t<wp::float32>, data) == 0);
static_assert(offsetof(wp::array_t<wp::float32>, grad) == 8);
static_assert(offsetof(wp::array_t<wp::float32>, shape) == 16);
static_assert(offsetof(wp::array_t<wp::float32>, strides) == 32);
static_assert(offsetof(wp::array_t<wp::float32>, ndim) == 48);
static_assert(offsetof(wp::array_t<wp::float32>, flags) == 50);
static_assert(sizeof(wp::launch_bounds_t<3>) == 32);
static_assert(alignof(wp::launch_bounds_t<3>) == 8);
static_assert(sizeof(wp::vec_t<3, wp::float32>) == 12);
static_assert(sizeof(wp::vec_t<6, wp::float32>) == 24);

int main() { return 0; }
