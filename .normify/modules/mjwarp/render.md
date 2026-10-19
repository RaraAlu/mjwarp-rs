+++
uid = "a66a7856"
id = "mjwarp.render"
state = "planned"
tags = ["planned", "engine", "render"]
fingerprint = "6575f51f593535bfed06c4a7deac4a0f6d62d7bac433e964774b3e8f32f91f42"

[name]
zh = "GPU批量渲染"
en = "Batched GPU rendering"

[description]
zh = """
提供无头批量渲染。维护渲染BVH。输出像素与完成状态。
"""
en = """
Plan headless batched rendering, rendering BVHs and image outputs without changing dynamic physics state.
"""

[[source]]
path = "src/render/mod.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::bvh::refit_bvh"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U004；R01、R05、R06；P8。
"""
en = """
Planned local function contract for upstream bvh.refit_bvh (U004). Matrix: R01、R05、R06. Phase: P8. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::render::bvh::refit_splat_bvh"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U005；R01、R07；P8。
"""
en = """
Planned local function contract for upstream bvh.refit_splat_bvh (U005). Matrix: R01、R07. Phase: P8. Not implemented; RPC is only an abstract contract notation.
"""

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
计划保留类型契约。保留冻结上游语义。 U095；R01；P8。
"""
en = """
Planned type contract for upstream types.RenderContext (U095). Matrix: R01. Phase: P8. Not implemented; RPC is only an abstract contract notation.
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

## 基础源码落地

2026-10-07：补齐基础行为。
ImageBatchLayout检查单相机布局。
深度与RGB各占四字节。
分割ID与类型共占八字节。
禁用输出拒绝访问。
单相机布局不冻结相机列表。
BVH与GPU批量渲染仍待实现。
本轮不激活产品模块。

证据：C:/Rust/mjwarp-rs/docs/foundation-architecture.md。
