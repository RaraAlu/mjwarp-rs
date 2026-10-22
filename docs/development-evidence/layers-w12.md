# W12 条件图节点评审

任务：`2026-10-18-layers-m7-conditional`。
日期：2026-10-22。
依据：`docs/architecture.md`、`docs/unified-launcher.md`、`docs/module-contracts.md`。
本轮只评审，不实现、不升级依赖、不改代码。
本轮结论不自动批准实现。
本轮不宣称条件图已实现，也不宣称 G14/G15 已实现。

## 结论

批准 W12 的技术路线：把 cudarc feature 升到 `cuda-12030`，经 `driver::sys` 使用条件节点 API，在统一发射器图面扩展 while 条件节点；但**实施延后**并另立交付任务。
不采用旧探针图实现作为产品条件图基础；旧探针图只覆盖内核节点。
G14/G15 的收敛迭代是 W12 的消费者，不是 W12 的完成条件；两者各自另立交付与数值验收任务。
升级 feature 与实现条件节点须分两个提交/两个验收面，不得混为一次提交。
条件未满足前不进入实施；条件图不进入产品路径。

结论：**有条件批准评审路线，实施延后**。

## 事实基线

| 事实 | 证据 | 结论 |
| --- | --- | --- |
| cudarc 0.18.2 提供 12.3 条件节点绑定 | crate `Cargo.toml` 有 `cuda-12030` feature；`sys/mod.rs` 在 `cuda-12030` 起导出 `CUgraphNodeParams_st` 的 `conditional` 联合成员、`CUDA_CONDITIONAL_NODE_PARAMS`、`CU_GRAPH_NODE_TYPE_CONDITIONAL=13`、`CUgraphConditionalNodeType`（IF/WHILE）与 `cuGraphConditionalHandleCreate` | 绑定已存在，无需自写 FFI |
| 条件句柄需 12.3 头/API | `cuGraphConditionalHandleCreate` 的 cfg 门从 `cuda-12030` 起 | 最低 API 版本 12030 |
| 通用加节点需 12.2 头/API | `cuGraphAddNode` 的 cfg 门从 `cuda-12020` 起（12.3+ 可见） | 与条件节点同属一次升级 |
| 当前产品未启用这些符号 | `Cargo.toml` 的 cudarc features 锁 `cuda-12000`；`src/runtime/launcher/` 下 grep 无 `cuGraph`/`begin_capture`/`instantiate` | 条件节点能力当前不可用 |
| 本机驱动满足门槛 | `nvidia-smi` 报驱动 596.36、CUDA 13.2；`docs/windows-external-probe.md` 记驱动 API 13020；`docs/evidence/three-layer/windows/env-nvidia-smi.txt` 同值 | 远超 12030/12040 接口门槛 |
| 冻结 Warp 对条件节点有 12.4 口径 | 冻结 `warp_util.py` 告警：`<12.3` 捕获不可靠、`<12.4` 条件图节点不可用，建议 `graph_conditional=False` | 产品门槛按 12.4 保守冻结 |
| 上游求解器依赖条件 while | 冻结 `solver.py:3994` `wp.capture_while(nsolving, while_body=_solver_iteration, ...)`；`derivative.py` PCG 与 `collision_flex.py` FPS 也用 `capture_while` | W12 是 G14/G15 迭代的前置 |
| G14/G15 尚未实现 | `docs/compatibility-matrix.md` G14/G15 为 P6；`docs/upstream-api-map.md` U069 指向 G14/G15；`src/` grep 无 `SolverType`/`graph_conditional`/options 实现 | W12 只解前置，不产生求解器 |
| `graph_conditional` 契约已登记 | `docs/physics-stage-contracts.md` 记 `graph_conditional` 默认 true、关联 A06 | 默认值与回退语义已有冻结口径 |
| 统一发射器图面是宿主策略层 | M6 `src/runtime/launcher/graph.rs` 只有 `CaptureHost`/`CaptureGuard`/`CapturedGraph`/`GraphInstance` 与重绑/回收策略，无真实 CUDA 图调用 | W12 实施需新增真实图捕获与执行接线 |
| 旧探针图只接受内核节点 | `src/runtime/cuda/graph.rs` 的 `ProbeGraph` 捕获后枚举节点，非 `CU_GRAPH_NODE_TYPE_KERNEL` 即拒绝；无通用加节点/条件句柄 | 不能直接泛化为产品条件图 |
| M4/M5/M6 已给出边界前置 | `c0a56b8` 的 `launch_l2` 拒绝捕获期发射并校验上下文；`fbbefd9` 的完成凭证/账本；`9ea28e9` 的捕获互斥、预分配真实显存（`TransferSession::preallocate`）与数值槽重绑 | W12 可复用这些边界，但尚未接入真实图对象 |

