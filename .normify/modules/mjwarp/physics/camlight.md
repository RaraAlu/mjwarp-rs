+++
uid = "cd673a89"
id = "mjwarp.physics.camlight"
fingerprint = "fd67f76d068f3916d3d36e5880e744e0345d890c7271540eb756e8f21791272a"

[name]
zh = "相机光源位姿"
en = "Camera light transforms"

[description]
zh = """
复用刚体与质心设备结果。
"""
en = """
Reuse resident FK and COM device results.
"""

[[source]]
path = "src/physics/camlight.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::CamLightOutput"

[apis.description]
zh = """
相机与光源辅助接口。
"""
en = """
Camera and light helper interface.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::CamLightWorld"

[apis.description]
zh = """
相机与光源辅助接口。
"""
en = """
Camera and light helper interface.
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

## 2026-10-14 内部量与输出映射

本轮核清十八组内部量。
四项公开输出均采用f32。
原五种模式与负目标回退保持。
三向量特化直接除以长度。
它不采用四元数的倒数乘法。
两段设备执行顺序已核对。
实际生成CUDA与四个PTX入口已核。
十二处归一化均为f32。
检查器追踪共享零寄存器复制链。
源码与既存产物哈希限定其适用域。
独立宿主目标三次，后两轮三过。
第二轮后补强两组光源位置覆盖。
原配置现已满额，禁止再执行。
首轮检查器断言失败如实保留。
本轮未发现产品公式修复依据。
产品源码及当前捕获身份保持。
旧满额选择与面控制目标未执行。
本轮未启动GPU或CUDA编译器。
完整机器码与数值证明仍待补。
原容差、覆盖、21系数界限保持。
本模块保持活动态。
证据：C:/Rust/mjwarp-rs/docs/t4-camlight-internal.md。
