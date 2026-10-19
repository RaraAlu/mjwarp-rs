+++
uid = "f1eac802"
id = "mjwarp.io.flex"
fingerprint = "6bed15405f498517133997bd37c650dbc18e291673316186f523cac988a867cd"

[name]
zh = "原生柔体位置转换"
en = "Native flex position conversion"

[description]
zh = """
独占原生柔体位置字段。严格转换并检查模型维度。
"""
en = """
Own native flex position fields and strictly convert f64 values; dimension checks do not establish model identity.
"""

[[source]]
path = "src/io/flex.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::io::NativeFlexPositionSnapshot"

[apis.description]
zh = """
复制十二项柔体字段。转换后接入位置辅助。
"""
en = """
Capture twelve flex fields and convert them into the resident position subset.
"""

[[deps]]
kind = "call"
to = "mjwarp.model.flex"
from_api = "rpc:mjwarp_rs::io::NativeFlexPositionSnapshot"
to_api = "rpc:mjwarp_rs::model::FlexPositionModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::io::NativeFlexPositionSnapshot"
to_api = "rpc:mjwarp_rs::model::NativeModelInfo"

[[deps]]
kind = "call"
to = "mjwarp.io.kinematic"
from_api = "rpc:mjwarp_rs::io::NativeFlexPositionSnapshot"
to_api = "rpc:mjwarp_rs::io::kinematic::native::convert"

[[deps]]
kind = "call"
to = "mjwarp.io.fields"
from_api = "rpc:mjwarp_rs::io::NativeFlexPositionSnapshot"
to_api = "rpc:mjwarp_rs::io::fields::native_staging"

[[deps]]
kind = "call"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::io::NativeFlexPositionSnapshot"
to_api = "rpc:mjwarp_rs::diagnostics::NativeProbeError"
+++

## 本轮局部验收

本叶子只提供位置字段转换。
它不提供完整put_model。
快照独占十二项原生字段。
转换拒绝二次插值。
维度检查不证明模型身份。
调用方保证身体编号同源。
内核验收独立使用NVRTC。
Linux验收仍待执行。

证据：C:/Rust/mjwarp-rs/docs/windows-native-flex-position.md。
