+++
uid = "cb74b4d5"
id = "mjwarp.render.context"
tags = ["engine", "render", "context"]
fingerprint = "a59b8630f71ace406ffaf1cb6237977554c15059ebf1c3c8062dd31c3b2a08f6"

[name]
zh = "渲染上下文与输出"
en = "Render context and outputs"

[description]
zh = "渲染上下文与输出"
en = "Render context and outputs"

[[source]]
path = "src/render/context.rs"

[[source]]
path = "src/render/context/camera_output.rs"

[[source]]
path = "src/render/context/hfield.rs"

[[source]]
path = "src/render/context/hfield_inputs.rs"

[[source]]
path = "src/render/context/lifecycle.rs"

[[source]]
path = "src/render/context/lighting.rs"

[[source]]
path = "src/render/context/model.rs"

[[source]]
path = "src/render/context/output.rs"

[[source]]
path = "src/render/context/owner.rs"

[[source]]
path = "src/render/context/params.rs"

[[source]]
path = "src/render/context/tests.rs"

[[source]]
path = "src/render/context/texture_inputs.rs"

[[source]]
path = "src/render/context/world/errors.rs"

[[source]]
path = "src/render/context/world/layout.rs"

[[source]]
path = "src/render/context/world/mod.rs"

[[source]]
path = "src/render/context/world/state.rs"

[[source]]
path = "src/render/texture.rs"

[[source]]
path = "src/render/texture/cuda.rs"

[[source]]
path = "src/render/texture/sample.cu"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::create_render_context"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U042；R01～R04；P8。
"""
en = """
Planned local function contract for upstream render_util.create_render_context (U042). Matrix: R01～R04. Phase: P8. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::get_depth"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U043；R01～R04；P8。
"""
en = """
Planned local function contract for upstream render_util.get_depth (U043). Matrix: R01～R04. Phase: P8. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::get_rgb"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U044；R01～R04；P8。
"""
en = """
Planned local function contract for upstream render_util.get_rgb (U044). Matrix: R01～R04. Phase: P8. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::get_segmentation"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U045；R01～R04；P8。
"""
en = """
Planned local function contract for upstream render_util.get_segmentation (U045). Matrix: R01～R04. Phase: P8. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::RenderContext"

[apis.description]
zh = """
已实现宿主生命周期。 拥有BVH、网格、纹理与输出缓冲。 显式refit与销毁。 像素内核待S7b。
"""
en = """
Implemented host lifecycle. Owns scene/mesh BVHs, texture objects and output buffers; explicit refit and destroy. Pixel kernel is S7b and not implemented.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::OutputChannels"

[apis.description]
zh = """
基础辅助接口已有源码。 标记启用的图像输出。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Host single-camera channel selection, not a rendering context.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::ImageOutput"

[apis.description]
zh = """
基础辅助接口已有源码。 区分三类图像元素布局。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Host RGB/depth/segmentation layout selector; not native ABI.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::ImageBatchLayout"

[apis.description]
zh = """
基础辅助接口已有源码。 检查批量像素与容量边界。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Checked single-camera batched host image layout; no pixels, camera list, BVH or rendering execution.
"""

[[deps]]
kind = "call"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::render::ImageBatchLayout"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[deps.label]
zh = "基础辅助的真实依赖"
en = "Foundation helper"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::render::ImageBatchLayout"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[deps.label]
zh = "基础辅助的真实依赖"
en = "Foundation helper"
+++
