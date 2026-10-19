+++
uid = "3b80fd65"
id = "mjwarp.physics.fluid-force"
tags = ["G04", "U038", "NP01", "NP02", "cuda", "fluid"]
fingerprint = "d1f4dcd4158a90d57ad97a547f4d3c6242d8e24a2bcf512f9d1085b5bb6c552e"

[name]
zh = "流体被动力"
en = "Fluid passive force"

[description]
zh = """
借用同源常驻位置、速度与可选附着位姿，在设备计算惯性盒或椭球流体力并投影广义力。独立入口现需可变速度状态；六类批量字段独立取模。共享入口按冻结f32顺序合并。
"""
en = """
Borrows same-source resident position, velocity, and optional attached frames to compute inertia-box or ellipsoid fluid forces on device and project generalized force. The standalone update now requires mutable velocity state; six batched fields keep independent periods. The shared path merges in frozen f32 order.
"""

[[source]]
path = "src/physics/fluid_force.rs"

[[source]]
path = "src/physics/com_velocity.rs"

[[source]]
path = "docs/fluid-force.md"

[[source]]
path = "docs/evidence/fluid-force/attempt-2-left-associative/audit.json"

[[source]]
path = "docs/evidence/fluid-force/ellipsoid-1/result.json"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FluidForcePlan"

[apis.description]
zh = """
建立惯性盒或椭球流体力计划。
"""
en = """
Builds the inertia-box or ellipsoid fluid force plan.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FluidForceData"

[apis.description]
zh = """
持有流体空间力与广义力。
"""
en = """
Owns fluid spatial and generalized force buffers.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FluidForceView"

[apis.description]
zh = """
借用同源速度结果。
"""
en = """
Borrows the same-source velocity result.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FluidForceSnapshot"

[apis.description]
zh = """
回读流体空间力与广义力。
"""
en = """
Reads fluid spatial and generalized forces.
"""

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.core"
from_api = "rpc:mjwarp_rs::physics::FluidForcePlan"
to_api = "rpc:mjwarp_rs::physics::smooth::com_vel"

[[deps]]
kind = "dataflow"
to = "mjwarp.physics.resident"
from_api = "rpc:mjwarp_rs::physics::FluidForcePlan"
to_api = "rpc:mjwarp_rs::physics::KinematicsPlan"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::FluidForcePlan"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"
+++

惯性盒与椭球分支均已实现。
共享入口现复用常驻运动学源。
它直接读取同源设备速度。
它不回传宿主中间量。
共享被动力最后加入流体力。
双禁用路径清零流体缓存。
异源与失效源都会拒绝。
设备失败会封锁公开结果。
清零工作量先检查边界。

定向GPU目标两次通过。
首次因NVRTC不可见失败。
新增纯宿主范围测试通过。
原容差仅作为契约记录。
共享测试使用按位组合比较。
它没有独立物理参考。
关节状态仍需调用方同步。
完整同代状态仍未实现。
冻结上游设备结果仍缺。
完整G04与U038仍未验收。


## 2026-10-15 椭球分支

椭球分支复用附着位姿。
几何三类输入已经接入。
六类批量字段独立取模。
常量与算序跟随冻结源码。
零负系数与小质量跳过。
输出不添加力臂力矩。
独立更新现需可变状态。
旧惯性盒定向回归通过。
共享流体定向回归通过。
椭球九世界比较288槽。
原容差保持2e-6。
最大误差为4.768e-7。
本轮没有捕获实际PTX。
上游设备输出仍未取得。
