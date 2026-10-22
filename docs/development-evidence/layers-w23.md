# W23 元数据重绑证明评审

任务：`2026-10-18-layers-m7-proof`。
日期：2026-10-22。
依据：`docs/architecture.md`、`docs/unified-launcher.md`、`docs/module-contracts.md`。
本轮只评审，不实现、不改代码、不新增依赖。
本轮结论不自动批准实现。
本轮不宣称元数据重绑已开放。

## 结论

有条件批准 W23 的目标契约：以 C4「后续规则」为准，受检元数据证明绑定内容、上下文、ABI、维度与版本，任何写入使证明失效，捕获图保留证明，重绑以新证明替换旧证明。
现有 W18 `Checked`、W22 槽位分类、A1-4 证明所有权登记与 M6 数值槽重绑可以作为底座，但**W23 接口尚未实现**：
捕获图不保留证明、没有证明替换/元数据重绑入口、内容摘要未参与发射复核、ABI 与版本未绑定。
首版继续禁止地址类重绑（M6 已实现并测试）；元数据重绑实现与负向验收另立交付任务。
条件未满足前不开放元数据重绑。

结论：**有条件批准目标契约，实现延后并另立任务**。

## 现状基线

| 事实 | 证据 | 结论 |
| --- | --- | --- |
| 证明绑定设备区间、维度与写入代际 | `src/runtime/launcher/checked.rs`：`Checked` 持 `DeviceSpan`、`M::Dims`、FNV-1a 摘要、`MetadataRevision` 与 generation | 内容/上下文/维度/写入代际已有承载 |
| 证明构造只来自宿主校验 | `assume_uploaded` 校验宿主镜像长度与 span、调用 `MetadataProof::validate`、记录摘要与代际；`Checked` 构造为 `unsafe` 且需宿主镜像 | 受检类型不可凭空伪造 |
| 发射前复核证明 | `Kernel::plan` 对每个地址槽要求同槽 proof：`ProofKind` 相等、span 相等、dims 相等、revision 代际未失效；缺失/重复/多余 proof 拒绝 | 已实现 |
| 上下文绑定 | `DeviceSpan` 含 context；span 不等即 `MetadataMismatch`；跨上下文绑定由 `validate_bindings` 拒绝 | 已实现 |
| 维度绑定 | `CheckedBinding::validate_for` 比较 dims；实际发射维度的派生发生在宿主构造新 proof 时（如 `field_batch.rs` 的 `proof(dims)`、`rne` 的 cacc/cfrc/backward 派生） | 发射面已实现；图节点侧未保留 dims |
| 内容摘要 | `Checked::digest()` 只提供只读访问；生产发射路径不比较摘要，只有测试用它核对刷新前后一致（如 `mass_solve/sparse/mixed/solve/tests/gpu_checks.rs`） | 摘要已存但未作为发射/重绑的复核条件 |
| ABI 绑定 | 地址槽只声明 `SlotRole::AddressBearing { proof: ProofKind }`；`Kernel::plan` 只比较 `ProofKind` 名称；`PROOF` 常量多为 `...metadata.v1`，也有 `applied_force_static_metadata`、`kinetic_energy_nv` 等无版本后缀命名 | ABI/入口指纹未绑定，命名规范不统一 |
| 版本绑定 | 无配方/PTX/ABI 版本字段；`MetadataRevision` 是写入代际，不是内容版本或 ABI 版本 | 未实现 |
| 写入失效 | 产品元数据上传后不可写；`invalidate()` 在生产路径只出现在元数据缓冲的刷新/重建处（如 `applied_force`、`rne`、`rigid_kinematics` 等 metadata buffer 的重建方法），现存 `overwrite_for_test`/`restore_for_test` 为 `#[cfg(test)]` | 失效协议存在；生产元数据刷新接口尚少 |
| A1-4 所有权登记 | `model::data::lifecycle` 重导出 `CheckedMetadata = Checked`、`MetadataProofOwner = MetadataProof`、`MetadataProofRevision = MetadataRevision`；`DataLifecycle` 另持 `metadata_generation` 与 `check_metadata_generation` | 所有权登记存在，但 physics 路径尚未接线（grep 只在定义与单测出现） |
| M6 数值槽重绑 | `GraphInstance::rebind_values` 只接受数值槽；地址类槽位直接 `AddressSlotRebind`；先提案、逐节点联合校验、通过后一次提交；失败返回原资源并保持原绑定 | 首版规则已实现并有测试 |
| 捕获图保留证明 | `CapturedNode` 只存 `AbiSignature`、绑定与最小跨度，不存 proof；`GraphInstance` 不持 proof | 未实现（C4 后续规则要求） |
| 旧证明/旧缓冲回收 | `RecycleRecord` 只持 `RetainedResources`；重绑等待在途重放完成后释放旧资源 | 资源回收已有；证明随旧缓冲回收未实现 |

