# R06-S3b 签名 allowlist 断言落库（GPU 用例实现签名 allowlist + CPU 三类自证）

任务：`2026-11-04-render-r06-s3b-flex-allowlist-assert-fix`（p80；只改用例，零 GPU、零判据改动）。
溯源：attempt5 `6e9aa585` FAILED_ROUND —— 判据实现缺口：**用例断言严于用户签的 allowlist**
（观测 cam2 depth `(1,0)`，落在签名允许范围内，但旧断言要求 `(0,0)`）；本卡把签名 allowlist
**显式实现进断言**，其余仍全部严格。本卡不重试 attempt5、不重跑 GPU、不改判据文本/容差/参考/fixtures。

## 1) 用户签的 allowlist（2026-11-04 22:14，签名级）

| 项 | 值 |
| --- | --- |
| 相机 | cam2（`ALLOWLIST_CAMERA = 2`） |
| 像素 | `(x=134, y=128)`，flat = `128*320+134 = 41094` |
| 图元 | `flexid = 0`（参考 seg = `(0, 9)`，代码里作为 allowlist 挂载正确性守卫） |
| 允许 | 该像素 depth `ULP <= MAX_DEPTH_ULP(4)` **且** `beyond4 = 0`（即 `ulp > 4` 仍失败） |
| 其余 | **全部严格**：cam0/cam1 全像素、cam2 其它像素 `bits == expected`（mismatched=0 / beyond=0）；所有相机 rgb、seg 全等、两跑 sha 逐一保持 |

## 2) 实现位置（两文件；受 500 行棘轮约束拆出支持模块）

主用例 `tests/render_flex_validate.rs` 原已 500 行，直接内联实现增长到 639 行被
normify `size/grew` 拒绝（上限 500）。因此拆分为：

- **`tests/support/render_bvh/allowlist.rs`（新增，156 行）**：显式常量
  `ALLOWLIST_CAMERA / ALLOWLIST_PIXEL_X / ALLOWLIST_PIXEL_Y / ALLOWLIST_FLEXID / ALLOWLIST_SEG_TYPE`
  + 生产断言路径 `assert_depth_allowlisted(camera, width, rendered, expected_bits, reference_seg)`
  + 4 条 CPU 自证（`#[cfg(test)]`，随主用例测试二进制运行）；
- **`tests/render_flex_validate.rs`（495 行，低于 500）**：仅新增
  `#[path = "support/render_bvh/allowlist.rs"] mod allowlist;` 与 `compare()` 内一行调用；
  rgb / seg 断言逐字未动。

断言语义：

- **严格区**：逐像素 `bits == expected`；任何失配立即失败，失败信息打印
  `mismatched / beyond / max_ulp / bits`（含 flat/x/y 定位）；
- **allowlist 像素**：单独判定 `ULP <= 4` 且 `beyond4 = 0`，失败信息打印
  `mismatched=1, beyond, max_ulp, bits, expected`；并校验参考 seg 为 `(0,9)`（防误挂图元）；
- 放宽只由 `camera == ALLOWLIST_CAMERA` 且 `flat == ALLOWLIST_PIXEL_Y*width + ALLOWLIST_PIXEL_X`
  同时命中触发，**不做相机级/区域级/条件式放宽**，无「总数减 1」。

改动：`artifacts/render-flex-validate.diff`（主用例 diff）；`artifacts/allowlist.rs`（新模块全文）。

## 3) CPU 三类自证（普通 `cargo test`，不碰 CUDA；`logs/cpu-selftest-run.log` 原始输出）

命令（无 GPU、无 `--ignored`）：

```text
cargo test --locked --features cuda-probe --test render_flex_validate -- --test-threads=1
```

结果：**17 passed / 0 failed / 1 ignored**（ignored = GPU 用例），其中新增 4 条
（均在 `tests/support/render_bvh/allowlist.rs` 的 `#[cfg(test)] mod tests` 内，直接调用生产断言）：

| 用例 | 构造 | 期望 | 结果 |
| --- | --- | --- | --- |
| `allowlist_pixel_within_4_ulp_passes_cpu` | allowlist 像素差 1 个 bit、ULP=3（`(1,0)`） | 生产断言 **PASS** | ok |
| `allowlist_pixel_beyond_4_ulp_fails_cpu` | allowlist 像素 ULP=5（`beyond4=1`） | 生产断言 **FAIL**，信息含 `超 4 ULP`/`mismatched=1` | ok |
| `non_allowlisted_pixel_mismatch_fails_cpu` | allowlist 像素 ULP=3 + 另一像素差 1 ULP | 生产断言 **FAIL**（严格区 `mismatched=1`、`bits=0x…`、`flat=100`） | ok |
| `allowlist_is_camera_scoped_cpu`（附） | 同一 flat 像素差 3 ULP 放到 cam0/cam1 | 两相机均 **FAIL**（证明未扩成相机级） | ok |

第 3 条输出的严格区 `mismatched=1` 是**正确口径**：严格区不统计 allowlist 像素（该像素单独判定），
所以严格区只看到那 1 个非白名单失配，不存在「总数减 1」。

## 4) attempt6 衔接

- 本卡 verified 后，attempt6 的执行序（另卡/Chief 控制）：`cargo test --locked --features
  cuda-probe --no-run --test render_flex_validate` 重建 → 按窗口规则**重冻 16/16（exes=2, inputs=14）**
  → 零编译跑实体 → Chief GO；本卡**不执行**这些步骤。
- 断言口径：cam2 depth 现在允许 `(1,0)/ULP=3/beyond4=0` 的这 1 个签名像素，其余保持 bit-first；
  与 `docs/development-evidence/render-bvh/r06-s3b-flex-f32-order-check/DECISION-SUPPORT-user-signature.md`
  的最小范围一致，allowlist 的签名/落库由 Arch 治理卡承载（本证据只记录用例实现）。

## 5) 纪律

- 零 GPU（普通 `cargo test`，无 `--ignored`）、未开窗口、未重冻、未改判据文本/容差/参考/fixtures；
- 只新增 `tests/support/render_bvh/allowlist.rs` + 修改 `tests/render_flex_validate.rs`（+ 本证据目录）；
  不动 r2/f32-order 与 tie-order 既有证据；
- 门禁：fmt / clippy / clippy-cuda / unit 全过；`unchanged fixtures/` 通过。