## G14/G15 需求

W12 面向的上游用法只有一种语义：设备端条件 while 节点。

| 需求 | 内容 |
| --- | --- |
| 条件量 | 设备端整数缓冲，每图 1 个元素；语义为“未收敛世界数”，0 表示全部收敛 |
| 循环体 | 一个 body 图，含一个或多个内核节点；现有 ABI/跨度/证明校验不得绕过 |
| 终止 | 循环体内核把条件量写 0，或达到最大迭代后由内核置 0；Rust 侧不逐次查询 |
| 重放 | 条件图按普通图重放返回完成凭证；宿主不阻塞迭代 |
| 回退 | `graph_conditional=false` 时回退到宿主循环（上游 `for _ in range(iterations)`），两路径等价由 G14/G15 交付任务验收 |
| 账本 | 图重放按捕获节点读写集登记；条件体重复执行不改变账本条目 |
| 精度 | 条件只控制执行次数，不改内核字符串、f32 路径、容差与编译选项 |

G14（CG）与 G15（Newton）共用同一条件 while 语义。
`derivative.py` 的 PCG 与 `collision_flex.py` 的 FPS 复用同一 W12 能力，但不在 W12 实现。
`wp.capture_if` 未在冻结求解路径中出现；本轮只评审 while，if 节点列为后续扩展，不冻结。

## 条件节点 ABI 冻结（评审建议）

| 项 | 冻结值 |
| --- | --- |
| 条件类型 | `CU_GRAPH_COND_TYPE_WHILE`；一图一条件句柄 |
| 句柄 | 图创建后在图内创建，不跨图复用；销毁随图实例 |
| 加节点 | `cuGraphAddNode` + `CUgraphNodeParams{type_: CU_GRAPH_NODE_TYPE_CONDITIONAL, conditional: {handle, type_, size, phGraph_out}}` |
| 条件缓冲 | 设备地址固定、类型 i32、元素数 1；按地址类槽位对待，禁止重绑 |
| 依赖 | 条件节点依赖由捕获顺序/显式依赖决定；body 内部依赖由驱动返回图保存 |
| 线程块 | 沿用各内核 `KernelAbi::BLOCK`；条件节点不引入块大小 |
| 共享内存 | 沿用 W14 结论；条件节点不改变 `shared_bytes` |
| 捕获 | 复用 M6 的上下文互斥/禁止分配/JIT/同步；条件缓冲写入只能作为捕获节点，不允许捕获期宿主写 |
| 重放 | 复用 M5 完成凭证与访问账本；条件图实例按 `RetainedResources` 持有模块/图/条件缓冲 |
| 失败面 | 驱动版本不足、能力缺失、句柄创建失败、加节点失败、实例化失败、重放失败、条件缓冲被别名/重绑全部显式拒绝 |

发射器不得改写条件类型、条件缓冲大小与 body 图归属。
条件节点本身不发射内核，只包裹既有安全包装的节点。

## feature 升级影响

路径：`Cargo.toml` 的 cudarc features 由 `cuda-12000` 改为 `cuda-12030`（cudarc 0.18.2 已有该 feature）。
升级只影响编译期绑定集合与少量结构别名，不改产品内核、PTX、配方、容差与数值路径。

| 影响项 | 评估 |
| --- | --- |
| 现有 sys 符号 | `cuda-12000`→`cuda-12030` 的 cfg 列表包含原符号；`CUDA_KERNEL_NODE_PARAMS` 对该区间仍解析为 v2（`extra` 字段仍在），旧探针图的 `cuGraphKernelNodeGetParams_v2`/`cuGraphExecKernelNodeSetParams_v2` 预期继续可用 |
| 新增符号 | 条件句柄、通用加节点、条件节点参数与类型在 12.3 起可见 |
| 构建矩阵 | 默认配置不带 cudarc，不受影响；`cuda-probe` 与全部 `--all-features` 构建需重编 |
| 产品行为 | 升级提交本身不调用新符号，产品调度与数值不变；W12 实施提交才接线 |
| 回退方案 | 若 feature 升级出现编译/行为回归，可回退 feature，改走手写 `extern "C"` + 动态加载（与 W13 同路线），但须另立评审 |
| Linux | feature 与驱动门槛双平台校验另行评审，本轮只评 Windows |

## 影响面

