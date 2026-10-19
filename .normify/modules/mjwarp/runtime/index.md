+++
uid = "4098c702"
id = "mjwarp.runtime"
tags = ["planned", "engine", "runtime"]
fingerprint = "e546970669f4a8a04037062b08dcbc71be222395a8a38ae793c16b63a6ba3839"

[name]
zh = "GPU运行时"
en = "GPU runtime"

[description]
zh = """
管理设备、缓冲与队列。提供事件和图。不解释物理字段。
"""
en = """
Plan device resources, buffers, queues, events, graphs and safe views without interpreting physics fields.
"""

[[source]]
path = "src/runtime/mod.rs"

[[source]]
path = "src/runtime/probe.ptx"

[[source]]
path = "tests/gpu_probe.rs"

[[source]]
path = "tests/cubecl_probe.rs"

[[source]]
path = "scripts/prepare-cuda-probe.ps1"

[[source]]
path = "scripts/prepare-gnu-probe.ps1"

[[source]]
path = "scripts/run-cubecl-probe.ps1"

[[source]]
path = "src/runtime/samples.rs"

[[source]]
path = "src/runtime/cuda/resources.rs"

[[source]]
path = "tests/runtime_resources.rs"

[[source]]
path = "src/runtime/cuda/artifacts.rs"

[[source]]
path = "tests/artifact_probe.rs"

[[source]]
path = "scripts/test-windows-deployment.ps1"

[[source]]
path = "src/runtime/cuda/external.rs"

[[source]]
path = "tests/external_resources.rs"

[[source]]
path = "include/mjwarp_probe.h"

[[source]]
path = "tests/native/external_descriptor.cpp"

[[source]]
path = "scripts/test-windows-external.ps1"

[[source]]
path = "tests/batch_transfer.rs"

[[source]]
path = "scripts/test-windows-transfers.ps1"

[[source]]
path = "src/runtime/launcher/mod.rs"

[[source]]
path = "src/runtime/launcher/abi.rs"

[[source]]
path = "src/runtime/launcher/alias.rs"

[[source]]
path = "src/runtime/launcher/recipe.rs"

[[source]]
path = "src/runtime/launcher/artifact.rs"

[[source]]
path = "src/runtime/launcher/module.rs"

[[source]]
path = "src/runtime/launcher/checked.rs"

[[source]]
path = "src/runtime/launcher/launch.rs"

[[source]]
path = "tests/launcher_abi.rs"

[[source]]
path = "tests/launcher_cache.rs"

[[source]]
path = "tests/launcher_sync.rs"
+++

## P1局部实现

2026-10-06：继续Windows验证。
两路共用八项Rust探针。
另有两个全局扫描辅助内核。
增加f32原子与全局扫描。
分层工作区先分配再捕获。
GPU递归扫描并回填块偏移。
每层总和与前缀检查守卫。
宿主只生成独立测试参考。
f32固定四分之一整数样本。
部分和范围保证精确表示。
非二进制小数仍需另定容限。
真实节点更新交替切换输出。
多节点扫描同步更新三节点。
每轮核对未选输出保持初值。
每路覆盖74组GPU配置。
含72组九种规模矩阵。
另有两组10轮长重放。
两路共472轮及584次节点更新。
调试与发行模式均通过GPU回归。
全特性宿主测试51项通过。
图先于输出及工作区释放。
本轮不修改依赖冻结提交。
原生PTX仍只支持affine。
保留GNU默认工具链。
Linux工作本轮暂不开展。
外部资源租约仍待验证。
图布局变更仍待验证。
清洁部署与缓存仍待验收。
探针不验收CubeCL运行时。
八项目标API仍属于计划。
本轮不激活该模块。

证据：C:/Rust/mjwarp-rs/docs/windows-kernel-probe.md。

## Windows资源租约原型

