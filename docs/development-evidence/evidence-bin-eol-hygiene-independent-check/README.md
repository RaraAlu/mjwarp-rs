# 证据二进制 checkout 卫生修复的独立复核（`.bin/.f32/.npz/.gz/.mjb`）

- 任务：`2026-10-25-evidence-bin-eol-hygiene-independent-check`（p55，纯 CPU）
- 复核对象：`526e7659`（父 `52609f5e`；修复前基线 `e00de497`）
- 判定：**pass（阻断 0、发现 0、观察 1）**
  - 修复前 fresh clone（`core.autocrlf=true`）：**7/1188 个二进制文件被 EOL 转换破坏，且 `git status` 仍干净（静默）**
  - 修复后 fresh clone：**1188/1188 逐字节一致**（修复提交时点的二进制家族全集）
  - 主仓工作树（HEAD `fa25e45f`）：**1440/1440 与 blob 一致**；属性：1440/1440 `text: unset` + `diff: unset`（含修复后新增的 252 个 `.bin`，规则同样生效）
  - 修复片范围/历史核对通过（只动 `.gitattributes` + `.normify` 记录 + 新增证据目录，线性历史，未动 `Cargo.*`/fixtures XML/S2–S8 证据）
- 观察 1：**原地升级不会自动修复已损坏工作树**（`git checkout` 到修复后提交时，blob 未变的文件不会被重写；索引里的旧 stat 让 `status` 继续显示干净，需 `git checkout -- .` 或重新 clone）——修复的保证面是「修复后的全新 checkout/clone」。

## 1. 方法与独立性

自建两个独立 clone，**不复用 Backend 的 `target/bin-eol`、`target/bin-eol-verify`**（两者存在，本轮只列出目录名，未读写其内容/日志）：

| 用途 | 路径 | 配置 |
| --- | --- | --- |
| 逆向判别（修复前） | `target/qa-bin-eol-check` | `git clone --shared --no-checkout`，`core.autocrlf=true`、`core.longpaths=true` |
| 对照（修复后 fresh checkout） | `target/qa-bin-eol-check-after` | 同上，全新 clone 后直接 checkout `526e7659` |

比较方式不做「`git status`/`git diff` 语义」判断，而是逐文件取 **blob 原始字节**（`git cat-file --batch`）与**工作树原始字节**对比：size、sha256、首个差异偏移、是否恰为 `\n`→`\r\n` 纯插入、`\n` 个数（即预期 +size）。全量覆盖 `.gitattributes` 新规则声明的 5 类扩展名：修复提交时点 `.bin` 1130、`.f32` 4、`.npz` 6、`.gz` 22、`.mjb` 26，**共 1188 个已跟踪文件**；HEAD 时点已增至 **1440**（`.bin` 1382，见 §2.1）。

环境：Windows、`git 2.53.0.windows.2`、python 3.12.13；主仓 HEAD 运行时 `fa25e45fe582db2fe06d08e54fa188fc21e43921`；仓库含 >260 字符长路径，clone/checkout 加 `core.longpaths=true`（与 Backend 相同坑，独立踩过）。

## 2. 关键结果

### 2.1 主仓工作树（HEAD `fa25e45f`，1440 个二进制家族文件）

- 1440/1440 与 blob 逐字节一致（mismatch 0）→ 当前工作树没有被历史 checkout 破坏。
- 属性全量：1440/1440 `text: unset`、`diff: unset`（逐族：`.bin` 1382、`.f32` 4、`.npz` 6、`.gz` 22、`.mjb` 26，每族全绿，violation 0）。
- 时点说明：修复提交 `526e7659` 时点已跟踪二进制为 **1188**（`.bin` 1130）；HEAD 已含其后 Backend 的 ccd 溢出位修正片新增的 **252 个 `.bin`**（`g10-p5-ccd-overflow-bit-fix` 证据），新规则对它们同样生效（1440 全绿）——说明规则不是只对旧文件生效。

### 2.2 反向判别力：修复前 `e00de497` + `core.autocrlf=true`

