# B3（primitive 分支覆盖 + swap 类型序规范化）QA 独立复核

结论：**pass（阻断 0 / 观察 6）**。复核对象：交付 `e221e6c2`（实现+证据，HEAD）、`docs/development-evidence/g10-p5-primitive-branch-coverage/`。本复核纯 CPU：未开 GPU 窗口，device 侧只核交付窗口原始日志与二进制签名；另做 pinned 公式独立分支谓词复算、源拼装/冒烟、非 ignored CPU 自检重跑、launcher/ABI/T4 核对。

## 1. 复核范围与不变量

- 交付 diff（`e221e6c2~1..e221e6c2`）：新增 `fixtures/g10-p5-primitive-branches/`（6 MJCF + `pentagon_prism.obj`）、`tests/g10_p5_primitive_branches_cuda.rs`(366)、`tests/support/primitive_branches_case.rs`(251)、`tests/support/primitive_compare.rs`(246)；改 `src/physics/collision/cuda/product.cu`(406，primitive 入口 12 行 pair 规范化)；证据目录。共 26 文件 +2160 −19。
- 独立核：`src/physics/collision/abi.rs`、`src/physics/mod.rs`、`src/runtime/case_evidence.rs`、`src/runtime/case_compare.rs`、`.normify/gates.toml` 本提交 **零 diff**；`fixtures/g10-p5-primitive/`、`fixtures/precision/` 零 diff；旧参考 `upstream-primitive-gpu.json` 提交内零 diff（实测 sha 仍 `3a6475ac…`）；0 新 ABI 字段；PTX 入口 **6→6**（我重跑冒烟确认）；`DEVICE_SOURCES` 仍 27 段；判据/容差未改。

## 2. 分支真实性（独立 CPU 复算，不复用交付 precheck 脚本）

我用 pinned venv（python 3.12.13 / mujoco 3.15.0）加载 6 个 fixture 取 `geom_type/size` 与 mesh graph 数据，再以**参考 JSON 里同一批 device 输入** `geom_xpos/geom_xmat` 按 pinned 公式 f32 复算分支谓词（pinned 源：`collision_primitive.py` L79 图分支条件、L81-84 零距 fallback、L117-121 clamp、L442-453 端面、L454-462 角点；`collision_primitive_core.py` L177-205 非平行 det）：

| case | 我复算的谓词 | 命中分支 | 交付 precheck 对照 | 参考 count |
| --- | --- | --- | --- | --- |
| plane-mesh-graph | `mesh_vertnum=10`、`mesh_graphadr=0`、`graph[0]=10` → `graphadr!=-1 && vertnum>=10` | plane_convex **graph hillclimb** | 一致 | 4 |
| sphere-sphere-concentric | 两球心 f32 距离 `0.0`（bits=0） | `dist==0` fallback `n=(1,0,0)` | 一致 | 1 |
| sphere-capsule-clamp | `projection=0.15000000596 > half_length=0.119999997`（f32 的 0.12） | 投影 **clamp** | 一致（其 half 写作 0.12） | 1 |
| sphere-cylinder-cap | `x=0.18`、`side=false`、`cap=true`、`x>0` | **端面 cap** | 一致 | 1 |
| sphere-cylinder-corner | `x=0.14`、`side=false`、`cap=false` | **角点 corner** | 一致 | 1 |
| capsule-capsule-nonparallel | 轴 90°；`det`（融合/非融合均）= `2.0736003353e-4` ≥ `1e-15` | **非平行 det** | 一致（量级；其 `axis_dot=0.0` 为理想化，见 O5） | 1 |

6/6 `hit=true`，与 `precheck-branches-cpu.json` 的谓词一致；参考 count 与 CPU 后端 nacon（4/1/1/1/1/1）无差异。**分支不是凑数**：graph 分支的 `vertnum>=10` 与 mesh graph 数据独立从 MuJoCo 模型读取确认。

## 3. swap 类型序规范化（小修）

- pinned `_add_geom_pair`（`collision_driver.py` L350-380）：`type1 > type2` 时 `pair=(geom2,geom1)`，即上游 `contact.geom` 恒按类型序；`_primitive_collisions` key 与 kernel 匹配也按类型序（L1474-1480 精确匹配）。我核过该段源码。
- 设备改动（`product.cu` primitive 入口 12 行）：`pair_swap = a.type1 > a.type2`，派生局部 `type1/type2/geom_index1/geom_index2`，**dispatch 全部分支改用局部变量**（我 grep 确认入口内无 `a.type1/a.g1` 残留），并覆写 `params.geoms` 为规范化顺序。入口内不新增 ABI 字段/入口。
- 既有调用方 no-op：B0/B1/B2 的 case 均低类 geom 在前（plane/sphere 为 geom0），`pair_swap=false` → 局部变量即 `a.*`、`params.geoms` 不变、wrapper 入参不变；因此源码级等价。
- swap 变体：测试对 4 个类型不同的 case 以 `(g1=1,g2=0,type1=高类,type2=低类)` 输入，设备规范化后与同一参考逐字段（含 `contact.geom` 零容差）比较；attempt2 日志 4/4 `failures=0`，无「顺序/数值不符」消息（0 ABI、entries 6 另见 §4）。

