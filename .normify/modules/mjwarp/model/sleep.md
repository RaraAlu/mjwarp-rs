+++
uid = "a762ce09"
id = "mjwarp.model.sleep"
fingerprint = "b75f99894134f36afa213a7af52be2213468e40e9f35608e27608ffdf712f0f4"

[name]
zh = "肌腱唤醒输入"
en = "Tendon wake input"

[description]
zh = """
检查树循环与肌腱限位。保留位置、边、面与矩阵。
"""
en = """
Checked tendon wake input preserving optional flex positions, edges, faces and Hessians.
"""

[[source]]
path = "src/model/sleep.rs"

[[source]]
path = "src/model/smooth_force.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::TendonWakeModelInput"

[apis.description]
zh = """
肌腱唤醒模型。
"""
en = """
Checked tendon wake model.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::TendonWakeFields"

[apis.description]
zh = """
独立限位字段。
"""
en = """
Independent tendon limit fields.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SleepTreeState"

[apis.description]
zh = """
可写树状态。
"""
en = """
Writable checked tree state.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::SmoothForceInput"

[apis.description]
zh = """
四力合并的体/DOF/树索引子集。保留-1与重复体，只检查范围，不证明完整原生拓扑。
"""
en = """
Checked body/DOF/tree index subset for four-force merge; preserve static -1 and repeated bodies; not canonical native model admission.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.tendon"
to_api = "rpc:mjwarp_rs::model::TendonModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexPositionModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[[deps]]
kind = "reference"
to = "mjwarp.model.parameters"
to_api = "rpc:mjwarp_rs::model::ParameterBatch"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexEdgeFields"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexFaceFields"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexHessianFields"

[deps.label]
zh = "矩阵字段"
en = "Hessian fields"
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

## 2026-10-14 平滑力映射辅助

新增SmoothForceInput的body_treeid/dof_bodyid/ntree检查。
只检查维数/索引/容量，保持重复/非identity映射及tree=-1。
ntree0全静态可用，nv0拒绝；不生成完整canonical模型。
独立新host两模型函数通过，旧SleepTreeState不改不重测。
Physics新分支只判tree_awake==0，非零含负值活动。
字段实际i32，与原bool编译选项适配分列。
旧肌腱唤醒循环与ABI保持，原UID/active不变。
证据C:/Rust/mjwarp-rs/docs/smooth-force.md。