修复前的 `.gitattributes` **存在**（`e00de497:.gitattributes` 可读），但**没有 `*.bin`/二进制规则**（`has_bin_rule=false`）。同一 clone 配置下 checkout `e00de497` 后：

- **7/1188 被破坏，全部是 `.bin`**；`git status --porcelain -uno` **为空**（静默损坏，不会提示）。
- 破坏形态全部是**纯 `\n`→`\r\n` 插入**（`pure_crlf_insertion=true`），size 增量恰等于 blob 中 `0x0A` 个数。

| # | 文件（`docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/`） | size | Δ | LF 数 | 首个差异偏移 |
| --- | --- | --- | --- | --- | --- |
| 1 | `box-box-margin/geom_margin.bin` | 8 → 10 | +2 | 2 | 0 |
| 2 | `box-box-margin/geom_size.bin` | 24 → 26 | +2 | 2 | 4 |
| 3 | `box-box-margin/out1/includemargin.bin` | 8 → 10 | +2 | 2 | 0 |
| 4 | `box-box-margin/out2/includemargin.bin` | 8 → 10 | +2 | 2 | 0 |
| 5 | `ellipsoid-box/geom_size.bin` | 24 → 25 | +1 | 1 | 4 |
| 6 | `mesh-mesh/witness1/x1.bin` | 24 → 25 | +1 | 1 | 20 |
| 7 | `mesh-mesh/witness2/x1.bin` | 24 → 25 | +1 | 1 | 20 |

sha256（blob → 被破坏后的工作树）：

```
1 71bd145b289665ed1a6f76fa379608f7067641dda9933724b20f7bca5b1529be -> 90aedc56309aa5155231ec054a79e8f59123060be8c7139e1b97b0f77191e841
2 4a47978227a7c450709ba2b1b24eb394982bac20117eecfe366f74b6a09996d2 -> c5ccb4b29ea4a8760ba7f55b868ce46dbe2866e811f05c8089d6a788e3cdbfc1
3 e8bc1851972c560fbc0ffd9915c22e07f4877914d2a39d16f54ce08bc0ff98f2 -> 32e4db88d8c7c91c430e385cd17aea0f8d9db989e15cda60b52ea823f9378dd2
4 e8bc1851972c560fbc0ffd9915c22e07f4877914d2a39d16f54ce08bc0ff98f2 -> 32e4db88d8c7c91c430e385cd17aea0f8d9db989e15cda60b52ea823f9378dd2
5 856be4402e5ac93dba96d9c3a740133abe4f65530950a4273ea6f00c66fd7954 -> 80cbcf77fd46b05bb9c8025453449d7d802afb7b217264ea86be79ba7ac755ad
6 0dab017570276789890833a20a0075dc26920ee2241c395350f748a4543a078f -> f2aa2cc056cd4343c4b634768036a00e169e2b15a7730087cbf7d2020cfb6642
7 0dab017570276789890833a20a0075dc26920ee2241c395350f748a4543a078f -> f2aa2cc056cd4343c4b634768036a00e169e2b15a7730087cbf7d2020cfb6642
```

### 2.3 对照：修复后 `526e7659` + `core.autocrlf=true`（fresh clone）

- 1188/1188 与 blob 逐字节一致（mismatch 0），`git status` 干净。
- 同一 clone 配置、同一级联命令，pre 7 个 vs post 0 个，差异唯一变量就是 `526e7659` 的 `.gitattributes` 5 条 `-text -diff` → 反向判别成立（不是「属性字符串看着对」）。

### 2.4 其余 4 类（58 个）修复前为何没坏

`e00de497` 时这 58 个文件的保护来源（QA 实测分类）：**20 个已有 fixture 级 `-text` 规则**（如 `fixtures/*/*.mjb -text -diff`），**38 个含 `0x00`，被 git 自动判为二进制而跳过转换**（`.gz`/`.npz`/`docs/**/*.f32` 里 `0x00` 与 `0x0A` 并存），0 个「无 NUL 且有 LF」的暴露文件。修复后这 58 个也全部命中新规则（逐族全绿）——把原本依赖「NUL 启发式 / 各目录零散规则」的隐式保护变成显式保证，避免新目录/新文件漏规则。

