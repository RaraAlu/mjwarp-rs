# perworld L v2 G-L1 采集窗执行与参考落库（双相机 scene-a-light）

- 任务：`2026-11-09-render-perworld-l-v2-capture-window-g-l1`（owner Backend、p65、P8-渲染与BVH；登记 `6f9201e3`）
- 授权：S-2 双签留证 `2f361d91` + QA 复核 `4a19e003`（PASS，可发 G-L1 token）；Chief **attempt 2 token 2026-11-09T00:02:00+08:00**（单次、失败即停、不重试、不放宽）
- 依据：已签 spec v3 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B（§2.3/§2.4/§6/§11.1）；S-2 材料 `docs/development-evidence/render-bvh/perworld-l-v2-capture-window-s2/`
- 执行者：**Backend（唯一写者）**；整窗 repo-write-lock `owner=Backend`（pid 60976，全程未释放；收口后释放）。窗口内零写仓、零编译、不响应群消息。
- 产物：正式参考根 `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/`（5 文件，**11,078,239 B**）；证据本目录。
- 定位：本参考是 **G-L2 验证窗的唯一输入**；本卡不做产品渲染比较，不做超出采集事实的声明。

## 1. 时间线与角色

| 阶段 | 时间（+08） | 结果 |
| --- | --- | --- |
| attempt 1 token 窗口 | 2026-11-08 22:15 起 | monitor exit 20（外部负载，`longest_clean=24`、`free_min=8,153 < 10,230`）→ **窗口不成立、停手**；记录留存 `target/render-bvh/perworld-l-v2-gl1/attempt1/`（不入库） |
| attempt 2 取锁 + T0 | 2026-11-09 00:00:00 冻结 | HEAD `4f6fe313`、树 clean、0 构建、锁 owner=Backend、参考目录不存在 |
| N1 + step-2 dry-run | 00:00:0x | N1 三态 PASS；dry-run exit 0、cameras 3、`[true,true,false]`、legacy 112、不建 out_dir |
| monitor（常规） | 00:00:13–00:30:11 | exit **20**：354 samples、longest_clean **22**、util 16–43%、free_min 13,024 MiB（min>floor 但 util 口径不达标） |
| monitor（authoring 例外） | 00:30:24–00:35:24 | exit **0**：**60/60 clean**、util 14–40%、free 13,029–13,034 MiB、`compute_filtered_max=0`、`build_max=0` |
| 三点基线 | 00:35:56–00:36:06 | free 13,029/13,029/13,029；util 36/37/21%；`floor_effective=max(10,230, 9,957)=10,230`；报 Chief 获 **GO**（00:38） |
| 单次采集 | 00:37:19 启动，wall 2.02 s | wrapper exit **0**；refit×2/render×2、序列 refit→render→refit→render；两遍逐位一致 |
| readout + 采后自查 | 00:37:2x–00:38 | verdict **pass-margin**；§2.4 对账全 OK；全部自查 PASS |

## 2. T0 冻结（`window/t0-freeze.txt`）

HEAD `4f6fe3134061dbcc39d20e83ddca5904380e09ab`、树 clean、0 构建进程、锁 `owner=Backend`。指纹（均实测吻合）：

| 项 | sha256 | bytes / lines |
| --- | --- | --- |
| `fixtures/render-bvh/scene-a-light/scene.xml` | `4b354054116ddb81c6209ed10c021d133ea30d6fd82c1a6f866bafff9f625716` | 5,134 / 86 |
| `scripts/capture_render_perworld.py` | `f3af09ab992981f2dc5d7f2e3479aaf9eebb97df0d2a774b247c8e0772ad6f07` | 23,637 / 490 |
| `scripts/perworld_capture_meta.py` | `3fdb85953681a1a695a9611fc48ddf0862318acf0e0767e831029a820127b87e` | 19,154 / 379 |
| `scripts/perworld_light_prep.py` | `9fd5f8c25894380992986ffecaa584ee4fdff53a042438a70812f04699113e2c` | 7,206 / 160 |
| `readout_c1_c2.py` | `4a054abafbec8e3a2567f75b7e71f5cec82c3cbd88d1d412e8a44f445370ad8b` | 16,711 / 364 |
| `spec.md` | `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` | 39,379 / 344 |

upstream `71da24d9` clean；venv `3.12.13 / 1.15.0 / 3.15.0 / 3.15.0 / 2.5.4`；正式参考目录 `…/perworld/nw3-light-hinge02040` **不存在（fresh）**；`…/scene-a-light/` 仅 1 个既有已跟踪文件（scene.xml）。S-2 材料 README：签名版 `c8bb71ef…`/18,299 B（`receipt.json.signature` 记录；token 所指），HEAD 为加签名块后的 `2ca9f1dd…`/19,502 B（材料自身写明属预期），见 `window/s2-material-identity.txt`。

