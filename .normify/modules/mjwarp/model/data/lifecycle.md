+++
uid = "6b66bd50"
id = "mjwarp.model.data.lifecycle"
fingerprint = "de3b13ba733d65afae04110bcac988fe3ac611f93b13518b920885f61eb6c07c"

[name]
zh = "设备数据生命周期"
en = "Device data lifecycle"

[description]
zh = """
设备数据生命周期契约：会话所有权、模型与布局身份分离、状态失效与证明代际。宿主类型已实现。计划级元数据证明所有权登记在共享归属下。
"""
en = """
Device-data lifecycle contract: session ownership, separate model/layout identities, state invalidation and proof generations. Host types implemented; plan-level metadata proof ownership is registered here.
"""

[[source]]
path = "src/model/data/lifecycle.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::DataLifecycle"

[apis.description]
zh = """
已实现的生命周期记录：会话核对、状态版本、派生失效与证明代际复核。
"""
en = """
Implemented lifecycle record: session check, state version, derived invalidation and proof-generation check.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::DataIdentity"

[apis.description]
zh = """
已实现的数据身份：模型身份与布局身份分开记录，同尺寸不代表同拓扑。
"""
en = """
Implemented data identity: model and layout identities recorded separately; equal sizes do not imply equal topology.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::SessionBinding"

[apis.description]
zh = """
会话绑定契约：克隆共享设备流与隔离状态，跨会话使用一律拒绝。
"""
en = """
Session binding contract: clones share device stream and isolation state; cross-session use is rejected.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::DeviceModelData"

[apis.description]
zh = """
io设备模型的公开兼容转发面：只读身份与主字段布局，旧签名不变。
"""
en = """
Public compatibility forwarding surface for io device models: read-only identity and primary layout; old signatures unchanged.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::CheckedMetadata"

[apis.description]
zh = """
计划级元数据证明的受检包装别名；绑定设备区间、维度与写入代际。
"""
en = """
Alias of the checked metadata proof; binds device span, dims and write generation.
"""
+++

## 生命周期契约

构造期依次检查宿主输入、上传模型与参数、上传元数据并建立证明。
运行期先核对身份、会话、证明与精确跨度，再同步发射。
状态写入使派生结果失效；元数据写入使证明代际失效。
销毁时设备缓冲随所属类型释放，不遗留半初始化设备模型组。