### 2.5 观察项：原地升级不自动修复

在同一个 clone 里先 checkout `e00de497`（7 个损坏）再 checkout `526e7659`：**7 个损坏文件仍保持损坏**（git 不为 blob 未变的文件重写工作树），且 `git status --porcelain -uno` **仍为空**（索引里保留的旧 stat 让 git 认为文件没动；触碰文件出现 stat 变化后才会显示 ` M`）。因此：修复保证「修复后全新 checkout/clone 正确」；既有损坏工作树需要手动 `git checkout -- .`（或重新 clone）来修复，这一点建议写进交付 README/后续操作说明（本片不要求改，记为观察）。

## 3. 属性抽样（`git check-attr`，覆盖不同模块目录）

全量 1440 已如 §2.1（修复提交时点 1188）；抽样 15 条（含控制项）：

| 目录/类型 | 样本 | text | diff | eol |
| --- | --- | --- | --- | --- |
| `docs/development-evidence/**`（.bin） | `g10-p5-gpu-kernels/gpu-window/cases/box-box-margin/geom_aabb.bin` | unset | unset | – |
| 同上 | `.../geom_dataid.bin`、`.../geom_gap.bin` | unset | unset | – |
| `docs/development-evidence/**`（.md/.json） | `a10-measure-release-timing-fix/README.md`、`a10-p11-windows/debug.json`、`.../receipt.json` | unspecified | unspecified | – |
| `.gitattributes` | 自身 | set | unspecified | lf |
| `docs/evidence/**`（.json） | `applied-force/audit.json` | set | unspecified | lf |
| `fixtures/**`（.mjb） | `attached-kinematics/attached-tree.mjb` | unset | unset | – |
| `fixtures/**`（.xml 控制项） | `mjwarp-complex-physics/box-box.xml` | set | unspecified | lf |
| `fixtures/**`（.json/.xml 无专用规则） | `actuation/actuation-cases.json`、`actuation/model.xml` | unspecified | unspecified | – |
| `include/**` | `include/mjwarp_collision_probe.h`、`include/mjwarp_native_probe.h` | unspecified | unspecified | – |
| `src/**` | `src/diagnostics/input.rs` | unspecified | unspecified | – |

结论：新的 5 条规则**没有误伤文本规则**（`fixtures/mjwarp-complex-physics/box-box.xml` 仍 `text set eol=lf`），且 `.bin`/`.f32`/`.npz`/`.gz`/`.mjb` 全族 `-text -diff`。

## 4. 修复片范围与历史核对

`git show --name-status --format= 526e7659` 共 15 条变更：

- `M .gitattributes`（唯一产品性改动）
- `M .normify/changes/2026-10-25-evidence-bin-eol-hygiene.json`、`M .normify/modules/mjwarp/index.md`（normify 记录）
- `A .normify/runs/2026-10-25-evidence-bin-eol-hygiene/{fmt,clippy,clippy-cuda,unit}.json`（门禁记录）
- `A docs/development-evidence/evidence-bin-eol-hygiene/{README.md,attr-summary.json,clone-before-compare.json,clone-after-compare.json,receipt.json,spot-checkout-roundtrip.json,summary.json,verify-bin-checkout.ps1}`（新证据目录）

核对结果：allowed 前缀（`.gitattributes`/`.normify/`/新证据目录）外 **violation 0**；`Cargo.toml`/`Cargo.lock`/`rust-toolchain.toml`、`fixtures/mjwarp-complex-physics/*.xml`、S2–S8 证据 **均未改**（forbidden 0）；`526e7659` 的父为 `52609f5e` 且是 HEAD 祖先，`--merges` 为空 → **线性历史、未重写**；`docs/development-evidence/**` 下只有新增目录，没有对既有证据内容的改写。

