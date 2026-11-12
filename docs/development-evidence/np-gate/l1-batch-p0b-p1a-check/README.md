# L1 批量复核：P0-b 能力索引 + P1-a 设备层 trait 骨架（一张卡）

- 复核范围：
  - **片 1 P0-b**：commit `17de20895`（卡 `2026-11-12-p0b-capability-index`）；证据 `docs/development-evidence/plan/kernel-exec-index/`。
  - **片 2 P1-a**：commits `a6649a80`（S1 骨架）+ `522f8661`（S2 mock/单测/证据）；卡 `2026-11-12-g10-p5-kernel-exec-p1a`；
    证据 `docs/development-evidence/plan/kernel-exec-layer/p1a-device-trait-skeleton/`。
- 复核基线 HEAD `522f8661`；**本卡登记提交时仓库 HEAD 为 `d9c863d45`**——两基之间 3 个提交
  （`15901d403` 计划「计算物理层归纳」、`ca7d8fc92` 其登记 in_progress、`d9c863d45` A0 定稿措辞）均为 **L0 文档/治理**，
  未触及两片任何文件，故卡 `revision.before` 取登记时 HEAD `d9c863d45`（事实见 §6）。
- 口径：新 L1 流程（一张卡一次 + 风险抽查 30%；必查判据数字/断言内容/边界/冻结件身份）；L1 证据不要求 manifest/逐字节 sha/policy 全扫。
- **结论：PASS（0 阻断 / 0 严重 / 0 一般 + 1 低 + 3 观察）**——1 低建议进 errata-ledger 批量修，不返工。

## 1. 必查项（不抽样）

### 1.1 范围边界 ✅

- 两片 diff 路径核对：
  - P0-b `17de20895`：仅 `.normify/changes|runs` + `docs/development-evidence/plan/kernel-exec-index/{README.md,capability-index.json}`；
  - P1-a `a6649a80^..522f8661`：`src/runtime/kernel_exec/` 9 个新文件 + `src/runtime/mod.rs` **仅加 1 行** `pub mod kernel_exec;` + `.normify`（卡/模块索引/runs）+ P1-a 证据 4 文件。
- **零 `.cu`/recipe/PTX/ABI/fixture/冻结集改动** ✅：`git diff --name-status 17de20895^..522f8661 -- '*.cu' '*.ptx' abi src/runtime/transfer fixtures scripts` 为空。
- `src/runtime/mod.rs` diff 恰为 1 行（`+pub mod kernel_exec;`）✅。
- 两片证据目录以外无新增文件 ✅。

### 1.2 行为（P1-a 先包不改）✅

- `kernel_exec` 在 `src/` 内**无外部引用**：全仓 `src/**/*.rs` 中除 `src/runtime/kernel_exec/**` 外仅 `src/runtime/mod.rs:7` 的模块声明 1 处；
  **未接任何调用点**、未改既有 `runtime`/其他模块逻辑。
- **29 个 `.cu` 源树 digest 复算 = `266bbf94fd92cce3202159df196d84824dacbaa6d22f4b848eddd83ac57fcb44`** ✅（与冻结值一致）。
- **现有测试：`unit.json` = 1435 passed / 0 failed / 402 ignored**（= 原 1410 + 新增 25）✅；`test.log` 25 passed（934 filtered）与 `unit-summary.log` 一致。

### 1.3 接口层（P1-a）✅

- `src/runtime/kernel_exec/` 内 `cudarc` 命中 **0**、`#[allow(dead_code)]` 命中 **0**、无 lint 配置/Cargo 改动（diff 空）✅。
- `unsafe`：文本命中 3 处**全部是注释/文档**（"不做宿主 unsafe 解引用"等）；`unsafe` 块/`unsafe fn` 实测 **0** ✅。
- `mock_backend`/`mock_tests` 为 `#[cfg(test)]`，模块经 `pub use` 从 crate 根可见（无 dead_code 豁免）✅。
- `NumericPolicy::FROZEN_F32 = { f32_only: true, fma: Enabled, fast_math: false }`，注释与 A0 §2 F.6（"f32 / FMA / fast-math 关闭"）及 `implementation-plan` F.6（"全程f32、乘加融合开、不开fast-math"）**一致** ✅。

### 1.4 行数（≤500）✅（但证据数字有 +1 偏差，见 §4 L1）

- 我按 **LF/`[IO.File]::ReadAllLines` 口径**（`split('\n')` 去尾部空行；不用 `Get-Content`）实测：
  `backend 200 / buffer 159 / device 168 / error 266 / mock_backend 456 / mock_tests 212 / mod 36 / module 86 / queue 151`，
  `src/runtime/mod.rs` **125**；全部 **≤500** ✅。
- README §1 表与 `artifacts.json::files[].lines` 写的是 201/160/169/267/457/213/37/87/152/126（**每个 +1**），
  `max_file_lines` 457（实测 456）→ 见 §4 L1。