M6 测试证据：`tests/launcher_graph.rs` 的 `rebind_rejects_address_slots_and_mismatched_buffers`、`rebind_checks_context_capacity_and_duplicates`、`rebind_rolls_back_alias_conflicts_and_keeps_old_bindings`、`rebind_holds_old_resources_until_pending_replays_complete`、`invalidated_graph_rejects_replay_and_rebind`、`graph_instance_plugs_into_m5_retained_resource_slot` 与 GPU 预分配用例；本轮没有新增测试。

## W23 目标契约（评审冻结建议）

| 绑定项 | 目标 | 当前 | 差距 |
| --- | --- | --- | --- |
| 内容 | 证明绑定元数据内容摘要；设备内容对外只读；任何写入使证明失效 | 宿主快照摘要已存；发射不复核；无只读设备类型 | 需要把摘要纳入发射与重绑复核，并冻结失效协议 |
| 上下文 | 证明绑定上下文（`DeviceSpan.context`），跨上下文拒绝 | 已实现 | 无 |
| ABI | 证明绑定 `ProofKind` 与 ABI 入口指纹（`AbiSignature` 的名称/槽位/块规则/别名例外，或配方摘要） | 只比较 `ProofKind` 名称 | 需冻结指纹来源与 v1 命名规范 |
| 维度 | 证明 dims 与节点捕获 dims 一致；重绑新证明 dims 必须等于节点 dims，不一致整批回滚 | 发射已校验；图节点未保留 dims | 捕获图需按地址槽保存 dims |
| 版本 | 证明绑定配方/PTX 摘要或显式 ABI 版本；版本变化使证明失效 | 无 | 需冻结版本字段来源与拒绝面 |
| 失效 | 写入前使代际失效；已失效证明不得用于发射或重绑 | `MetadataRevision` 已实现；生产路径尚未统一接线 A1-4 代际 | 需统一单一权威失效源 |
| 图保留证明 | 捕获时图实例按节点地址槽保留证明；重放前复核 | 未实现 | W23 实现任务 |
| 重绑替换 | 元数据重绑以 (node, slot, 新证明) 为单位；成功后新证明替换旧证明；旧证明随旧缓冲延迟回收 | 未实现（首版只数值槽） | W23 实现任务 |
| 首版限制 | 地址类重绑继续禁止；元数据重绑开放前另立负向验收 | M6 已按 C4 首版禁止地址重绑 | 维持不变 |

## 失效与重绑协议（建议冻结）

1. 元数据写入只能经证明所有者完成：写入前 `invalidate`，写入后重新 `assume_uploaded` 生成新证明；旧证明立即失效。
2. 证明身份 = `ProofKind`（命名与版本） + ABI 指纹 + `M::Dims` + `DeviceSpan`（含上下文） + 内容摘要 + 写入代际。
3. 捕获图按节点地址槽保存证明；重放前逐地址槽复核：代际未失效、span 与当前绑定一致、dims 与节点捕获 dims 一致、`ProofKind`/ABI/版本一致；任一不符拒绝重放，不静默回退。
4. 元数据重绑以 (node, slot) 为更新单位，附带新证明；先提案后整批校验：槽位必须是地址类、`ProofKind`/ABI/版本一致、dims 等于节点 dims、上下文一致、容量不小于原、整图别名联合校验通过；失败整批回滚并保留旧证明。
5. 重绑成功后旧证明随旧缓冲进入待回收列表，等待在途重放完成；连续重绑按提交顺序生效，一次可含多槽。
6. 失效来源：元数据写、revision 失效、图作废、上下文隔离；错误映射沿用 `StaleMetadata`、`MetadataMismatch`、`GraphInvalid`、`AddressSlotRebind` 等。
7. 访问账本仍按节点访问登记；元数据重绑不改变读写区间语义，不新增隐式依赖。
8. 负向验收至少覆盖：错 `ProofKind`、错 ABI/版本、错 dims、跨上下文 span、span 容量变小、失效代际、重复更新、未知节点/槽、别名冲突、部分失败回滚、作废图拒绝重绑。

## 现有风险

