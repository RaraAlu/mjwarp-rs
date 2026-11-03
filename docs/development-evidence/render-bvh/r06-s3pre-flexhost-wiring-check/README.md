# R06-S3pre FlexHostInputs 接线 QA 独立复核

任务：`2026-11-04-render-r06-s3pre-flexhost-wiring-check`（QA）。对象：`2026-11-04-render-r06-s3pre-flexhost-wiring`（verified），提交 `0b82c676`（12 文件 +579/−9；2 个源码文件 + 证据/卡/模块/runs）。复核时点 2026-11-04，rustc/cargo 1.99.0。零 GPU；只写本复核记录。

## 结论

**PASS-with-observations**（三态覆盖/边界/三条口径均成立；1 条需修：`check_sources` 有一条可触发的 panic 路径；2 条观察：无生产调用点、无 flex 两次运行 sha 证据改由 3 条机器可判口径替代）。不宣称 flex 渲染可用。

## 一、对象与门禁

- `src/render/context/flex_host.rs`（487 行）+ `src/render/context.rs`（+4：`#[cfg(feature = "cuda-probe")] pub mod flex_host` + re-export）；模块登记/证据/runs 同提交。
- `src/render/kernel/cuda/` 与全部 `.cu` **0 文件改动**；fixtures 0；f64→f32 在新文件中 **0 处**（`f64/mjtNum/convert/as f32` 命中 0，转换仍集中在 S2b 的 io 边界）。
- 门禁 runs：fmt/clippy/clippy-cuda passed；unit **1289 passed / 0 failed / 398 ignored**（merged 卡 1283 → +6，与本片新增 6 个单测一致）。
- 证据 manifest 3/3 sha+bytes 通过；`flex-host-unit-tests.txt` 与实跑一致。

## 二、三态单测（独立复跑）

`cargo test --locked --lib flex_host` → **6 passed / 0 failed**（独立复跑，与本片证据一致）：
- `no_flex_returns_none_deterministically`（无 flex → `Ok(None)`，两次一致）
- `single_flex_assembles_full_channel`（有 flex → `Ok(Some)`，9 数组/计数/映射逐项落位 + `validate` 通过）
- `multi_flex_multi_world_assembles_mapping`（多 flex/多 world，1D 每边一图元、2D 每 flex 一图元，`edgeid` 0/-1）
- `length_mismatches_are_rejected_explicitly`（xpos/bvh_id/group_root 三处长度不一致 → 匹配 `FlexWiringError::Length{field}`，不是只断言不 panic）
- `topology_mismatches_are_rejected_explicitly`（顶点链断裂、边端点越界 → `FlexWiringError::Count`）
- `assembly_is_bit_identical_across_calls`（同输入两次组装，xpos 逐位 + vertadr 相等）

## 三、边界与代码审查（A1）

- `FlexHostAssembly::build`：`nflex == 0` → 立即 `Ok(None)`（调用方走既有 `render`）；有 flex 依次 `FlexRenderFields::validate()` → `check_sources` → `geom_mapping()` → `FlexHostInputs::validate(nworld)` 第二层；错误全部为 `FlexWiringError`（`Length`/`Count`/`Model`/`Channel`），非测试代码**无 `unwrap/expect/panic!`**。
- `check_sources` 覆盖：`flex_vertadr` 长度=nflex、`flex_vertbodyid` 长度=nflexvert、`flex_edge` 长度=2×nflexedge、`flex_vertadr` 首 0+逐 flex 累加= nflexvert、每条边局部端点 < 对应 `flex_vertnum`、`flexvert_xpos`=3×nworld×nflexvert、`flex_bvh_ids`=nflex、`flex_group_root`=nworld×nflex；`FlexRenderFields::validate()`（model 侧）已保证 dim/radius/rgba/edgeadr/edgenum 长度与连续性。
- **需修（panic 路径）**：`check_sources` 未校验 `positions.flex_vertnum.len() == model.nflex`，但循环 `for flex in 0..model.nflex` 直接索引 `positions.flex_vertnum[flex]`（`flex_host.rs:200-203`）。`FlexPositionFields` 字段为 `pub`、无自身 validate，调用方传 `flex_vertnum` 较短（而 `flex_vertadr` 长度正确）时会**索引越界 panic**，违反本片 A1「不 panic、不静默回退」。修法建议：在循环前加 `expect("flex_vertnum", model.nflex, positions.flex_vertnum.len())?`（并顺带考虑 `FlexPositionFields` 长度不变量）。
- 观察（范围）：`FlexHostAssembly` 目前只有 re-export 与本模块单测使用；`render_with_flex` 在 `src/` 仍**无生产调用点**（README §1 给的是示例调用形态）。若卡面 A0 的「真实调用方接线」按生产路径理解，则接线尚未落地（留 S3 本体）；按「提供组装 API 供 S3 调用」理解则成立。
- 观察（口径）：检查卡 A0 要求「无 flex 路径同一输入两次运行 sha 相同」；本片证据 README §5 明确改为 3 条机器可判口径（①`render_with_flex(None)` ≡ `render()` 由 S2 实现保证、②零设备 diff、③`None` determinism + assembly 逐位一致），GPU 级两次运行位级证据**留 S3 窗口**。该替代口径已在证据写明，判为观察。

## 四、三条口径（A2）

1. 只交数据层/组装层：`src/render/kernel/cuda/` 与全部 `.cu` 零 diff；设备 flex 分支未实现 ✓。
2. 调用方接线不新增 S2c：卡面与证据一致，未注册 S2c；真实设备路径留 S3 本体 ✓。
3. 3.12 vs 3.15 只登记差异：本片未碰参考/判据/fixtures，未给数值结论 ✓。

## 五、结论与分类

- **PASS-with-observations**；**需修 1**：`check_sources` 的 `flex_vertnum` 长度缺口导致可触发 panic（建议在后续 fix 片补一条长度断言 + 一个「vertnum 短于 nflex 返回 Length 错误」的单测）。
- **观察 2**：无生产调用点；两次运行 sha 证据改为 3 条机器可判口径（S3 补）。
- 边界：零 GPU、零窗口；未改产品代码以外的判据/容差/fixtures；本文件与检查卡状态是唯一写入。不宣称 flex 渲染可用、不宣称 NP01/NP03 通过。

## 六、需修项闭合（2026-11-04 增量复核）

- 需修（panic 路径）已由 `55d2c264`（卡 `2026-11-04-render-r06-s3pre-flexhost-panic-fix` verified）订正：`check_sources` 循环前新增 `expect("flex_vertnum", model.nflex, positions.flex_vertnum.len())?`（`flex_host.rs:183`），不一致返回显式 `FlexWiringError::Length{field:`flex_vertnum`}`；新增单测 `short_flex_vertnum_returns_length_error`（:446），既有 6 测断言未动。QA 独立复跑 `cargo test --locked --lib flex_host` = **7 passed/0 failed**；unit 记录 **1290/0/398**；文件 499 行（<500）；仅改 `flex_host.rs`，`src/render/kernel/cuda/`/`.cu`/fixtures 零 diff。本复核结论闭合；两条观察（无生产调用点、两次运行 sha 由 3 条机器可判口径替代）不变、留 S3。
