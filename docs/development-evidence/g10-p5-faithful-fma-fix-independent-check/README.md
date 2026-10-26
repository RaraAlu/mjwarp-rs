# A 版 faithful FMA 修复（GJK `face_norm2` 显式非融合）· QA 独立复核

- 复核任务：`2026-10-25-g10-p5-faithful-fma-fix-independent-check`（QA，p54）
- 被复核任务：`2026-10-25-g10-p5-faithful-fma-fix-r2`，交付提交 **`50a0acd1`**
- 复核 HEAD：`50a0acd1`；性质：**纯 CPU**（自建 clean target / 只读副本 + 自写脚本），**未开 GPU 窗口**
- **结论：pass（阻断 0 / 发现 0 / 观察 5）**。只改 host `gjk/polytope.rs::face_norm2`（渲染归一化恢复
  `fma_f32`，属 r2 任务明确允许的 megakernel 段）；5 个 case 与 CPU 参考 **dist/pos/frame 全部分量位级 0 差异**
  （`dist 16/16`、`pos 48/48`、`frame 144/144`，max 0）；三构建全绿且判据行逐位相同；G5 pre/post 逐位相同；
  默认 `cargo test --locked` = **604/0/7**（111 binary）；判据/容差/参考路径/fixtures/gates/G01 一字未动；
  NP01 站点级登记实证；证据链与交付日志一致。

## 1. 范围独立核（`50a0acd1~1..50a0acd1`）

`git diff --name-status` = **31 文件 / +972 −14**（实际值；Arch 消息里的「6 文件 +223/−15」与提交不符，见观察 2）：
- 产品源码 2：`src/physics/collision/gjk/polytope.rs`（`face_norm2` → 源序两次舍入）、
  `src/render/kernel/megakernel_arrays.rs`（光源归一化恢复 `crate::math::fma::fma_f32`）；
- 证据 22：`docs/development-evidence/faithful-fma-fix/`（README/receipt/20 份日志含 `src.patch`）；
- NP01 2：`np01-type-mapping/{README.md,inventory.json}`；`.normify` 5（r2 任务 JSON + 4 个 runs）。

`qa-scope-static.json`：禁止路径（`relmat.rs`、`render/bvh/aabb.rs`、`case_compare.rs`、`case_evidence.rs`、
`gates.toml`、`scripts/`、`fixtures/`、`Cargo*`、旧 CPU 参考、GPU 参考）在该提交中 **0 命中**。
- `polytope.rs` **代码部分**：无 `mul_add`、无 `fma_f32`、无 f64；`head = r[0]*r[0]; tail = r[1]*r[1] + head;
  face_norm2 = r[2]*r[2] + tail;`，注释写明「对齐上游 CPU 参考 recipe（两次舍入），GPU PTX `fma.rn`
  属 device 侧口径」。
- `render` 光源归一化：`crate::math::fma::fma_f32(z, z, fma_f32(y, y, x*x))`（fused）。
- 全 `src/` 站点扫描分类：产品 host 非融合 = polytope（本片）；产品 host fused = render 光源、relmat（helper）、
  aabb 的 `fused_mul_add`（实现未动）；其余为测试/诊断；**产品代码里 `mul_add` 调用点 = 0**。
- 观察 1：QA acceptance 第 1 条字面写「render 不得改」，但 r2 任务 acceptance 明确允许
  「megakernel 段可复用」且 A 版设计就是 render 保留 helper；提交按 r2 任务执行，属在范围内（见观察 1）。

## 2. 5 case 位级（自写脚本独立复算）

方法（不读交付证据）：`git archive` r2 到 `target/qa-a-check/dump-tree`，仅追加一个 test-only dump 把
`upstream_mjwarp_contacts` 与 `product_contacts_upstream` 按产品 `sort_contacts` 排序后的逐行位型写入 JSON；
我的 Python 脚本再独立解码/重排/配对，并与 **CPU 参考 JSON**（`upstream-mjwarp-f32.json`）逐行对照
（`qa-5case-bitwise.json`；dump 测试源码与副本 sha 见 `qa-copies-info.json`）。

| case | count（world×2） | dist 位等 | pos 位等分量 | frame 位等分量 | frame max_abs |
| --- | --- | --- | --- | --- | --- |
| cylinder-box | 8（4+4） | 8/8 | 24/24 | 72/72 | 0 |
| ellipsoid-box | 2（1+1） | 2/2 | 6/6 | 18/18 | 0 |
| mesh-mesh | 2（1+1） | 2/2 | 6/6 | 18/18 | 0 |
| convex-mesh | 2（1+1） | 2/2 | 6/6 | 18/18 | 0 |
| degenerate-mesh | 2（1+1） | 2/2 | 6/6 | 18/18 | 0 |

