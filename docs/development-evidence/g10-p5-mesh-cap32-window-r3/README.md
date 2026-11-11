# W2/U7U8：U7/U8 pass_rule 全条件断言窗（attempt 2：r3 冻结；最小窗 1 实体）

- 任务：`2026-11-12-w2-u7u8-full-condition-assert`（卡保持 in_progress；本目录为 r3 窗后证据）。
- 窗口：**attempt 2**（r3 冻结轮；单实体 `g10_p5_branch_closure_cuda`；由 Chief 按四前提开窗并给 GO）。
- 窗口前基线 HEAD：`2ccc182b`；窗内无仓写、无群消息、零 Rust 编译。
- **attempt 1（r2 冻结轮）FAILED**，append-only 记录在
  `docs/development-evidence/g10-p5-mesh-cap32-window-r2-fail/`（verdict=FAILED、21 条不一致、exit_code_capture_bug）。

## 1. 一句话结论

**窗口有效执行且 PASS**：1/1 实体 exit **0**（41.25 s）；sha 守卫 **18/18**；
floor pre 13440 / post 13440 MiB ≥ 10,240；monitor 60/60（Chief）；零 Rust 编译 PASS；无残留 compute。
**U7/U8 pass_rule 全条件（comparator）**：U7 `failures=0`、
U8 `failures=0`；两 case `contact.geom=[1,0]`；
**判定：PASS**。测试结果：`test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 40.99s`。

## 2. 根因闭合依据（r2 FAIL → r3 PASS）

1. **输入归一化（本窗实配）**：`pair_meta()` 按上游 `_add_geom_pair`（`collision_driver.py:351-380`）归一化；
   U7/U8 实传 **`g1=1(box), g2=0(mesh), type1=6, type2=7`**（与上游捕获的 pair 序一致），期望/实际
   `contact.geom=[1,0]`。CPU 断言 `pair_normalization_matches_upstream_reference` 已对 11 case 机证。
2. **同键行序无歧义**：上游多接触 `geomcollisionid = 0..n-1` 每行唯一（`collision_convex.py:982-988`、
   `collision_core.py:287`），行序由 `multicontact()` witness 顺序决定；本窗 U8 设备 `geomcollisionid`
   与参考按 `(worldid, geomcollisionid)` 配对后 **pos 逐行相同、行序不再轮换**（`pos_order_equal=true`）。
3. **M2 对照**：M2 `mesh-box` 参考 count=4、`geom=[[1,0]]×4`、`geomcollisionid=0..3`（与 U8 同结构），
   A2-3 窗 M2 exe 全条件 comparator PASS → 设备 multicontact 路径在归一化 pair 下与上游一致；
   本窗 U8 修复后同样全条件 PASS，r2 FAIL 的“对序/行序”差异确认为**用例输入序问题**而非产品内核偏离。
4. r2 FAIL 断言（`geom=[0,1]`、frame 符号翻转、pos 行序轮换）在本窗全部消失：见 §4 明细。

## 3. 判定条件（机检汇总）

| 条件 | 结果 |
| --- | --- |
| 实体 exit code / 耗时 | 0 / 41.25 s（≤300 s；总 ≤900 s） |
| sha 守卫 | 18/18 OK，FAIL 0 |
| `SUMMARY entities=1 failed=0 exit_codes=0` | 见 facts（runner exit 捕获已修复，非空） |
| floor（pre/post） | 13440 / 13440 MiB ≥ 10240 → OK |
| monitor 机检（Chief CSV） | {"ok": true, "rows": 60, "util_max": 4, "free_min": 13440, "compute_filtered_max": 0, "build_max": 0, "clean_all": true, "streak_first": 1, "streak_last": 60} |
| 零 Rust 编译（窗后复核） | PASS |
| fail-stop 命中 | 0（无） |
| U7 comparator failures | 0 |
| U8 comparator failures | 0 |
| 两 case 均观测（RECORD-ONLY + comparator 行） | True |
| 窗内 0.25 s 采样 | WINDOW-SAMPLING target=g10_p5_branch_closure_cuda samples=138 interval_target_ms=250 interval_min_ms=290.6 interval_max_ms=361.3 util_max=4 free_min=11793 csv=window-samples.csv |
| 汇总 verdict | **PASS** |

## 4. U7/U8 全条件明细（comparator 输出）

### U7 `mesh-limits-cone-box`（fan-cone-24；degree 24 ≤ 32）