### 1.5 P1-a 测试断言（25 个，抽 30% = 8 + 必查类别补核 3）

- **抽样方法（可复算）**：按 `sha256(test_name)` 升序取前 8 条（25 的 32%）。
  抽中：`launch_defaults_use_frozen_numeric_policy`、`three_classes_are_not_merged`、`device_side_traits_are_usable_via_generic_bounds`、
  `compile_options_default_to_frozen_policy_and_carry_defines`、`module_compile_returns_kernel_and_rejects_empty_source`、
  `submit_is_not_implicitly_synchronous`、`dtype_sizes_are_explicit`、`launch_error_variants_are_distinct`。
- **必查类别补核（性质必查，不参与抽样）**：`range_check_is_explicit_on_overflow`、`allocator_capacity_is_explicit`、`launch_and_capture_are_counted`。
- 断言内容核验 ✅：
  - **错误三分类**：`three_classes_are_not_merged` 断言 `Compile/Launch/DeviceLost` 的 `class()` 分别返回对应 `ErrorClass` 且非 unsupported；
    `unsupported_is_explicit_and_not_one_of_three` 断言 `class()==None` + `is_unsupported()` + Display 文案；三分类不合并，`Unsupported` 显式单列。
  - **容量/越界显式**：`allocator_capacity_is_explicit` 断言 `CapacityExceeded{requested:32,capacity:16}` 与越界上传 `OutOfRange`；
    `range_check_is_explicit_on_overflow` 断言精确 `OutOfRange{offset,len,byte_len}` 且 **u64 溢出也显式越界（不 panic/不回绕）**。
  - **submit 不隐式同步 vs wait_idle**：`submit` 后 `idle=false`，`wait_idle` 后 `idle=true` 且 submissions 计数 1；语义与 README 一致。
  - **launch/capture/replay**：submit(D2) 后 capture 计数 1、replay 计数 1，断言各计数显式。
  - 抽样其余项：NumericPolicy 冻结默认、object-safe `dyn`/泛型边界、模块 compile 拒绝空源、DType 字节尺寸、launch 错误变体互异——断言均为具体值/枚举匹配，非"跑通即可"。

### 1.6 P0-b 数据（85 条）✅

- **JSON 可解析 ✅**，顶层 `row_count=85`、`counts={done:43, part:25, none:14, n/a:3}`、`gpu_used=false`、`upstream_rev=71da24d…`。
- **id 唯一 ✅**：85 个 `capability_id` 无重复、无空。
- **字段无空 ✅**：`contract`/`evidence_anchor`/`a1`/`owner`/`done_criteria`/`status` 全 85 行非空；
  `done_criteria` 四子键（contract/cuda_backend/test_or_evidence/pu_map）逐行齐全。
- **计数 43/25/14/3 = 85 ✅**（与 README 自检、卡 `/counts/part=25` 一致）。
- **part+none = 39 行，owner 39/39 非空 ✅**；词表合规：`PU-xx（待批）` 或 `backlog（…，未建卡）`（含 `PU-18a/PU-18b（待批）`）；
  抽样 26/85 行（sha256 升序前 26）**0 字段问题**。
- **n/a 3 项 ✅**：H.7/H.8 为"当前手写 CUDA 架构下不适用"，D.10 为"并入 I.3（待并入）"，status_note 均非空。
- **A1 pin ✅**：README 四文件 sha/bytes 与 target 实际 **4/4 逐条一致**（A1 inventory 37,413 B / host 140,411 / kernel 146,631 / hotspots 8,067）。
- **A0 映射抽查 ✅**：`E.1→DeviceMath（标量13）`、`F.6→NumericPolicy`、`K.1→Graph::capture/replay`、`L.1→Queue::wait_idle()`
  与 A0 §3 表逐条一致；设备层 44 行 = A0 §3「44 项」。
- **未改 A0/implementation-plan ✅**：P0-b 提交只新增证据；未改判断性内容。
- **禁词 0 ✅**：对 JSON 全行扫描 README 声明词表（已实现/已验收/已通过/完成验收/通过验收/等价/缺口已清）= 0；卡 `forbid_claim`（4 词）检查通过。

### 1.7 证据真实性 ✅

- P1-a：`test.log` 25 passed、`unit-summary.log` 1435/0/402、`artifacts.json` 文件清单/门禁与 `.normify/runs` 一致；
  README 复跑命令与 `gates.toml` 命令一致。P0-b：README 行数/计数/自检与 JSON 实测一致。
- **L1 不建 manifest/不逐文件 sha ✅（不算缺项）**；两片证据均 UTF-8、无 BOM、0 NUL、0 CR。
- **Python 生成脚本只在 target ✅**：`target/impl-plan-review/gen_capability_index.py`、`check_capability_index.py` 在盘，
  两片提交无 `.py`、仓内无对应脚本、未进构建/测试链（Cargo/门禁配置无引用）。

## 2. 边界/卡状态