## 4. 源拼装 / 冒烟（CPU，无 context）

- 我在本机重跑 `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored`：**26 passed / 0 failed**，`nvrtc_smoke_ok ptx_bytes=3786510 entries=6 source_bytes=237922 source_sha256=92ee236e…`，与交付 §5 完全一致。
- 自写 27 段拼装（`scripts/assemble_b3.py`）：工作树字节 = **237922 / `92ee236e…`**（与冒烟逐字节一致）；git blob（LF）= **237442 / `b1408f9b…`**（B3 证据未列 LF 口径，我补记）。与 B2 的 237212/`054aaa40…` 相比只差 `product.cu` 入口文本，符合「只改入口」。
- `product.cu` 406 行 <450；新测试/支持 366/251/246 均 ≤500；`DEVICE_SOURCES=27`。

## 5. 参考可信性（新参考 `a8407a90…`）

- 参考文件实测 sha256 = `a8407a90ba7ab491aac7e73c94108e5abd07ce3a3d4218382477b109dc62fca0`（9212 B）；**与交付两份 raw 逐字节相同**：`target/g10-p5-primitive-branches-capture/attempt1/upstream-primitive-branches-gpu-{A,B}.json` 两者 sha 均 `a8407a90…`，且 `compare.log` 记录 `RAW BYTE-IDENTICAL: PASS` → 两轮独立进程/独立 warp cache（run-a/run-b 日志分别指向 `warp-cache-a`/`warp-cache-b`）成立。
- 每 case `two_captures_bitwise_identical=true`（6/6，run 日志 `two_captures=True`）；`qpos_bits` 与 qpos f32 位型逐 case 一致；`xml_sha256` 与 `inputs-sha256.json` 及实际 fixture 三者一致。
- provenance 齐全且与实测一致：python 3.12.13、mujoco 3.15.0、warp 1.15.0、mujoco_warp 3.15.0、numpy 2.5.4；GPU `RTX 4070 Ti SUPER`、driver `596.36`、sm 8.9（与 `nvidia-smi` 实测相同）；`authoring_script_sha256` = 证据副本 sha = 执行副本 sha（`9cff9b31…`）；`captured_at=2026-10-26T21:53:51.581+08:00` = 捕获窗口 `window_start`，落在 `window_end=21:54:13.438` 内（baseline/run-a/run-b/compare/facts mtime 21:53:50–21:54:13 同区间）；`kernel_cache_parent` 存在且 A/B 子目录独立。
- 捕获窗口采样（我按 CSV 重算）：70 样本、peak 3874、free_min 12188、util_max 36、delta 240、residual 0、时间戳单调；基线 `3629/12433`，结束 `3629/12433`。
- **旧参考零改动**：`upstream-primitive-gpu.json` 提交内无 diff，实测 sha 仍 `3a6475ac…`。

## 6. 验证窗口 attempt1（aborted）与修复边界

- attempt1（21:55:42.076–21:56:23.829，141 样本、peak 4672、free_min 11390、util 68、delta 1203、residual 0）：`branches` exit 101，其余 4 个二进制 exit 0。
- 红因核实：`run-branches-b3.err.log` 显示 panic 在 `tests/support/primitive_compare.rs:48:35`（`flatten_f32` 的 `.expect("f32")`）；run 日志**没有任何 `case: nacon=` 行** → panic 发生在第一个 case 的 device 运行之前，无 per-case 数值/分支结论。原因与交付描述一致：把 3 层 `geom_xpos/geom_xmat` 喂给两层 flatten。
- 修复边界：交付包含新 `flatten_f32_nested`（三层）、把 `reference_rows` 抽成 `reference_expectation`、新增 non-ignored CPU 自检；比较器字段集/阈值/期望/参考来源我逐项核过，与 B1/B2 同口径，无放宽。attempt1 与 attempt2 签名仅 `branches` 变化（`1e3c2adf`→`25410b3d`），另 4 个二进制度不变；该窗口 attempts 事实支持「修复只重建了 branches 二进制」。
- 新增 CPU 自检：我独立重跑 `reference_support_parses_without_gpu` → **1 passed**，输出 6 case `support_parse_ok`；测试函数只读 JSON（无 `CollisionGpuProgram::new`/`run`），执行耗时 0.00 s，未建 context。其覆盖 sha/三层长度/字段行宽/reference_rows/分支谓词；对 attempt1 这类**调用方误用两层 flatten** 只能部分兜底（见 O2）。

## 7. 验证窗口 attempt2（收口轮）

