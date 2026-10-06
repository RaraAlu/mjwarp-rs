# 基础源码与架构收口

日期：2026-10-07。
范围：Windows基础层。
本轮不冻结正式GPU路线。
本轮不开展Linux验收。

本文记录基础落地时的快照。
后续实现见[字段交换报告](windows-transfer-probe.md)。

## 目标与结果

本轮处理六项架构警告。
运行时原先集中25项映射。
其余五模块缺少源码证据。

本轮按职责细化运行时。
五模块各新增实际基础行为。
测试覆盖正常及失败边界。
本轮不添加空壳或规则豁免。
本轮不改依赖与默认工具链。

Normify校验取得零错误。
Normify校验取得零警告。
当轮15节点均保持计划态。
零警告不证明引擎完成。

## 基础落地源码快照

```text
C:/Rust/mjwarp-rs/
  src/
    lib.rs
    main.rs
    diagnostics/mod.rs
    model/mod.rs
    io/mod.rs
    math/mod.rs
    physics/mod.rs
    render/mod.rs
    runtime/
      mod.rs
      cuda.rs
      cuda/
        resources.rs
        artifacts.rs
        external.rs
      lease.rs
      completion.rs
      external.rs
      cubecl.rs
      samples.rs
      cache.rs
      probe.ptx
  THIRD_PARTY_NOTICES.md
  LICENSES/Apache-2.0.txt
```

七顶层职责保持原有归属。
物理层不依赖渲染层。
本轮不新增CPU play后端。

## 基础落地架构快照

以下节点表示架构职责。
节点不要求照搬Rust目录。

```text
mjwarp.runtime
  driver       驱动与执行图
  buffers      内部缓冲租约
  completion   完成队列
  external     外部资源导入
  compiler     内核编译探针
  cache        可信产物缓存
  tooling      工具与命令入口
```

每个叶子登记主实现文件。
容器登记跨叶集成证据。
集成证据含门面与CUDA适配。
集成证据也含测试和脚本。
25项旧映射各保留一次。
本轮不删除任何旧映射。

| 架构叶子 | 主实现文件 | 职责 |
| --- | --- | --- |
| driver | `src/runtime/cuda.rs` | 驱动、内核、图与节点更新 |
| buffers | `src/runtime/lease.rs` | 所有权、范围、版本与别名 |
| completion | `src/runtime/completion.rs` | 令牌、事件、退役与隔离 |
| external | `src/runtime/external.rs` | 描述符、身份与重复导入 |
| compiler | `src/runtime/cubecl.rs` | 冻结双路内核编译 |
| cache | `src/runtime/cache.rs` | 可信PTX配方与完整性 |
| tooling | `src/main.rs` | 命令解析与显式配置 |

运行时容器保留18项映射：

```text
src/runtime/mod.rs
src/runtime/probe.ptx
src/runtime/samples.rs
src/runtime/cuda/resources.rs
src/runtime/cuda/artifacts.rs
src/runtime/cuda/external.rs
tests/gpu_probe.rs
tests/cubecl_probe.rs
tests/runtime_resources.rs
tests/artifact_probe.rs
tests/external_resources.rs
scripts/prepare-cuda-probe.ps1
scripts/prepare-gnu-probe.ps1
scripts/run-cubecl-probe.ps1
scripts/test-windows-deployment.ps1
scripts/test-windows-external.ps1
include/mjwarp_probe.h
tests/native/external_descriptor.cpp
```

原八项运行时契约路径不变。
设备与图契约归driver。
分配与复制契约归buffers。
提交与等待契约归completion。
设备视图契约归external。
相关出入边指向新的叶子。
这些生产接口继续保持计划。

原141项契约全部保留。
本轮另登记12项基础接口。
运行时另登记10项探针入口。
当轮共登记163项契约。
rpc只表达抽象工具协议。
项目不设计网络RPC服务。

## 五模块基础行为

| 模块 | 已实现行为 | 关键边界 | 新增测试 |
| --- | --- | --- | ---: |
| model | `BatchLayout`连续批量布局 | 溢出、切片容量、世界越界；允许空字段 | 4 |
| io | 浮点转换与世界片段复制 | 全量验证后写入；保留复制位模式 | 6 |
| math | 八项f32数学公式 | wxyz、共轭、非单位输入与非有限传播 | 8 |
| physics | `HistoryLayout`形状与时间检查 | 阈值显式传入；拒绝过短间隔 | 5 |
| render | 单相机批量图像布局 | 输出开关、像素索引、分割双元素与容量 | 5 |

