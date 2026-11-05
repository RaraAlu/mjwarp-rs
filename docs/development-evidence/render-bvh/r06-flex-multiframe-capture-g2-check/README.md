# R06 多帧采集窗 G-2 —— QA 只读复核

- 任务：`2026-11-05-render-r06-flex-multiframe-capture-g2-check`（owner QA；**QA 自登记**，Chief 2026-11-05 夜指派；登记提交 `652b3a0f`）
- 复核对象：G-2 采集窗卡 `2026-11-05-render-r06-flex-multiframe-capture-g2`（**verified**）→ 窗口提交 `5f2bd0bb` + 证据补充 `38a7767c`；**复核目标 = HEAD `38a7767c`**
- 日期：2026-11-05（18:00–24:00 空闲段）；边界：**只读复核 + 只写本目录**；零 GPU、不采集、不改 G-2 证据/fixtures/spec/脚本
- 方法：**不采信执行方结论**，从落库 npz 独立重算三条硬自查与两段变化计数，独立核对 sha/bytes、provenance、窗口日志、编码卫生，并做 **fresh clone（core.autocrlf=true）字节安全**测试；QA 自检 **34/34 PASS**

## 1. 结论

**PASS-with-observations（0 阻断）。** G-2 采集有效（verdict `pass-margin`；cam2 `000→001` depth/seg = **1,167/1,074**、`001→002` = **1,262/1,126**，均 ≥576，独立重算一致）；三条硬自查与 provenance 独立复算全过；15 个落库文件 sha256/bytes 与证据 15/15 一致，`reference/` 与单帧参考零改动；窗口合规与 G-1 supersede 叙事正确；编码/卫生通过。**3 条低观察（O1–O3）不阻断**，其中 O1 为需要治理动作的字节稳定性建议（npz 安全，3 个文本夹具在 autocrlf 下不逐字节稳定）。

## 2. 逐项核对（7 条口径）

### 2.1 采集有效性（PASS）
- `capture.json.delta_selfcheck.overall = `**`pass-margin`**；两段 cam2 `depth_changed/seg_changed` = **1,167/1,074**、**1,262/1,126**（均 ≥576，阈值 384/576 未变）。
- **独立重算**：从落库 `frame-{000,001,002}/{depth,seg}.npz` 的 cam2 数组按位重算，得同一组数字（1167/1074、1262/1126）→ 与证据记录逐值一致。
- `two_captures_bitwise_identical=true`、`legacy_reference_untouched=true`；`refit.actual_sequence=[0,1,2,0,1,2]`、`actual_calls=6`、`capture.json.refit_calls=6`；单次采集、无重试。

### 2.2 三条硬自查（独立复算，全 PASS）
| # | 检查 | 独立复核结果 |
| --- | --- | --- |
| ① | frame-000 vs `reference/base/` 数组级位级全等 | 独立加载落库 npz 与 base：**15/15 数组位级全等**，mismatches=[] |
| ② | 附着顶点位移 = k×0.100 m（±1e-6，读采集侧 state） | 真实模型 `flex_vertbodyid`（附着 5 / world 1）+ 落库 `state.npz.flexvert_xpos`：均值 **0.0 / 0.100000012 / 0.2 m**，逐帧 |Δ| ≤ 1e-6 |
| ③ | 两段变化像素均 ≥576 | 与 2.1 同一独立重算：1167/1074、1262/1126 均 ≥576 |

### 2.3 落库物（PASS）
- `fixtures/render-bvh/scene-flex/multiframe/` 恰 **15 个文件**（3 帧×4 npz + `capture.json` + `cam-poses.json` + `README.md`），无多余/缺失。
- 15 个文件 sha256/bytes 与证据 `artifacts/files-sha256.json` **15/15 一致**；与 `capture.json.per_frame_files_sha256` 一致；证据 `artifacts/capture.json` 副本与落库 `capture.json` 同 sha。
- `fixtures/render-bvh/scene-flex/reference/`（`reference/base/` + 现有单帧参考）相对窗口前提交 `2b8be485` **零 diff**；窗口对 `fixtures/` 的改动只有新增 `multiframe/`。

### 2.4 provenance（PASS）
- `spec_sha256 = 1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6` / **10,506 B**，与实际 spec 一致；`refit_policy=every-frame`、`refit_bvh=true`。
- 三脚本 sha256 三处一致（`capture.json.provenance`、`artifacts/script-sha256.txt`、实际文件/窗口时点提交 blob）：`capture_render_flex.py` `f50e150e…` / `capture_render_flex_multiframe.py` `46d75002…` / `capture_render_flex_multiframe_checks.py` `ef77f38b…`。

