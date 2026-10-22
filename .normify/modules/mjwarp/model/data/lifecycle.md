+++
uid = "6b66bd50"
id = "mjwarp.model.data.lifecycle"
fingerprint = "fdf9e476f6fd21d74251fa4631bb9e1a6453dfff6021ebcd857f68ad73df2171"

[name]
zh = "设备数据生命周期"
en = "Device data lifecycle"

[description]
zh = """
设备数据生命周期契约：会话所有权、模型与布局身份分离、状态失效与证明代际。宿主类型已实现；常驻状态写入递增状态代际与qpos/qvel/mocap/sleep字段版本，元数据写入递增证明代际。计划级元数据证明所有权登记在共享归属下。
"""
en = """
Device-data lifecycle contract: session ownership, separate model/layout identities, state invalidation and proof generations. Host types are implemented; resident state writes advance the state generation and per-field versions (qpos/qvel/mocap/sleep), and metadata writes advance the proof generation. Plan-level metadata proof ownership is registered here.
"""

[[source]]
path = "src/model/data/lifecycle.rs"

[[source]]
path = "src/model/data/versions.rs"

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

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::StateField"

[apis.description]
zh = """
常驻状态字段标签：qpos、qvel、mocap与休眠状态；用于按字段递增与核对版本。
"""
en = """
Resident state field tag: qpos, qvel, mocap and sleep state; used to advance and check per-field versions.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::data::StateFieldVersions"

[apis.description]
zh = """
状态字段版本只读快照；零表示尚未写入，每次成功写入加一，不携带可变借用。
"""
en = """
Read-only snapshot of state field versions: zero means not yet written, each successful write adds one, no mutable borrow is carried.
"""
+++

## 生命周期契约

构造期依次检查宿主输入、上传模型与参数、上传元数据并建立证明。
运行期先核对身份、会话、证明与精确跨度，再同步发射。
状态写入使派生结果失效；元数据写入使证明代际失效。
状态写入成功后递增状态代际与对应字段版本。
下游模块只读核对来源字段版本是否同代。
常驻元数据缓冲在构造后不可改写。
销毁时设备缓冲随所属类型释放，不遗留半初始化设备模型组。