合计 **dist 16/16、pos 48/48、frame 144/144，max_frame_abs 0.0**；dump 的 expected 行与 CPU 参考 JSON
逐行一致（`expected_rows_match_cpu_reference_json = true`）；无 count/结构差异。**任何非 0 位差都会阻断——
实测为 0。**

## 3. 三构建矩阵（clean `CARGO_TARGET_DIR`）

| 配置 | 工具链 / flags | print（`case_evidence`） | assert（`upstream_f32_within_4ulp`） |
| --- | --- | --- | --- |
| GNU 默认 | `1.99.0 x86_64-pc-windows-gnu`（仓库默认） | 3 passed / 0 failed / 2 ignored | 1 passed / 0 failed |
| MSVC | `+1.99.0-x86_64-pc-windows-msvc` | 同上 | 同上 |
| GNU + FMA | GNU 默认 + `RUSTFLAGS=-C target-feature=+fma` | 同上 | 同上 |

- 三构建的 **5 条 `[判据]` 行逐字相同**（`qa-three-build.json`），每条均为
  `dist 位等 n/n max 0 ULP`、`pos 位等接触 n/n max 0.00 ULP@尺度`、`frame 位等分量 9n/9n max abs 0e0`。
- `RUSTFLAGS` 交互实测：强制重编后 rustc 命令行含 `-C target-feature=+fma`、**不含** `link-dead-code`
  → 印证 `RUSTFLAGS` **替换** `.cargo/config.toml` 的 gnu rustflags（日志 `gnu-fma-rustc-line.txt`）。

## 4. G5 位级不变（pre/post 自跑）

pre = r2 parent `025d47fc` 只读副本；post = r2 主树；两者各自 clean target、同一命令、逐行对照
（libtest 顺序无关，按语义行多重集比较；`qa-g5-diff.json`）：

| 目标 | pre/post | 关键数字 |
| --- | --- | --- |
| `--lib relmat` | 6 行相同 | 4 passed |
| `--test g10_p5_sdf_relmat` | 13 行相同 | `rel_mat_warp 10/10 world`；6 条 `80/80` 逐位全等；dist/pos/frame max_abs 0e0 |
| `--test g10_p5_sdf_host` | 3 行相同 | 1 passed |
| `--test gjk_epa` | 10 行相同 | 3 passed |

→ `rel_mat 10/10`、SDF 40/40 与修复前逐位相同（diff=0），无回归。

## 5. 默认门禁（clean GNU target）

`cargo test --locked`：111 个 test binary，**604 passed / 0 failed / 7 ignored**（`qa-default-gate.json`）；
`case_evidence::upstream_f32_within_4ulp ... ok`、`our_contacts_match_upstream_f32_within_4ulp ... ok`
均在默认运行中出现，不依赖 seam/env；与交付 `.normify/runs/...r2/unit.json` 的 604/0/7 一致。

## 6. G01 定向（不跑全量）

- `host-gnu`（G01 内即默认 `cargo test --locked`）：我实跑 **604/0/7 绿**；**pre 也是绿的**
  （自跑 parent 副本：pass，ellipsoid frame **3.838112e-7**，靠非合规 libm 回退偶合）。
- `host-msvc`（G01 内为 MSVC 全量命令；交付只跑定向 `case_evidence`）：我自跑 parent 副本
  **FAIL（exit 101，frame 6.351978e-7）** → r2 **pass（0e0）**；交付「host-gnu 不是红、真正红→绿是 host-msvc」
  的更正与日志/实测一致（未夸大）。
- 全量 G01 未运行（证据无 `*g01*` 日志，交付 README 也如实写「未执行」）；G01 脚本在提交中未改动。

## 7. 抓凑绿（一字未动）

`git diff` 逐路径为空：`case_evidence.rs`（4.8e-7 / 4 ULP 仍在）、`case_compare.rs`（仍读
`upstream-mjwarp-f32.json` CPU 参考，不读 GPU 文件）、`fixtures/`、`.normify/gates.toml`、`scripts/`；
参考 sha 不变：CPU `a00c7228…20f04f`、GPU `d86c73cb…c87aa`（保持归档、非判据）。未发现任何为凑绿改
期望/阈值/参考/exclusions 的行为。

## 8. NP01 站点级登记（实证）

`np01-type-mapping/inventory.json` 新增 `hostSiteRegistration`（`qa-np01.json`）：
- `siteSemantics` **3 条**：`gjk_ccd_epa_host`（host=CPU recipe 两次舍入、device=pinned GPU PTX `fma.rn`）、
  `render_light_dir_normalize`（fused）、`relmat_relative_matrix`（fused）；
- `hostCoverage` **3 条**：`gjk_host`/`ccd_host`/`epa_host`，各含 productFiles、types（`doubleInProduct=0`）、
  recipe、upstream、已有证据与 missing 项的如实状态；
