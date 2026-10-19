+++
uid = "af628ce1"
id = "mjwarp.model.inertial"
fingerprint = "0e1966eeda5d18b0c5c2dce92d995d756cdfd95e3d251f1d16aa16ad089b1e0e"

[name]
zh = "惯性字段与自由度链"
en = "Inertial fields and DOF ancestry"

[description]
zh = """
校验九项惯性字段。组合只读运动学子集。
"""
en = """
Nine-field inertial helper with exact DOF ancestry, composed with the kinematic subset; not a complete production model.
"""

[[source]]
path = "src/model/inertial.rs"

[[source]]
path = "src/model/inertial/sparse.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::InertialFields"

[apis.description]
zh = """
九项未检查惯性字段。
"""
en = """
Raw nine-field f32/i32 inertial subset.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::InertialModelInput"

[apis.description]
zh = """
组合并校验二十一字段。仅导出只读借用。
"""
en = """
Validated immutable 21-field shared parameter subset; not full ModelInput.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SparseMassFields"

[apis.description]
zh = """
原生CSR与冻结分块分层元数据。大块只接受完整祖先前缀。
"""
en = """
Native CSR and frozen block/layer metadata; oversized sparse rows require full ancestor prefixes.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SparseMassInput"

[apis.description]
zh = """
原生CSR与冻结分块分层元数据。大块只接受完整祖先前缀。
"""
en = """
Native CSR and frozen block/layer metadata; oversized sparse rows require full ancestor prefixes.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SparseMassBlock"

[apis.description]
zh = """
原生CSR与冻结分块分层元数据。大块只接受完整祖先前缀。
"""
en = """
Native CSR and frozen block/layer metadata; oversized sparse rows require full ancestor prefixes.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SparseMassRoute"

[apis.description]
zh = """
原生CSR与冻结分块分层元数据。大块只接受完整祖先前缀。
"""
en = """
Native CSR and frozen block/layer metadata; oversized sparse rows require full ancestor prefixes.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SparseMassInput::dense_block_sources"

[apis.description]
zh = """
按qLD前缀排列对称CSR原矩阵源槽。缺项nC哨兵，不计算因子。
"""
en = """
Checked i32 symmetric CSR source slots in qLD prefix address order; nC sentinel for structural zero, not factors.
"""

[[deps]]
kind = "call"
to = "mjwarp.model.topology"
from_api = "rpc:mjwarp_rs::model::InertialModelInput"
to_api = "rpc:mjwarp_rs::model::KinematicModelInput"

[deps.label]
zh = "借用运动学字段"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::model::InertialModelInput"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[deps.label]
zh = "报告字段错误"
en = "Inertial subset helper"

[[deps]]
kind = "call"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::model::InertialModelInput"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[deps.label]
zh = "检查连续容量"
en = "Inertial subset helper"
+++

## 2026-10-14 新元数据实际验证

新集成第二次含两项宿主测试通过。
四模型元数据独立对照通过。
空树前中后过滤与坏CSR拒绝通过。
原生作者3/3冻结，不追加执行。
原惯性字段与原始ABI保持不变。

## 2026-10-14 真实CSR元数据

保留原惯性九字段，不改变ABI。
独立SparseMassFields接收原生CSR与树区间。
SparseMassInput只读验证并构造整数元数据。
冻结块分类检查尺寸与结构。
标量六维、稠密64维、其余稀疏。
生成qLD块地址、前缀大小与分层更新。
空树按num零过滤，不依赖adr。
稀疏大块要求完整祖先行前缀。
压缩大块仍拒绝，不冒充完整支持。
宿主不计算任何物理因子。
新数值验收仍独立，不继承旧结果。
证据：C:/Rust/mjwarp-rs/docs/resident-sparse-factor.md。

## 本轮局部验收

本叶子仅登记窄辅助契约。
二十一字段取得Windows证据。
测试覆盖正常与失败边界。
完整产品模块继续保持计划。
本叶子不声明物理等价。
本叶子不声明Linux验收。

证据：C:/Rust/mjwarp-rs/docs/windows-inertial-fields.md。


## 2026-10-14 非紧凑块源槽

新增dense_block_sources只读转换。
按qLD前缀地址映射对称原矩阵。
CSR取max行/min列，缺项用nC哨兵。
索引i32，不分配全nv平方。
Small/Tile均提供前缀原矩阵映射。
Compact/Sparse不占前缀。
并非M_tiles分组顺序或因子内容。
新宿主1过0.02秒，闭式核对第四模型49槽。
另7/64叉树、双Small与空前缀通过。
旧构造与分层元数据不改，旧用途不重跑。
证据：C:/Rust/mjwarp-rs/docs/sparse-mass-dense.md。
