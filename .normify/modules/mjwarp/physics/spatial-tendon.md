+++
uid = "a80c3152"
id = "mjwarp.physics.spatial-tendon"
fingerprint = "b8d1b15de4cf5356eedcf1a7647cdeb3a184c11ffd9fd59773828a1c4fa93187"

[name]
zh = "空间肌腱GPU子集"
en = "Spatial tendon GPU subset"

[description]
zh = """
三段GPU计算长度与力臂。内部暂存十三值。内侧方向先于接点舍入。公开结果仍用f32。
"""
en = """
Three GPU stages compute lengths, CSR moments and integer wraps. Private scratch holds 13 f32 values per record including pre-rounding inside directions; public physics outputs remain f32.
"""

[[source]]
path = "src/physics/spatial_tendon.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SpatialTendonOutput"

[apis.description]
zh = """
空间肌腱辅助接口。
"""
en = """
Spatial tendon helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SpatialTendonWorld"

[apis.description]
zh = """
空间肌腱辅助接口。
"""
en = """
Spatial tendon helper interface.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.spatial-tendon"
to_api = "rpc:mjwarp_rs::model::SpatialTendonRows"

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[[deps]]
kind = "reference"
to = "mjwarp.physics.tendon-wrap"
to_api = "rpc:mjwarp_rs::physics::tendon_wrap::TENDON_WRAP_CUDA"
+++

## 2026-10-14 site链与输出精度映射

本轮核对十六组局部关系。
六项输出分为f32与i32。
三类归一化保留不同分支。
微小方向采用X轴回退。
六来源与十一既存产物哈希已核。
产品site与上游几何各含四组库归约。
库级f64影响物理输出，不属诊断。
产品串行与上游原子累加仍待证。
球柱及内侧完整公式仍待独立审查。
三段同步设备顺序已核对。
新宿主目标一次四过，未重复。
本轮未发现必要产品公式修复依据。
本轮不运行CUDA编译器或GPU。
当前面捕获身份及旧失败保持。
原容差、覆盖与21系数界限保持。
本模块保持活动态。
完整G02仍有P2/P3/G01前置。
不能把前置阻塞当旧测试满额。
证据：C:/Rust/mjwarp-rs/docs/t4-spatial-tendon-internal.md。
