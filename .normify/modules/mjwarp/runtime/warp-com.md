+++
uid = "9f2c7a31"
id = "mjwarp.runtime.warp-com"
tags = ["runtime", "U038", "NP01", "NP02", "cuda"]
fingerprint = "c053a11aebff8bf44183ed94296cd4c94b8719ef555b652c75af84b8f4e1e226"

[name]
zh = "冻结Warp质心输入"
en = "Frozen Warp COM inputs"

[description]
zh = """
核验冻结Warp质心产物，保留独立上游中间量位型，并拒绝候选反馈。
"""
en = """
Authenticates frozen Warp COM artifacts, preserves independent upstream intermediate bit patterns, and rejects candidate feedback.
"""

[[source]]
path = "src/runtime/warp_com.rs"

[[source]]
path = "tests/t4_gravity_upstream_com.rs"

[[source]]
path = "fixtures/gravity-upstream-abi/independent-inputs/receipt.json"

[[source]]
path = "docs/evidence/gravity-upstream-com/verification.json"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpComArtifact"

[apis.description]
zh = """
核验并加载冻结质心产物。
"""
en = """
Authenticates and loads frozen COM artifacts.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpComInput"

[apis.description]
zh = """
承载独立质心输入字段。
"""
en = """
Carries independent COM input fields.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpComOutput"

[apis.description]
zh = """
返回有限质心与自由度值。
"""
en = """
Returns finite COM and DOF values.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpComReadback"

[apis.description]
zh = """
先保留原始位型。
"""
en = """
Preserves raw bit patterns first.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpComReceipt"

[apis.description]
zh = """
暴露固定作者来源。
"""
en = """
Exposes pinned authoring provenance.
"""

[[deps]]
kind = "call"
to = "mjwarp.runtime.driver"
from_api = "rpc:mjwarp_rs::runtime::FrozenWarpComArtifact"

[deps.label]
zh = "加载并启动固定PTX"
en = "Load pinned PTX"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::runtime::FrozenWarpComArtifact"

[deps.label]
zh = "报告产物与设备错误"
en = "Report errors"
+++

## 当前范围

固定上游Smooth生成产物。
独立生成subtree_com与cdof。
作者链不读取候选中间量。
原始位型先于有限值门禁。
静态运动学输入由作者给定。
完整qpos链仍不独立。
完整NP01至NP04仍开放。
