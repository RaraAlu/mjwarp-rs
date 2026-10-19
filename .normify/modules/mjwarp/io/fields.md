+++
uid = "8db34276"
id = "mjwarp.io.fields"
fingerprint = "ce4654e25083fb335d8980b0294fc3d31b64f21e35383eef22e3c80aa9c0e977"

[name]
zh = "连续字段转换与交换"
en = "Dense field conversion and exchange"

[description]
zh = """
复用连续字段与暂存分配。隔离模型组与原生所有者。
"""
en = """
Lower-level dense field exchange, strict scalar conversion and fallible native staging; independent of model groups and native ownership.
"""

[[source]]
path = "src/io/fields.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::convert_f64_to_f32_into"

[apis.description]
zh = """
基础辅助接口已有源码。 先校验，再转换宿主浮点。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Strict host conversion validates all inputs before writing. It does not replace equivalent put_data/put_model.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::copy_world_f32"

[apis.description]
zh = """
基础辅助接口已有源码。 复制指定世界的连续字段。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Checked host copy preserves f32 bit patterns and other worlds; not device state exchange.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::DeviceBatch"

[apis.description]
zh = """
连续字段支持四类标量。世界复制保留实际宽度。
"""
en = """
Synchronous dense f32/f64/i32/u32 GPU field exchange with checked per-world reads, writes and copies; not full Model/Data or put_data.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::upload_f64_batch"

[apis.description]
zh = """
基础传输辅助已有源码。 严格宿主转换后上传GPU。
"""
en = """
Strict f64-to-f32 conversion before uploading one dense field; not equivalent model/state import.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::fields::native_staging"

[apis.description]
zh = """
内部原生快照分配。它不是公开Rust入口。
"""
en = """
Internal fallible owned native staging allocation, not a public Rust API.
"""

[[deps]]
kind = "call"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::io::copy_world_f32"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[deps.label]
zh = "基础辅助的真实依赖"
en = "Foundation helper"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::convert_f64_to_f32_into"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[deps.label]
zh = "基础辅助的真实依赖"
en = "Foundation helper"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::copy_world_f32"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[deps.label]
zh = "基础辅助的真实依赖"
en = "Foundation helper"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
from_api = "rpc:mjwarp_rs::io::DeviceBatch"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "GPU同步复制"
en = "GPU synchronous copy"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::DeviceBatch"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[deps.label]
zh = "传输错误"
en = "Transfer errors"

[[deps]]
kind = "call"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::io::DeviceBatch"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[deps.label]
zh = "世界布局"
en = "World layout"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
from_api = "rpc:mjwarp_rs::io::upload_f64_batch"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "转换后上传"
en = "Converted upload"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::fields::native_staging"
to_api = "rpc:mjwarp_rs::diagnostics::NativeProbeError"

[deps.label]
zh = "保留分配错误"
en = "Preserve allocation failure"
+++

## 连续字段底层

DeviceBatch复用封闭传输类型。
它支持f32、f64、i32与u32。
原样f64路径保留八字节位模式。
世界复制检查维度与元素宽度。
upload_f64_batch仍严格窄化。
既有模型字段组不改变类型。
底层不依赖原生模型所有者。
本模块不替代完整put_data。

证据：C:/Rust/mjwarp-rs/docs/windows-transfer-probe.md。
证据：C:/Rust/mjwarp-rs/docs/windows-mass-solve-probe.md。