真实CUDA所有者移入包装。
字节范围记录偏移与布局版本。
在途写租约拒绝重叠读写。
队列先保留资源再提交内核。
完成令牌只保留状态回执。
句柄提前释放不悬空设备地址。
遗忘令牌不影响队列所有权。
队列析构等待实际完成事件。
未记录栅栏以流同步清理。
提交失败隔离资源且保留错误。
清理失败保守保留资源。
此故障路径会泄漏上下文。
宿主模拟验证故障与panic。
GPU只模拟提交后的宿主失败。
本轮不注入真实设备非法访问。
三路各验证五种GPU规模。
每次矩阵共90个成功任务。
另有15个宿主失败任务。
矩阵共240次边界拒绝。
矩阵共15次真实图重放。
图预热采用独立输出缓冲。
图路径提前移交内核模块。
同设备异域使用真实上下文。
调试及发行模式均通过。
默认46项宿主测试通过。
GNU C++宿主测试50项通过。
MSVC全特性51项通过。
内部原型不冻结外部ABI。
目标submit等接口仍属计划。
物理字段布局与设备视图待定。
缓存与清洁部署继续待验。
Linux工作仍暂缓。
本轮不激活该模块。

证据：C:/Rust/mjwarp-rs/docs/windows-resource-probe.md。

## Windows产物缓存原型

三路生成17份PTX产物。
全局扫描产物包含三个阶段。
源码及锁文件摘要进入缓存键。
键保留路线、配方与设备版本。
结构解析拒绝未知与重复字段。
摘要只检查完整性，不认证来源。
外部PTX入口要求unsafe担保。
CLI要求显式信任缓存目录。
缺失或损坏缓存不自动编译。
显式刷新先落临时文件再替换。
只驱动消费程序不依赖前端。
受限PATH子进程验证68组配置。
共204轮图重放与240次更新。
NVRTC不可见检查真实通过。
PE依赖仍含Windows及VC运行库。
此烟测不等于清洁部署验收。
NVRTC补丁与头文件指纹仍待定。
物理、渲染与外部ABI仍未实现。
八项目标API继续保持计划。
本轮不激活该模块。

证据：C:/Rust/mjwarp-rs/docs/windows-artifact-probe.md。

本轮全特性宿主66项通过。
MSVC调试及发行GPU各12项通过。
默认宿主48项通过。
缓存宿主13项、CLI6项通过。
GNU C++及隔离LLVM均通过。
三路缓存覆盖157组GPU配置。

## Windows外部资源ABI原型

Rust与C++核对96字节布局。
驱动查询传统分配实际范围。
导入器拒绝托管、池与宿主内存。
真实异域分配验证上下文拒绝。
私有unsafe契约保护所有者移交。
记录生产事件，再移交外部访问。
进程弱注册表拒绝重复导入。
在途租约保留外部所有者与事件。
三路每组实际释放15个所有者。
调试及发行各通过五种规模。
矩阵各有225次释放及255次拒绝。
MSVC全特性宿主各74项通过。
MSVC全量GPU各16项通过。
GNU C++隔离宿主73项通过。
GNU外部GPU三项通过。
仅驱动部署重新通过68组配置。
此原型不导出生产导入入口。
C回调及生产张量ABI仍待决议。
物理字段与模型输入ABI仍待决议。
八项目标API继续保持计划。
本轮不激活该模块。

证据：C:/Rust/mjwarp-rs/docs/windows-external-probe.md。

## 实际职责拆分

运行时现在作为容器。
七子模块各持有真实源码。
容器记录跨叶集成证据。
含门面、CUDA适配与测试脚本。
叶子各记录主实现文件。
25项旧映射全部保留。
八项原目标契约移入叶子。
探针与脚本入口单独登记。
本轮不激活产品模块。

## 同步传输辅助

新增transfer职责叶子。
类型化缓冲不导出裸地址。
缓冲保留会话与设备所有者。
同步接口在返回前等待完成。
未知完成保守保留资源。
图租约仍采用原有独立原型。
本轮不冻结生产运行时。

证据：C:/Rust/mjwarp-rs/docs/windows-transfer-probe.md。
