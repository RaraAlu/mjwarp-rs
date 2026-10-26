# G10@P5 M0b（32 数组 mesh case 生产 + INPUT_BINS 兼容）QA 独立复核

- 任务：`2026-10-26-mesh-multicontact-m0b-independent-check`（owner QA；纯 CPU，未建 CUDA context、未开窗）。
- 被复核片：`2026-10-26-mesh-multicontact-m0b-case-production`（verified），提交 `ac01d86d`
  （父提交 `d9c31703`）。复核时主树 HEAD `c89d84c2`（后续为治理/证据提交；代码树仍含 M0b/M2）。
- 结论：**pass（阻断 0 / 观察 6：2 一般 + 4 建议）**；未 commit、未 done。

## 0. 结论

1. **bins 与 manifest 一致**：两 case 各 32 个 `.bin` + `meta.txt`；我逐文件复算 sha256/count/dtype，
   **32/32 × 2 case 全一致**；`mesh-cases-manifest.json` 实测 sha `b7c0ed16…` = receipt 声明值；
   生产脚本实测 sha `133fedb9…` = receipt/sha256.json/manifest provenance 三处一致。
2. **同一来源（独立复算）**：我用 pinned mujoco 3.15 从同一 fixture XML 的 `capture` keyframe 现场重算
   `xml_sha256`、`qpos_bits`、`geom_xpos`、`geom_xmat` 位型，与 M1 GPU 参考**逐位一致**（两 case 全项 true，
   不是只读交付日志）；meta.txt 的 `friction/solref/solreffriction/solimp/adhesion` 也与 M1 参考首行**逐位一致**；
   另在 pinned CPU 后端独立复算 collision count = **4/4**（仅作 fixture 有效性交叉证据）。
3. **9 个 polygon 数组形状/规则正确**：实测长度 `(1,1,18,6,6,24,8,8,24)`；`sum(polynum)=nmeshpoly`、
   `sum(polyvertnum)=len(polyvert)`、`sum(polymapnum)=len(polymap)`、逐 mesh/polygon/vertex 区间、
   adr==cumsum、非负全部成立；9 数组数值与我从 pinned mjModel 现场 dump 的值**逐值相等**。
4. **CPU 自检门与探针证据链如实**：生产脚本 12 条规则两 case 全 PASS（manifest `cpu_selfcheck`）；
   「临时 `#[cfg(test)]` 探针调真实 `validate_inputs`」的证据链可复核——该提交 **`src/` 零 diff**、
   `ac01d86d` 提交树与当前树都**没有探针函数**、target 侧 raw 探针日志实测 sha 与 gates-summary 声明一致
   （`60b97128…`），日志内容含真实编译告警 + `PROBE_REAL_VALIDATE_OK` 三行 + `test result: ok`。
   探针属**历史留档**（不可复跑），常驻覆盖已由 M2 的 `validate_collision_inputs` + 真实 bins 测试接管
   （我上一片已核并直接跑过预构建二进制）。
5. **23→32 兼容真实**：`INPUT_BINS` 23→32（前 23 项顺序不变、尾部追加 9 个 mesh_poly 名）；
   sha 守卫改为按 **case QA 参考实际声明的 `input_bins` 集合**执行（未知名字 panic）；10 个旧 G5 case
   目录各 **23 个 bin、0 个 poly bin**，QA 参考均只声明 23 项；装配侧缺 9 bin → 空 slice + null sidecar，
   且 support 里显式断言「9 个同时存在或同时缺失」（不静默半装配）；4 条常驻测试我逐条核过断言并实跑
   **5 passed**（4 条 M0b + M2 追加的真实校验测试）。
6. **零改动**：M0b 提交只改 `tests/`（4 文件）+ 证据 + `.normify`；`src/`、`fixtures/`、G5 case bins/参考、
   M1/B1/B2/B3 参考、`prepare_cases.py` 全部零 diff；判据/容差文件未动（无 compare support 变更）。
