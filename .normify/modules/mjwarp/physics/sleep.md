+++
uid = "a362cc08"
id = "mjwarp.physics.sleep"
fingerprint = "8bd419b928c48ed0324d351e2b7dfb3e7b6762fd07e277e84fec1ae909c5cfbb"

[name]
zh = "肌腱唤醒子集"
en = "Tendon wake subset"

[description]
zh = """
GPU唤醒循环并刷新树状态。
"""
en = """
GPU tendon wake cycles and lightweight tree refresh.
"""

[[source]]
path = "src/physics/sleep.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SleepTreeOutput"

[apis.description]
zh = """
树状态快照。
"""
en = """
Owned tree state snapshot.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SleepTreeWorld"

[apis.description]
zh = """
单世界树状态。
"""
en = """
World tree state view.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++

## 2026-10-14 肌腱唤醒精度映射

本轮核对十八组内部量。
五项公开输出均采用i32。
限位距离仍采用f32。
树链接、最小值与计数采用i32。
原活动标记保留至扫描末尾。
复制、唤醒、交换路径已核对。
回读f64转换仅作诊断。
现有六份产品产物哈希已核。
独立新宿主目标一次四过。
通过后未重复执行。
本轮未发现产品公式修复依据。
上游sleep生成产物仍缺。
绑定常量实例化仍待取证。
并行语义及完整G25仍未验收。
本轮不启动GPU或CUDA编译器。
原容差、覆盖与21系数界限保持。
本模块保持活动态。
证据：C:/Rust/mjwarp-rs/docs/t4-tendon-wake-internal.md。
