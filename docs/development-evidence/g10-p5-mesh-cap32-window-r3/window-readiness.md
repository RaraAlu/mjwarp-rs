# W2/U7U8 r3 最小窗就绪材料（窗口前；不构成通过声明）

- 任务：`2026-11-12-w2-u7u8-full-condition-assert`（卡保持 in_progress；0 窗修正片）。
- 本片目的：修正 r2 最小窗 FAIL 的**用例输入序**并重建 branch exe（r3），为全新最小窗做准备。
- **窗口前 U7/U8 全条件结果未知，不宣称通过**；新窗由 Chief 按四前提开（fresh monitor、三点基线、floor、群留痕）。

## 1. r2 FAIL → r3 修正（事实链）

- r2 最小窗（attempt 1）**FAILED**：`test result: FAILED. 1 passed; 1 failed`，comparator 21 条不一致
  （U7 4 + U8 17）。失败窗证据（append-only）：`docs/development-evidence/g10-p5-mesh-cap32-window-r2-fail/`
  （verdict=FAILED、attempt=1、u7-u8-failures.json、exit_code_capture_bug、manifest）。
- 根因（Chief 已核源码批准）：S4 `LIMIT_CASES` 的 `geom_type=[7,6]` + `build_meta` 硬编码 `g1=0,g2=1`
  → 传给 ContactMulti 的 pair **未按上游 `_add_geom_pair` 归一化**（上游 narrowphase 输入恒为类型序）。
- 修正：`tests/support/branch_closure_case.rs` 新增 `pair_meta()`（上游 `_add_geom_pair` 同构：
  `type1 > type2` 时交换 pair，相等保持调用顺序），`build_meta` 使用归一化 pair。
  - U7/U8 归一化明细：**`g1=1(box), g2=0(mesh), type1=6, type2=7`** → 期望 `contact.geom=[1,0]`（与参考一致）。
  - 其余 branch case 类型升序/相等 → pair 不变（no-op）。
  - 上游多接触写入门：`geomcollisionid = 0..n-1` 每行唯一（`collision_convex.py:982-988` + `collision_core.py:287`），
    行序由 `multicontact()` witness 顺序决定；comparator 按 `(worldid, geomcollisionid)` 配对单值确定。
- 新增 CPU 断言（零 GPU）：`pair_normalization_matches_upstream_reference`——对全部 11 case 断言
  `pair_meta()` 与 S3 参考 `contact.geom` 逐行一致，并断言 U7/U8 明细 = `(1, 0, 6, 7)`。
- 产品 `src/` **零改动**；内核身份不变（source `fb03c5b1…`/254,434、PTX `78da9dea…`/3,883,643/6、
  29 `.cu` digest `266bbf94fd92cce3202159df196d84824dacbaa6d22f4b848eddd83ac57fcb44`）；不改判据/容差/参考/comparator。
- **PASS 口径**：窗内 comparator `failures=0`（U7/U8 各）且 exe exit 0 才算 PASS；任一 fail-stop 命中即停、不重试、不放宽。

## 2. r3 冻结轮（只重建 branch exe）

- 冻结目录 `target/a2-freeze-r3/`；来源 `target/a2-freeze-r2/`（9 个未重建，sha/bytes 与 r2 清单 **9/9 全等**）；
  旧 `target/a2-freeze-r2/`、`target/a2-freeze/` 与 A2-3/r2 fail 证据**保留不动**。
- 新 branch exe：`g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe` / **41782272 B** /
  `ac823b58b90ddebaf86ceea10a8d82bb299dd326a9f7f98e9d0d8b60cb7508ec`（supersedes r2 branch `6d2b39fcad182e43c85a9faf64ab48a1369614889a60d00befc723efe0373677`）。
- 逐项表：`refreeze-10-exe-r3.json` / `.tsv`；计数：{"rows": 10, "rebuilt": 1, "copied_unchanged": 9, "old_freeze_match": 9, "copy_match_total": 10}。

| target | file | bytes | sha256 | source |
| --- | --- | --- | --- | --- |
| g10_p5_branch_closure_cuda | `g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe` | 41782272 | `ac823b58b90ddebaf86ceea10a8d82bb299dd326a9f7f98e9d0d8b60cb7508ec` | rebuilt / old_match=False |
| g10_p5_nvrtc_launcher | `g10_p5_nvrtc_launcher-719917fb0657e1fe.exe` | 41714176 | `6c306f58ef1eaf4a08659b4350ce5b2dc7d36e814c8358cf9b33a89a27310a99` | copied_unchanged / old_match=True |
| g10_p5_multicontact_cuda | `g10_p5_multicontact_cuda-3c769ef530070ffe.exe` | 41698816 | `4c035bfac51d0f269b9b684a74ae05060bba953156d03a1313b2da37cddc9ec3` | copied_unchanged / old_match=True |
| g10_p5_primitive_cuda | `g10_p5_primitive_cuda-0908b0f61ffbc599.exe` | 41701376 | `6f949c44ddb3991256f164889af95ad1f1a2eb5f5a377b798b06324613ae1944` | copied_unchanged / old_match=True |
| g10_p5_primitive_plane_cuda | `g10_p5_primitive_plane_cuda-b1e8897a38248f9f.exe` | 41573376 | `83a6864fa9206a0cb27224bdbc237abf342833083502b08b575ae31fb270b0d4` | copied_unchanged / old_match=True |
| g10_p5_primitive_sphere_capsule_cuda | `g10_p5_primitive_sphere_capsule_cuda-effe95bf0fed2063.exe` | 41562112 | `b4eaeaa4922c368d7c653e1e40ddc1ab075de70f7f16f5accdd3c025b02a8f15` | copied_unchanged / old_match=True |
| g10_p5_primitive_branches_cuda | `g10_p5_primitive_branches_cuda-555175a2d3995ff7.exe` | 41638400 | `3d2fb1e36105c80fc58a81efe2a81f41f2751c027a8c8c4ea58593434c061e74` | copied_unchanged / old_match=True |
| g10_p5_mesh_multicontact_m2_cuda | `g10_p5_mesh_multicontact_m2_cuda-e11ae6eb82e10bd5.exe` | 41753088 | `2e2afa00b78b47c7b03ac8a0176fd2662b4a3c2f03646eaf53e826d84efd47a8` | copied_unchanged / old_match=True |
| g10_p5_mesh_multicontact_m3_cuda | `g10_p5_mesh_multicontact_m3_cuda-31f06b6f3d1d660f.exe` | 41756160 | `9efcfe18c75e783712b0ce7ef70411bc06782f3a633c7c23a4559e63ccf97fca` | copied_unchanged / old_match=True |
| stage2_equivalence_window | `stage2_equivalence_window-b7e65259db9435d8.exe` | 42277888 | `013617a81ed55243b45cd5eccd9a71443aac625d8f19e8e559916400c2364c75` | copied_unchanged / old_match=True |

