# G10@P5 G6 独立复核：验收映射 / perf 方法学 / native 落点（用户拍板 B）

- 任务：`2026-10-25-g10-p5-acceptance-perf-native-independent-check`（p60）
- 复核对象：G6 收口 `ad53eaff`（治理补记 `2bb45c55`，HEAD）、native 拍板 B 契约 `09896792`（登记 `ce9baa5a`）、上游 pinned `71da24d956378a87a703b6e1442b13aec0c4ac29`
- 判定：**pass（阻断 0、发现 0、观察 3）**。本复核只做只读复算与文件核对：**未重跑设备计时**（用原始样本独立重算），未开 GPU 窗口，未改任何交付证据。

## 1. 结论要点

| 面 | 结果 |
| --- | --- |
| 验收映射 | 12 条需求 = **8 met / 4 partial / 0 unmet**、6 uncovered，与 `map.json.summary` 声明一致；**72 条 `evidence_paths` 独立核对 72/72 存在**（其中 71 条已入库、1 条为 `target/` 下未入库文件，见观察 2） |
| met 抽查（4 条） | G6-1.4（设备 sdf-box）、G6-1.5（设备 CCD 溢出位）、G6-2.2（独占窗口 perf）、G6-3.1（native 拍板 B）——判据原文 / 实测数字 / 证据路径 / 状态四者一致；数字均从被引证据文件独立重取 |
| partial 逐条（4 条） | G6-1.2、G6-1.7 同层级（关键实测只在 QA 独立复核证据，修复片自身只写常量/被覆盖在主判据内）；G6-2.1（模块写入范围未闭合：multicontact/NVRTC/primitive 缺口）；G6-2.4（NP03 native 接触字段按拍板 B 只诊断，NP04 只「已记录」）—— **均不应判 met，map 的 partial 判定成立** |
| perf 方法学 | host 9 / device 7 / upstream 9 全部用原始样本独立重算 **0 处不一致**；kernel-only 与 e2e 分栏；编译/JIT/加载排除；无跨硬件外推、无性能类达标宣称；四列覆盖非同集与「跨列 contacts/次 不同、不可直接相除」已披露 |
| 窗口合规 | HEAD 绑定 `9c0a0480`、相对 `6510317f` 产品 diff 为空、基线 3663 / 峰值 4834（+1171）/ 结束 3663 MiB、7 点采样、34 个图形上下文、本任务计算进程 0、wall 5.5 s、守卫（3072 MiB / 50%）未触发 —— 与 `perf-summary.json` 窗口字段全部一致 |
| native 落点 | 契约 `## 接触字段（native 侧）` 与提交 `09896792` **逐字一致**（HEAD 未再改）；四条措辞齐备；`native-framework.md` 与 `map.json` 无「把诊断写成放行」的表述；E 复算用产品输出、样本量 1 case/2 触点已显著标注；`hfield-box` 不可配对、样本不增成立 |
| 边界 | 未改产品/已 verified 证据/`Cargo.*`/fixtures XML；证据目录无 `.bin`/`.pyc`/warp-cache |

## 2. 验收映射：独立复算

脚本 `qa_g6_check.py` 读 `map.json`，逐条核对「要求 → 判据原文 → measured → evidence_paths → status」：

- 计数独立重算：requirements 12、met 8、partial 4、unmet 0、uncovered 6 —— 与 `summary` 声明一致。
- 状态一致性：8 条 met 的 `reason`/`impact` 均为空且有 `measured`；4 条 partial 的 `reason`/`impact` 非空；无 unmet。
- **72 条 evidence_paths**：72/72 存在（路径里的「（说明）」「::符号」后缀已剥离后核对）；其中 71 条被 git 跟踪，`target/g10-p5-native-tolerance-candidates-v3.md` 未入库（观察 2）。
- 上游 pinned：`map.upstream` = `71da24d9…`，与 `upstream-cpu.json.upstream`、`upstream_head_matches_pinned=true` 一致。

### 2.1 met 抽查（4 条，数字均从证据文件独立重取）