7. **门禁**：提交时 M0b gate unit **986/0/372**（116 binaries，fmt/clippy/clippy-cuda exit 0；我按
   `target/m0b/unit.log` 独立聚合得 986/0/372，raw sha 与 gates-summary 一致）；当前树 **987/0/373**，
   差异恰好 = M2 追加的 **+1 非 ignored**（`real_cpu_validation_accepts_m0b_and_old_cases`）**+1 ignored**
   （`mesh_multicontact_launcher_matches_m1_upstream_gpu`），我从 `git diff ac01d86d 16214a22 -- tests/` 核过。

## 1. 复核对象

| 项 | 值 |
| --- | --- |
| M0b 提交 | `ac01d86d75f46ba53d39a0db094cf4f17e746aa6`（83 文件：证据 74 + tests 4 + normify 5） |
| 父提交 | `d9c3170349f16c66d4efad256551aee1262b316d` |
| 复核时 HEAD | `c89d84c2`（治理/证据提交，无代码影响） |
| 产物 | `cases/{mesh-box,mesh-cylinder}/`（各 32 bin + meta.txt）、`mesh-cases-manifest.json`(14316 B)、`scripts/prepare_mesh_cases.py`(17393 B) |

## 2. bins 与 manifest（逐文件复算）

- 32 个数组的 `file/dtype/count/size/sha256` 全部与 manifest 逐项一致（两 case 32/32）；
  `arrays_order` 与 receipt 的 32 项清单一致；case 目录各 **32 bin + 1 meta = 33 文件**。
- manifest 实测 sha256 `b7c0ed16a229f361b2bbc99110848aa2133259e7d6c1d24f80ba7dfb6c2503ba`（= receipt）；
  manifest `result=PASS`、`failures=0`、每 case `cpu_selfcheck.result=PASS`（12 条规则）。
- 生产脚本 sha `133fedb99eef50817d1b550803ea3aba8337f09d8ea34176b985bb17e9b049c2`
  = receipt / `scripts/sha256.json` / manifest `provenance.authoring_script_sha256`。

## 3. 同一来源（我现场重算，不读交付日志）

| 检查 | mesh-box | mesh-cylinder |
| --- | --- | --- |
| `xml_sha256` 现场重算 = manifest = M1 参考 | ✅ `88b8d7af…` | ✅ `d0b75f91…` |
| `qpos_bits`（capture keyframe f32） | ✅ `[0,0,1041194025,1065353216,0,0,0]` | ✅ `[0,0,1044549468,1065353216,0,0,0]` |
| `geom_xpos` 位型（pinned mj_forward，f32 bits） | ✅ = bins = M1 参考 | ✅ 同 |
| `geom_xmat` 位型 | ✅ = bins = M1 参考 | ✅ 同 |
| meta `friction/solref/solreffriction/solimp/adhesion` 位型 = M1 参考首行 | ✅ | ✅ |
| meta `nworld/ngeom/g1/g2/type1/type2` | ✅（1/2/1/0/6/7） | ✅（1/2/1/0/5/7） |
| CPU collision count（pinned CPU 后端复算，fixture 有效性交叉） | ✅ 4/4 | ✅ 4/4 |

说明：M1 参考里 `geom_xpos/geom_xmat` 位于 `cases.<case>.output.geom_xpos/geom_xmat`（`{value,Count}` 包装）；
我按 `value` 展平为 f32 位型比较。`meta.txt` 的 pair/标量取自 fixture 与参考首行，我逐位核过。

## 4. 9 个 polygon 数组（pinned 形状 + 我自己的 mjModel dump）

- 实测长度：`polynum/polyadr 1,1`；`polynormal 18`（=nmeshpoly 6×3）；`polyvertadr/num 6,6`；
  `polyvert 24`；`polymapadr/num 8,8`；`polymap 24` → **`[1,1,18,6,6,24,8,8,24]`**（与 pinned types.py 一致）。
- `sum(polynum)=6=nmeshpoly`、`sum(polyvertnum)=24=len(polyvert)`、`sum(polymapnum)=24=len(polymap)`；
  逐 mesh/polygon/vertex 区间全部在界内；`polyadr/polyvertadr/polymapadr == cumsum(num)` 全 True；
  所有 adr/num 非负。
