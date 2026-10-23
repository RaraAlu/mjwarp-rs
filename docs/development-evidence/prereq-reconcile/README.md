# 前置任务依赖修复与资源核实

任务：`2026-10-23-prereq-dependency-reconcile`。日期：2026-10-23。
只改 proposed 任务的 `depends_on`/`note` 并产出证据：未改产品代码、未 start 任何被修任务、未 abandon/promote 任何任务、未动 backlog 条目。

## 结论

- 修复前：proposed 共 **16** 条，按依赖规则 **7 ready / 9 blocked**（与 Chief 所述 6+9 略有差异：现在多了一条 `2026-10-23-u038-3d-gravity-propagation-debt-2`，且 7 条 ready 中含该条）。
- 修复后：**9 ready / 7 blocked**。
- 替换依赖 **4 条**（假阻塞）：`np02-frozen-inventory`、`np03-dual-platform`、`np04-performance`、`u038-reference`。
- 无 verified 后继、按指示**不硬改**并标出：`t4-artifacts`、`t4-gate`（旧 `t4-types` 已 abandon）。`t4-numeric`/`t4-performance` 属其下游合法阻塞。
- 其余 7 条保持 blocked 均为**真实阻塞**：`np03`/`np04` 缺 np02 未验收 + 外部 Linux GPU；`toolchain-clippy-acceptance` 缺外部 Linux 环境；`t4-*` 链待裁决。

## 原依赖 → 新依赖（4 条替换）

| 任务 | 原依赖（坏项） | 新依赖 | 依据 |
| --- | --- | --- | --- |
| `2026-10-23-np02-frozen-inventory` | `2026-10-23-np01-type-mapping`(abandoned) | `2026-10-23-precision-type-mapping`(verified)、`2026-10-23-precision-source-paths`(verified) | 旧 np01 因 task id 触发 `forbid_claim` 自匹配而 abandon；其后继为 `2026-10-23-precision-type-mapping`（3756487），后续补全 `2026-10-23-precision-source-paths`（9c411de） |
| `2026-10-23-np03-dual-platform` | 旧 np01(abandoned) | 上述两个 verified 后继 + `np02-frozen-inventory`(proposed) | 同源替换；np02 未验收仍是真前置 |
| `2026-10-23-np04-performance` | 旧 np01(abandoned) | 同上 | 同源替换；np02 未验收 + 外部 Linux GPU 是真前置 |
| `2026-10-16-next-u038-reference` | `2026-10-16-next-u038-shared`(abandoned) | `2026-10-22-next-u038-shared-2`(verified) + `done-upstream`、`done-gravity`(verified) | 旧 shared 因设计预检前置未落地而 abandon；重登记版 `2026-10-22-next-u038-shared-2` 已 verified |

> 说明：Chief 消息中写的是 `2026-10-23-next-u038-shared-2`；仓库内实际 verified id 为 **`2026-10-22-next-u038-shared-2`**，按真实 id 修复（未虚构任务）。

## 无 verified 后继、标出不硬改（2 条）

| 任务 | 悬空依赖 | 说明 |
| --- | --- | --- |
| `2026-10-16-next-t4-artifacts` | `2026-10-16-next-t4-types`(abandoned) | 用户 2026-10-16 裁决「当前不继续 T4」，该任务被 abandon 且仓库内无 verified 后继；不硬换，保持 blocked，待用户裁决是否由 `2026-10-22-t4-historical-capture-rebaseline` 等新链接替 |
| `2026-10-16-next-t4-gate` | `t4-types`(abandoned) + proposed 的 artifacts/numeric/performance | 同上；依赖链未闭合前保持 blocked |

## 修复后 ready 清单与建议执行顺序

