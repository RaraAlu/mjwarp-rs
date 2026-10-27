# S4 · UNC-3 分支收口：设备对照 target + launcher 扩 hfield-capsule（开窗前证据）

- 任务：`2026-10-27-unc3-branch-closure-s4-validation`（owner Backend，HEAD `e2197106`，工作区干净起步）
- 上游 pinned revision：`71da24d956378a87a703b6e1442b13aec0c4ac29`
- 本片分两段：**开窗前**（实现 + 定向测试 + 门禁 + 10 实体冻结 + 窗口计划）与**验证窗后**（窗口结果回填）。
  本文件当前只记录**开窗前**已落盘的事实；窗口结果段在窗口 run 后追加，**不写「窗口通过」**、不写任何
  分支「已覆盖」。
- 判据/容差/参考/期望值：**零改动**；产品代码与设备源：**零改动**（`git status src/` 为空）。

## 1. 交付 A：新 target `tests/g10_p5_branch_closure_cuda.rs`

- 结构（≤500 行限制内）：`tests/g10_p5_branch_closure_cuda.rs` **341 行**；
  输入装配/常量拆到 `tests/support/branch_closure_case.rs`（354 行）与
  `tests/support/branch_closure_mesh_data.rs`（418 行）；判据复用既有
  `tests/support/primitive_compare.rs`（B1/B2/B3 同口径，**零放宽**）。
- 参考：S3 捕获窗两份新参考（只读、先锁 sha 再解析）：
  - `docs/development-evidence/g10-p5-branch-closure-capture/upstream-branch-closure-gpu.json`
    sha `24d89e9248cccece96baff2089954e438ca7956fe8a71ccc61df318208709db1`（16932 B，9 case）
  - `…/upstream-mesh-limits-gpu.json`
    sha `b6abb8fa5820c103f6d880936d6ae95fa60035e5b9ce49a0364b15de1def19ad`（4581 B，2 case）
- **branch 验收组（进 pass/fail 分母）9 case = U1/U3×5/U4/U5/U6**：
  `plane-cylinder-degenerate`、`plane-box-corner-face-down`、`plane-box-corner-face-up`、
  `plane-box-corner-tilt-x`、`plane-box-corner-tilt-y`、`plane-box-corner-single`、
  `plane-capsule-lying`、`plane-cylinder-deep-tilt`、`mesh-mesh-two-assets`。
  逐 case 对照 **count / geom / dist / pos / frame + 结构字段**（`worldid`/`type`/`dim`/
  `geomcollisionid`/`includemargin`/`friction`/`solref`/`solreffriction`/`solimp`/`adhesion`），
  判据 = **bit_equal 优先 + 4 ULP@scale 兜底**（`dist` 非位级 ≤4 ULP@自身量级、ref=0 用 4 ULP@1.0；
  `pos` 4·ULP(参考三轴最大绝对值)；`frame` 4 ULP@1.0；结构/参数零容差；`efc_address` 全 -1、
  `overflow` 全 0 设备侧不变量）。
- 输入来源（只读）：`geom_xpos`/`geom_xmat` 逐位取 S3 参考 JSON 的同一帧；`geom_size`/`geom_type`/
  `geom_dataid` 来自 S3 fixture 的 `mjModel`（case 表逐值写死）；mesh 拓扑（`mesh_vert`/`mesh_graph` +
  9 个 polygon 数组）来自 `branch_closure_mesh_data.rs`（pinned mujoco 3.15 对同一批 S3 fixture 的解析；
  authoring-only 生成，生成脚本已入库 `scripts/authoring/g10-p5/author_branch_closure_mesh_data.py`，
  生成命令 / 输入 fixture sha / 42 组数组 sha 见 [`mesh-data-provenance.md`](mesh-data-provenance.md)；
  S4 CPU 测试逐项复核拓扑统计与值域）。S4 target 不运行
  Python/muJoCo。