- 9 数组与我从 pinned `mujoco 3.15` mjModel 现场 dump 的值**逐值相等**（含 `-0.0` 位型）。

## 5. CPU 自检门 + 真实 `validate_inputs` 探针（证据链）

- 生产脚本内 12 条规则（9 项齐全、逐 mesh/polygon/vertex 形状、`sum(polynum)`、两条强一致 sum、非负）
  与 M0a 修正后的 `validate_mesh_poly` 同序；manifest 记录两 case 全 PASS（我按 bins 独立重算同结论，§4）。
- 临时探针：`validate_inputs` 是 `pub(crate)`，集成测试不可达；M0b 当时把 `#[cfg(test)]` 探针临时追加到
  `src/runtime/collision_gpu/validate_tests.rs`，读**真实 bins + meta.txt** 装配 32 项后调用真实函数，跑完
  `git checkout --` 还原。我的核对：
  ① `git diff ac01d86d~1 ac01d86d -- src/` **为空**（src 零改动）；
  ② `git grep probe_m0b_real_bins_pass_m0a_validate_inputs ac01d86d -- src/` **无命中**，当前树也无残留；
  ③ 探针日志（target raw 与证据副本 sha 均 `60b97128…`）含真实编译告警（`validate_tests.rs:456` 的
  `unused mut`，正是临时探针代码行）+ 3 行 `PROBE_REAL_VALIDATE_OK`（两 mesh case sidecar=present
  lens `[1,1,18,6,6,24,8,8,24]`；`sphere-box-shallow` sidecar=empty lens 全 0 nworld=2 ngeom=2）+
  `test result: ok`；
  ④ 该轮**不可复跑**（探针源码未留档），属历史证据；常驻覆盖已由 M2 的
  `validate_collision_inputs` + `real_cpu_validation_accepts_m0b_and_old_cases` 接管（我在 M2 复核里
  核过其非 ignored、真实 bins、默认套件通过，并直接跑了预构建二进制）。

## 6. `INPUT_BINS` 23→32 兼容

- 两个 support：`INPUT_BINS` 23→32，**前 23 项顺序不变**，尾部追加 9 个 mesh_poly 名；
  sha 守卫改为遍历 case QA 参考**实际声明**的 `input_bins` 集合，声明名必须在 32 项内（否则 panic）。
- 装配：新增 `read_optional_bytes/f32/i32`（文件缺失→空 slice）；support 内显式断言
  「mesh_poly bin 应 9 个同时存在或同时缺失，实际 N 个」→ **不会静默半装配**。
- 旧 G5：10 个 case 目录（不是 12，见 O1）各 **23 bin / 0 poly bin**；我核了 10 个 case 的 QA 参考
  `input_bins` 均**只声明 23 项**；`old_g5_cases_keep_23_bin_compat` 断言 23 项声明数、9 slice 全空、
  目录不存在 9 个 bin、声明名都在 32 项内。
- 4 条常驻测试我逐条核了断言（manifest sha/count、装配长度/指针/规则镜像、旧 case 兼容、同一来源复算），
  并在当前树实跑：`cargo test --locked --test g10_p5_mesh_poly_case_assembly` → **5 passed / 0 failed**
  （4 条 M0b + M2 追加的 `real_cpu_validation_accepts_m0b_and_old_cases`）；交付日志（M0b 时点）为
  **4 passed**（sha `7b6ee8fe…`，与 target 原始日志一致）。

## 7. 零改动核对

| 路径 | 结果 |
| --- | --- |
| `src/` | `git diff d9c31703..ac01d86d -- src/` 为空（探针已 revert） |
| `fixtures/` | 零 diff |
| G5 case bins + QA 参考、M1/B1/B2/B3 参考 | 零 diff（`git diff` 这些证据路径为空） |
| `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/prepare_cases.py` | 零 diff |
| 判据/容差/参考比较器 | 未动（提交只改 4 个 tests 文件 + 证据 + .normify） |
| 提交改动分组 | evidence 74 / tests 4 / normify 5，无其它 |

## 8. 门禁数字