| 顺序 | 任务 | 优先级 | 资源评估 | 说明 |
| --- | --- | --- | --- | --- |
| 1 | `2026-10-23-np02-frozen-inventory` | p30 | 无 GPU（本地证据） | 本地可执行；完成后方可为 np03/np04 提供前置 |
| 2 | `2026-10-22-split-ptx-identity` | p2 | gpu:windows-t4 本机可满足 | 纯本地 PTX 对照 |
| 3 | `2026-10-22-t4-historical-capture-rebaseline` | p2 | gpu:windows-t4 本机可满足；需历史 T4 冻结产物 | 启动前确认 `target/t4-g03-retest` 等产物可用 |
| 4 | `2026-10-16-next-u038-reference` | p82 | gpu:windows-t4 本机可满足 | GPU 定向对照，排在本地证据任务后 |
| 5+ | `2026-10-16-next-g03-parity` | p84 | gpu:windows-t4 本机可满足，但预检结论阻塞 | 依赖虽全 verified，需 MathDx/nvJitLink 许可与 Linux GPU 裁决，暂不派发 |
| 5+ | `2026-10-23-w13-nvjitlink-binding` | p10 | 需用户裁决 nvJitLink 许可/部署 | 裁决前不宜启动 |
| 5+ | `2026-10-18-layers-linux`、`2026-10-18-linux-toolchain-validation` | p1/p2 | **外部 Linux RTX 5080 / linux-acceptance** | 本机不可满足 |
| 5+ | `2026-10-23-u038-3d-gravity-propagation-debt-2` | p82 | gpu:windows-t4 本机可满足 | 裁决 B 债务跟踪，是否启动需用户裁决 |

## 仍 blocked 及原因（7 条）

| 任务 | 剩余坏前置 | 原因 |
| --- | --- | --- |
| `2026-10-23-np03-dual-platform` | `np02-frozen-inventory`(proposed) | np02 未验收；另需外部 Linux RTX 5080 |
| `2026-10-23-np04-performance` | `np02-frozen-inventory`(proposed) | 同上，且性能/显存双平台测量 |
| `2026-10-16-next-t4-artifacts` | `t4-types`(abandoned，无后继) | 待用户裁决替代链 |
| `2026-10-16-next-t4-gate` | `t4-types` + artifacts/numeric/performance(proposed) | 链未闭合 |
| `2026-10-16-next-t4-numeric` | `t4-artifacts`(proposed) | 链未闭合（合法阻塞） |
| `2026-10-16-next-t4-performance` | `t4-numeric`(proposed) | 链未闭合（合法阻塞） |
| `2026-10-18-toolchain-clippy-acceptance` | `2026-10-18-linux-toolchain-validation`(proposed) | 需外部 Linux 环境 |

## 仍缺外部资源

1. **Linux RTX 5080 / linux-acceptance**：`layers-linux`、`linux-toolchain-validation`、`np03`、`np04`（本机不可满足）。
2. **历史 T4 冻结产物**：`t4-historical-capture-rebaseline`（需 `target/t4-g03-retest` 等历史捕获与重采集工具）。
3. **nvJitLink / MathDx 许可与部署**：`w13-nvjitlink-binding`、`g03-parity`（CUDA 12.8.1 无 nvJitLink 组件；MathDx SDK 头/库/许可不在仓内）。
4. **用户裁决**：`u038-3d-gravity-propagation-debt-2`（裁决 B 债务跟踪是否启动）、`t4-*` 链替代方案。

## backlog 悬空依赖（只读观察，未修改）

按指示未动 backlog 条目。扫描发现 2 条 backlog 任务依赖已 abandon 任务，留待后续分诊处理：

- `2026-10-11-roadmap-g02-p4` → `2026-10-16-next-rne-repair`(abandoned)
- `2026-10-11-roadmap-g04-p4` → `2026-10-16-next-u038-shared`(abandoned)、`2026-10-16-next-u038-3d`(abandoned)

## 方法

1. 读取 `.normify/changes/*.json`，按规则计算 readiness：proposed 且全部 `depends_on` 为 verified/不存在 → ready，否则 blocked。
2. 对 16 条 proposed 逐条核对依赖状态；有 verified 后继的替换，无后继的标出且不改；为每条补 `note` 资源核实说明。
3. 用引擎复核：`normify status`（就绪 9 / 阻塞 7）、`normify validate`（0 错误）。
4. 未 start 被修任务、未 abandon/promote、未动 backlog 条目；证据见 `reconcile.json`。

## 边界

- 只改 `depends_on` 与 `note` 字段；未改 acceptance/checks/files/status；未开始任何功能任务。
- 未改产品代码、测试、容差、fixtures；未跑 GPU。
- 「ready」只表示依赖已满足，不代表外部资源或产品结论就绪（见上表资源评估）。
