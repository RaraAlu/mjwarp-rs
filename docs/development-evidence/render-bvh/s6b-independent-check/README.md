# 渲染/BVH S6b 独立复核（100 次 create/destroy / 陈旧 BVH / 显式 refit / 输出 / drop）

- 任务：`2026-10-24-render-bvh-s6b-independent-check`（QA；Arch 30 分钟 GPU 独占窗口，实测 GPU 约 1 分钟）
- 复核对象：`a171044`（S6b 功能提交，任务 `2026-10-24-render-bvh-s6b`），复核基线 HEAD `eea75f7`
- 日期：2026-10-25
- 结论：**验收面全部成立（0 阻断）**。100 次 create/destroy 显存增长 **0 B**（10 次采样全部等于基线、created/destroyed 各 +100）；陈旧 BVH 语义（不 refit 时 A/B 均 miss、显式 refit 后 B 命中 d=4.0、A miss、refits 0→1、render 不隐式重拟合/不改位姿）；禁用 depth/seg 的 `byte_len`/`read` 均 `DisabledOutput`、rgb 480,000 B/120,000 像素且新分配为零；drop 后 `retained=0 B`、context 占用 2,097,152 B、释放后可重建；5/5 GPU、2/2 host、531/0。
- **1 个非阻断证据问题（建议 `normify fix`）**：交付回执/README 中 5 个产物的**行数**与提交 blob 不符（哈希、字节均正确，见 §6）。

## 1. 复核方式（独立重算，不读交付测试断言）

| 检查 | QA 独立做法 |
| --- | --- |
| 100 次循环 | QA harness（`target/qa-s6b-harness`，产品 crate path 依赖、不导入交付测试）预热 1 次后取 `device_memory()` 基线，循环 100 次 create→render→destroy，每 10 次采样 free 字节，比较增长；`release_stats()` 计数差 |
| 陈旧 BVH / refit | 单 box 场景：初始 A 命中 d=4；`update_poses` 到 B 不 refit → A/B 均 miss、refits=0、位姿未被 render 修改；`refit_bvh()` 后 B 命中 d=4、A miss、refits=1；后续 render 不再改变 refits |
| 禁用输出 | 只开 rgb：`byte_len/read` 的 depth/seg 均 `DisabledOutput`；rgb 字节=480,000、值长 120,000、全零 |
| drop | 预热两次后取基线：带网格+3 纹理的 context 占用、`set_points` 后 mesh 陈旧（-1）→ 显式 refit 后命中（0）、纹理采样有限；`destroy()` 后 retained；销毁后重建成功 |
| 证据/边界 | 5 个产物 git blob 哈希/行数/字节；窗口内 GPU 显存记录；`cargo test --locked` 531/0 |

## 2. 逐项结果

### ① 100 次 create/destroy（成立）

```text
baseline_free=15,811,477,504 B
10 次采样 = [15,811,477,504 × 10]（全部等于基线）
free_after=15,811,477,504 B  growth=0 B（阈值 16,777,216 B）
created_delta=100  destroyed_delta=100  elapsed≈6.2 s
```

- QA 独立运行与交付 GPU 用例（6,570 ms）数字一致；无 UAF（`destroy(self)` 消费所有权，所有访问经 owner 借用）。

### ② 陈旧 BVH 与显式 refit（成立）

```text
initial: geom=0 distance=4.000000000
update_poses(B), 不 refit: stale_b geom=-1, stale_a geom=-1, refits=0, 位姿未被 render 修改
refit_bvh(): refits=1 → fresh_b geom=0 distance=4.000000000, fresh_a geom=-1
之后 render 不再改变 refits（=1）
```

- 与上游语义一致：陈旧 BVH 仍可渲染（命中反映旧加速结构），重拟合只能由显式 `refit_bvh()` 触发。

### ③ 无隐式刷新（成立）

- render 前后 `refits()` 恒 0（refit 前）/ 1（refit 后）；`scene().poses()` 在 render 前后不变（`render_changed_poses=false`）；render 不触发物理/不写位姿。

### ④ 禁用输出（成立）

- 只开 rgb：`rgb_bytes=480,000`（=120,000 像素×4 B）、`rgb_values_len=120,000`、全部为 0（新分配清零）；
- depth/seg：`is_enabled=false`，`byte_len` 与 `read` **均返回 `DisabledOutput`**（S6a 布局的 `rgb_buffer_bytes` 与产品一致）。