- 两卡均 `verified`（P0-b closed_at `15:15:50Z`；P1-a `15:27:36Z`）；P0-b 卡检查 10 项（4 gate + unchanged + 4 json + forbid_claim）、
  P1-a 卡检查 5 项（4 gate + unchanged）；我逐条解析 P0-b 的 4 条 json 指针 **4/4 满足**（task_id/gpu_used/row_count/counts.part）。
- 两片均 0 窗、零 GPU（P1-a 未跑 GPU 测试；P0-b `gpu_used=false`）✅。

## 3. 抽样清单（本次）

| # | 对象 | 条目 | 抽中 | 方法 | 结果 |
| --- | --- | --- | --- | --- | --- |
| 1 | P1-a | 25 个单测 | 8/25（32%） | sha256(name) 升序前 8 | ✅ 断言具体 |
| 2 | P1-a | 必查类别补核 | 3（errors/容量越界/launch-capture） | 类别必查 | ✅ |
| 3 | P1-a | 10 个文件行数 | 10/10（项少全查） | LF ReadAllLines | ✅ ≤500；数字 +1 偏差见 L1 |
| 4 | P0-b | 85 行能力条目 | 26/85（30.6%） | sha256(id) 升序前 26 | ✅ 0 字段问题 |
| 5 | P0-b | A1 pin 四文件 | 4/4 | 逐文件 sha/bytes | ✅ |
| 6 | P0-b | A0 §3 映射 | 4 行（E.1/F.6/K.1/L.1） | 原文对照 | ✅ |

## 4. 发现

### 低（建议进 errata-ledger 批量修，不返工）
- **L1 P1-a 证据行数全部 +1**：README §1 表与 `artifacts.json::files[].lines` 的 10 个值（201/160/169/267/457/213/37/87/152/126）
  均比 LF 实测（200/159/168/266/456/212/36/86/151/125）+1，`max_file_lines=457` 实测 456。
  判据（≤500）不受影响；建议改用 LF/`ReadAllLines` 口径重算或在证据里注明计数工具（本机 `Get-Content` 会多算 1）。
  处置（Chief）：不返工，进 errata-ledger；Backend 在 P1-b 首个提交顺手更正。

### 观察（记录即可；待 errata-ledger 建库后转入）
- **O1 P0-b「85 行」措辞**：README 标题「P0-b，85 行」指 **85 条能力行**，而 JSON 物理行 2,172；建议写「85 条（row_count=85）」，
  避免与文件行数混读。
- **O2 P0-b `level` 复合值**：实际存在「必；64位整数可退：双32位」等复合字符串，README 口径写「必/退/射」；属描述性复合值、可接受，
  建议 errata 注明允许复合写法。
- **O3 P1-a 门禁耗时两轮并列**：README §4 为「手动+normify 各一次」的手动耗时（fmt 1.77 s / unit 88.34 s），
  与 `.normify/runs` 的 gate 耗时不同轮次；建议加一句「两轮独立」避免误读为同一轮数据。

## 5. 结论

**PASS（0 阻断 / 0 严重 / 0 一般 + 1 低 + 3 观察）。** 两片范围/行为/接口层/行数/测试断言/P0-b 数据/证据真实性均通过；
L1 低项与 3 观察记录，待 errata-ledger 批量收口。

## 6. 与外部 agent 的事实报告（不处置）

- 只读复核期间（约 23:31:52 起）仓库写锁由 **`owner=claude`（pid 38312）** 持有，reason「plan doc: physics layer summary」；
  HEAD 由 `522f8661` 推进到 `ca7d8fc92`（卡片 `2026-11-12-physics-layer-summary-doc` 登记 in_progress，纯文档）；
  其暂存区有 `docs/implementation-plan.md` 等改动。**我未碰其文件、未写仓、未取锁**。
- 写卡阶段（23:34:57 前后）核对：锁 `REPO_WRITE_LOCK_FREE`、HEAD `d9c863d45`、树仅外部 Codex 两项 untracked、
  `.normify/locks` 空；经 Chief 授权后 `acquire -Owner qa` 成功（`pid=81384 owner=qa ttl_min=90`）才写本卡证据。
- 两基线（`522f8661` → `d9c863d45`）之间的 3 个提交均为 L0 文档/治理（`15901d403`/`ca7d8fc92`/`d9c863d45`），
  未触及 P0-b / P1-a 两片文件；本卡范围与结论不受影响。
- 我未在外部 Codex 两项 untracked（`.normify/changes/2026-11-11-code-size-quick-screen.json`、`docs/development-evidence/code-size-quick-screen/`）上做任何操作。

## 7. 非声明

本复核只对上述两片的范围/行为/接口/数据/证据真实性负责；**不宣称**任何能力已实现/已验收、mesh 容量缺口已清、
与上游等价（over-32）、G10@P5 整族通过、NP01/NP02 门禁通过、性能结论；**不作位级等价声明、不提供位级等价保证**；
零 GPU、零窗；不改判据/参考/容差/夹具/源码。
