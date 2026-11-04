# R06-S3b 验收收口证据独立复核 ——**PASS-with-observations（0 阻断）**

- 复核卡：`2026-11-04-render-r06-s3b-closure-check`（owner QA，p85；PASS 后由 Chief 汇总给用户）
- 对象：收口卡 `9351a339` → 证据提交 **`db082561`**，路径 `docs/development-evidence/render-bvh/r06-s3b-closure/`（README `49a4b5fc…`/6,188 B、`facts.json` `6ecc9e0a…`/4,557 B、manifest 首行 `#`、2/2 自核通过）
- 依据：Chief 2026-11-04 裁**选项①**（attempt6 `7a48205d` PASS 接受，不跑 attempt7）
- 纪律：只读、零 GPU、未重跑窗口、未改任何证据；写锁仅用于本证据落库

## 1. B 类前提（六项之首）——逐条独立核过

| # | 前提 | 独立核对结果 |
| --- | --- | --- |
| P1 | 冻结参考 6 文件 | **6/6 逐字节复算一致**：`cam-poses.json 9b6eabf2…`/3,987、`capture.json 4713e2c7…`/16,166、`depth.npz c3ee4adc…`/787,918、`rgb.npz 5ed50287…`/2,362,306、`seg.npz f3b00fa8…`/1,575,106、`state.npz 7f2fff50…`/1,748 |
| P2 | 同 GPU | `capture.json` provenance：`NVIDIA GeForce RTX 4070 Ti SUPER`、driver **`596.36`**（arch 89 / CUDA 12.9）——与声明一致 |
| P3 | 同 recipe | provenance 7 项与声明**逐项一致**：`use_fast_math=false`、`use_shadows=true`、`samples_per_pixel=1`、`flex_render_smooth=true`、`use_precomputed_rays=true`、`render_skybox=false`、`enable_backface_culling=true` |
| P4 | 同编译形态 | attempt6 窗口内 canonical rebuild：先 0.16 s（无重建）→ 强制 `cargo clean -p mjwarp-rs` 后 **1 次 Compiling / 14.25 s**，同刻记录 `exe_sha256=e48c7268…`、**107,854,859 B**、`lastwrite=23:03:33.909`（与声明一致）；产品 bin `c5014d83…`/85,041,741 B；guards `attempt=2 / status=frozen / 16 entries (2 exes + 14 inputs)`、guards sha **`c72caea0…`**/7,508 B、aggregate_all **`c5263d33…`**、aggregate_inputs **`ec6fa222…`**——按 json 声明算法**独立重算 2/2 一致**；frozen exes `stash==recorded` 2/2、`pending_freeze=false` |
| P5 | 同一判定实现 | `git diff --name-only d35cab68..8c15ad75 -- src/` **空输出**（只动 `tests/`） |

## 2. 签名 allowlist（`3406f1ca`）最小范围——逐字一致、无扩大

- README §2 与签名卡逐字对应：**仅 `cam2 / (x=134, y=128) / flexid0` 允许 `ULP ≤ 4` 且 `beyond4 = 0`；`beyond4 ≠ 0` 即失败**；
- 其余全部 bit-first：cam0/cam1、cam2 除该像素全部像素、所有 seg、所有 rgb；不得扩大、不得二次放宽、仅限本次验收——与签名卡 `3406f1ca` 逐条一致（我此前已核签名卡与材料；本次复核收口文本无扩大措辞）。

## 3. D8 的 321 px 与 1 px 分开——不混算

- D8 `a1cc22ae`：修复前 cam2 **321 px**（beyond4 297、maxULP 259、seg 0/76,800）；
- f32 修复 r2 `d35cab68`：降到 **1 px**（beyond4=0）；attempt6 PASS 建立在该 1 px 走签名 allowlist 之上；两条记录分开引用、不重算不重解释（与 D8 证据一致，我此前已独立复算）。

## 4. 跨轮闭环三条链——均可独立验证

| 链 | 独立核对结果 |
| --- | --- |
| a) 实现未触渲染器 | `git diff d35cab68..8c15ad75 -- src/` → **空输出** ✓ |
| b) 两跑 sha 与采集轮逐位相同 | attempt6 `two-run-sha.txt`：rgb `d84ecaed…`、depth `98292153…`、seg `c4be9853…`；与 r2 `verification-summary.two_run_sha`（`two_run_sha_equal=true`）**三者逐位相同** ✓ |
| c) r2 像素明细 | 我按提交内 artifacts 复算：`flat=41094`、产品 `0x400015ba` vs 参考 `0x400015b7`、**ULP=3**、`beyond4=0`、seg 双方 `[0,9]`、flexid0；cam2 flex **654 px = flex0 403 + flex1 251**；全 cam2 失配数 **1** ✓ |

## 5. attempt6 PASS 与窗口纪律（复核收口所引原始记录）

