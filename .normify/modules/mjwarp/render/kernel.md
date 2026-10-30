+++
uid = "bc631296"
id = "mjwarp.render.kernel"
tags = ["engine", "render", "kernel", "gpu"]
fingerprint = "28d7248ced2f05f62fcc6d486eea1c798a917c83477eec30e40e250f85d85802"

[name]
zh = "渲染内核与内核实参"
en = "Render kernel and argument wiring"

[description]
zh = "渲染内核与内核实参"
en = "Render kernel and argument wiring"

[[source]]
path = "src/render/camera.rs"

[[source]]
path = "src/render/camera/cuda/build_rays.cu"

[[source]]
path = "src/render/kernel/cast.rs"

[[source]]
path = "src/render/kernel/cast_mesh.rs"

[[source]]
path = "src/render/kernel/cuda/aa_resolve.cu"

[[source]]
path = "src/render/kernel/cuda/render.cu"

[[source]]
path = "src/render/kernel/cuda/render_common.cu"

[[source]]
path = "src/render/kernel/cuda/render_math.cu"

[[source]]
path = "src/render/kernel/cuda/render_megakernel.cu"

[[source]]
path = "src/render/kernel/cuda/render_shade.cu"

[[source]]
path = "src/render/kernel/cuda/render_shadow.cu"

[[source]]
path = "src/render/kernel/launch.rs"

[[source]]
path = "src/render/kernel/megakernel.rs"

[[source]]
path = "src/render/kernel/megakernel_aa.rs"

[[source]]
path = "src/render/kernel/megakernel_arrays.rs"

[[source]]
path = "src/render/kernel/megakernel_uploads.rs"

[[source]]
path = "src/render/kernel/mod.rs"

[[source]]
path = "src/render/kernel/output.rs"

[[source]]
path = "src/render/kernel/renderer.rs"

[[source]]
path = "src/render/kernel/world_arrays.rs"

[[source]]
path = "src/render/kernel/world_camera.rs"

[[source]]
path = "src/render/rays.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::render"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U041；R01～R10；P8。
"""
en = """
Planned local function contract for upstream render.render (U041). Matrix: R01～R10. Phase: P8. Not implemented; RPC is only an abstract contract notation.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::render::render"
to_api = "rpc:mjwarp_rs::model::data::Data"

[deps.label]
zh = "只读物理派生缓冲"
en = "Derived-state view"

[[deps]]
kind = "call"
to = "mjwarp.runtime.completion"
from_api = "rpc:mjwarp_rs::render::render"
to_api = "rpc:mjwarp_rs::runtime::submit"

[deps.label]
zh = "提交无头渲染计划"
en = "Render submission"

[[deps]]
kind = "reference"
to = "mjwarp.math"
from_api = "rpc:mjwarp_rs::render::render"
to_api = "rpc:mjwarp_rs::math::rot_vec_quat"

[deps.label]
zh = "使用相机与几何公式"
en = "Camera geometry"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::render::render"
to_api = "rpc:mjwarp_rs::diagnostics::RenderReport"

[deps.label]
zh = "返回图像有效性报告"
en = "Image validity"
+++
