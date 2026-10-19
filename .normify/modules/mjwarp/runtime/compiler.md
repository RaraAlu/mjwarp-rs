+++
uid = "5cedba77"
id = "mjwarp.runtime.compiler"
state = "planned"
fingerprint = "b40ba0dc6b4b87f517dbeb32694f52a510b2e90313af27c9068524c9ad1d5130"

[name]
zh = "内核编译探针"
en = "Kernel compiler probes"

[description]
zh = """
编译冻结CubeCL双路内核。 产品目标接口仍属计划。
"""
en = """
Own frozen CubeCL C++ and LLVM kernel compilation probes, not a production backend choice. Production target APIs remain planned.
"""

[[source]]
path = "src/runtime/cubecl.rs"

[[source]]
path = "scripts/native-mathdx-probe.cpp"

[[source]]
path = "scripts/probe-windows-mathdx.ps1"

[[source]]
path = "scripts/native-mathdx7-link.cpp"

[[source]]
path = "scripts/native-mathdx7-call.cu"

[[source]]
path = "scripts/probe-windows-mathdx7-link.ps1"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::run_probe"

[apis.description]
zh = """
现有局部探针入口。 不提供完整引擎API。 rpc仅作契约标记。
"""
en = """
Existing working prototype entry point, not a complete engine API. RPC is an abstract contract notation, not a network service.
"""
+++

## 2026-10-14 七维MathDx原生依赖探针

新增独立MSVC C++能力工具。
仅公开C ABI，WHEEL校验与受限DLL装载。
实际Warp1.15.0/MathDx0.3.1/enabled1。
七维POTRF直接编译一次成功，1/3冻结。
REAL/F32 precision5,rowmajor,upper,SM89,32线程。
LTO1034872/fatbin506960字节。
无wp_init/Python/矩阵/物理内核/旧作者调用。
输出二进制只留ignored target，不分发SDK。
Warp Apache2与SDK协议分开登记。
不反汇编SDK，不代用户判发布许可。
只证明本地编译能力，非数值/内部类型/机器等价。
正式compiler仍planned，不激活完整实现。
源树UID/主锁与前置不变。
Small FAILED3/3和旧Clippy/回代冻结保持。
后续实际链接/受检加载/混合精度仍待实现。
证据：C:/Rust/mjwarp-rs/docs/native-mathdx7.md。

## 2026-10-14 七维实际链接与设备依赖

新增离线22参数编译链接和Driver设备工具。
复用原7维LTO与fatbin，无能力查询/solver重生成。
同新purpose首轮/I:准备失败，第二轮完整通过2/3冻结。
实际NVRTC12.9、release/O3/fmad true/fast_math false。
CUBIN2086544字节，经ELF身份检查装载SM89。
六批世界1/2/257，两静态独立解析U，25480槽。
原abs/rel2e-4及重构2e-6保持。
maxabs9.536743e-7/maxrecon1.748402e-7。
10920正零下三角、输入/输出/status双端守卫。
首末坏主元info1/7，全部8启动成功。
正式compiler仍planned，不替代正式产品路径。
不修改/绕过Rust产品f32 PTX门禁。
SDK和链接二进制仅ignored target，不反汇编。
不证明原MJWarp7维模板、内部精度或SASS。
旧FAILED3/3/成功/Clippy/满额预算保持冻结。
证据：C:/Rust/mjwarp-rs/docs/native-mathdx7-link.md。