| 风险 | 说明 | 缓解 |
| --- | --- | --- |
| 内容摘要不复核 | 旁路写设备内容不会被发现 | 元数据写入只经 owner；重绑/重放前复核摘要或证明构造路径 |
| ABI 指纹缺失 | 不同入口若复用同名 `ProofKind` 可能误配 | 冻结 ABI 指纹并纳入证明身份 |
| 命名规范不统一 | 部分证明名无版本后缀 | W23 冻结命名与版本规范，旧名保留兼容映射 |
| 两套失效表示 | `MetadataRevision` 与 `DataLifecycle::metadata_generation` 并存，A1-4 未接线 | 实施任务统一为单一权威失效源 |
| 图不保留证明 | 图重放可绕过证明生命周期 | 实现前不开放元数据重绑 |
| 地址类误重绑 | 重绑地址会绕过原校验证明 | 首版禁止；重绑提案阶段显式拒绝 |
| 旧证明回收 | 证明与资源回收未联动 | 回收记录同时持证明与资源，按在途重放代际释放 |
| 只覆盖 i32 元数据 | 现有证明对象为 i32 缓冲 | 冻结为同一类型；其他元素类型另行评审 |

## 影响面

| 对象 | 影响 |
| --- | --- |
| `mjwarp.runtime` 证明/图面 | 新增证明身份字段、图节点保留证明、元数据重绑入口；本轮不改 |
| `mjwarp.model::data` | A1-4 所有权/代际接线需统一；不改变公开兼容路径 |
| `mjwarp.physics` 元数据缓冲 | 生产元数据刷新需走 owner/失效协议；内核与容差不变 |
| M5 完成凭证/账本 | 图重放与重绑需保持资源持有与账本语义 |
| M6 数值槽重绑 | 保持现有行为与测试不变 |
| 产品精度 | 不改内核、PTX、容差、f32 路径 |
| G14/G15 | 后续求解器元数据若需重绑，依赖 W23 结论 |
| Linux | 同为宿主侧逻辑，流程另行评审 |

## 前置条件

实施任务启动前须满足：

1. 冻结证明身份字段：`ProofKind` 命名与版本规范、ABI 指纹来源、dims、上下文、内容摘要与写入代际的语义。
2. 冻结失效协议与元数据写入 API（owner-only，写入前失效、写入后重建证明），并统一 A1-4 代际与 `MetadataRevision`。
3. 扩展 `CapturedNode`/`GraphInstance` 按地址槽保留证明；定义重放前复核与错误映射。
4. 冻结元数据重绑 API 与事务语义：提案、整批联合校验、成功提交、失败回滚、作废图拒绝。
5. 冻结旧证明与旧资源的回收路径：在途重放完成后再释放。
6. 冻结拒绝面与负向验收清单（见上节）。
7. 首版继续禁止地址类重绑；数值槽重绑路径与测试回归不变。
8. 内核字符串、PTX、容差、fixtures 不动；无 Python 构建或运行链。
9. 验证覆盖：宿主反例 + GPU 元数据重绑后重放对照（确定性路径逐位、原子路径沿用冻结容差），且不得豁免原容差。
10. Linux 另行评审，不与 Windows 同轮放行。

## 本轮验证

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| `cargo test --locked` | 通过，退出码 0 |
| `unchanged` 检查 | `src`、`native`、`include`、`tests`、`fixtures`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml` 未改动 |
| `git status` | 仅 `?.claude/` 未跟踪 |

本轮未实现证明身份扩展。本轮未改图捕获/重绑。本轮未开放元数据重绑。本轮未运行 GPU 重放测试。

## 依据

- `docs/architecture.md`（执行计划、地址变化例外、C4 引用）。
- `docs/unified-launcher.md`（C2/C3/C4、W18/W22/W23、M7 阶段）。
- `docs/module-contracts.md`（runtime 图捕获/重放与内容/地址失效边界）。
- `src/runtime/launcher/{checked.rs,launch.rs,abi.rs,alias.rs,graph.rs,submit.rs}`。
- `src/model/data/lifecycle.rs`、`src/model/data/mod.rs`。
- 元数据缓冲实现示例：`src/physics/rne/metadata_buffer.rs`、`src/physics/resident/sleep_copy.rs`、`src/physics/resident/field_batch.rs`。
- 已落地提交：W18 核心 `683f6e4`、W22 槽位分类 `330132c`、A1-4 所有权 `edfa582`、M5 `fbbefd9`、M6 `9ea28e9`、M4 `c0a56b8`。
- `tests/launcher_graph.rs`（重绑与回收测试）。