- M0b 提交时（`.normify/runs/2026-10-26-mesh-multicontact-m0b-case-production/`）：
  fmt/clippy/clippy-cuda exit 0；unit **986 passed / 0 failed / 372 ignored**（116 binaries）。
  我按 `target/m0b/unit.log`（sha `25396227…`，与 gates-summary 声明一致）独立聚合 = **986/0/372**。
- 兼容性单测（M0b 时点）：4 passed（`logs/assembly-test.log`，sha `7b6ee8fe…`）。
- 当前树：unit **987/0/373**（117 binaries）。差异说明：M2 提交 `16214a22` 追加
  1 条非 ignored（`real_cpu_validation_accepts_m0b_and_old_cases`）+ 1 条 ignored
  （`g10_p5_mesh_multicontact_m2_cuda::...`）→ 986→987、372→373；我从 diff 逐条核过。
- 我另在 M0b 复核时点实际跑过：assembly 测试 5 passed（含 M0b 4 条）、unit 门禁（M2 片 gate）987/0/373。

## 9. 观察（按级别，均不阻断）

- **O1 一般**：「旧 12 个 G5 case」与事实不符。README §1/§4 与 commit message 写 12；实际
  `gpu-window/cases/` 只有 **10** 个 case 目录，`OLD_G5_CASES: [&str; 10]`、README §3.3 表也写 10；
  M2 证据已把「12→10」作为笔误更正。建议 M0b 证据同步改为 10（或注明「原写 12 为笔误」）。
- **O2 一般**：README §5 的文件行数与实测不符（`prepare_mesh_cases.py` 331 对（非空行）✓；
  但 `primitive_qa_support.rs` 写 300 实测 raw 331/非空 312、`multicontact_qa_support.rs` 写 313
  实测 347/326、`mesh_poly_case_support.rs` 写 328 实测 362/338、`g10_p5_mesh_poly_case_assembly.rs`
  写 347 实测 385/361）。所有文件 ≤500 行，无红线影响；建议按同一口径重算。
- **O3 建议**：manifest 的 `cross_check.geom_xpos_bits/geom_xmat_bits` 只有 `{"match": true}`，没有
  actual/reference 位型（`xml_sha256`/`qpos_bits`/`collision_count` 都有 actual/reference）。常驻测试会
  现场复算、我也独立复算过，所以结论可信；但 manifest 自身的可审计性弱，建议补 actual/reference。
- **O4 建议**：交付证据 2 个日志是 UTF-8 **带 BOM**（`logs/assembly-test.log`、
  `logs/real-validate-probe.log`）；其余文件无 BOM，行数均 ≤500。建议统一转无 BOM。
- **O5 建议**：探针日志不含 cargo exit code 行（只有 `test result: ok` + PROBE 输出），且探针源码
  未留档 → 该轮只能「按日志 + 事后零残留」复核，不可独立复跑；建议 M0b 证据加一句
  「已被 M2 常驻测试取代，探针日志仅历史留档」，避免后来者把它当可复跑证据。
- **O6 建议**：探针日志里保留了当时编译告警 `validate_tests.rs:456 unused mut`，可佐证探针确实编译执行过；
  建议后续临时证据同时附探针源码文本/sha（本轮内容真实、可接受）。

## 10. 边界

- 纯 CPU：未开 GPU 窗口、未建 CUDA context；本片本来就不跑 GPU 内核。
- 9 个 mesh_poly 数组只针对单 mesh cube fixture（nmesh=1）；多 mesh 不在本片范围。
- 临时探针为历史证据（不可复跑）；常驻覆盖在 M2（我已在 M2 复核中验证并直接运行预构建二进制）。
- 不宣称 mesh 已支持；M2 设备行为不在本片范围。

## 11. 复现

```text
# 我的独立复算（pinned venv，CPU 设备，不建 CUDA context）
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-m0b/verify_m0b.py

# 兼容性/装配测试（当前树）
cargo test --locked --test g10_p5_mesh_poly_case_assembly -- --nocapture
```

完整脚本在 `scripts/verify_m0b.py`；原始 JSON 在 `target/qa-m0b/verify_m0b.json`。
