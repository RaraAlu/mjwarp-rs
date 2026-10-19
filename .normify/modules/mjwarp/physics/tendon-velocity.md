+++
uid = "adb9b6a2"
id = "mjwarp.physics.tendon-velocity"
tags = ["P4", "U020", "G02"]
fingerprint = "b8f64954ee28ec965c327e501645c1c5323bf3636455f75aebf0e3dd810827a9"

[name]
zh = "常驻肌腱速度消费者"
en = "Resident tendon velocity"

[description]
zh = """
直接借用全局雅可比。独占速度与结果。完整速度阶段仍缺。
"""
en = """
Actual resident global J consumer with owned qvel, not complete U020/G02.
"""

[[source]]
path = "src/physics/tendon_velocity.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonVelocityPlan"

[apis.description]
zh = """
实际设备雅可比消费者。完整速度阶段仍缺。
"""
en = """
Actual resident J and owned qvel consumer; no full U020/G02 admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonVelocityData"

[apis.description]
zh = """
实际设备雅可比消费者。完整速度阶段仍缺。
"""
en = """
Actual resident J and owned qvel consumer; no full U020/G02 admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonVelocityView"

[apis.description]
zh = """
实际设备雅可比消费者。完整速度阶段仍缺。
"""
en = """
Actual resident J and owned qvel consumer; no full U020/G02 admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::TendonVelocitySnapshot"

[apis.description]
zh = """
实际设备雅可比消费者。完整速度阶段仍缺。
"""
en = """
Actual resident J and owned qvel consumer; no full U020/G02 admission.
"""

[[deps]]
kind = "reference"
to = "mjwarp.physics.resident"
from_api = "rpc:mjwarp_rs::physics::TendonVelocityPlan"
to_api = "rpc:mjwarp_rs::physics::KinematicsPlan"

[deps.label]
zh = "借用实际常驻源"
en = "Borrow actual resident source"

[[deps]]
kind = "reference"
to = "mjwarp.model.tendon"
from_api = "rpc:mjwarp_rs::physics::TendonVelocityPlan"
to_api = "rpc:mjwarp_rs::model::TendonRows"

[deps.label]
zh = "读取全局稀疏行"
en = "Read global CSR rows"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::TendonVelocityPlan"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[deps.label]
zh = "提交同步内核"
en = "Launch synchronous kernel"
+++

实际resident J设备借用。独占qvel。视图锁定源与数据。快照只作历史。保持f32正零、CSR行序、J非零分支与原融合许可。新宿主两项通过。新GPU两项通过。合并预算2/3成功冻结。36批15600槽零位差。54项故障恢复。旧源19次更新只作同一新GPU命令的依赖。完整U020、G02、NP、一般空间J与完整生成入口仍缺。首次映射既有tendon叶子漏登记反向依赖。收尾预检发现环并拒绝该边。新独立叶子预检零错误零警告。登记晚于源码落地。保留流程偏差。原树UID、状态与主排程不替换。证据见C:/Rust/mjwarp-rs/docs/tendon-velocity.md。
