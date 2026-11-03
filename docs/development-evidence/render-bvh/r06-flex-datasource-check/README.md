# R06-S2b 合并卡 QA 独立复核（T4 重钉 + 数据层）

任务：`2026-11-04-render-r06-s2b-t4-merged-check`（QA）。对象：合并卡 `2026-11-04-render-r06-s2b-flex-datasource-t4-merged`，提交 `1f199737`（32 文件 +1046/−24）；后续订正 `ceeb4554`（证据 README 11→12 行、fixtures 口径精确化、审计 addenda）。复核时点 2026-11-04，rustc/cargo 1.99.0，HEAD `12bbb4b5`。零 GPU；只写本复核记录。

## 结论

**PASS-with-observations**（数据/门禁/边界全部通过；1 条需修-低：证据 manifest 在 `ceeb4554` 改 README 后未重算）。不宣称 flex 渲染可用、不宣称 NP01/NP03 通过。

## 一、重钉 12/12 逐行复算（A0）

用父提交/HEAD 的**原始 blob LF-sha256** 独立复算，与 README §5 表逐行一致：

| # | 文件/条目 | old | new | 复算 |
| --- | --- | --- | --- | --- |
| 1 | `fixtures/precision/t4-current-kernels.json` / `src/physics/flex.rs` | `cb2977a3…` | `0299be86…` | ✓（old=父 blob、new=HEAD blob） |
| 2-4 | `fixtures/precision/g01-inputs.json` / `include/mjwarp_native_probe.h`、`native/model_probe.cpp`、`src/io/flex.rs` | `03151bf5…` / `6a01eb01…` / `c0f3e0bb…` | `e40aff13…` / `f5272854…` / `f74256b3…` | ✓ |
| 5 | `g01-inputs.json` / `src/model/flex.rs` | `8355b0f1…` | `477355d8…` | ✓ |
| 6-7 | `g01-packing-routes.json` / `src/physics/flex.rs`、`src/model/flex.rs` | `cb2977a3…` / `8355b0f1…` | `0299be86…` / `477355d8…` | ✓ |
| 8 | `g01-state-packing.json` / `src/physics/flex.rs` | `cb2977a3…` | `0299be86…` | ✓ |
| 9 | `docs/evidence/t4-g03-retest/flex-internal/map.json` / `src/physics/flex.rs` | `cb2977a3…` | `0299be86…` | ✓ |
| 10 | 同上 / `src/model/flex.rs` **五文件组合**（根+edge+face+input+position，`\n` 连接） | `b3603225…` | `9834e3a0…` | ✓（独立复算组合哈希两侧一致） |
| 11 | `flex-matrices/map.json` / 同组合 | `b3603225…` | `9834e3a0…` | ✓ |
| 12 | `rigid-rotate-recapture/previous-kernels.json` / `src/physics/flex.rs` | `cb2977a3…` | `0299be86…` | ✓ |

深度 JSON 对比（父提交 vs `1f199737`）：7 个文件**只有上述 12 处 sha 字面量变化、0 处其它字段变化**（上游 pin/revision、排除清单、断言逻辑、容差、参考均未动）；提交不含 `tests/`、reference/criteria 文件。

## 二、原生与前后证据（A1）

- `repin-before-fail-identity.txt`：`expected=cb2977a3… actual=0299be86…`（1/20 差异，exit 101）——与重钉目标一致。
- `repin-before-fail.txt`：重钉前全量失败块（lib recapture `unexpected lineage change`；`t4_input_precision` 31 passed/5 failed，drift 含 header/cpp/io-flex/model-flex/physics-flex）。
- `repin-after-pass.txt`：`t4_identity_live_check` **2/0**、`t4_input_precision` **36/0/12**、`recapture::tests_host` **2/0**。
- `native-verify.txt`：native additive 冒烟 **1 passed / 0 failed**（MuJoCo 3.12 夹具 `flex-tree.mjb`，nflex=4）。
- 证据 manifest 6/6（README/receipt/repin×3/native）中 **5 条 sha+bytes 通过**；README 一条见需修项。

## 三、零设备 diff 与边界（A2）

- 提交 `1f199737`：`.cu` **0 文件**、`src/render/kernel/cuda/` **0 文件**；fixtures 仅 4 个登记文件（`fixtures/precision/` 的 t4/g01×3）与 3 个 `t4-g03-retest` map 做身份重钉，其余 fixtures 0。
- 原生 ABI：`mjwarp_flex_render_info` 32 B / `mjwarp_flex_render_targets` 72 B（header static 字段 + `static_assert` 偏移），**`flex_edge` 未重复导出**（仅 edgeadr/edgenum，注释明确 flex_edge 留在 G01 字段集）。
- f64→f32 **单点**：`io::kinematic::native::convert("flex_radius", …)`（仅 1 处 `convert`；`flex_rgba` 为原生 float、仅 finite 校验；其余 i32 passthrough）。
- `normify check 2026-11-04-render-r06-s2b-flex-datasource-t4-merged` → **全部检查通过**（合并卡无 waivers；新卡注册即收窄 `unchanged src/render/kernel/cuda/`，旧手写 waiver 随原卡 abandon，已在 addenda 记录）。
- unit **1283 passed / 0 failed / 398 ignored**（`runs/…merged/unit.json`）；fmt/clippy/clippy-cuda 全 passed。

## 四、结论与发现

- **PASS**：12 处重钉可复算、只碰身份字面量；前后证据与产品一致；零设备 diff；口径未越界。
- **需修-低**：`ceeb4554` 只改了 README（9278→**9599** bytes），未重算 `manifest.txt` ⇒ 现 manifest 中 README 条目 stale（manifest `6c09930a…/9278`，实际 `9376d46c…/**9599**`）。建议重算 manifest（或按约定把 README 排除在 manifest 外），否则证据自校验不再 6/6。
- **观察**：卡 note 正文仍写「11 行」，与 addenda/README §5/manifest 的 12 行不一致（addenda 已订正，属预期但读者需看 addenda）；建议在下次 fix 时把 note 正文口径一并刷新。
- **观察**：合并卡无 `waivers` 字段（以收窄 check 取代旧手写 waiver），审计链靠 addenda 记录；本轮 `normify check` 已通过。

## 五、边界

零 GPU、零窗口；未改产品代码/判据/容差/fixtures；本文件与检查卡状态是唯一写入。不宣称 flex 渲染可用、不宣称 NP01/NP03 通过。

## 六、需修项闭合（2026-11-04 增量复核）

- 需修-低（manifest stale）已由 `88e19c44` 订正：`manifest.txt` 的 README 条目按实际 `9376d46c…/9599` 重算；QA 增量实测 **6/6 条 sha+bytes 通过（0 stale）**，证据自校验恢复 6/6。本复核结论闭合。