- `entity-run.log`：`running 1 test` → `test result: ok. 1 passed; 0 failed; 17 filtered out; finished in 1.41s`；**0 个 Compiling 行**、`Finished ... in 0.30s`（冻结后零编译/零 relink）；`freeze-attempt2.log`：`GUARDS_OK=16/16 exes=2 inputs=14`（aggregate/guards sha 与 guards-after.json 相同）；
- `monitor-official.csv`：**60 行全部 clean**（util 2–7%、free_min **14,414** ≥ floor 11,688、compute_filtered/build 全程 0）。

## 6. 非声明与索引

- 非声明齐全：不宣称上游等价 / parity pass / flex 渲染整体可用 / 对外发布依据；allowlist 仅限本次验收；
- **索引 11 项**（facts `evidence_index`）逐一 `git cat-file -t ^{commit}` 解析：**11/11 均为真实 commit**（attempt6 `7a48205d`、r2 `d35cab68`、allowlist 实现 `8c15ad75`、QA 复核 `1927d62e`、签名卡 `3406f1ca`、attempt5 材料 `ecc93b56`、D8 `a1cc22ae`、tie 负结果 `9a1f102b`、attempt5 `6e9aa585`、attempt6 计划 `0a176abf`、增强 backlog `5dc62d66`）。

## 7. 观察（不阻塞收口）

1. **O1（低，建议修）**：attempt6 的 `two-run-sha.txt` 带 **UTF-8 BOM**（`EF BB BF`），使 `check-evidence-bom.ps1` 从 **128→129** non-.ps1 BOM 文件（基线漂移）。建议以最小治理动作剥掉该数据文件 BOM（不动 attempt6 README/其它证据），或明确更新基线；否则后续「BOM 基线 128+4」的说法不再准确。
2. **O2（低）**：attempt6 的 `entity-run.log`/`freeze-attempt2.log`/`canonical-rebuild.txt` 为 **UTF-16LE + PowerShell 包装噪声行**（`+ CategoryInfo ...`），grep/审计不便；建议后续窗口日志统一 UTF-8/LF（不阻塞）。
3. **O3（低）**：收口 README §7 索引表 **10 行**，而 facts `evidence_index` **11 项**（README 未列 `stats_print_backlog: 5dc62d66`）。两者不矛盾但宜一致；建议 README 表补该行（或注明 backlog 属后续增强）。
4. **O4（低）**：收口 README 的「缺逐相机运行时打印」已如实登记并由跨轮闭环覆盖，我确认该缺口的替代证据链成立（§4a–c）；后续增强卡 `5dc62d66` 不阻塞收口——与卡面一致。

## 8. 结论

**PASS-with-observations（0 阻断）**：收口证据的六项必含内容均可独立复核、数字与既有证据逐位一致；R06-S3b attempt6 = VALID_ROUND / PASS 的收口结论**成立**（在 B 类前提同时成立、签名 allowlist 逐字范围内）。观察 1–4 均为低级别、不改变收口结论；O1 建议在对外汇总前处理（BOM 基线）。

不宣称上游等价 / parity pass / 对外发布依据；本卡只核收口证据，不重开 attempt6 判定。

## 9. 复现

```powershell
$ev = "docs\development-evidence\render-bvh\r06-s3b-closure-check"
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_closure_check.py" --out "$ev\logs\closure-check.log"
```

## 10. 产物清单

- `README.md`（本文）、`facts.json`、`manifest.txt`
- `scripts/qa_closure_check.py`、`logs/closure-check.log`

## 11. 闭合 addenda（O1/O3 已修，判定不变）

Arch 于复核后按观察做最小治理（各自独立提交、未动其它证据）：

- **O1（BOM 漂移）已修**：`ac18a4d1` 只剥 `r06-s3b-flex-attempt6/two-run-sha.txt` 的 UTF-8 BOM（**736 → 733 B**，去掉 BOM 后正文**逐字节相同**，新 sha `579ad3e6…`）；该提交非 `.normify` 文件**仅此 1 个**；attempt6 README 未动（blob `9b2bbde2…` 不变）。我复跑 `check-evidence-bom.ps1`：**`WARN (128 non-.ps1 BOM files, 4 manifest header violations)` —— BOM 基线已回 128+4**，对外汇总里的基线数字现在准确。
- **O3（索引表 10 vs 11）已修**：`e836418c` 在收口 README §7 补入 `attempt6 统计打印 backlog | 5dc62d66` 一行（现 **11 行**，与 `facts.evidence_index` 11 项对齐）；README 现 6,428 B / `ddb88b06…`，`facts.json` 不变（`6ecc9e0a…`/4,557 B），`manifest.txt` 已重算且 **2/2 自核通过**。
- **O2（UTF-16LE + PowerShell 噪声）/O4（明细缺口由跨轮闭环覆盖）**：按 Arch 决定**不修**，保留为已知观察（改编码会再动 attempt6 证据、收益低；O4 由 backlog `5dc62d66` 承载）——与我原意见一致。
- **判定影响：无**。两笔只改 BOM 与索引行，不动任何数值/结论；收口六项复核结论与 `8aec9619` 的 **PASS-with-observations（0 阻断）** 保持不变。复核脚本 `scripts/verify_addenda.py`、日志 `logs/observation-closure.log`。