### ⑤ drop 释放回基线（成立）

```text
baseline_free=15,811,477,504 B
context in_use=15,809,380,352 B（占用 2,097,152 B）
destroy 后 free=15,811,477,504 B → retained=0 B
textures=3；mesh stale=-1 → fresh=0，mesh_refits 0→1
销毁后重建成功（无 use-after-free）
```

### ⑥ 32 MiB 一次性代码缓存观察（交付证据成立，QA 本轮未复现）

- 交付 GPU 用例：预热前 `retention=33,554,432 B`（32 MiB，首次加载 scene/mesh/sample 三个模块的驱动代码缓存），预热后基线，destroy `retained=0 B` —— 判定用预热后第二轮，阈值 16 MiB 未放宽；
- QA harness 本轮同一进程内 `warmup_retention=0 B`（模块加载时机在会话/预热内已计入），但 100 次循环增长 0、drop retained 0 结论一致；该观察属驱动一次性缓存语义记录，非泄漏。

### ⑦ 证据 / 门禁（成立，除 §6 行数问题）

- 5 个产物 **5/5 sha256+bytes** 与 git blob 一致；
- `cargo test --locked`：**531 passed / 0 failed**；交付 GPU 测试原样重跑：**5 passed / 0 failed**（测试体 7.56 s），两条主机用例（S6b 边界/拒绝面）在此前的 531 中执行；
- 窗口内 GPU 显存：开始 `2,899 MiB`、harness/测试 `2,871–2,878 MiB`、结束 `2,871 MiB`，无本任务计算进程。

## 3. GPU 窗口

- 开始 `2,899 MiB`（桌面基线，驱动 596.36 / CUDA 13.2 / RTX 4070 Ti SUPER，无其他满载任务）；harness 100 次循环 6.2 s、交付 GPU 用例 7.6 s。
- 结束 `2,871 MiB`，无本任务 CUDA 计算进程，已回桌面基线（桌面应用波动 2.87–2.90 GiB）。

## 4. 未满足项 / 限制

- **像素渲染内核未实现**：RGB/depth/seg 着色、AA resolve、阴影/天空盒/聚光灯、纹理/顶点法线/镜面/自发光求值属 S7b（依赖 S7a 发射 ABI）；本片只做输出缓冲所有权与 BVH 生命周期，`render_rays` 是「渲染消费 BVH」的查询代理，不宣称图像验收；
- 预计算射线缓冲仍由 S5 `RayBuilder` 按需构建，不在 S6b owner 内（接入渲染 context 留 S7b/S8）；
- 多 world 场景 GPU 用例仅 `worlds=1`（多 world BVH 分组已由 S2a/S2b 验收）；
- 上游上下文 retain/push/pop 仅 S4 纹理对象路径需要，S6b 不新增上下文操作；
- 未跑全量 G01（fast-loop 口径）。

## 5. 复核产物

- `harness-report.json`：QA 独立生命周期数字（cycles/stale/refits/disabled outputs/drop/worlds mismatch）
- `artifact-check.json`：产物哈希/字节/行数对照（含 §6 行数差异）
- `product-gpu-test.log`：交付 GPU 用例原样重跑
- `unit-tests-summary.txt`、`gpu-window.txt`

## 6. 非阻断证据问题（建议 `normify fix`）

交付回执/README 的 5 个产物**行数**与提交 blob 不符，**哈希与字节完全一致**（属计数口径/陈旧计数，非内容差异）：

| 文件 | 记录行数 | 实际 blob 行数 | bytes | hash |
| --- | --- | --- | --- | --- |
| `src/render/context/owner.rs` | 364 | **403** | 13,054（一致） | 一致 |
| `src/render/context/lifecycle.rs` | 322 | **358** | 12,157（一致） | 一致 |
| `src/render/context.rs` | 397 | **426** | 14,743（一致） | 一致 |
| `src/render/mod.rs` | 359 | **368** | 11,393（一致） | 一致 |
| `tests/render_context_lifecycle.rs` | 477 | **497** | 18,258（一致） | 一致 |

建议同 S1 `field-diff.json` 行数那样用 `normify fix` 修正 README/receipt 的 lines 字段（不影响验收结论，也未改变「新文件 ≤500 行」判定：最大 497）。
