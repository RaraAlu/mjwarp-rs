+++
uid = "6f2c1a9d"
id = "mjwarp.physics.gravity-compensation"
tags = ["G04", "U038", "NP01", "NP02", "cuda"]
fingerprint = "bd8a28eb32066e5b875aa2e2c1e796825bc8065fe294b1b1fc4076974bd4fb6c"

[name]
zh = "重力补偿"
en = "Gravity compensation"

[description]
zh = """
计算并合并刚体重力补偿。零与非零位形链均使用同状态实时产品结果。非零平面链1134项逐位一致，完整三维状态与NP03仍开放。
"""
en = """
Computes and composes rigid gravity compensation. Zero and nonzero qpos chains use live product results from the same state. The planar nonzero chain is bit-identical across 1,134 values; full 3D state coverage and NP03 remain open.
"""

[[source]]
path = "src/physics/gravity_compensation.rs"

[[source]]
path = "tests/t4_gravity_upstream_abi.rs"

[[source]]
path = "tests/t4_gravity_upstream_com.rs"

[[source]]
path = "fixtures/gravity-upstream-abi/package.json"

[[source]]
path = "fixtures/gravity-upstream-abi/independent-package.json"

[[source]]
path = "docs/gravity-compensation.md"

[[source]]
path = "docs/evidence/gravity-upstream-generated/verification.json"

[[source]]
path = "docs/evidence/gravity-upstream-com/verification.json"

[[source]]
path = "tests/t4_gravity_product_chain.rs"

[[source]]
path = "fixtures/gravity-upstream-abi/product-chain-package.json"

[[source]]
path = "docs/evidence/gravity-product-chain/verification.json"

[[source]]
path = "tests/t4_gravity_nonzero_product_chain.rs"

[[source]]
path = "fixtures/gravity-upstream-abi/nonzero-product-chain-package.json"

[[source]]
path = "docs/evidence/gravity-nonzero-product-chain/verification.json"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::GravityCompensationPlan"

[apis.description]
zh = """
建立重力补偿设备计划。
"""
en = """
Builds the gravity compensation device plan.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::GravityCompensationData"

[apis.description]
zh = """
持有分阶段补偿结果。
"""
en = """
Owns staged gravity compensation results.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::GravityCompensationSnapshot"

[apis.description]
zh = """
回读补偿自由度结果。
"""
en = """
Reads compensated DOF results.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::GravityCompensationView"

[apis.description]
zh = """
借用常驻运动学源。
"""
en = """
Borrows the resident kinematic source.
"""

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.resident"
from_api = "rpc:mjwarp_rs::physics::GravityCompensationPlan"

[deps.label]
zh = "借用刚体与质心结果"
en = "Borrow rigid and COM results"

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::physics::GravityCompensationPlan"

[deps.label]
zh = "读取参数与拓扑"
en = "Read parameters and topology"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::GravityCompensationPlan"

[deps.label]
zh = "启动补偿内核"
en = "Launch compensation kernels"

[[deps]]
kind = "reference"
to = "mjwarp.runtime.warp-com"
from_api = "rpc:mjwarp_rs::physics::GravityCompensationPlan"

[deps.label]
zh = "核对独立上游中间量"
en = "Check upstream inputs"
+++

实现冻结重力补偿公式。

参数保持独立周期。
刚体质量与补偿使用f32。
雅可比借用常驻设备结果。
最终按冻结顺序完成合成。

冻结Warp生成产物已入库。
Rust直接加载固定PTX。
真实上游入口已通过设备测试。
二十八项输出保持原容差。
候选链仍提供质心与自由度。
完整链仍不独立。
完整U038与NP仍开放。
