# 证据二进制 `.bin` 的 checkout 卫生修复（`.gitattributes` 二进制规则）

- 任务：`2026-10-25-evidence-bin-eol-hygiene`（P5，工程卫生片；纯配置，不占 GPU）
- 状态：实现 + 验证完成，待提交（提交由 Arch 串行执行）
- 取证时 HEAD：`52609f5ec38c06f9533b1f5a1013b4ad229ce6b6`（取证时 `.gitattributes` 修复为工作树未提交内容，sha256 `516481470246ae65ee90ec88760f9ab6985089656b2a71e430ef27f66dc70b95`）
- 不重写历史、不改任何已提交证据内容、不改 S2–S8 证据语义

## 结论

`.gitattributes` 追加 `*.bin -text -diff`（及实测同为二进制的 `.f32/.npz/.gz/.mjb`）后：

1. 全部 **1130 个已跟踪 `.bin`** 的 `text`/`diff` 属性均为 unset（`git check-attr` 1130/1130）。
2. 用 `core.autocrlf=true` 全新 clone **修复前 HEAD**：1130 个 `.bin` 中 **7 个** 与工作树逐字节不一致（CRLF 转换，尺寸 +1/+2 字节）；clone 后 `git status` 仍显干净。
3. 全新 clone **修复后规则**（clone 内一次性验证提交承载，再强制全量重建工作树）：**1130/1130 逐文件 sha256 一致**，clone 后 `git status` 干净。
4. 5 个非空小 `.bin`（含 3 个 clone-before 已复现损坏的样本）`git checkout -f` 往返 sha256 不变。

即：**修复前会坏（可复现）、修复后不坏（1130/1130）**，且修复未重写任何文件内容。

## 环境与前提

| 项 | 值 |
| --- | --- |
| 平台 | Windows 10/11 x64，仓库根 `D:\Rust\mjwarp-rs` |
| git | `git version 2.53.0.windows.2` |
| `core.autocrlf` | `true`（`C:/Program Files/Git/etc/gitconfig`，全局） |
| `core.longpaths` | `true`（仓库 `.git/config`；仓库存在 >260 字符的 warp-cache 路径，clone 需显式带上，**与 `.bin` 规则无关**，属既有前提） |
| 已跟踪文件 | 12274（其中 `.bin` 1130，全部位于 `docs/development-evidence/g10-p5-gpu-kernels/`） |

## 根因

- 仓库 `core.autocrlf=true`，但 `.gitattributes` 没有任何 `*.bin` 规则（修复前 284 行，全是 text/fixtures 规则）。
- git 对没有 `text` 属性声明的文件做**内容启发式**判断：前 8000 字节内出现 NUL 即视为二进制、不做 EOL 转换；否则按文本处理。
- 1130 个 `.bin` 实测：**865** 个前 8000 字节含 NUL（被当二进制，侥幸安全）；**265** 个不含 NUL；其中 **仅 7 个** 含 `0x0A` 字节 → checkout 时被 `autocrlf` 按文本执行 LF→CRLF，字节被改写。这 7 个就是本次复现出的损坏样本（见下表）。
- **为什么以前没暴露**：这些 `.bin` 由本地探针直接生成并 `git add`，从未经历 clone/checkout 往返；同时 `git status`/`git diff` 也用同一套 autocrlf 归一化，转换是幂等的，于是「工作树已损坏」仍显示为干净，掩盖了问题。
- **影响面**：全部 1130 个已跟踪 `.bin`，以及今后任何新增 `.bin` 证据；风险发生在 **checkout/clone 方向**，不影响已提交对象。
- **界定**：当前索引/提交 blob 的原始字节与工作树一致（clone-after 用 `-text` 规则逐字节还原出 1130/1130 即可证明），**不是「已提交证据已损坏」**；只是旧规则下无法保证别人 clone 后拿到相同字节。

## 改动

只改 `.gitattributes`（+9 行，追加在文件末尾；gitattributes 以最后匹配为准，不影响任何既有 `text eol=lf` 规则）：

```gitattributes

# 证据二进制文件：禁止 EOL 转换与文本 diff。
# 背景：core.autocrlf=true 时，未声明 -text 的二进制会在 checkout 期被当作文本改写
# （G5 证据 1130 个 .bin 中已有 7 个含 0x0A 的文件会被破坏，见任务 2026-10-25-evidence-bin-eol-hygiene）。
*.bin -text -diff
*.f32 -text -diff
*.npz -text -diff
*.gz -text -diff
*.mjb -text -diff
```

