+++
uid = "3ce2a1b7"
id = "mjwarp.runtime.warp-kinematics"
tags = ["runtime", "U038", "NP01", "NP02", "cuda"]
fingerprint = "d107b3c8d02dcde6b6168b97e35304f5b397dca57320f9f82f9d9464ed180bff"

[name]
zh = "冻结Warp刚体运动学链"
en = "Frozen Warp rigid kinematics chain"

[description]
zh = """
核验冻结Warp刚体运动学产物。零与非零qpos均由冻结作者链生成质心前置量。产品构建与测试不依赖Python。
"""
en = """
Authenticates frozen Warp rigid kinematics artifacts. Frozen authoring produces the COM prerequisites for both zero and nonzero qpos. Product builds and tests do not depend on Python.
"""

[[source]]
path = "src/runtime/warp_kinematics.rs"

[[source]]
path = "tests/t4_gravity_upstream_kinematics.rs"

[[source]]
path = "fixtures/gravity-upstream-abi/independent-kinematics/receipt.json"

[[source]]
path = "docs/evidence/gravity-upstream-kinematics/verification.json"

[[source]]
path = "fixtures/gravity-upstream-abi/nonzero-kinematics/receipt.json"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpKinematicsArtifact"

[apis.description]
zh = """
核验并启动冻结刚体链。
"""
en = """
Authenticates and launches the frozen rigid-body chain.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpKinematicsInput"

[apis.description]
zh = """
承载固定模型与qpos。
"""
en = """
Carries the pinned model and qpos.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::FrozenWarpKinematicsReadback"

[apis.description]
zh = """
保留位姿与质心原始位型。
"""
en = """
Preserves raw pose and COM bit patterns.
"""

[[deps]]
kind = "call"
to = "mjwarp.runtime.driver"
from_api = "rpc:mjwarp_rs::runtime::FrozenWarpKinematicsArtifact"

[deps.label]
zh = "加载并启动固定PTX"
en = "Load pinned PTX"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::runtime::FrozenWarpKinematicsArtifact"

[deps.label]
zh = "报告产物与设备错误"
en = "Report errors"
+++