## 3. N1 与 step-2 dry-run

- N1（`window/n1-check.txt`）：`MJWARP_CAPTURE_AT` **missing / empty / invalid 三态全部显式失败**，token 值 `2026-11-09T00:02:00+08:00` 通过；逐字写入 `window/token-capture-at.txt`（语义=窗口授权时间，非采集瞬间）。
- dry-run（`window/dryrun.json` / `dryrun-summary.txt`）：exit 0、`model_static.cameras` 3 条（cam-persp/cam-hinge/cam-track）、`cam_active_fixed_only=[true,true,false]`、`legacy_manifest_entries=112`、未创建 out_dir/perworld root。

## 4. monitor（常规不达标 → authoring 例外，floor 未放松）

- 常规 `-MaxUtil 40`：**exit 20**，354 samples、`longest_clean=22 < 60`、util **16–43%**、free 13,024–13,034 MiB、`compute_filtered_max=0`、`build_max=0`（`window/monitor-regular-attempt2.csv`）。
- authoring 例外 `-MaxUtil 60`（用户 2026-11-08 预批、仅本采集窗）：**exit 0**，**60 samples / 60 clean**、`longest_clean=60`、util **14–40%**、free **13,029–13,034 MiB**、`compute_filtered_max=0`、`build_max=0`；`exception_load_bearing=true`（`window/monitor-exception.csv`、`monitor-summary.json`、`monitor-exit-code.txt`）。
- **floor 未放松**：post-hoc 校验**每个 clean sample free ≥ floor_effective 10,230 MiB = true**。

## 5. 三点基线与 floor

`window/baseline-three-points.txt` / `baseline.txt`（monitor PASS 后 5 分钟内；实际 00:35:56–00:36:06）：

| 点 | used | free | util | temp |
| --- | --- | --- | --- | --- |
| 1 | 3,033 | 13,029 | 36% | 28℃ |
| 2 | 3,033 | 13,029 | 37% | 28℃ |
| 3 | 3,033 | 13,029 | 21% | 28℃ |

`baseline_free = min(三点 free) = 13,029 MiB`；`onspot_floor = 13,029 − 3,072 = 9,957 MiB`；**`floor_effective = max(signed 10,230, onspot 9,957) = 10,230 MiB`**（更高更严）；三点 free ≥ floor、util <40；报 Chief 获 GO；采集启动距基线末点 1 分 13 秒（<5 分钟）。

## 6. 单次采集（`window/capture-*`、`refit-counter.json`）

- wrapper：`docs/development-evidence/render-bvh/perworld-l-v2-pilot-g-l0/materials/run_pilot_with_refit_counter.py`（authoring-only，不改行为）；`MJWARP_CAPTURE_AT=2026-11-09T00:02:00+08:00`；out-root `fixtures/render-bvh/scene-a-light/perworld`；`--repeat 2`、tag `nw3-light-hinge02040`、`--worlds 3 --hinge-deg 0,20,40`、`--use-fast-math false --use-shadows true --seed 20261030`。
- **唯一一次采集执行**：exit **0**、wall **2.02 s**；`refit_calls=2`、`render_calls=2`、序列 **refit→render→refit→render**、`pattern_ok=true`、`capture_exit_code=0`。
- `two_captures_bitwise_identical=true`（两遍采集逐位一致）；`legacy_reference_untouched=true`（legacy 112）；**无采集重试**。

## 7. 采后判读与 §2.4 对账

readout（`window/readout.json`，exit 0）：**verdict `pass-margin`**、`all_sets_pass_margin=true`、`any_below_accept=false`、`metadata_selfcheck.failed=[]`。

| 项 | 0→1 | 1→2 |
| --- | --- | --- |
| C1 magnitude（cam-persp） | **38,285** | **40,644** |
| C1 bitwise（cam-persp） | 43,604 | 41,815 |
| C2 depth（cam-hinge） | **35,828** | **33,883** |
| C2 seg（cam-hinge） | **16,041** | **16,754** |
| 诊断 cam-persp depth/seg | 0 / 0 | 0 / 0 |
| 诊断 cam-hinge rgb magnitude / bitwise | 50,363 / 54,380 | 52,329 / 54,250 |

§2.4 计数对账（`window/counts-crosscheck.txt`）：capture.json 的五个 §2.4 字段 vs readout.json，**两对 world 各 8 项逐项一致，全部 OK**（`SUMMARY PASS`）。

## 8. 参考文件（正式参考根，5 文件）

