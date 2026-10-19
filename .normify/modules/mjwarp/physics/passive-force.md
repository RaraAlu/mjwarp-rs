+++
uid = "a38c1e7b"
id = "mjwarp.physics.passive-force"
tags = ["G04", "U038", "NP01", "NP02", "cuda"]
fingerprint = "10ca7cdc963a3cc0040fc7451bedf8ab93037a19659a97a644284966f39677cd"

[name]
zh = "被动力共享累加"
en = "Passive force accumulation"

[description]
zh = """
按冻结顺序合并关节、肌腱、柔体边、重力补偿与流体力。复用同一常驻运动学源，并过滤休眠自由度。保留f32分组、双禁用清零与失败封锁。
"""
en = """
Combines joint, tendon, flex-edge, gravity-compensation, and fluid forces in frozen order. It reuses one resident kinematics source, filters sleeping DOFs, and preserves f32 grouping, dual-disabled clearing, and failed-publication blocking.
"""

[[source]]
path = "src/physics/passive_force.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::PassiveForcePlan"

[apis.description]
zh = """
建立共享被动力累加计划。
"""
en = """
Builds the passive accumulation plan.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::PassiveForceData"

[apis.description]
zh = """
持有共享分量与平滑结果。
"""
en = """
Owns shared passive parts and smooth results.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::PassiveForceSnapshot"

[apis.description]
zh = """
回读共享被动力分量。
"""
en = """
Reads passive force components.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::PassiveForceView"

[apis.description]
zh = """
借用全部活动生产者。
"""
en = """
Borrows all active producers.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::ClearedPassiveForceView"

[apis.description]
zh = """
读取双禁用冷清零结果。
"""
en = """
Reads dual-disabled cold clear.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SleepClearedPassiveForceView"

[apis.description]
zh = """
借用树状态读取休眠结果。
"""
en = """
Borrows tree state for sleep merge.
"""

[[deps]]
kind = "reference"
to = "mjwarp.physics.core"
from_api = "rpc:mjwarp_rs::physics::PassiveForcePlan"

[deps.label]
zh = "复用关节与平滑生产者"
en = "Reuse joint and smooth"

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.tendon-passive"
from_api = "rpc:mjwarp_rs::physics::PassiveForcePlan"

[deps.label]
zh = "累加肌腱贡献"
en = "Accumulate tendon"

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.flex-edge-passive"
from_api = "rpc:mjwarp_rs::physics::PassiveForcePlan"

[deps.label]
zh = "累加柔体边贡献"
en = "Accumulate flex edge"

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.gravity-compensation"
from_api = "rpc:mjwarp_rs::physics::PassiveForcePlan"

[deps.label]
zh = "合并重力补偿"
en = "Merge gravity compensation"

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.fluid-force"
from_api = "rpc:mjwarp_rs::physics::PassiveForcePlan"
to_api = "rpc:mjwarp_rs::physics::FluidForcePlan"

[deps.label]
zh = "合并同源流体力"
en = "Merge same-source fluid force"

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.resident"
from_api = "rpc:mjwarp_rs::physics::PassiveForcePlan"
to_api = "rpc:mjwarp_rs::physics::KinematicsData"

[deps.label]
zh = "借用设备树活动状态"
en = "Borrow tree state"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::PassiveForcePlan"

[deps.label]
zh = "执行共享缓冲内核"
en = "Launch shared kernels"
+++

实现真实共享累加链。

关节先直接写入。
肌腱随后原子累加。
柔体边最后原子累加。
弹簧与阻尼先合并。
允许的重力随后加入。
流体力最后加入。
流体复用同一常驻运动学源。
链路不打包宿主中间字段。
休眠自由度写入正零。
双禁用同步清零全部分量。
设备失败会封锁公开结果。
修复输入后可以恢复。

新增GPU目标两次通过。
首次因NVRTC不可见失败。
失败未进入物理执行。
五个活动世界完成按位核对。
两个休眠世界完成过滤核对。
原容差仅作为契约记录。
共享测试没有独立物理参考。
关节状态仍由调用方同步。
代码不核对副本值一致。
完整同代状态仍未实现。
上游设备静态结果仍缺。

性能与完整G04仍未验收。
完整U038与NP01至NP04仍缺。


## 2026-10-15 椭球接线边界

共享计划可接收椭球输入。
它借用同源附着位姿。
旧惯性盒共享回归通过。
共享椭球尚未定向实测。
该缺口不影响独立实测。