## 5. 与交付声明的一致性

交付声明（提交标题与 Arch 转述）：克隆前 7/1130 损坏、克隆后 1130/1130 一致。QA 独立复算：**7/1130 `.bin` 损坏、7 个全为纯 CRLF 插入、修复后 fresh clone 1130/1130（5 类合计 1188/1188）**——数量与形态一致（本复核未读取 Backend 的日志/结果文件，仅按同一任务输入独立复现）。

## 6. 限制

1. 只验证 `core.autocrlf=true` 这一条会触发转换的配置；`core.autocrlf=false/input` 不会造成该破坏，未重跑（结论不依赖）。
2. `--shared` clone 复用主仓对象库（不复制对象），checkout 语义与普通 clone 相同；clone 位于 `target/` 下，不入库。
3. 只覆盖**已跟踪**的 5 类二进制文件（修复提交时点 1188 个、HEAD 1440 个）；未跟踪的 `.bin`（如 G5 的 995 个本地捕获 bin）不在版本卫生范围。
4. `.gitattributes` 其它历史规则只做了抽样回归（控制项），未做全仓规则级回归。
5. 本片只审 checkout 卫生，不涉及产品运行时代码；gate 的 fmt/clippy/clippy-cuda/unit 记录在 `.normify/runs/2026-10-25-evidence-bin-eol-hygiene-independent-check/`（本任务）与 `.normify/runs/2026-10-25-evidence-bin-eol-hygiene/`（被复核的修复片）。

## 7. 复现

```powershell
cd D:\Rust\mjwarp-rs\docs\development-evidence\evidence-bin-eol-hygiene-independent-check
python qa_bin_eol_check.py            # 生成 qa-bin-eol-check.json（clone 在 target/qa-bin-eol-check[-after]）
python qa_artifacts_sha256.py         # 生成 artifacts-sha256.json
```

`qa_bin_eol_check.py` 内部：主仓 HEAD 全量对比 → `check-attr` 全量+抽样 → 两个 fresh `--shared --no-checkout` clone（`autocrlf=true`）→ pre-fix/fresh post-fix 逐字节对比 → 原地升级观察 → 范围/历史核对 → 输出 JSON 与 `qa-check.log`。

## 8. 提交路径与不入库

入库（`docs/development-evidence/evidence-bin-eol-hygiene-independent-check/`）：

```
README.md  receipt.json  artifacts-sha256.json  qa_artifacts_sha256.py
qa_bin_eol_check.py  qa-bin-eol-check.json  qa-check.log
```

不入库（实际存在的）：`target/qa-bin-eol-check/`、`target/qa-bin-eol-check-after/`（本轮自建 clone，位于 `target/` 下，可由脚本重建）；Backend 的 `target/bin-eol/`、`target/bin-eol-verify/` 未触碰；证据目录内无 `.bin`/`.o`/`.pyc`/warp-cache。

## 9. 结论

- 修复有效且判别力已被独立证明：同一 `autocrlf=true` 配置下，修复前 7 个 `.bin` 被静默改写（纯 CRLF 插入、`status` 不报），修复后 fresh clone 1188/1188（修复提交时点）逐字节一致；HEAD 时点全量 1440/1440 一致。
- 属性覆盖正确：1440/1440（修复提交时点 1188/1188）`text: unset` + `diff: unset`，控制项文本规则未被误伤；修复提交后新增的 252 个 `.bin` 也自动被覆盖。
- 修复片范围干净、历史线性、未改产品/其它证据。
- 唯一观察项：**已有损坏工作树不会被原地升级自动修复**（需 `git checkout -- .` 或重新 clone），建议在交付 README 里补一句操作提示；不构成阻断。
- 门禁：`normify gate 2026-10-25-evidence-bin-eol-hygiene-independent-check` 的 fmt/clippy/clippy-cuda/unit 全部通过（记录在 `.normify/runs/2026-10-25-evidence-bin-eol-hygiene-independent-check/`）。
