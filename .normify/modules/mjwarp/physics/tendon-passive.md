+++
uid = "f201cefe"
id = "mjwarp.physics.tendon-passive"
tags = ["P4", "U038", "G04"]
fingerprint = "099a9c1749cf1cb3290a59212816a406b0cd3434cd528470b19fa54a09faa653"

[name]
zh = "常驻肌腱被动力"
en = "Resident tendon passive force"

[description]
zh = """
直接消费常驻长度、雅可比与设备速度。完整G04仍缺。
"""
en = """
Direct resident length, Jacobian, and device velocity consumer; not complete G04.
"""

[[source]]
path = "src/physics/tendon_passive.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonPassivePlan"

[apis.description]
zh = """
建立肌腱被动力计划。
"""
en = """
Build tendon passive force plan.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonPassiveData"

[apis.description]
zh = """
持有肌腱被动力设备结果。
"""
en = """
Own tendon passive device results.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonPassiveView"

[apis.description]
zh = """
锁定本次输入与结果。
"""
en = """
Bind current inputs and results.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonPassiveSnapshot"

[apis.description]
zh = """
回读弹簧与阻尼力。
"""
en = """
Read spring and damper snapshots.
"""

[[deps]]
kind = "reference"
to = "mjwarp.physics.resident"
from_api = "rpc:mjwarp_rs::physics::TendonPassivePlan"
to_api = "rpc:mjwarp_rs::physics::KinematicsPlan"

[deps.label]
zh = "借用长度与雅可比"
en = "Borrow length and Jacobian"

[[deps]]
kind = "reference"
to = "mjwarp.physics.tendon-velocity"
from_api = "rpc:mjwarp_rs::physics::TendonPassivePlan"
to_api = "rpc:mjwarp_rs::physics::TendonVelocityView"

[deps.label]
zh = "借用设备速度"
en = "Borrow device velocity"

[[deps]]
kind = "reference"
to = "mjwarp.model.parameters"
from_api = "rpc:mjwarp_rs::physics::TendonPassivePlan"
to_api = "rpc:mjwarp_rs::model::ParameterBatch"

[deps.label]
zh = "读取独立参数周期"
en = "Read parameter periods"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::TendonPassivePlan"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[deps.label]
zh = "提交同步内核"
en = "Launch sync kernel"
+++

实现U038肌腱被动力局部切片。直接消费实际常驻长度、全局雅可比与设备肌腱速度。五类ParameterBatch保持独立周期。全链保持f32，索引保持i32。死区、禁用、多项式次序与atomicAdd遵循冻结基线。首轮完成260世界与1560槽固定参数子集零位差。续查发现验证器错误读取禁用、零系数与空CSR行输入；计算核也在无力判断前形成J读取。现按冻结顺序先判有力，再判空行，最后读取J、长度与速度。提交后条件读取测试一次通过，但对应空行修复前源码；最终源码另以新窄测试验证11世界、2/3/5/7/11五类变化周期与11条空行非有限输入，1项通过，旧成功选择器未重跑。最终测试1.80秒，编译18.96秒，过滤307项。两新计划自有容量2540字节，不含生产者和模块，非峰值。PTX精度门禁通过，但未归档CU/PTX，产物谱系仍缺。一般原子重叠误差仍未验收。未运行满额、Python、Clippy、Linux或发布模式。完整U038、G04与NP01至NP04保持开放。证据见C:/Rust/mjwarp-rs/docs/tendon-passive.md。