- **limits 组（U7/U8）**：同一 target 内对 `upstream-mesh-limits-gpu.json` 只做**描述性记录**：
  每 case 打印 `RECORD-ONLY limits case=… device_count=… ref_count=… device_dist=[…] ref_dist=[…]
  device_pos=[…] ref_pos=[…]`（**count/dist/pos 同时记**，不只比 count）；不参与 pass/fail 断言。

### 1.1 U7/U8 口径（Chief 硬口径，逐条照办）

1. **U8 措辞（定死）**：上游支持 20 顶点 polygon（动态容量），设备 `FACE_MAX=16` 逐 polygon 回退
   → **行为差异 / 限制存在**。若设备实测 count ≠ 4（如 1），如实记差距，不动 fixture、不改判据、
   不挪代码凑绿。**不使用**「上游也限制」「与上游一致」「U8 已覆盖」这类表述。
2. **U7 措辞**：`count=1` **与 degree>10 限制语义一致**；不写「证明限制触发」。
3. **record-only**：U7/U8 不进 pass/fail 分母；U7 上游 count=1 与设备单接触回退 count=1 可能**偶合**，
   因此必须同时记录 dist/pos（见 QA O9）。
4. 静态映射依据（S2 `docs/development-evidence/unc3-branch-layer-static-map/branch-map.json`）：
   - **M-05**（degree>10 guard）：设备 `src/physics/collision/cuda/multicontact_mesh.cu:120-122,148-150`
     固定 `G5_MC_MESH_DEG_MAX(10)`；pinned 动态取 `nmeshdegmax`（`collision_convex.py:1262-1289`）。
   - **M-09**（polygon 顶点>16 guard）：设备 `multicontact_mesh.cu:180-182` 逐 polygon `nvert > FACE_MAX`
     返回 0；pinned 动态取 `npolygonmax`。二者**语义不同**，本片如实按 S4 设备实测记录。

### 1.2 O1 数字统一（S3 已更正口径，本片不引入 5/6）

- CPU precheck vs GPU 参考：count 与 `qpos_bits` **11/11 一致**；`dist`+`pos` **同时位级一致的只有 4 个 case**
  （`plane-cylinder-degenerate` / `plane-box-corner-face-down` / `plane-box-corner-face-up` /
  `plane-capsule-lying`）；`plane-box-corner-single` **仅 dist 位级**（pos x GPU `1.1678e-8` vs CPU
  `1.4901e-8`）；3 个 tilt pose 差 **dist 8–16 ULP / pos 8–48 ULP**；U6/U7/U8 的 CPU precheck
  **无 dist/pos**（只对账 count / `qpos_bits` / 拓扑）。**S4 证据不再出现 5 或 6** 作为位级 case 数。
- S3 U3 角点并集 8/8（face-down `[0,1,2,3]` + face-up `[4,5,6,7]` + tilt-x `[0,1]` + tilt-y `[1,3]` +
  single `[7]`；count 4/4/2/2/1）来自 S3 CPU precheck，S4 仅引用。

## 2. 交付 B：`tests/g10_p5_nvrtc_launcher.rs` 扩 hfield-capsule

- 只加 **hfield-capsule 一个 case**（`CASES` 由 3 → 4）；对比对象 = 冻结探针
  `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/hfield-capsule/out1/`；
  判据 = **位级优先 + 4ULP@scale 兜底**（与 hfield-sphere 先例一致，复用 `launcher_compare_support.rs`）。
- 新增**非 ignored CPU 前检** `hfield_capsule_reference_matches_frozen_probe`：核对上游参考
  `fixtures/g10-p5/references/hfield-capsule.json`（9712 B，sha
  `feb8c65c4066a36b04862177812826926d054063c787d5d556ba3b218e15c9b0`）与 `out1/` 的
  `count/geom/dist/worldid` 逐位一致（`count=7`），不建 CUDA context。
- **既有 3 个 case（hfield-sphere / sdf-box / ellipsoid-box）判据与期望值一个字未改**。
- 文件 499 行（≤500）；改动仅新增 case、doc 与上述 CPU 测试。

## 3. 定向测试（开窗前，CPU-only）

