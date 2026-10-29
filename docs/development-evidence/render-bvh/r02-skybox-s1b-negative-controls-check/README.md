# R02-S1b 负控断言更新 · QA 独立复核

任务卡：`2026-10-29-render-bvh-r02-skybox-s1b-negative-controls-check`（priority 57，Owner QA）
被核对象：`2026-10-29-render-bvh-r02-skybox-s1b-negative-controls`（**verified**）
被核提交：`f2ee32d0`（测试）→ `1d3c7159`（证据）→ `df24283c`（收口）
复核基线 HEAD：`8a967468`。**只读**：未开 GPU、未跑 `--ignored` 设备用例、未改代码/证据。

---

## 1. 结论

**verdict：pass / 阻断 0 / 观察 6。**

本片把天空盒解锁后的 3 处 stale 负控从「必须被拒绝」改成「必须被接受并发射成功」，我用 diff 逐断言核过：**没有删断言或弱化成「只看不 panic」**，反而在新行为面增加了更强的断言；仍不支持的动态射线与 splat 负控保留。`src/**` 零改动、`e2e` 净 **−4**（≤500）、unit 我自跑 **1104 / 0 / 390（136 result 行）**，均与证据一致。

更硬的一条交叉证据（我另查）：S3 窗口 attempt2（在本片之后执行）的三个实体日志里可见**新断言对应的 println 文案**并 `test result: ok.` ——
`render_e2e`：`s8[negative] shadows+skybox accepted; dynamic-rays 写前拒绝 + AA 射线数…判定成立`；
`render_kernel_flags`：`s7b3 rejection surface: AA + shadows + skybox supported; precomputed=false rejected; bad AA …`；
`render_kernel_rgb`：`s7b2 rejection surface: shadows+skybox supported; precomputed=false/splat rejected …`。
⇒ 新断言不仅在静态上正确，也已在设备侧被真实执行并通过（该日志属 S3 证据，本卡只作交叉印证）。

---

## 2. 三份测试的断言审计（复核点 1）

| 文件 | 行数（LF，我自算） | 移除 | 新增 | 保留的负控 |
| --- | --- | --- | --- | --- |
| `tests/render_e2e.rs` | 500 → **496**（净 −4，≤500 ✓） | skybox 拒绝 case + `assert_eq!(rejected, 2)` 计数 | `resolved().skybox.is_some()`、`render` 返回 `Ok(())`、`outputs().read(ImageOutput::Rgb).is_ok()` | 动态射线 `Unsupported`（直接调用，panic 会失败）+ AA `InvalidArray("ray_count")` |
| `tests/render_kernel_flags.rs` | 475 → **495**（≤500 ✓） | skybox 拒绝 case | context 可创建（catch_unwind → panic!）+ `skybox.is_some()` + `render Ok` | 动态射线 `Unsupported` |
| `tests/render_kernel_rgb.rs` | 270 → **309**（≤500 ✓） | skybox 拒绝 case | 同上的接受面断言 + **新增 splat 负控**（`SplatInputs` 四组参数 → `Unsupported`） | 动态射线 `Unsupported` + splat `Unsupported` |

- 「是否有弱化」：**没有**。三个文件的新断言都钉在「**新行为**」上（context 能建、`skybox.is_some()`、`render` 成功），而旧断言钉的「必须被拒绝」在 R02-S1 之后已不可能成立（S1 移除了 host 前置拒绝）。
- 「e2e 是否真净 ≤0」：**是**，500 → 496，净 −4；红线 500 未破。
- `flex`：我核实现侧当前拒绝面（`megakernel.rs` L98 动态射线 / L106 splat / L110 SDF / L113 light_mode / L136 enabled_geom_ids），**没有 flex 的前置拒绝**，所以「flex 不作负控候选（R06 范围）」的说法与代码一致 ✓。
- 三份文件都由 `#[ignore]` GPU 用例承载，本片不跑 GPU ⇒ 本片只能静态核；设备侧执行证据见 §1 的 S3 attempt2 交叉印证。

---

## 3. 仓库级 stale 扫描（复核点：是否漏掉其他 stale 断言）

我自己 `git grep -n render_skybox -- tests src` 得 **44** 条，证据 `materials/render-skybox-scan.txt` 列 **42** 条：

- **漏 2 条**：`src/render/kernel/cuda/render_megakernel.cu:30` 与 `:195`（R02-S1 的内核注释）。属注释、非断言，不影响「没有其他 stale 拒绝断言」的结论，但材料写「全仓（tests/ + src/）」略有超出（O2）。
- 我把每条命中按上下文分类：**没有**任何一条仍处于「必须在写前拒绝」的断言窗口内（`stale_rejection_hits = []`）✓；`tests/render_context_params.rs` 的 `render_skybox` 是解析断言（`render_skybox=true` → `skybox.texture_id/face_width`），`tests/render_lighting_params.rs` 是「skybox 与光照解耦」断言，均非拒绝断言 ✓。
- **另发现一处不含 `render_skybox` 字面的 stale 陈述**：`src/render/kernel/megakernel.rs:9` 模块文档仍写「阴影产品路径 R09-S2b 起已解锁（天空盒/splat/flex/hfield/SDF 仍写前拒绝）」。天空盒（R02-S1）与 hfield mesh（R05-S2）均已解锁，该括号清单已过时（O1）。因为不含 `render_skybox` 字面，扫描材料按字面 grep 抓不到——这也说明「按字面扫描」的局限。

