+++
uid = "49e2a1c7"
id = "mjwarp.physics.flex-face"
fingerprint = "0e0440779cb6575147a0fa02a4c71b74140b76dba15fd3e345f4fd648641b6a6"

[name]
zh = "柔体面运动学"
en = "Flex face kinematics"

[description]
zh = """
线性壳体面位置与极分解旋转。
"""
en = """
Linear shell face positions and frozen iterative polar rotations.
"""

[[source]]
path = "src/physics/flex_face.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexFaceOutput"

[apis.description]
zh = """
拥有面快照。
"""
en = """
Owned face snapshot.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::FlexFaceWorld"

[apis.description]
zh = """
提供单世界面视图。
"""
en = """
Single world face view.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[[deps]]
kind = "reference"
to = "mjwarp.model.flex"
to_api = "rpc:mjwarp_rs::model::FlexFaceFields"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"
+++

## 2026-10-19 严格逐位门禁复验

当前面源码含三轴末项融合修复。
离线机器码与上游逐项等价。
旧FIX_M阴性对照只差三处末项。
严格门禁1033664项零位差。
因子复测五配置掩码10零位差。
旧2078只属FIX_M旧源码。
本次未改产品内核与容差。
三角大角路径未作机器码符号核对。
NP01至NP04整体继续开放。
原UID与活动态保持。
证据：docs/development-evidence/flex-face-strict-bit-gate/README.md。

## 2026-10-14 独立归一化控制域

本轮补齐归一化分支覆盖。
本轮未发现新产品公式缺陷。
两侧gt与leu谓词互补。
NaN同样选择单位回退。
正无穷同样进入倒数路径。
两侧四分量复用单次f32倒数。
检查器固定既有PTX与源码。
新宿主目标首轮四过零败。
八项篡改均遭检查器拒绝。
旧满额目标本轮零调用。
当前设备位差仍未知。
旧2078只属历史结果。
原容差与全部物理覆盖保持。
本轮不新增机器码或数值证据。
NP01至NP04整体继续开放。
主排程及资源锁保持。
原UID与活动态保持。
证据：C:/Rust/mjwarp-rs/docs/t4-face-normalization-control.md。

## 2026-10-14 三角控制域实核

本轮未发现产品三角缺陷。
本轮不改产品公式或编译选项。
源码局部与累加继续采用f32。
两侧面PTX各含四条库归约链。
库内部采用整数-f64-f32转换。
该f64量影响物理输出。
它不属于纯诊断计算。
不能称全部编译指令采用f32。
有限小坐标也能触发慢归约。
独立宿主目标两次，后轮四过。
首轮文本断言失败如实保留。
旧满额选择均未执行。
当前机器码及数值仍未取证。
当前位差未知，前轮2078保留。
原容差、覆盖与21系数范围保持。
主排程与原模块状态保持。
证据：C:/Rust/mjwarp-rs/docs/t4-face-trig-control.md。

2026-10-14 当前CUDA与PTX已实编译。
原编译选择第3次1过，现已满额。
当前面PTX8057e350…采用末项三轴fma。
当前机器码和修复后数值仍未取得。
前轮2078失败仍保留，当前位差未知。
旧捕获保持历史身份。
原容差、覆盖、f32与模块状态保持。
详见docs/t4-field-internal.md。

2026-10-14 三轴末项改为显式fmaf。
其余物理源码保持。
旧捕获已陈旧，新CUDA未编译。
修复后数值尚未执行。
前轮2078失败不归零。
宿主新目标两次各三过。
原容差与全部覆盖保持。
模块保持活动态。
详见docs/t4-field-internal.md。

2026-10-14 局部修复已落实。
基向量旋转与倒数归一化对齐。
乘积固定已证实的融合次序。
全链保持f32及原阈值。
原容差与全部覆盖保持。
独立产品仍有2078项位差。
严格零位差门禁继续失败。
当前样本跨编译器位差为零。
此结论不外推全部输入。
三项新选择各累计三次。
后续仅作静态残差核对。
模块保持活动态。
完整精度仍待验收。
证据见docs/t4-field-internal.md。