| 项 | 值 | 判定 |
| --- | --- | --- |
| nacon / ref_count | 1 / 1 | 零容差 OK |
| bit_equal 行 | 1/1 | **全位级** |
| max_dist_ulp / max_pos_ulp / max_frame_ulp | 0 / 0 / 0 | 0 ULP（≤4 判据内） |
| contact.geom | [1,0] | pinned 类型序 `[1,0]` |
| comparator failures | 0 | 全条件成立 |

### U8 `mesh-limits-prism-box`（prism-20；polygon 20 ≤ 32）

| 项 | 值 | 判定 |
| --- | --- | --- |
| nacon / ref_count | 4 / 4 | 零容差 OK |
| bit_equal 行 | 4/4 | **全位级** |
| max_dist_ulp / max_pos_ulp / max_frame_ulp | 0 / 0 / 0 | 0 ULP |
| contact.geom | [1,0] | pinned 类型序 `[1,0]` |
| pos 行序 / geomcollisionid | 逐行相同（`pos_order_equal=True`） | 0..3 唯一配对 |
| comparator failures | 0 | 全条件成立 |

comparator 内部逐字段断言：nacon 零容差、结构键 `(worldid, geomcollisionid)` 排序配对、11 个零容差字段
（friction/solref/solreffriction/solimp/includemargin/adhesion/worldid/type/dim/geom/geomcollisionid）、
overflow 全 0、efc_address 全 -1、geom 序 `[1,0]`；失败会打印 FAIL 行并使测试失败。

## 5. 执行事实与 runner 修复

- 实体：`target/a2-freeze-r3/g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe` / **41782272 B** / `ac823b58b90ddebaf86ceea10a8d82bb299dd326a9f7f98e9d0d8b60cb7508ec`；
  参数 `--ignored --nocapture --test-threads=1`。
- runner 修复生效（r2 attempt 1 缺陷）：`.NET ProcessStartInfo` 捕获 exit code → `results.exit_code=0`；
  `SUMMARY` 用 `@(...)` 计数 → `failed=0`；自测日志 `logs/runner-selftest.log`（exit0→0、exit3→3、failed_count=1、PASS）。
- 窗内 0.25 s 采样：WINDOW-SAMPLING target=g10_p5_branch_closure_cuda samples=138 interval_target_ms=250 interval_min_ms=290.6 interval_max_ms=361.3 util_max=4 free_min=11793 csv=window-samples.csv；CSV `logs/window-samples.csv`。
- 环境：GPU pre :: 2 %, 2622 MiB, 13440 MiB, 25, 596.36；GPU post :: 2 %, 2622 MiB, 13440 MiB, 25, 596.36；FLOOR gate: free=13440 MiB >= 10240 MiB (floor_effective=10240)；残留：POST compute-apps filtered residual: 0。
- monitor：CSV `logs/monitor-pre-window-r3.csv`（60/60 clean、util_max=4、free_min=13440）；
- 三点基线 raw：`logs/three-point-baseline-r3.txt`（Chief 提供；util 2/4/2 %、used 2,614 MiB、free 13,448 MiB、25 °C、596.36）。

## 6. 边界、限制与非声明

- 本窗只证明 U7/U8 pass_rule 全条件（comparator 口径与冻结参考下）：nacon、结构键配对、dist/pos/frame 4 ULP、
  11 零容差字段、overflow==0、efc_address==-1、geom 序。`efc_address`/`overflow` 为设备侧不变量（参考未捕获）；
  参考 `flex/elem/vert` 不在 pass_rule 字段清单、不比对（与 M2/M3 同口径）。
- 不宣称 mesh 容量缺口已清、与上游等价（over-32）、G10@P5 整族通过、NP01/NP02 门禁通过、性能结论；
  **不作位级等价声明、不提供位级等价保证**（本表 `bit_equal` 仅记录 comparator 统计口径）。
- 无失败轮（attempt 2）、无重试、无放宽；attempt 1 FAILED 记录保留在 `...-r2-fail/`，不改历史字节。
- 未改判据/容差/参考/comparator；候选/参考/fixture 守卫与 A2-3 一致。

## 7. 证据文件

`README.md`、`receipt.json`、`window-readiness.md`、`refreeze-10-exe-r3.json`/`.tsv`、`guards-r3.tsv`、
`results.json`、`facts.txt`、`sha-guards.txt`、`u7-u8-full.json`、`logs/`（实体日志 + `.err` + runner console +
monitor r3 CSV + baseline raw + window-samples + runner selftest）、`manifest.txt`。