### 2.5 窗口合规与 supersede（PASS）
- 基线三点 21:45:08/11/14：free **14,338 / 14,374 / 14,442 MiB** ≥ floor **11,688**（取严公式 max(min_free−3,072, 11,688)）、util 17% <40、compute_filtered=0、build=0。
- monitor **60/60** clean（CSV 60 行进证据：free_min **14,432** ≥ floor、util 2–4%、compute_filtered/build 全 0）；**S-2 未启用**。
- dry-run exit 0、`out_dir_created=false`、`refit_calls=6`；采集 **exit 0**、`out_dir_existed_before=false`（无预存残留）、无 `.repeat2` 残留；窗后 GPU 15,41/14,521 MiB、build=0 并释放锁。
- G-1 负结果证据与卡**保留不删**（相对窗口前零 diff），supersede 叙事与 G-1 数字一致（697/616、244/183 → 根因缺每帧 refit，同签名 spec/帧集重采 pass-margin）。

### 2.6 编码/卫生 + fresh clone（PASS，含 1 条观察）
- 证据 manifest 首行 `#`、**16/16 自核**；证据与 fixtures 文本文件 UTF-8/LF/无 BOM、**裸 CR/TAB 0 命中、无 U+FFFD**。
- **fresh clone（core.autocrlf=true，target 38a7767c）**：**12/12 npz 逐字节一致**（`.gitattributes` 的 `*.npz -text -diff` 生效）；证据目录 manifest 在 clone 中 **16/16** 自洽。
- **观察 O1**：3 个文本夹具（`capture.json`/`cam-poses.json`/`README.md`）在 fresh clone 中被 LF→CRLF（checkout 加 292/162/16 处 CRLF），sha256/bytes 与 `files-sha256.json` 记录不一致——不影响 npz 参考数据与 JSON 语义，但影响「按字节复核」的可复现性；建议补 `.gitattributes` eol 规则（详见 §3）。

### 2.7 非声明（PASS）
- 证据文本 `多帧验收通过`、`动画一致性已覆盖`、`上游等价` **0 命中**；收口口径为「采集有效 → 可进入验证窗（G-2 判据门）」；无发布依据声明。

## 3. 观察清单（按严重度，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低-中（字节稳定性） | fresh clone `core.autocrlf=true` 后 3 个文本夹具被转 CRLF，sha256/bytes 与证据记录不一致（npz 12/12 安全、证据 manifest 16/16 安全） | 补 `.gitattributes` 规则（如 `fixtures/render-bvh/** text eol=lf`，或 `fixtures/render-bvh/scene-flex/** text eol=lf`；npz 已被 `*.npz -text -diff` 覆盖），**无需改已有 blob**；属治理/发布准入卡动作，建议报 Chief 决定 |
| O2 | 低（证据措辞） | 证据 README §6 写 `capture-log-header.txt` 是「`capture.log` 前 11 行」，实际为前 **10** 行 + 合成 `# ---- end (exit_code=0) ----` 行（内容等价、计数差 1） | 下次触碰该证据时把措辞改为「前 10 行 + 合成 end 行」，或在文件头注明 |
| O3 | 低（证据措辞） | 证据 README §3 写 fixtures 每文件 sha「并逐条进入 `manifest.txt`」，实际 `manifest.txt` 16 条均为**证据目录内**条目；15 个落库文件 sha 在 `artifacts/files-sha256.json`（已复核 15/15） | 措辞改为「15 文件清单在 files-sha256.json，manifest 只含证据目录条目」；数据本身完整 |

## 4. 复现与证据

- `scripts/verify_g2.py`：QA 独立复核脚本（A 采集有效性/落库 npz 重算、B 落库物、C provenance、D 窗口合规、E 编码/卫生/fresh clone、F 非声明）；零 GPU、只读、`sys.dont_write_bytecode`。
- `logs/verify-g2.log` / `logs/verify-g2.json`：**RESULT: PASS (34/34)**，逐项结果与 3 条观察。
- `scripts/verify_fresh_clone.py`：fresh clone 字节安全复核（`git clone --no-checkout` → `core.longpaths=true` + `core.autocrlf=true` checkout → 15 文件 sha/bytes 与证据比对 + 证据 manifest 自核）；clone 落 `target/qag2-verify/clone`（不落仓）。
- `logs/fresh-clone.log`：clone 复核原始输出（npz 12/12 一致、3 文本夹具 CRLF、证据 manifest 16/16）。
- `facts.json`：机器可读事实（复核目标/判据数字/落库 sha/provenance/窗口/门禁/观察）。
- `manifest.txt`：本目录 sha256 + bytes + 证据目录内相对路径（首行 `#`，不自列）。

## 5. 非声明

- 本复核只确认 **G-2 采集有效**；**不声明**多帧判据整体通过、动画一致性已覆盖、与上游实现的等价性、任何发布依据。
- 下一步（多帧测试实体实现 + 验证窗设计）以本复核 PASS 为 G-2 判据门前置；判据执行/验证另窗另卡。