| 要求 | 独立重取结果（来源） | 判定 |
| --- | --- | --- |
| G6-1.4 设备 sdf-box | `gpu-window/result.json` → device_count 80、reference_count 80、dist 80/80 位等（0 ULP）、pos 203/240（`pos_max_abs` 7.4505805969238e-9 = 1.0 ULP@scale 0.0983355194）、frame 675/720（max 2.3841857910156e-7 ≤4 ULP@1.0）、nccd 0、overflow [0,0]、两次运行排序后一致 | met 成立 |
| G6-1.5 设备 CCD 溢出位 | `g10-p5-ccd-overflow-bit-fix/overflow-result.json` → 修前 272 → 修后 256（5/6 world hit FLOAT_MAX，dist bits 1900671690 与上游位级相同，ncontact/multiccd_idx 相等，两次运行位级相同） | met 成立 |
| G6-2.2 独占窗口 perf | 见 §4：原始样本重算 0 不一致、窗口自洽、无达标宣称 | met 成立 |
| G6-3.1 native 拍板 B | 契约与 `09896792` 逐字一致、四条措辞齐备、无放行表述、E 复算样本量已标注 | met 成立 |

### 2.2 partial 逐条（4 条，逐条独立判断）

| 要求 | 独立判断 | 为什么不是 met |
| --- | --- | --- |
| G6-1.2 设备地形接触 + `MAXCONPAIR=50→HFIELD(32)` | **partial 成立** | 地形接触主判据在 G5 交付证据内；但 `overflow=32`（HFIELD 1<<5）的**实测**只出现在 QA 独立复核 `qa-compare-gpu.json`（`hfield-maxconpair-overflow`: nacon 4/4、prism_count [50]、overflow [32]=上游、接触判据过），G5 修复片 README 只写常量映射、无该实测 → 与 G6-1.7 同为「关键证据在 QA 层」 |
| G6-1.7 `naconmax`/`naccdmax` 截断 | **partial 成立** | `sdf-box-naconmax40`（nacon 80/80、写行 40/40、truncated、无越界）、`ellipsoid-box-naccdmax1`（nacon 1/1、写行 1/1、overflow 位多重集相等）同样只在 QA 独立复核证据；G5 修复片自身未造这两个 case（G5 receipt 的 unmet 也列了截断未实装项） → 与 G6-1.2 **同层级** |
| G6-2.1 模块写入范围 | **partial 成立** | G2–G5 已顺序写入产品路径，但 multicontact 设备版、NVRTC 产品 launcher、primitive narrowphase 仍是模块缺口（对应 uncovered 1–3）；范围未闭合不能判 met |
| G6-2.4 先核 NP01/NP02、再 NP03/NP04 | **partial 成立** | 契约明确「先核对NP01与NP02、再执行NP03与NP04」且「NP01至NP04仍不整体放行」「机制落地不表示NP03全量通过」；NP03 的 native 接触字段按拍板 B 只作诊断、5% 红线不适用；NP04 只「已记录耗时与显存」→ 不判 met |

## 3. native 落点核对（用户拍板 B）

- 契约小节 `## 接触字段（native 侧）`（`docs/numerical-precision-contract.md` 第 219–239 行）与提交 `09896792` 中的同名小节**逐字一致**；`git diff 09896792..HEAD -- docs/numerical-precision-contract.md` 为空（HEAD 未再改）。
- 四条措辞逐条命中：①native 侧只作诊断、**待验收不放行**（且「该判定不参与通过或失败」）；②`qa-native-f64.json` 读数只作诊断与趋势、不构成准入依据；③**接触精度仍由对上游 f32 的判据承担**（既有定稿口径：向量尺度 ULP / frame 绝对 ULP / 结构零容差）；④红线 `E_candidate <= 1.05 * E_mjwarp` **在其它字段照旧生效、接触字段 native 侧不适用**。
- 另核：`本节不写入任何生效atol或rtol`、`依据：真native仅 ellipsoid-box 1 个 case、2 个接触点` 均在契约原文中。
- `native-framework.md`：B/C 选项表与「A 留待扩样」记录齐全；文件明确列出禁止的通过/等价/放行类表述（四个禁用短语见交付 README，本证据不复述字面）；E 复算表区分「设备（产品）vs native」与「上游 f32 vs native」，数字与 `qa-compare-gpu.json.native_f64_readings.ellipsoid-box` 一致（dist 3.979e-7、pos 4.546e-4、frame 1.170e-3、normal 1.170e-3）；样本量 1 case / 2 接触点显著标注、声明不可外推。
- `hfield-box` 配对：`native/model_probe_collision.cpp` 全文只做模型字段拷贝（`mjwarp_native_copy_primitive_geometry`：geom_type/bodyid/contype/conaffinity/condim/priority/solmix/solref/solimp/size/friction/margin/gap），**无 contact/narrowphase/hfield 输出入口** → 「不可配对、样本不增」成立（不伪造配对）。
- `map.json` / `native-framework.md` 放行词扫描：未发现正向放行表述（命中的均是「不得写 / 不表示 / 不提供放行信号」类否定语境）。