| 对象 | 影响 |
| --- | --- |
| `mjwarp.runtime` 图面 | 新增条件句柄、通用加节点、body 图与重放接线；本轮不改 |
| `mjwarp.runtime` 发射面 | `launch_l2` 已拒绝捕获期发射；W12 不改其语义 |
| M5 完成凭证/账本 | 条件图重放走同一提交与账本；实施任务需把图实例接入资源槽 |
| M6 捕获/重绑 | 条件缓冲属地址类，禁止重绑；捕获期约束沿用 |
| G14/G15 | 解锁迭代循环，但不产生求解器；求解器由各自任务实现与验收 |
| 产品精度 | 不改内核、容差、f32 路径；精度验收仍归 G14/G15 |
| 旧探针图 | 保留；不作为产品条件图基础 |
| 部署 | 驱动门槛新增到启动检查；不新增 SDK 二进制依赖（cudarc 已有绑定） |

## 风险与缓解

| 风险 | 缓解 |
| --- | --- |
| 设备/驱动不满足条件节点 | 启动时用 `cuDriverGetVersion`/设备能力核对；不足则拒绝条件图并回退宿主循环 |
| feature 升级影响现有探针/构建 | 升级单独提交，先跑双配置 check/clippy/unit 与既有图测试；不与条件节点实现混提交 |
| 条件 node 语法随 CUDA 版本漂移 | 冻结为 12.3+ 的 `cuGraphAddNode`+conditional 参数；不混用旧 conditional API 变体 |
| 条件缓冲被别名或重绑 | 地址类槽位禁止重绑（C4 首版规则）并纳入联合校验 |
| 条件体重复执行影响账本/回收 | 账本按捕获节点登记；资源持有按图实例，不按迭代次数 |
| 死循环 | 循环上限由上游迭代内核计数置零；Rust 侧不注入额外终止语义 |
| 捕获期宿主写条件缓冲 | 捕获作用域只记录节点，拒绝同步/无关入队（C3） |
| 误把 W12 当 G14/G15 完成 | 回执与文档显式声明只解前置，求解器数值验收另立任务 |

## 前置条件

实施任务启动前须满足：

1. 冻结条件节点 ABI：句柄、类型、依赖、条件缓冲布局与禁止项。
2. 冻结驱动门槛（API 版本 12030/12040 与设备能力）及拒绝/回退策略；启动检查位置写入模块契约。
3. feature 升级单独验收：双配置 check/clippy、unit、既有 `launcher_graph`/`launcher_sync`/`launcher_frozen` 测试全过；不得以 G01 既有债务当门禁。
4. 冻结图捕获/重放接线：真实捕获开始/结束、实例化、重放完成凭证、资源持有与回收。
5. 冻结 G14/G15 的 `graph_conditional` 选项与宿主循环回退语义；默认 true、false 回退。
6. GPU 测试覆盖：驱动版本不足拒绝、while 循环 2 次以上、条件缓冲地址禁重绑、捕获期禁止分配/同步、失败隔离、完成凭证等待、回放后资源回收。
7. 内核字符串、PTX、容差、fixtures 不动；无 Python 构建或运行链。
8. Linux 另行评审，不与 Windows 同轮放行。

## 本轮验证

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| `cargo test --locked` | 通过，退出码 0 |
| `unchanged` 检查 | `src`、`native`、`include`、`tests`、`fixtures`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml` 未改动 |
| `git status` | 仅 `?.claude/` 未跟踪 |

本轮未升级 cudarc feature。本轮未实现条件节点。本轮未改图捕获实现。本轮未运行 GPU 条件图测试。本轮未宣称 G14/G15 或精度验收。

## 依据

- `docs/architecture.md`（三层划分、执行计划、GPU 运行时契约）。
- `docs/unified-launcher.md`（C2/C3/C4、cudarc 能力边界、W12、M7 阶段、待决清单）。
- `docs/module-contracts.md`（runtime 职责、图捕获/重放边界）。
- `docs/compatibility-matrix.md`（G14/G15）、`docs/upstream-api-map.md`（U069）、`docs/physics-stage-contracts.md`（`graph_conditional`）。
- `src/runtime/launcher/{graph.rs,launch.rs,submit.rs,abi.rs}`、`src/runtime/cuda/graph.rs`、`src/runtime/transfer.rs`、`Cargo.toml`。
- 已落地提交：M4 `c0a56b8`、M5 `fbbefd9`、M6 `9ea28e9`。
- cudarc 0.18.2 crate 源码（`Cargo.toml` features 与 `src/driver/sys/mod.rs` 条件节点绑定）。
- 冻结 Warp 源码：`warp_util.py`、`solver.py`、`derivative.py`、`collision_flex.py`。
- 设备与工具链：`nvidia-smi`（596.36 / CUDA 13.2）、`docs/windows-external-probe.md`（驱动 API 13020）、`docs/evidence/three-layer/windows/env-nvidia-smi.txt`。