| 文件 | bytes | sha256 |
| --- | --- | --- |
| rgb.npz | 5,531,026 | `602574c213d848ddc674244b11ac518fe42e54cf373f43784e143292e960493f` |
| depth.npz | 1,844,650 | `82eec6dab866f5ba54e3df0f3592d7198efb6ab5fc832627d98f4d56f079e92d` |
| seg.npz | 3,687,826 | `c3f09b16b6c02f972d5f7b535a39a94a6850bc1362e8399543cdb384781142f0` |
| state.npz | 2,486 | `a1c5a32154370551923a8a4ef394c62d5f9e0d9dbb468e707f77175f8c0c6aca` |
| capture.json | 12,251 | `8b53e1221faf9c524b9f42640764a850d7e723de61ebc84342ee6b63137720a9` |
| **合计** | **11,078,239** | — |

- spec §2.3 的 11,076,054 B 是**估算**；冻结以实测 **11,078,239 B** 为准（差值来自 npz/capture.json 实际体积）。
- 四个 npz 的 sha256 与 G-L0 pilot 同名文件**逐位相同**，属上游确定性预期（pilot 已记录；本次 provenance 以本窗为准）。
- `capture.json.files_sha256` 与上表实测一致；`capture.json.capture_json_sha256`（包装器输出）为 `8b53e122…`。

## 9. 采后自查（`window/`）

metadata-selfcheck（command 非空 / `captured_at=2026-11-09T00:02:00+08:00` / tag / per_world_scope light+geom / cameras 2×10 字段 / cam-hinge sensorsize `[0,0]` fovy 路径 / two-run / legacy / files 集合）、`seg-geom-sets.json`（每 world×相机可见 `(id,type)`：cam-persp 三 world 均 `{(-1,-1),(0,5)}`；cam-hinge world0 六对、world1/world2 七对（多 `(6,5)`））、`legacy-selfcheck.txt`、`environment-provenance.txt`（python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4 / upstream `71da24d9` / RTX 4070 Ti SUPER·driver 596.36）、`zero-compile-fingerprint.txt`（0 构建）、`git-before/after.txt`、`post-gpu.txt`、`files-sha256.txt`。

## 10. incidents（`window/window-incidents.txt`）

1. **attempt 1**（2026-11-08 22:15 token）：monitor exit 20（外部负载；`longest_clean=24`、`free_min=8,153 < 10,230`）→ 窗口不成立、停手未采集；记录只留 `target/render-bvh/perworld-l-v2-gl1/attempt1/`（非入库）。
2. **attempt 2 常规 monitor 不达标**：util 峰值 43% ≥40 → 按用户预批启用 authoring 例外（`-MaxUtil 60`，60/60 clean、`exception_load_bearing=true`、floor 未放松）；**不是**放宽判据，也不是早解锁。
3. **采集前一次 shell 引号错误**：首次调用因 cmd 相对路径/引号错误在**执行前**即失败（exit 1、0.01 s、无 GPU 工作、无文件写入）；改用 Start-Process 后**唯一一次**采集执行成功（无任何采集重试）。
4. S-2 材料 README sha 差异（签名版 `c8bb71ef…`/18,299 B vs HEAD 加签名块 `2ca9f1dd…`/19,502 B）属材料自身写明的预期演进，非指纹违规。

## 11. 边界与非声明

- `unchanged src/`、`unchanged tests/`、`unchanged scripts/`；spec 目录、S-2 材料目录与 pilot 材料目录只读；fixtures **仅新增**本参考 5 文件（未改既有 fixtures）。
- 四门禁 fmt/clippy/clippy-cuda/unit 全过；证据 manifest sha256+bytes 自核；UTF-8/LF/无 BOM。
- **非声明**：本卡只落 G-L1 参考与采集事实；不宣称产品渲染通过、不宣称与上游实现等价、不宣称判别力恢复。产品侧实体比对属 G-L2 验证窗。

## 12. 证据清单

`README.md` / `receipt.json` / `manifest.txt`；`window/` 含 t0-freeze、token-capture-at、n1-check、dryrun.json(+summary/exit/stderr)、monitor-regular-attempt2.csv、monitor-exception.csv、monitor-summary.json、monitor-exit-code.txt、monitor/exc stdout·stderr、baseline-three-points、baseline、capture-command/stdout/stderr/exit、capture.json、refit-counter.json、readout.json(+stdout/exit/stderr)、counts-crosscheck.txt、files-sha256.txt、metadata-selfcheck.json、seg-geom-sets.json、legacy-selfcheck.txt、environment-provenance.txt、zero-compile-fingerprint.txt、git-before/after.txt、post-gpu.txt、window-incidents.txt、s2-material-identity.txt、preconditions.txt。