扩展名由 `git ls-files` 实测得出（含 NUL 判定为二进制）：`.bin` 1130、`.f32` 4、`.npz` 6、`.gz` 22、`.mjb` 26。
`fixtures/**/*.mjb` 原有逐目录 `-text -diff` 规则保留，新增的全局 `*.mjb` 兜底了此前未覆盖的 5 个 `.mjb`（`fixtures/g10-p5/mjb-312/*`、`fixtures/g10-p5/mjb-315/*`、`fixtures/render-bvh/scene-a/*.mjb`、`fixtures/collision-reference-input/plane-sphere.mjb`）。
`.ptx/.cu/.log/.json/.jsonl/.csv/.txt` 等保持文本或既有规则不变；本片不改任何已提交证据文件内容。

## 验证方法

脚本：`docs/development-evidence/evidence-bin-eol-hygiene/verify-bin-checkout.ps1`（Windows PowerShell 5.1 可跑；UTF-8 BOM）。

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File `
  docs/development-evidence/evidence-bin-eol-hygiene/verify-bin-checkout.ps1
```

脚本做四件事（临时 clone 全部落在 `target/bin-eol-verify/`，该目录被 `.gitignore` 忽略，不入库）：

1. 记录仓库 HEAD、`core.autocrlf`、已跟踪 `.bin` 清单，并用 `git check-attr --stdin text diff` 对 1130 个 `.bin` 逐个统计属性（文件重定向喂 stdin，避免 2260 行输出触发管道死锁）。
2. **clone-before**：`git clone --no-hardlinks --config core.autocrlf=true --config core.longpaths=true <repo> target/bin-eol-verify/clone-before`，对 clone 出的 1130 个 `.bin` 逐文件 sha256 与主工作树同路径文件对比。
3. **clone-after**：同参 clone，随后**仅在临时 clone 内**复制工作树的 `.gitattributes` 并做一次性本地验证提交（commit subject `verify-only: .bin -text -diff（一次性验证提交，仅存在于临时 clone）`），再 `git rm -r --cached .` + `git reset --hard HEAD` 强制全量重建工作树，模拟「修复提交后的全新 checkout」，再做 1130 个 `.bin` 逐文件 sha256 对比。
4. 在 clone-after 中对 5 个非空小 `.bin` 做 `git checkout -f` 往返 sha256 对照（选取 = clone-before 已复现损坏的 3 个样本 + 2 个未受影响的最小非空文件）。

> 主仓库工作树若已含修复（本片提交后），重跑时 clone-before 阶段预期不再出现差异（0 不一致）；这是预期行为，clone-after 恒应 1130/1130。是否出现 7 个不一致，取决于 HEAD 是否已包含修复规则。

## 验证结果

### 1. 属性判定（1130 个 `.bin`）

`check_attr_lines=2260`（1130 文件 × text/diff 两属性）：`text: unset` **1130**，`diff: unset` **1130**，unexpected **0**。

样例（`git check-attr -a`）：

```text
docs/.../box-box-margin/geom_margin.bin: diff: unset
docs/.../box-box-margin/geom_margin.bin: text: unset
docs/.../sdf-box/oct_child.bin: diff: unset
docs/.../sdf-box/oct_child.bin: text: unset
fixtures/g10-p5/mjb-312/hfield-box.mjb: diff: unset
fixtures/g10-p5/mjb-312/hfield-box.mjb: text: unset
fixtures/render-bvh/scene-a/reference/depth.npz: diff: unset
fixtures/render-bvh/scene-a/reference/depth.npz: text: unset
docs/.../t4-historical-capture-archive/t4-historical-captures.tar.gz: diff: unset
docs/.../t4-historical-capture-archive/t4-historical-captures.tar.gz: text: unset
fixtures/g10-p5/sdf-f32/oct_coeff.f32: diff: unset
fixtures/g10-p5/sdf-f32/oct_coeff.f32: text: unset
```

### 2. clone-before（修复前 HEAD 的全新 checkout）

- clone：`target/bin-eol-verify/clone-before`（HEAD `52609f5e…`，`core.autocrlf=true`，旧 `.gitattributes` sha256 `506849fb…`）
- 1130 个 `.bin`：**1123 一致，7 不一致，0 缺失**
- clone 后 `git status --porcelain`：**空（干净）** ← 这正是问题长期不可见的机制

7 个不一致样本（尺寸差即 `0x0A` 字节数；`-text` 缺失时被 CRLF 改写）：