## 4. perf 方法学：原始样本独立重算

重算口径：`host/<case>.json.samples_ns[200]`、`device/<case>.json.kernel_ns[50]/e2e_ns[50]`、`upstream-cpu.json.cases[].samples_ns[200]`；中位用 `median`（n=200 时取中间两值均值，与存储值一致），p95 按 **perf-summary 的 nearest-rank** 与 **upstream JSON 的 numpy 线性插值**分别核对（交付 README §5.8 已披露两种口径，差异 <1%）。

| 列 | case 数 | 样本 | 中位/p95 重算与存储不一致 | 备注 |
| --- | ---: | --- | ---: | --- |
| host_cpu_single_thread | 9 | 200/例 | **0** | p95=nearest-rank 与 summary 一致 |
| device_kernel_only | 7 | 50/例 | **0** | 与 summary 及 `device_case_meta`（kernel_kind/nworld/naconmax/efc_width/nacon/nccd/overflow）全部一致 |
| device_end_to_end | 7 | 50/例 | **0** | 同上 |
| upstream_f32_cpu | 9 | 200/例 | **0** | summary=nearest-rank、upstream JSON=线性插值，两者差值 <1%（如 cylinder-box p95 1588100 vs 1588175 ns） |

- 分栏与方法：device 侧 `kernel_only_method`=「one kernel launch，CUDA events，输出清零在 start 事件前」、`end_to_end_method`=「H2D + 清零 + kernel + 全部输出 D2H + sync，host steady_clock」；host 列为产品 Rust 路径单线程 warmup 20 + 200；上游为冻结 f32 CPU warmup 20 + 200。**编译/JIT/加载**在 `perf-summary.notes` 明确「不计入任何列」（上游 load/compile/make_data 单列）。
- 覆盖缺口：`coverage.missing` = host 缺 box-box-margin、device 缺 convex-mesh/degenerate-mesh/hfield-box、upstream 缺 ellipsoid-box —— 与 README §4 一致，未补齐、未外推。
- 跨列可比性：四列不存在同 case 齐全子集；`contacts/次` 各列自身定义不同（box-box-margin 设备 2 vs 上游 10；cylinder-box host/上游 8 vs 设备 2）已在 README §5.9 与吞吐表脚注披露「不可直接相除」。
- 证据完整性：`perf/artifacts-sha256.txt` **41/41 条** sha256 复算一致（另有 1 行 `#` 注释说明 probe 二进制为本地构建产物）；`build-report.txt` 的 3 个探针源 sha256 一致；`target/g6-perf/probe_perf.exe` 仍在且 sha256 与构建记录一致（`ffc3e25c…`）。
- 宣称扫描：`perf/README.md`、`map.json`、`receipt.json` 中无性能类达标/加速比/等价声明；README 明确「性能只作证据、不进精度判据、不跨硬件外推、不用于 NP03/NP04 放行声明」。

## 5. 窗口合规

| 项 | 值（`perf/window-log.txt` / `window-memory-samples.txt`） | 与 summary 一致 |
| --- | --- | --- |
| 窗口起点 HEAD | `9c0a0480cd4a1be9508155261c8286e9eae1b54b` | ✓ |
| 相对 `6510317f` 产品 diff | 空（`src`/`tests`/`Cargo.toml`/`Cargo.lock`，在窗口 HEAD `9c0a0480` 上复算为空） | ✓ |
| 窗口时未提交改动 | 9 项（host harness 5 处 + 证据目录/脚本），已在 `perf/README §1` 披露 | ✓ |
| 基线 / 峰值 / 结束 | 3663 / **4834（+1171）** / 3663 MiB（16376），7 点 0.5 s 采样，峰值与结束均与采样文件逐值一致 | ✓ |
| 守卫 | `increment_limit_mib=3072`、`total_half_limit_mib=8188`，**未触发** | ✓ |
| 计算进程 | 基线 34 个图形上下文、本任务相关 0；结束仍 34、无 probe/python 残留（窗口日志 + 我复算 | ✓ |
| wall / GPU 负载 | 5.5 s；util max 90%/avg 20%；temp max 31°C | ✓ |
| 探针一致性 | `probe_sha_matches_build_report=true`，与 `build-report.txt` 的 `ffc3e25c…` 一致（我复算了本地二进制） | ✓ |

