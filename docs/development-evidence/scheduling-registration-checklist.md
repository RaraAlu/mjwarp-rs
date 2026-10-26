# 排程登记检查清单（Normify 片登记规范）

> 来源：2026-10-26 M0a 形状修正片（`…-fix-mesh-poly-shapes`）登记时 `files` 漏写证据目录，导致任务 verified 后提交被判越界，只能另开「证据归档片」补救。本清单把该教训写死，登记任何片前逐条自查。

## 1. 登记前（新片 JSON）

`schedule.files` **必须**覆盖该片的全部写域，逐项确认：

1. **源码目录**：会改的 `src/` 子树（写目录而不是单个文件；例如 `src/physics/collision/cuda/`、`src/runtime/collision_gpu/`）。
2. **测试目录**：`tests/`（含 `tests/support/`，`#[path]` 引入的 support 文件也在 `tests/` 下）。
3. **fixtures**：会新增/修改时写具体新目录（如 `fixtures/g10-p5-mesh-multicontact/`）；**只读引用不必写**。
4. **该片证据目录**：`docs/development-evidence/<片名>/`；若 checks 里有 `json` 指向该目录文件，**必须显式写进 files**（否则提交越界，正是本次教训）。
5. **治理回执目录**（若与证据目录分开）：`docs/development-evidence/<片名>-governance/` 等。

**自查口径**：把「本片预计会 new/modify 的文件」逐条映射到 files 的一项前缀上，任何一条映射不到 → 补 files 再提交登记。

## 2. 登记后约束

- `normify start` 之后**不能**再改任务的 acceptance/checks/files（`fix` 只追加修正记录，不改声明；`waive` 只对候选/待办任务有效）。
- `verified` 任务视为终态，不能 abandon、不能改冻结字段。

## 3. 漏登记的补救路径（按代价从低到高）

1. **证据单独归档片**：证据内容零改写，只需一个只含证据目录的治理片把目录提交（本仓库已用一次，`…-fix-evidence-archive`）。
2. **abandon + 重登记 r2**：仅适用于**任务尚未 verified** 时；重登记时把 files 补全，工作树内容不动，重跑 gate/done。
3. 不推荐手改 closed 任务字段 —— 会被 hook 拒绝（`关闭时不能改写已冻结字段` / `已关闭，修正须用 Normify-Fixes`）。

## 4. 本次自查结论（2026-10-26）

| 片 | files 是否覆盖写域 | 处置 |
| --- | --- | --- |
| M0b `…-m0b-case-production` | 是（证据 scripts/ + 证据目录 + `tests/` + `tests/support/`；不写 fixtures/src） | 不动（已 start） |
| M2 `…-m2-convex-vs-mesh` | 原缺 `src/runtime/collision_gpu/`（段数/顺序断言与 test 模块）与 `tests/support/` | **已补**（`9c06b9fd`） |
| M3 `…-m3-mesh-mesh` | 同上 | **已补**（`9c06b9fd`） |
| M1 捕获片 / M0a 修正片 / 各 QA 片 | 已按实际写域登记；M0a 修正片的证据目录漏登记已由归档片补救 | 记录留痕 |

## 5. 边界

- 本清单只管「登记与范围」，不改任何判据、容差、验收口径。
- 仍遵守既有纪律：每片 QA 预登记、窗口先报基线+签名 sha、attempt 留档、作废轮不拼。

## 6. 父文件与目录必须分别写（2026-10-27 更新）

两次 scope 事故（M0a 修正片、M2）都源于同一个错误假设：把目录 `src/x/y` 当成也覆盖父文件 `src/x/y.rs`。
实际规则：**目录 scope 只匹配 `src/x/y/...`，父文件 `src/x/y.rs` 必须单独写一行**。同理：

- `src/x/y` 与 `src/x/y.rs` → 两行都写；
- 新证据目录、fixture 目录、脚本目录各自单列；
- 如果提交前才发现漏写，补救成本是「abandon + 重登记」或「单独治理片」，都不如登记时写全。

## 7. 提交前机械自查（强制）

提交前必须跑：

```text
node scripts/scope-selfcheck.mjs <task-id>
```

它把 `git status --porcelain`（含未跟踪新文件）的改动路径与任务 `schedule.files` 做集合差：

- 目录 scope 只匹配 `dir/...`，父文件 `dir.rs` 不匹配 → 能挡住「父文件漏写」；
- 忽略 `.normify/` 与 `target/`；
- **越界路径非空 → 退出码 1，停手补 files 或移出本片**；零越界打印 OK。

自查通过后再 `git add` / `commit`；M2 观察收口片起生效。

**多写者工作区补充（2026-10-27）**：Arch 收口时工作区常同时有在途片的未跟踪 WIP（例如 M3 的
`docs/development-evidence/mesh-multicontact-m3/`、`fixtures/…/mesh-mesh.xml`）。这些路径不在 Arch 当前任务的
`files` 里，`scope-selfcheck` 会报越界——**这不等于本片越界**。处置：核对越界路径确实属于另一个在途任务的声明范围，
记录在提交说明「工作树另有 X 在途 WIP、未纳入本提交」，然后**只 `git add` 本片路径**（不要 `git add -A`），
不要把别人的 WIP 捎带提交。