- 原 `rows` **30 条逐字未变**（`rows_unchanged=true`），其余顶层键未变；README 追加专节。
→ 不是「只在 README 写一句话」，登记内容可核。

## 9. 证据链事实审计

`qa-evidence-chain.json`：
- 交付 post 三构建 `[判据]` 行与我自跑三构建 **逐字相同**；pre 交付数值与我自跑 parent 副本一致
  （GNU 3.838112e-7 pass、MSVC 6.351978e-7 fail）。
- `logs/src.patch` 与 `git diff 50a0acd1~1..50a0acd1 -- src` **字节一致**（2022 B，仅 2 文件），无隐藏改动。
- 链路要素均在 README 中：GNU `mul_add` 回退非 IEEE 单次舍入 → 参考是 CPU 后端非融合 →
  B 实测不可行（4/5 红，ellipsoid 8.1929e-7 > 4.8e-7）→ A 的理由与 C/D 不放宽；引用证据文件均存在。
- 交付 `receipt.json`：`gpu_windows=0`、`full_g01 not run`；修复证据目录无 GPU window 日志。
- 观察 3：B「4/5 红」数据引自 Arch 的 CPU scratch，证据目录内**无该 scratch 原始日志**；其中
  ellipsoid/mesh 的 frame 量级与我此前独立测得的 CPU-vs-GPU 参考差（8.192876e-7 / 5.364418e-7）一致，
  但「4/5 红 + exit 101」组合无法从仓库已提交产物复算。

## 10. 观察（非阻断，5）

1. **QA acceptance 与 r2 acceptance 的范围表述冲突**：QA acceptance 第 1 条字面列 `render/kernel/megakernel_arrays.rs`
   为「不得改」，但 r2 任务明确允许 megakernel 段复用（A 设计：render 参考是 GPU fast-math fused）。
   提交按 r2 执行；建议修正 QA acceptance 措辞（改为「render 不得改回 `mul_add`」或列为允许项）。
2. **Arch 消息的提交规模与事实不符**：消息称 `50a0acd1` 为「6 文件 +223/−15」，实际 `git show --stat` 为
   **31 文件 +972/−14**（2 src + 22 证据 + 2 NP01 + 5 `.normify`）；不影响内容判断，但汇总数字应以 git 为准。
3. **B 不可行性的 scratch 原始日志未入库**（见 §9）；A 的理由主要依赖可复现的参考侧事实与本次三构建/5 case。
4. 交付 pre 日志为**复用主树缓存**的运行（无 `Compiling` 行），post 才重新编译主树；我的复核用独立 parent
   副本重编 pre（见 §4/§6），不受该差异影响。
5. 我自己的首次矩阵脚本有 PowerShell 参数名 bug（`$args` 与自动变量冲突，cargo 只打印 help，exit 0），
   该轮**全部作废**；改用 Python subprocess 按字节抓取 stdout 重跑全部测试后得到本报告数字（不拼接）。

## 11. 口径与边界

- 真实无 FMA 机器未实机验证（本片未新增/未实机验兜底路径；沿用 `exact-f32-fma-relmat` 的 seam + 默认 gate 覆盖）。
- NaN payload 不作位级判据；边界测试计数以 **16** 为准；gate unit 引 **604**。
- 不宣称精度结论、跨环境全绿、上游等价、跨平台结论、原生路径放行。
- 未跑全量 G01；未开 GPU 窗口（本复核纯 CPU，未建 CUDA 上下文）；未改交付证据/产品树/判据；
  **未 commit、未 done**。

## 12. 复现与产物

```powershell
$py = 'target\mjwarp-parallel\venv\Scripts\python.exe'
& $py docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\qa_scope_static.py
& $py docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\qa_prepare_copies.py
& $py docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\qa_run_tests.py
& $py docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\qa_verify_dump.py
& $py docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\qa_three_build.py
& $py docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\qa_g5_diff.py
& $py docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\qa_default_gate.py
& $py docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\qa_np01.py
& $py docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\qa_evidence_chain.py
# 证据冻结后 gate/check（stdout 写 target/，避免 gate-scope 自失效）
& docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\run_gate_check.ps1
```

证据目录 `docs/development-evidence/g10-p5-faithful-fma-fix-independent-check/`：
`README.md`、`receipt.json`、`artifacts-sha256.json`、`qa-scope-static.json`、`qa-5case-bitwise.json`、
`qa-three-build.json`、`qa-g5-diff.json`、`qa-default-gate.json`、`qa-np01.json`、`qa-evidence-chain.json`、
`qa-copies-info.json`、`logs/`（三构建/三 pre 构建/G5 pre-post/默认门禁摘要/工具链/dump/exits）、
脚本 9 个（`qa_*.py` + `run_gate_check.ps1`）。不入库：`target/qa-a-check/**`（副本、clean target、dump）。