| 目标 | 命令 | 结果 |
| --- | --- | --- |
| branch_closure | `cargo test --locked --features cuda-probe --test g10_p5_branch_closure_cuda` | **3 passed / 0 failed / 2 ignored**（`fixtures_match_s3_signature`、`reference_support_parses_without_gpu`、`mesh_topology_matches_s3_precheck_and_validates`） |
| launcher | `cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher` | **2 passed / 0 failed / 1 ignored**（`hfield_capsule_reference_matches_frozen_probe`、既有 `world_order_…`） |

CPU 测试覆盖：13 个 S3 fixture 的 sha256 签名复核（逐文件，与 S3 §2 清单一致）、S3 两份参考的
sha 锁 + case 集/count/`two_captures` 校验、U6/U7/U8 拓扑与 S3 precheck 统计（nmesh/nmeshvert/
nmeshpoly、U7 polynum=25、U8 polynum=22）与 polyvert/polymap 值域，以及真实
`validate_collision_inputs`（O7 值域校验）对三组 mesh 输入 `Ok`。

## 4. 门禁（同一棵树；收口时 `normify gate` 绑最终源重跑）

| 项 | 命令 | 结果 |
| --- | --- | --- |
| fmt | `cargo fmt --all -- --check` | exit 0 |
| clippy | `cargo clippy --locked --all-targets -- -D warnings` | exit 0 |
| clippy-cuda | `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings` | exit 0 |
| unit | `cargo test --locked --no-fail-fast` | **994 passed / 0 failed / 376 ignored**（120 binaries；M3 基线 990/0/374 → +4 passed +2 ignored：3 条 branch CPU + 1 条 launcher CPU，2 条 branch GPU） |

收口门禁由 `normify gate 2026-10-27-unc3-branch-closure-s4-validation` 绑最终源写入
`.normify/runs/`；本段为同树手动结果，两者应一致。

## 5. 验证窗 10 实体（冻结，sha 签名集合）