## 3. 最小窗守卫表（18 = 1 exe + 15 fixture + 2 参考）

逐项 sha 见 `guards-r3.tsv`（runner 同源生成）；fixture/参考 sha 与 A2-3 窗口一致。

## 4. runner 修复 + 自测（本片实跑）

- r2 runner 缺陷：`Start-Process -PassThru`（无 `-Wait`）在本机 PowerShell 读不到 `ExitCode` → `results.exit_code=null`；
  `SUMMARY failed` 用 `Where-Object | .Count` 对 OrderedDictionary 取到键数（4）。**本窗 runner 已修复**：
  - 实体执行改 `.NET ProcessStartInfo`（异步读 stdout/stderr + `WaitForExit` + `ExitCode`）；
  - 失败计数用 `@(...)` 强转数组；SUMMARY 打印 `exit_codes`；
  - `-SelfTest` 用同一执行路径跑 dummy `cmd /c exit 0` / `cmd /c exit 3`。
- 自测日志：`logs/runner-selftest.log`（实跑）：exit0 捕获 0、exit3 捕获 3、failed_count=1、RESULT=PASS。
- 本文件与 runner 一同提交；窗内 runner 位于 `target/a2-window-r3/run-a2-window-r3.ps1`（target 工作材料，不入库）。

## 5. 下一窗（Chief 四前提 + GO 后执行）

- 实体：`target/a2-freeze-r3/g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe --ignored --nocapture --test-threads=1`（只此 1 个；含 S4 branch 9/9 回归 + U7/U8 全条件）。
- 预计：窗内 **40–55 s**（r2 实测 40.05 s；r3 仅用例输入序变化，comparator 断言耗时≈0）；墙钟 ~70–90 s；上限 300 s/实体、900 s 总。
- 窗内 0.25 s 采样（r2 L2 收口持续）：CSV `window-samples.csv` 入窗证据。
- fail-stop 8 条沿用（A2-3 就绪包 §5）；第 6 条具体化：**U7/U8 comparator `failures≠0`（nacon/结构键配对/dist/pos/frame/
  11 零容差/overflow/efc/geom 序任一）即停**，不重试不放宽。
- monitor CSV 建议落 `target/render-bvh/gl2-ptx-diff/w2-a2-precheck/monitor-pre-window-r3.csv`；
  三点基线 raw 建议落 `three-point-baseline-r3.txt`（无则按群留痕如实记录，不虚构）。

## 6. 四前提勾表（Chief 填）

| # | 前提 | 判据 | 状态 |
| --- | --- | --- | --- |
| 1 | fresh freeze | `src/` 自冻结零 diff（`git diff 2ee0540d..HEAD -- src/` 与 `git diff HEAD -- src/` 均 0）；29 `.cu` digest `266bbf94fd92cce3202159df196d84824dacbaa6d22f4b848eddd83ac57fcb44`；r3 10 exe 与 `refreeze-10-exe-r3.json` 全等；9 旧 exe sha 与 r2 清单全等；旧 r2 branch `6d2b39fcad182e43…` 未重建 | ☐ |
| 2 | monitor 60/60 | 官方脚本连续 60 干净样本、过滤后 compute 0、构建进程 0；CSV 落 `monitor-pre-window-r3.csv` | ☐ |
| 3 | 三点基线 | pre ×3 util/used/free/temp/driver 记录并贴群 | ☐ |
| 4 | floor 取严 | free ≥ 10,240 MiB；`floor=min(10240, baseline_free−3072)` 取更严者；pre/post 复核 | ☐ |
| 附 | 零 Rust 编译 | 起跑前无 cargo/rustc/cl/nvcc；窗内不构建；窗后复核 PASS | ☐ |
| 附 | 单窗互斥 | 全队同一时间仅本窗；不写仓、不发群消息 | ☐ |

## 7. 非声明

- 窗口前 U7/U8 全条件结果未知，不宣称通过；不宣称 mesh 容量缺口已清、与上游等价（over-32）、G10@P5 整族通过、
  NP01/NP02 门禁通过、性能结论；不作位级等价声明、不提供位级等价保证。
- r2 FAIL 记录保留（`g10-p5-mesh-cap32-window-r2-fail/`）；本 r3 材料不覆盖、不修改任何历史证据字节。
