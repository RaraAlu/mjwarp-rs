# R06 多帧实体 A（含小修卡）—— QA 只读复核

- 任务：`2026-11-05-render-r06-flex-multiframe-entity-check`（owner QA；**QA 自登记**，Chief 2026-11-05 夜指派；登记提交 `b418cae0`）
- 复核对象（**最终状态 HEAD `a175c7f8`**）：
  - 实体卡 `2026-11-05-render-r06-flex-multiframe-entity`（verified；登记 `3fa62181`、start `895c1aad`、实现 `2a27c2fd`、数字校正 `241d702f`）
  - 小修卡 `2026-11-05-render-r06-flex-multiframe-entity-fix`（verified；登记 `ddcf78f8`、start `bb03af57`、实现 `a2d7d4c5`、裁决 addenda `a175c7f8`）
- 日期：2026-11-05（18:00–24:00 空闲段）；边界：**只读复核 + 只写本目录**；零 GPU、不采集、不执行 freeze、不改 tests/实体与修卡证据/spec/fixtures
- 方法：不采信执行方结论，独立静态/AST 核对 + 从落库 npz 重算 + guards 逐条哈希/aggregate 复算 + `-GuardsOnly` 与 CPU 用例复跑；QA 自检 **41/41 PASS**（7 条低观察）

## 1. 结论

**PASS-with-observations（0 阻断）。** 小修卡两项窗口前必修均已落实并对最终状态核过：① 跨帧判据 **cam2-only**（`change_pixels_cam2` 为唯一断言口径，全相机合计仅诊断；独立重算 cam0/cam1=0、cam2=`(1167,1074)`/`(1262,1126)` 逐值、384=验收阈）；② **选项 A 诚实口径**（不补 body_pos 偏移步骤，裁决留痕齐全，实现/证据无「偏移模型建 context」现行表述）。逐帧判据、参考完整性 15 文件、provenance、allowlist 边界、guards 2+26=28（aggregate `b8110cee…` 复算）、回归面 498/210/≤500、四门禁 unit 1359/0 全部通过。7 条低观察均属诊断/文案/验证强度/不可改写卡面文字，不影响判据与数值。

## 2. 逐项核对

### 2.1 cam2-only 跨帧判据（PASS）
- `tests/support/render_bvh/multiframe.rs`：新增 `CAM2=2`、`cam2_slot()`、`change_pixels_cam2()`（**唯一判据口径**）；原 3 相机求和改名 `change_pixels_all_cameras()`，只在 CPU 自证⑦与 GPU 跨帧段 `println!` 诊断。
- 断言接线：`verify_change_pixels()` 只用 `change_pixels_cam2`（≥384）；CPU 自证 ⑤⑥ 用 `change_pixels_cam2(0/1)`；GPU 用例跨帧段 cam2 断言 + 全相机诊断打印；全相机合计无 ≥384 阈值断言。
- **独立重算**（落库 npz，逐相机）：`000→001` cam0/cam1=(0,0)、**cam2=(1167,1074)**；`001→002` cam0/cam1=(0,0)、**cam2=(1262,1126)**；合计与 cam2 逐对相同（本夹具 cam0/1 结构性为 0）。
- 阈值：`CHANGE_PIXELS_MIN=384`（spec §5 验收），576 仅采集余量、不作验收阈（全文无 576 阈值用法）。

### 2.2 选项 A 口径与裁决留痕（PASS）
- 实现/证据/脚本中**无「偏移模型建 context」现行表述**（`git grep` 限定 tests/docs/scripts；`artifacts/fix.diff` 内的出现是被删除旧措辞的 − 行，不计）。
- 口径 = 逐帧 state 刚性位姿建 3 个 context + 刚性签名（叶几何+位姿位型）一致 + 同一 context 渲染对照 + `refits()==0`；spec §2 实现映射（模型 `body_pos` 偏移对 flex 渲染惰性）写进实体 README §5。
- 裁决留痕四处：实体卡 addenda（`a175c7f8`）、小修卡 acceptance #2、修复卡 receipt `option_a`（`no_body_pos_step=true`、`chief_ruling_at=2026-11-05`、`wording`、`entity_acceptance_immutable=true`）、实体证据 README §5、GPU 用例注释。
- 实体卡 intent/acceptance 原文仍含旧短语——verified 卡 acceptance 不可改写（normify 规范），以 addenda 留痕（见 O7）。

### 2.3 逐帧判据实现（PASS）
- rgb：逐相机 `max_byte_diff <= 1` 且 `beyond_1_lsb == 0`；depth：frame-000 仅 cam2 `(134,128)`、参考 seg `(flexid0,9)` 走 allowlist（ULP≤4、beyond4=0），frame-001/002 全严格 bit-first + allowlist 漂移 panic（停手报用户）；seg 逐像素全等；两跑 = **整块多相机缓冲** rgb/depth/seg sha 相等（更强、等价逐相机相等）。
- GPU 用例 `multiframe_renders_match_captured_reference` 1 个且 `#[ignore]`；CPU 自证 22 项；`#[test]` 共 23 个。

