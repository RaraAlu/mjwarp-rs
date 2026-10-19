+++
uid = "b59e0d41"
id = "mjwarp.runtime.warp-gravity"
tags = ["runtime", "U038", "NP01", "NP02", "cuda"]
fingerprint = "acfd5de628018c05c6236a43c2dd568b864421e1256e8406588ba2fbc015193b"

[name]
zh = "冻结Warp重力导入"
en = "Frozen Warp gravity import"

[description]
zh = """
严格核验冻结Warp重力产物，直接加载PTX，并保留原始设备位型。
"""
en = """
Strictly authenticates a frozen Warp gravity artifact, loads PTX directly, and preserves raw device bit patterns.
"""

[[source]]
path = "src/runtime/warp_gravity.rs"

[[source]]
path = "tests/t4_gravity_upstream_abi.rs"

[[source]]
path = "fixtures/gravity-upstream-abi/package.json"

[[source]]
path = "fixtures/gravity-upstream-abi/generated/receipt.json"

[[source]]
path = "docs/evidence/gravity-upstream-generated/verification.json"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpGravityArtifact"

[apis.description]
zh = """
核验并加载冻结重力产物。
"""
en = """
Authenticates and loads the frozen gravity artifact.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpGravityInput"

[apis.description]
zh = """
承载十一项冻结输入字段。
"""
en = """
Carries the eleven frozen input fields.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpGravityReceipt"

[apis.description]
zh = """
暴露固定生成来源回执。
"""
en = """
Exposes the pinned generation provenance receipt.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpGravityReadback"

[apis.description]
zh = """
先保留原始位型，再执行有限值门禁。
"""
en = """
Preserves raw bits before enforcing the finite-value gate.
"""

[[deps]]
kind = "call"
to = "mjwarp.runtime.driver"
from_api = "rpc:mjwarp_rs::runtime::FrozenWarpGravityArtifact"

[deps.label]
zh = "加载并启动固定PTX"
en = "Load pinned PTX"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::runtime::FrozenWarpGravityArtifact"

[deps.label]
zh = "报告产物与设备错误"
en = "Report errors"
+++

## 当前范围

固定上游提交与Warp版本。
固定CUDA、PTX与生成回执哈希。
Rust直接加载真实PTX。
设备回读先保存原始位型。
有限值门禁随后拒绝异常。

真实生成核已通过定向设备测试。
候选链仍提供质心与自由度。
完整链仍不独立。
完整NP01至NP04仍开放。