## 6. 观察（不构成阻断，建议后续片处理）

1. **`product_diff_6510317f..HEAD` 的措辞要绑定窗口起点**：在窗口起点 HEAD `9c0a0480` 上该 diff 为空（我复算为空）；但**当前 HEAD 非空**——G6 收口提交 `ad53eaff` 在窗口之后把 host harness 提交进来（`src/physics/collision/perf_report.rs` 为 `#[cfg(test)] mod`，另 4 处为 helper 抽取 + `#[ignore]` 性能测试，`perf/README §8` 已逐条说明均为 test-only）。`perf/README §1` 已披露窗口时 9 项未提交改动，属如实记录；建议后续 acceptance 文案统一写「在窗口起点 HEAD 上为空」，避免被误读为「当前 HEAD 的产品代码零变更」。
2. **1 条证据路径未入库**：`map.json` G6-3.1 的 `target/g10-p5-native-tolerance-candidates-v3.md` 在 `target/`（gitignored）下，仓库内不可复现；其结论摘要已收录于 `docs/development-evidence/native-contact-tolerance-decision-b/README.md §2`（逐字引用契约 + E 数字表），证据链可用，但建议把 v3 摘要正式入库或改为引用 decision-b 文档，消除对本地 `target/` 的依赖。
3. **窗口采样稀疏（已如实披露）**：0.5 s × 7 点，峰值是采样内最大值，间隙更高瞬时值不可排除；`perf/README §5.4` 已标注，守卫未触发 + 结束回落到基线。

## 7. 方法、范围与限制

- 方法：`qa_g6_check.py` 只读复算 —— map 路径/状态/计数；G5 `result.json`、CCD 修复 `overflow-result.json` 取数；perf 原始样本重算 + artifacts sha256 + 窗口日志解析 + git 历史/范围核对；契约逐字比对（HEAD vs `09896792`）+ 放行词扫描 + native ABI 核对。
- 未重跑设备计时（Arch 授权可只复算原始样本）；未开 GPU 窗口，未产生 GPU 负载。
- 未改任何交付证据/产品代码/`Cargo.*`/fixtures XML；本证据目录无 `.bin`/`.pyc`/warp-cache。
- 限制：本复核未重新采集 host/上游计时（同机桌面负载下的噪声只在交付 README 中以「单轮、量级参考」披露）；未独立核对 G6-1.1/G6-1.3/G6-1.6 的全部数字（抽了 G6-1.5，其余只核 evidence 存在性与状态）；不构成对 Linux/T4 或跨硬件的任何结论。

## 8. 提交路径与不入库

入库（`docs/development-evidence/g10-p5-acceptance-perf-native-independent-check/`）：

```
README.md  receipt.json  artifacts-sha256.json  qa_artifacts_sha256.py
qa_g6_check.py  qa-g6-check.json  qa-g6-check.log
```

不入库：本目录无 `.bin`/`.pyc`/`.o`/warp-cache；未复制交付的 `perf/` 原始样本（只读引用）。

## 9. 结论

- **阻断 0**：验收映射 72/72 路径存在、8/4/0/6 计数与状态自洽；4 条 met 抽查四者一致、4 条 partial 逐条成立（G6-1.2 与 G6-1.7 同层级）。
- **发现 0**：perf 原始样本重算 0 不一致、artifacts 41/41 哈希一致、窗口自洽；native 契约与 `09896792` 逐字一致且无放行表述。
- **观察 3**：窗口 diff 措辞绑定窗口起点、1 条证据路径依赖本地 `target/`、采样稀疏（均已在交付侧部分披露）。