- 窗口 22:18:38.121–22:19:09.213（body 26.5 s），5 二进制并行、5/5 exit 0；采样 103 点（我按 CSV 重算：peak 5242、free_min 10820、util 45、delta 22、residual 0、时间戳单调）；基线/结束基线 `5215/10847`；post `query-compute-apps` 无本任务进程；窗口内无 `Compiling`/`rustc` 记录。
- **6 分支 case**：plane-mesh-graph nacon=4、其余各 1，`failures=0`；**swap 变体** 4/4 `failures=0`（plane-mesh (7,0)、sphere-capsule (3,2)、sphere-cylinder-cap/corner (5,2)）。
- **13 组合回归**：B1 plane 6/6 `failures=0`；B2 sphere/capsule 4/4 `failures=0`；primitive 3/3 `bit_equal=80/80, max_dist/pos/frame_ulp=0`；launcher 3/3 `max_*_ulp=0`（`bit_equal 314/314、3122/3122、80/80`）。注：B1/B2 的「位级」只有 `failures=0` 支持（判据含 4 ULP 容差），见 O1。
- 签名集合实测：`target/debug/deps` 现存 5 个二进制 sha 与 facts/README/receipt **逐字节一致**：branches `25410b3d…`、plane `115b5c99…`、B2 `280f2e47…`、primitive `ec56ff74…`、launcher `84ee6074…`。

## 8. ABI / T4 / 只读声明

- 0 新 ABI 字段（`abi.rs` 无 diff）；0 新入口（entries 6）；`src/physics/mod.rs` 零 diff、未另立 repin、既有 `fixtures/` 零改动（新 fixture 只在新目录）；旧参考/B1/B2 证据零改动（本提交内）。

## 9. 未覆盖 / 边界

- 不重跑 device（按纪律 CPU-only、未开窗）；swap 的 device 行为只核窗口日志，非我独立执行。
- 不宣称全分支/全路径覆盖：plane-mesh 只覆盖一条 10 顶点棱柱路径；capsule-capsule 只覆盖一组 90° 非平行轴；sphere-cylinder 只覆盖 `x>0` 侧端面/角点；capsule 平行轴分支仍未覆盖。
- 13 组合「位级无回退」对 B1/B2 是判据内通过 + 源码 no-op 论证，非实测 bit_equal（O1）。
- 未做 release/Linux；不宣称精度或跨环境结论。

## 10. 门禁记录（主树，stdout 写 `target/qa-b3/gate-check/`）

- `normify gate 2026-10-26-g10-p5-primitive-branch-coverage-independent-check --repo D:\Rust\mjwarp-rs`：**exit 0**，fmt/clippy/clippy-cuda/unit 全过（unit 计数见 `.normify/runs/<task>/unit.json`；该 run 绑定本证据目录内容）。
- `normify check`：**exit 0**（紧跟最终 gate 之后运行；stdout 存 `target/qa-b3/gate-check/check.log`，不拷入本目录以免范围变化使门禁记录过期；结果见 receipt.check）。

## 11. 复现命令

```text
# 分支谓词独立复算（pinned venv，不建 CUDA context）
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-b3/recompute_branches.py

# 27 段拼装（工作树 vs git blob）
python target/qa-b3/assemble_b3.py

# CPU 自检（non-ignored）
cargo test --locked --features cuda-probe --test g10_p5_primitive_branches_cuda reference_support_parses_without_gpu -- --nocapture

# 冒烟（无 context）
$env:PATH="D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;$env:PATH"
cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored --nocapture

# 门禁
normify gate 2026-10-26-g10-p5-primitive-branch-coverage-independent-check --repo D:\Rust\mjwarp-rs
normify check 2026-10-26-g10-p5-primitive-branch-coverage-independent-check --repo D:\Rust\mjwarp-rs
```

## 12. 观察清单（详见 `qa-observations.json`）

| id | 级别 | 摘要 |
| --- | --- | --- |
| O1 | 一般 | 「13 组合位级无回退」对 B1/B2（10/13）只有 `failures=0` 证据，判据含 4 ULP；只有 primitive/launcher 打了 `bit_equal`。建议改口径或补计数 |
| O2 | 建议 | CPU 自检不经过 `run_case` 的 geom 解析路径，`device_touched=false` 为恒真断言；attempt1 类调用方误用只能部分兜底 |
| O3 | 一般 | 执行脚本留在仓库根 `scripts/` 未提交（untracked），参考 provenance 记的路径在提交树不可解析；证据副本实测与执行副本逐字节相同（已留 sha） |
| O4 | 建议 | B3 README 第 204/206 行含 2 个 BEL 控制字符（`\authoring…`/`\8…` 显示丢失首字符），§9 标题重复（ABI 与脚本归档都编号 9） |
| O5 | 建议 | precheck 的 `axis_dot=0.0` 为理想化；按参考 `geom_xmat` f32 实算 `axis_dot≈-7.57e-9`、`det≈2.073600335e-4`（与其 `2.0736003310918745e-4` 差 ~4e-15 绝对值），分支命中不受影响，建议标注来源口径 |
| O6 | 建议 | attempt1 `branches` 二进制 `1e3c2adf…` 未归档，无法二进制级验证「修复只碰 tests/」；panic 日志（device 运行前）+ 提交内容 + dirty_entries 10→11 支持该说法 |