### 2.4 参考完整性 / provenance（PASS）
- 独立重算：12 个 npz sha 对齐 `capture.json.files_sha256`、3 个文本夹具 sha 锚定 = **15 文件**全对。
- provenance：spec `1d9fa5ce…`/10,506 B（文件实测一致）、三脚本 sha 逐项一致、`refit_calls=6`、`capture_checks.refit.actual_sequence=[0,1,2,0,1,2]`、`refit_policy=every-frame`。

### 2.5 guards 与回归面（PASS）
- guards：2 exes（pending）+ 26 inputs = **28**；26 条 input sha256/bytes 逐条与实际文件一致；aggregate 独立复算 = `b8110cee547a6a6a12dbc4b8201f1571bebb31f7450e2d252f0a4015484afb7f`；分类 input=25 / guard_script=1；`status=pending_freeze`、stash 目录不存在 → **未执行 freeze**；小修卡未动 guards ps1/json。
- 回归面：`tests/render_flex_validate.rs` **498 行零 diff**（实体卡+小修卡全程）；`allowlist.rs` **210**（仅 +14/−0）；两新 rs **499/496** 均 ≤500；`src/`、`fixtures/`、冻结 spec 相对实体卡登记前零 diff。
- 门禁：实体卡与小修卡各 fmt/clippy/clippy-cuda/unit 全过，unit **1359 过 / 0 败 / 400 ignored**。
- 复跑：`cargo test --locked --features cuda-probe --test render_flex_multiframe -- --nocapture` → exit 0、**38 passed / 0 failed / 1 ignored**，含 `pair 0: cam2=(1167,1074) all=(1167,1074)`、`pair 1: cam2=(1262,1126) all=(1262,1126)`；`-GuardsOnly` → exit 0、`GUARDS_STRUCTURE_OK …`（与证据一致）。

### 2.6 证据校正（PASS）
- 实体 README：476 残留清零、行数 499/496、§3 改「整块多相机缓冲 sha」、§4 cam2-only、§5 裁决段；entity `receipt.line_ratchet` 同步 499/496。
- 实体证据 manifest **8/8**、修卡证据 manifest **6/6** 自核；UTF-8/LF/无 BOM、裸 CR/TAB 0、无 U+FFFD；`artifacts/fix.diff` 与实际 diff（tests + 实体证据）逐字一致；修卡 CPU 日志 38/0/1 + cam2/all 两行诊断。

## 3. 观察清单（按 Chief 定级规则：只影响诊断/文案/验证强度/不可改写卡面文字 → observation，0 阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低 | `change_pixels_all_cameras()` 在 CPU 自证⑦参与「同输入两次调用相等」的确定性断言（**非 ≥384 判据断言**），与「只作诊断打印、不参与断言」字面略有出入 | 可保留（不影响判据）；下次触碰时把该断言改到 cam2 函数或注明为确定性检查 |
| O2 | 低 | 裁决留痕路径写 `receipt.json.criteria.option_a.chief_ruling`；实际**修复卡** receipt 为顶层 `option_a`（`criteria.option_a` 存在于实体卡 receipt） | 追溯按顶层字段即可；建议文案统一 |
| O3 | 低 | allowlist 漂移 panic 打印 `x/y/bits/expected/ulp/seg` 但缺 `flat`（卡面要求 `flat/x/y/...`）；strict 路径仍有 flat 信息 | 补 `flat={flat}` 一行即可 |
| O4 | 低 | `refits()==0` 只在每帧渲染**前**断言，渲染后仅 println；未真正证明「render 不 refit」 | 渲染后追加断言（验证强度，不影响判据/数值） |
| O5 | 低 | freeze 脚本预算抛错文案写「expected 16 frozen entries」（实际条件检查 28） | 文案改 28 |
| O6 | 低 | 实体 receipt 非声明句含 exact 禁词「上游等价」（否定式；README/代码无） | 改「与上游实现的等价性」，与既往口径一致 |
| O7 | 低 | 实体卡 intent/acceptance 原文仍含「偏移模型建 3 个 context」（verified 卡 acceptance 不可改写，规范禁止） | 裁决已由 addenda + 小修卡 acceptance + README §5 + receipt 留痕；如需字面清除只能重登记卡，不建议 |

## 4. 复现与证据

- `scripts/verify_entity_review.py`：QA 独立复核脚本（A 行数/边界、B cam2-only 独立重算、C 选项 A/裁决、D 证据校正、E guards/门禁、F 判据/参考/provenance）；只读、零 GPU、`sys.dont_write_bytecode`。
- `logs/verify-entity.log` / `logs/verify-entity.json`：**RESULT: PASS (41/41，7 observations)**。
- `logs/cpu-tests-repro.log`：复跑 CPU 用例（38/0/1 + cam2/all 诊断，exit 0）。
- `logs/guards-only-repro.log`：复跑 `-GuardsOnly`（结构自检，exit 0；未执行 freeze）。
- `facts.json`：机器可读事实（复核目标/判据/guards/回归/观察）。
- `manifest.txt`：本目录 sha256 + bytes + 证据目录内相对路径（首行 `#`，不自列）。

## 5. 非声明

- 本复核只确认实体 A（含小修卡）的实现口径与证据，**不产生验证窗结论**；不写 R06 整族通过、动画通用性、与上游实现的等价结论。
- GPU `#[ignore]` 用例的真实结论留验证窗执行（真冻结 28 条 → 基线/floor → monitor 60/60 → 零编译跑该用例 → 证据），由 Chief 发 token。