---

## 4. 门禁与我自跑结果（复核点 4）

| 门禁 | 证据 | 我自跑 |
| --- | --- | --- |
| fmt | 0 | **exit 0**（`logs/fmt.log` 空） |
| clippy `--all-targets --features cuda -D warnings` | 0 | **exit 0**（`logs/clippy.log`） |
| `cargo test --locked` | 136 result 行 / 1104 passed / 0 failed / 390 ignored | **136 / 1104 / 0 / 390**（我逐 result 行求和，`logs/test-summary.txt`） |
| `--no-run` | 0 | 未重跑（证据记录 0；本片仅测试断言变更，编译已由 `cargo test` 覆盖） |
| `validate --fingerprints` | 0 错误 / 5 警告 | 未重跑（无模块变动） |
| GPU 用例 | 未执行 | 未执行 ✓（与本片声明一致） |

⚠ 派工消息写「unit **135** result 行」，证据与我的实测都是 **136 result 行**（1104/0/390）→ 以证据/实测为准（O6）。

---

## 5. 证据与范围（复核点 5）

- `git diff f2ee32d0^..f2ee32d0`：**只有** 3 个测试文件 + 任务卡；`src/**` 零改动 ✓。
- 整片（代码+证据+收口）：只有 3 个测试 + 本卡证据 + `.normify` 卡/runs ✓；未改判据/容差/参考/fixture。
- manifest：**5 行 = 5/5 committed blob = 5/5 现盘**，不自引用、无未列文件 ✓。
- 禁词：见 O5（README §6 在「不宣称…」句中字面列出了三个禁词）。

---

## 6. 观察（6 条，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | `src/render/kernel/megakernel.rs:9` 模块文档仍写「天空盒/splat/flex/hfield/SDF 仍写前拒绝」——天空盒（R02-S1）与 hfield mesh（R05-S2）已解锁，现有拒绝面是动态射线/splat/SDF/light_mode/enabled_geom_ids 覆盖不全 | 修注释（不含 `render_skybox` 字面，故 S1b 扫描未覆盖）；可随下一片带上 |
| O2 | 建议 | `materials/render-skybox-scan.txt` 标「全仓（tests/ + src/）」但漏了 `src/render/kernel/cuda/render_megakernel.cu` 的 2 条命中（我 grep 44 条 vs 材料 42 条） | 改口径为「tests/ 与 src/ 的 .rs」或补齐 2 行；结论不受影响 |
| O3 | 记录 | `render_e2e` 的 `outputs().read(ImageOutput::Rgb).is_ok()` 只证明「RGB 输出已启用 + 设备→主机读取成功」，**不校验内容**（全零缓冲也会 Ok）；证据写作「output buffer written」略强于断言事实 | 措辞改为「RGB 输出可读」；像素面由 S3 位级比较承担（已有） |
| O4 | 记录 | `render_kernel_flags.rs`（`let Ok(context) = created else { continue; }`）与 `render_kernel_rgb.rs`（`Err(_) => continue`）的动态射线负控保留「context 创建 panic 就静默跳过该 case」的预存在模式；`render_e2e` 版是直接调用（panic 即失败）。三处合并仍能判定，且本片未新增此类模式 | 记录；后续可统一为「创建失败即失败」 |
| O5 | 记录 | 证据 README §6 在「不宣称…」句里**字面**列出了 3 个禁词（逐条口径见 `recompute.json` 的 `forbidden_literal_hits`）；`normify check` 只按它自己的口径判定因此通过，但按「证据禁词 0」的字面规则不合格（该句是否定式，无任何结论性主张） | 改成不含字面的表述（例如「不写任何窗口/精度/等价类结论」）；我自己的证据用码点构造规避 |
| O6 | 记录 | 派工消息称 unit「135 result 行」，证据与实测均为 **136**（1104/0/390） | 以证据/实测为准；派工消息数字陈旧 |

---

## 7. 边界

1. 未开 GPU、未跑 `--ignored` 设备用例：本片改的三份用例都是 `#[ignore]`，我只能静态核断言；设备侧执行由 S3 attempt2 窗口覆盖（§1 交叉印证用的是 S3 证据，不是本卡重跑）。
2. 未重跑 `--no-run` / `validate --fingerprints`（证据记录 0；本片只改测试断言，编译已由 `cargo test` 覆盖）。
3. `outputs().read()` 的语义我是读源码判定的（只做 device→host 拷贝，不校验内容），未在设备上实测其失败面。
4. 未改代码/证据/卡；未产出任何渲染数值结论。

---

## 8. 复现命令（仓库根目录）

```text
target\render-bvh-precheck\venv312\Scripts\python.exe -B docs\development-evidence\render-bvh\r02-skybox-s1b-negative-controls-check\recompute.py
cargo fmt --all --check
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked
git grep -n render_skybox -- tests src
```

（脚本 `problems` 为空，见 `recompute.json`；门禁输出与 unit 汇总在 `logs/`。）

---

## 9. 收口

- 本卡证据提交：本目录；被核对象三段提交 `f2ee32d0` + `1d3c7159` + `df24283c` 保持不动（本卡只读）。
- 本片是测试修正片：设备数值验证已由后续 S3 窗口 attempt2 承担，本卡不重复也不替代。
