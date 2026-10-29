+++
uid = "63873c25"
id = "mjwarp.runtime.collision-gpu"
tags = ["implemented-subset", "gpu", "collision", "windows"]
fingerprint = "0000000000000000000000000000000000000000000000000000000000000000"

[name]
zh = "GPU 碰撞运行时"
en = "GPU collision runtime"

[description]
zh = """
GPU 碰撞运行时。产品入口 run_product_collision、32 项宿主输入 ABI 校验、输入快照转换、上下文/回退/错误分类、NVRTC 源码与校验测试。代表实现子集，不等于完整碰撞验收。
"""
en = """
GPU collision runtime: the run_product_collision product entry, 32-item host input ABI validation, input snapshot conversion, context/fallback/error classification, NVRTC sources and validation tests. Implemented subset, not full collision acceptance.
"""

[[source]]
path = "src/runtime/collision_gpu.rs"

[[source]]
path = "src/runtime/collision_gpu/abi.rs"

[[source]]
path = "src/runtime/collision_gpu/abi_tests.rs"

[[source]]
path = "src/runtime/collision_gpu/buffers.rs"

[[source]]
path = "src/runtime/collision_gpu/context.rs"

[[source]]
path = "src/runtime/collision_gpu/error.rs"

[[source]]
path = "src/runtime/collision_gpu/fallback_tests.rs"

[[source]]
path = "src/runtime/collision_gpu/inputs_from_snapshot.rs"

[[source]]
path = "src/runtime/collision_gpu/multicontact_tests.rs"

[[source]]
path = "src/runtime/collision_gpu/nvrtc_smoke.rs"

[[source]]
path = "src/runtime/collision_gpu/primitive_tests.rs"

[[source]]
path = "src/runtime/collision_gpu/product_entry.rs"

[[source]]
path = "src/runtime/collision_gpu/product_entry_tests.rs"

[[source]]
path = "src/runtime/collision_gpu/source.rs"

[[source]]
path = "src/runtime/collision_gpu/tests.rs"

[[source]]
path = "src/runtime/collision_gpu/types.rs"

[[source]]
path = "src/runtime/collision_gpu/validate.rs"

[[source]]
path = "src/runtime/collision_gpu/validate_mesh_poly_tests.rs"

[[source]]
path = "src/runtime/collision_gpu/validate_tests.rs"

[[source]]
path = "src/runtime/collision_upload.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::collision_gpu::run_product_collision"

[apis.description]
zh = """
已实现的产品入口。接收 ProductCollisionRequest，经 32 项宿主输入校验后提交 GPU 碰撞内核并返回 ProductCollisionOutcome；失败按 ProductHostReason 分类，回退与上下文生命周期由本模块维护。
"""
en = """
Implemented product entry: validates 32 host inputs, submits the GPU collision kernel and returns ProductCollisionOutcome with ProductHostReason; fallback and context lifecycle stay here.
"""
+++