| # | 文件（`docs/development-evidence/g10-p5-gpu-kernels/` 下） | ref 字节 | clone 字节 |
| --- | --- | --- | --- |
| 1 | `gpu-window/cases/box-box-margin/geom_margin.bin` | 8 | 10 |
| 2 | `gpu-window/cases/box-box-margin/geom_size.bin` | 24 | 26 |
| 3 | `gpu-window/cases/box-box-margin/out1/includemargin.bin` | 8 | 10 |
| 4 | `gpu-window/cases/box-box-margin/out2/includemargin.bin` | 8 | 10 |
| 5 | `gpu-window/cases/ellipsoid-box/geom_size.bin` | 24 | 25 |
| 6 | `gpu-window/cases/mesh-mesh/witness1/x1.bin` | 24 | 25 |
| 7 | `gpu-window/cases/mesh-mesh/witness2/x1.bin` | 24 | 25 |

逐文件 sha256 与 `clone-before-compare.json` 逐项对应。

### 3. clone-after（修复规则落地后的全新 checkout）

- clone：`target/bin-eol-verify/clone-after`；一次性验证提交 `684461b90d0f21d6880b2913181d88c89a0c8ac4`（仅存在于该临时 clone），强制全量重建工作树后再对比
- 1130 个 `.bin`：**1130 一致、0 缺失、0 不一致**
- clone 后 `git status --porcelain`：**空（干净）**

### 4. `checkout -f` 往返（小 `.bin` 抽查）

5 个非空样本的 `git checkout -f` 往返 sha256 全部不变（`roundtrip_all_identical=true`）：

| 文件 | 字节 | clone-before 是否已损坏 |
| --- | --- | --- |
| `.../box-box-margin/geom_margin.bin` | 8 | 是 |
| `.../box-box-margin/out1/includemargin.bin` | 8 | 是 |
| `.../box-box-margin/out2/includemargin.bin` | 8 | 是 |
| `.../box-box-margin/out1/nacon.bin` | 4 | 否 |
| `.../box-box-margin/out1/nccd.bin` | 4 | 否 |

### 5. 反向判别力（请按此口径理解 7 vs 1123）

**修复前 clone 的 7 个 mismatch 不是「只有 7 个有问题」**，而是：旧规则依赖 git 的内容启发式，**恰好这 7 个样本的字节序列（无 NUL 且含 `0x0A`）会触发 CRLF 改写**；其余 1123 个要么前 8000 字节含 NUL、要么不含 `0x0A`，在本次内容下不发生转换。旧规则**并非 100% 触发，但触发即破坏证据复现**，且触发与否取决于数据内容（换一批 `.bin`、换一个值就可能命中），因此必须以 `-text` 规则强制二进制，而不是依赖启发式。

### 6. 工作区状态

修复后主工作树 `git status --porcelain` 仅有：` M .gitattributes`、本证据目录（`??`）、以及 QA 未跟踪的 `g10-p5-gpu-kernels-independent-check/qa-gpu/...`；**没有任何 `.bin` 被翻动**，未重写文件内容（`git diff --numstat` 只有 `.gitattributes` 9 行新增）。

## 未覆盖 / unmet

- 只在 Windows + `core.autocrlf=true` 实测；Linux/macOS 默认 `autocrlf=false` 或无 autocrlf，本片未实机复测（`-text` 规则在那些平台同样是「不转换」，但未纳入本次证据）。
- 规则只覆盖实测到的二进制证据扩展名（`.bin/.f32/.npz/.gz/.mjb`）；其它内容型二进制（如未来新增的 `.npy/.stl/.glb` 或第三方工具产物）需按同一模式补规则。
- 不重写历史、不执行 `git add --renormalize`：已提交 blob 未做任何归一化重写；本片未验证「历史提交在旧规则下曾被转换过」的情形（经 clone-after 逐字节还原证明当前 blob 与工作树一致，未发现已被转换的 blob）。
- 临时 clone 目录（`target/bin-eol-verify/`）为可复现的临时物，不入库；证据中记录其生成命令与对比结果。
- 不声称「所有 checkout/EOL 风险已消灭」；本片只保证 `.bin` 等上述二进制扩展名经 `git clone`/`checkout` 后字节不变。

## 证据文件

| 文件 | 内容 |
| --- | --- |
| `verify-bin-checkout.ps1` | 可复现验证脚本（属性统计 + clone-before/after 逐文件 sha256 + checkout -f 往返） |
| `attr-summary.json` | HEAD、git 版本、`core.autocrlf`、1130 `.bin` 属性计数、`.gitattributes` sha256 |
| `clone-before-compare.json` | clone 路径/HEAD/属性/status + 1123 一致/7 不一致明细（含逐文件 sha256 与尺寸） |
| `clone-after-compare.json` | 同上，1130/1130 一致 + 一次性验证提交 sha |
| `spot-checkout-roundtrip.json` | 5 个样本 `checkout -f` 往返 sha256 对照 |
| `summary.json` | 汇总（HEAD / 属性计数 / before 7 / after 0 / spot ok） |
| `receipt.json` | 任务回执（scope、验证数值、证据 sha256、门禁、unmet） |