冻结方式：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-run --test <name>`
（`MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package`）；复制到 `target/s4-freeze/` 后
**重哈希**（copy_match=true），源码 mtime（10:16:51–10:17:07）早于构建时间（10:17:56）。

| target | 文件 | 字节 | sha256 |
| --- | --- | --- | --- |
| branch_closure | `g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe` | 41617920 | `e46b0c3262e16e6b847769be8e26b6ae3c1f282bb6b0858be30cd63478cf1a2a` |
| launcher | `g10_p5_nvrtc_launcher-719917fb0657e1fe.exe` | 41701376 | `a7c1d0331ea1ea9274469850aed27613b6b62c1a7786c9751207ba4ee57c4236` |
| multicontact | `g10_p5_multicontact_cuda-3c769ef530070ffe.exe` | 41686016 | `0901879b47bea6d7d7ff60b5248fb8d270ee9ddf15dd5d4b43668191beb7c8d1` |
| primitive | `g10_p5_primitive_cuda-0908b0f61ffbc599.exe` | 41689600 | `5511da57e8c8ebd09ef8472f6d2cdf05f52abf463158fd2d388196bcbf0e73da` |
| plane (B1) | `g10_p5_primitive_plane_cuda-b1e8897a38248f9f.exe` | 41561088 | `e7dccc078092357cf905bb47f6b947dc4c093ac8444ff149b5401176b62ef016` |
| sphere_capsule (B2) | `g10_p5_primitive_sphere_capsule_cuda-effe95bf0fed2063.exe` | 41549312 | `6cd4f7a6755ba5af0dcb384ea734a7f1c7428b5124b0f5273d0c6cd3627373fd` |
| branches (B3) | `g10_p5_primitive_branches_cuda-555175a2d3995ff7.exe` | 41625600 | `ff61a349d87ddd0cee8643271ec533ed2080be336860152b30dab47e54ba92ef` |
| mesh_m2 | `g10_p5_mesh_multicontact_m2_cuda-e11ae6eb82e10bd5.exe` | 41740800 | `277b043594b4c1ab4167c66258405146b5eedf5bf784fd1ac4d15dc7ebf3e3de` |
| mesh_m3 | `g10_p5_mesh_multicontact_m3_cuda-31f06b6f3d1d660f.exe` | 41743360 | `468c1a967318ae309ea6757fb495f014f417c3e7827219f2b499a9cb790991c3` |
| stage2 (record-C) | `stage2_equivalence_window-b7e65259db9435d8.exe` | 42265600 | `9e5f0b0968d7cca4a9e71093a466f5d39eb0b45e996dc49fe0e994dfb9009b70` |

runner（不入库）：`target/s4-branch-closure/window/run-validation-attempt1.ps1`，11735 B，sha256
`1731f56c2bc5757c2f3957a12a520aaf2a516c2e6c605b68e13f1aaedd31dcd6`。

## 6. 窗口计划（待签后开窗；attempt 计数=0）

- **内容**：10 实体顺序执行。stage2 用 `window_product_equivalence_dump …`（record-C 只记录），
  其余 `--ignored --nocapture --test-threads=1`。
- **回归口径**：branch 验收组 9 case（含 U6）+ limits 2 case record-only；
  launcher 4 case（hfield-capsule 新）；G5 9 设备 case（multicontact 3 + launcher 3 + primitive 3）；
  B1 6 + B2 4 + B3 6 分支 + 4 swap；M2 2 case；M3 1 case；stage2 9 synth（record-C）。
- **runner 纪律**：
  - 逐实体 sha 守卫（10 exe + 13 fixture + 2 参考），`SHA_OK …` 行写入 `sha-guards.txt` **且**进
    `facts.txt`（不只 stdout）；
  - 负载 precheck：`cargo/rustc/cl/nvcc` 任一存在 → **abort exit 9**；CPU 3 采样 + 进程快照落盘；
  - guard v2：open 门 `free≥10240`；`floor = min(10240, baseline_free−3072)`；0.25 s 采样；
    pre/post/结束基线 3 点；无残留；窗内**零 Rust 编译**；
  - 每实体独立 exit code；attempt ≤2 有效轮；abort/污染轮留档不拼；
  - 所有 runner 写文件用 `[IO.File]::WriteAllText(..., UTF8Encoding($false))`（UTF-8 无 BOM）。
- **计划值（开窗时 runner 重采；此处为报窗材料）**：
  - 3 点基线：`2819,13243,38,31,596.36` / `2819,13243,39,31,596.36` / `2819,13243,40,31,596.36`；
    free 13243 ≥ 10240；按公式 floor = min(10240, 13243−3072) = **10171**。
  - compute-apps：29 条（其中 6 条 `[Insufficient Permissions]`，桌面/系统进程），按
    `python|g10_p5|cargo|mjwarp` 过滤 = **0**；负载快照 CPU 3 采样 `4,4,1`、无 cargo/rustc/cl/nvcc。
  - wall 预估：M3 9 实体 202.1 s → 10 实体 **≈220–300 s**（含 branch_closure 新 target 的 JIT）。
  - 显存预算：按 M3 实测 `delta_peak=278 MiB` 量级（10 实体串行，单进程峰值 ~1.5 GiB）；
    基线 free 13243 下预计 free_min ≥ 11000 > floor 10171。
- **attempt 计数**：当前 0（尚未开窗）；≤2 有效轮。

## 7. 边界（不宣称项）

- 本文件是**开窗前证据**；窗口结果与窗口事实在 run 后回填（另一提交），窗口结果只按判据内
  零失败记录，**不写「窗口通过」**、不写分支「已覆盖」。
- U7/U8 保持 **record-only**（不进分母，同时记 count/dist/pos）；U8 设备行为若与上游不同，
  按 §1.1 措辞如实记录，不改 fixture/判据/代码。
- 本片不新增容差、不改写期望值；产品代码/设备源零改动；fixtures 只用 S3 落盘 13 个，未新造。
- 机器可读 JSON（`receipt.json`）按既有惯例可超过 500 行限制；README/代码 ≤500 行。