### 7.1 提交说明禁词自查（2026-10-27 起，强制，与 scope-selfcheck 同级）

提交前**还要**跑：

```text
node scripts/commit-msg-selfcheck.mjs <task-id> <提交信息文件>
# 自检：node scripts/commit-msg-selfcheck.mjs --self-test
```

- 词表来源：任务 JSON 的 `checks[kind=forbid_claim].terms`（与 commit-msg hook 同源）；任务文件缺失或无该检查时
  回退内置默认表（`G10已实现`/`完整物理通过`/`双平台通过`/`上游完全等价`/`原生放行`/`精度达标`）。
- 扫描提交信息全文（标题 + 正文 + trailer），命中即打印「行号 + 命中词」并**退出码 1**（hook 会拒绝该提交）；
  零命中打印 OK；退出码 2 = 用法/文件错误。
- **触发事实**：2026-10-27 M3 提交时提交信息里写了字面词「精度达标」（在否定句里），被 commit-msg hook 的
  `forbid_claim` 拒了一次——手写容易漏，故做成脚本。改写口径示例：「不宣称精度目标已达成」。
- 脚本自带 `--self-test`（4 条：干净/含禁词/CRLF/BOM），可复跑；提交说明措辞改完再跑，exit 0 才能提交。

### 7.2 字节级自查（2026-10-27 起，强制，与 scope/commit-msg 同级）

改动 `.ps1`/`.psm1`，或任何**声称「只改注释/只改文档文字」**的代码/脚本文件时，提交前必须跑：

```text
node scripts/byte-selfcheck.mjs --base <rev> [--paths a,b] [--comments-only] [--allow-bom-change]
# 自检：node scripts/byte-selfcheck.mjs --self-test
```

- 检查项：① 改动文件严格 UTF-8 可解码（二进制自动跳过）；② **BOM 状态相对 base 提交不得变化**（例如 `.ps1` 原有 UTF-8 BOM 不能被编辑工具悄悄去掉；确需改编码时显式 `--allow-bom-change`）；③ `--comments-only` 时「去注释行/空行后的行序列」必须与 base 完全一致，即「声明只改注释」要字节级成立。
- 退出码：0 通过 / 1 违规（打印文件与具体原因）/ 2 用法错误；`--self-test` 5 条可复跑。
- **触发事实**：2026-10-27 T4 债务 B 落盘（`c8753bdb`）标称「仅注释改动」，实际去掉了 `scripts/test-windows-g01.ps1` 的 UTF-8 BOM（WinPS 5.1 会按 ANSI 读无 BOM 文件），由 `9d8db76c` 恢复。**教训：仅注释改动也要做字节级检查，不能靠自审偶发发现。**
- 常用组合：`--base <本片起点> --comments-only`（注释类改动）；`--base <本片起点>`（普通脚本改动，仅校验编码/BOM）；新增文件会因 base 不存在而只做 UTF-8 校验。


## 8. 夜间负载与重测试互斥（2026-10-27 更新，Chief 指令）

**触发事实**：用户 2026-10-27 夜间问「是不是在跑大量 CPU 测试」；Chief 查询时点 0 个 `cargo`/`rustc` 进程，
但当晚数次峰值来自**门禁全量测试**与 **QA clean 重建**在时间上叠加（不是单一任务持续满载）。为避免夜间再出现
100% CPU 峰值，加两条纪律：

1. **同一时间只允许一个 clean 构建 / 全量测试**。Backend 与 QA 的重测试必须排队：先跑者持锁（简单文件锁或
   群里一句「我开始跑 clean 全量了」的互斥约定均可），跑完释放；另一方的 clean 重建/全量测试等到对方结束后再起。
   轻量定向测试（单文件测试、`cargo test --locked --test <单测>`）不受此限。
2. **夜里优先实现 / 证据 / 只读复核**。23:00–08:00 期间，全量重测试能攒就攒：攒到 GPU 窗口前、或攒到早上跑；
   夜里只跑必要的定向测试与快门禁（fmt/clippy/clippy-cuda/unit 的门禁记录可复用缓存）。
3. **口径说明**：门禁缓存命中不算重测试；`--retry` 重跑、clean 重建、`--all-features` 全量、`cargo test --locked`
   全量都算重测试。查询 GPU/CPU 进程时点为空，不等于之前没有峰值——记录峰值要按采样，不按单点。

## 9. GPU 窗口前置：机器负载检查（2026-10-27 补）

窗口计划与开窗前必须包含：

1. **零并行重构建**：开窗前确认无 `cargo` / `rustc` / `cl`（MSVC）/ `nvcc` 进程；窗口内仍**零 Rust 编译**（只跑预构建二进制）。
2. **机器负载入报告**：除 3 点 GPU 基线（used/free/util/temp）与 `compute-apps` 原文外，附 CPU 负载/进程快照，
   并注明「无 clean 重建、无全量测试并行」；这条与预构建 sha 一起报 Arch 转 Chief 双签。
3. **窗口后**：post 快照 + 结束基线 + 无残留检查照旧；若窗口期间有他人重构建，该轮按污染作废并留档。