diagnostics另增加一项测试。
InputError保留字段与索引。
本轮共新增29项宿主测试。

### 数学约定

四元数顺序采用wxyz。
空间向量先转动，后平动。
矩阵采用行主序。
quat_inv只执行共轭。
公式不隐式归一化。
非单位四元数保留缩放效应。
纯公式保留IEEE非有限传播。

八项函数如下：

```text
mul_quat
quat_mul_axis
rot_vec_quat
axis_angle_to_quat
quat_to_mat
quat_inv
motion_cross
motion_cross_force
```

本轮读取冻结上游快照。
本轮不执行上游Python。
解析四分之一转角另设期望。
测试固定绝对容限为1e-6。
整数样本采用精确比较。
矩阵对旋转检查仅作辅助。
GPU等价与广域数值仍待验收。

[冻结数学源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py)提供公式依据。
[第三方声明](../THIRD_PARTY_NOTICES.md)保留来源。
[许可副本](../LICENSES/Apache-2.0.txt)保留上游许可。
项目总许可仍待P0决议。

### 严格辅助边界

浮点转换拒绝非有限输入。
转换也拒绝f32溢出。
转换允许舍入与下溢。
世界复制保留NaN位模式。
失败路径不改写目标切片。
这些辅助不替代put_data。
这些辅助也不替代put_model。

历史辅助要求正样本与通道。
调用方显式传入原生阈值。
相邻间隔可以等于阈值。
阈值不是固定的新物理常量。
时间检查仅提供宿主辅助。
MC-38与MC-39仍缺完整证据。
历史槽位、延迟与插值仍待实现。

图像布局只描述单相机。
RGB采用四字节打包元素。
深度采用四字节浮点元素。
分割采用ID与类型双元素。
全部输出禁用时不申请缓冲。
禁用输出直接返回错误。
本轮不猜测RGB字节通道顺序。
本轮不冻结空相机列表语义。
异构相机、BVH与像素仍待实现。

## Windows验证

GPU：RTX 4070 Ti SUPER。
驱动：596.36。
Rust：1.99.0。
主机：Windows 10.0.26200.0 x64。
测试不调用Python。
默认工具链仍采用GNU。
LLVM检查显式选择MSVC。

| 检查 | 本轮结果 |
| --- | --- |
| GNU默认宿主测试 | 85通过，零失败 |
| GNU隔离C++宿主测试 | 102通过，零失败；GPU另行执行 |
| MSVC全特性宿主测试 | 调试与发行各103通过；默认忽略16项GPU测试 |
| MSVC全特性GPU测试 | 调试与发行各16通过，零失败、零忽略 |
| MSVC隔离LLVM宿主测试 | 102通过，零失败；GPU另行执行 |
| fmt、clippy与rustdoc | fmt及五种特性clippy通过；全特性rustdoc通过 |
| Normify证据刷新与校验 | 零错误、零警告；全部保持计划态 |

命令使用锁定依赖：

```text
cargo test --locked
cargo +stable-x86_64-pc-windows-gnu test --locked --features cubecl-cpp-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --features cubecl-llvm-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --release --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --release --all-features -- --ignored --test-threads=1
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-gnu clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-gnu clippy --locked --all-targets --features cubecl-cpp-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --features cubecl-llvm-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
```

rustdoc设置`-D warnings`。
初检发现四元数文档链接错误。
修正代码标记后复验通过。
MSVC命令先加载VS与CUDA环境。
日志位于`target/foundation-check/`。
GPU测试核对现有探针回归。
GPU回归覆盖三条现有路线。
GPU测试不验收新增宿主公式。
本轮不重做清洁机器部署验收。

## 剩余工作

原生模型ABI与字段仍待收口。
Model、Data与上传仍待实现。
GPU动力学与碰撞仍待实现。
GPU历史与回调仍待实现。
GPU批量渲染仍待实现。
完整数值证据仍待建立。
正式发布仍要求双平台验收。

架构产物位于以下目录：

```text
C:/Users/zhang/.codex/normify/normify-mjwarp-rs/
```

变更编号如下：

```text
2026-10-07-foundation-architecture-warnings
```
