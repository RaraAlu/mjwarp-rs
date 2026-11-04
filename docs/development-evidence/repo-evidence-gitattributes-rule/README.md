# `docs/development-evidence/**` 的 `-text -diff` checkout 卫生规则（`.gitattributes` 追加一行）

- 任务：`2026-11-05-repo-evidence-gitattributes-rule`（P11-发布准入；纯配置，不占 GPU）
- 取数时 HEAD：`48bc2840e0af7d505951a93226aae86cbbdbf06a`（`.gitattributes` 修复为工作树未提交内容，提交后本证据目录随同入库）
- 环境：Windows x64；`git version 2.53.0.windows.2`；`core.autocrlf=true`、`core.longpaths=true`（仓库 `.git/config`）
- 状态：实现 + 验证完成；本提交 = `.gitattributes`（1 行新增）+ 本证据目录

## 结论

在 `.gitattributes` **末尾追加一行**：

```gitattributes
docs/development-evidence/** -text -diff
```

效果：`docs/development-evidence/**` 下全部已跟踪文件在 **checkout/clone** 时不再做 EOL 转换、
`git diff` 不再按文本处理 → fresh checkout 得到的字节与提交 blob 一致；证据 manifest 里按字节记的
`sha256 + bytes` 在别人 clone 后不再漂移。**不改任何既有 blob、不 renormalize、不动 `.ps1` 规则、
不动 `.git/info/attributes`。**

## 改动（1 行）

追加在文件末尾（第 323 行；前 322 行逐字未动）：

```text
docs/development-evidence/** -text -diff
```

- 选择追加在末尾：git 属性按「后匹配优先」；在 `docs/development-evidence/**` 下已有的逐目录
  `-text -diff` 规则（`g10-p5-mesh-cap20/**`、`render-bvh/**`、`criteria-purpose-audit/**`、
  `render-bvh/sdf-validate-window/**`）**值相同**，不冲突；新规则只补齐此前后未覆盖的目录；
- 该路径下**没有**任何 `text eol=lf` 规则会被覆盖（已核对 `.gitattributes` 相关行）；
- 完整 diff 见 `gitattributes-diff.txt`（`git diff --stat` 仅 `.gitattributes | 1 +`）。

### 为什么不 renormalize

- 追加属性本身**不重写任何 blob**：已验证提交内只有 `.gitattributes` 1 文件变更，证据文件 blob 不动；
- `git add --renormalize`（或 `git checkout --renormalize`）会按新属性重写 blob，等价于**改既有证据字节**，
  会让历史 manifest 里按字节记的 sha 与 blob 失配——本卡明令禁止；
- 规则只影响**此后**的 checkout/clone/diff，不回溯改历史；旧工作树若已漂移，按「回滚/修复」节用
  `git checkout -- <paths>` 从 blob 恢复即可（blob 从未变）。

## 效果范围

| 范围 | 影响 |
| --- | --- |
| `docs/development-evidence/**`（全部已跟踪文件，本卡取数时 **10580** 个） | **生效**：checkout/clone 不做 EOL 转换；`git diff` 按二进制（`-diff`） |
| `docs/development-evidence/**` 下已有的逐目录 `-text -diff` 规则 | 值不变（仍 `unset/unset`），无行为变化 |
| `src/`、`tests/`、`fixtures/`、`docs/evidence/` | **不受影响**（各自既有规则不变；`check-attr` 实样见下） |
| `.ps1` 的 BOM 规则 | **未动**（本卡不改任何 `.ps1` 行） |
| `.git/info/attributes` | **已存在、本次未改动**：本卡取数时该文件已存在（2026-10-30 起，3 条 guards 规则 `scripts/validate/render-bvh/guards.json -text` / `guards-r02-skybox.json -text` / `guards-r08-rgb-hetero.json -text`）；本卡只追加 `.gitattributes` 一行，未读改该文件（见「证据修订记录」O1） |

## 属性前后对照（主工作树，`git check-attr text diff -- <files>`）

原始输出：`attr/worktree-attr-before.txt` / `attr/worktree-attr-after.txt`。摘要：

| 文件 | 改前 | 改后 |
| --- | --- | --- |
| `docs/development-evidence/g10-p5-summary/receipt.json` | text: **unspecified**, diff: unspecified | text: **unset**, diff: **unset** |
| `docs/development-evidence/g10-p5-summary/manifest.txt` | unspecified / unspecified | unset / unset |
| `docs/development-evidence/np01-e-criterion-a2/manifest.txt` | unspecified / unspecified | unset / unset |
| `docs/development-evidence/write-lock/manifest.txt` | unspecified / unspecified | unset / unset |
| `docs/development-evidence/render-bvh/r06-s3b-flex-allowlist-assert/manifest.txt` | unset / unset（旧规则已覆盖） | unset / unset |
| `fixtures/render-bvh/scene-a/model-312.json` | unspecified / unspecified | unspecified / unspecified（不受影响） |
| `src/lib.rs` | unspecified / unspecified | unspecified / unspecified（不受影响） |

## fresh checkout 验证（不只看工作树）

脚本：`verify/verify_fresh_checkout.py`（临时 clone 全部落 `target/gitattributes-verify/`，`target/` 被
`.gitignore` 忽略、不入库）。验证口径：

1. **clone-before**：`git clone --no-hardlinks --config core.autocrlf=true --config core.longpaths=true
   <repo> target/gitattributes-verify/clone-before`（HEAD 无新规则）→ 抽样 **18 个**
   `docs/development-evidence/**/manifest.txt` 逐 sha256 与主工作树对比：**17/18 一致**
   （未覆盖目录被 autocrlf 改写，证明本规则有必要）；
2. **clone-after**：重新 clone 后**仅在临时 clone 内**应用新 `.gitattributes` → `git add && commit`
   （verify-only）→ `git rm -r --cached .` + `git reset --hard HEAD` 强制按新属性全量重建工作树，
   模拟「规则提交后的 fresh checkout」→ 同 18 个 manifest 逐 sha256 对比：
   **18/18 一致，0 缺失、0 漂移**；
   clone 内 `git status --porcelain` = `''`（干净）；
   clone 内 `git check-attr` 样例均 `text: unset` / `diff: unset`。

完整命令与逐文件 sha 见 `verify/clone-compare.json`（含 clone 前后两段、每文件 `worktree_sha256`/`clone_sha256`/`equal`）。

> 备注：主工作树样本 manifest 已逐字节等于 HEAD blob（验证时抽样即 18/18 一致），因此 clone-after 的
> 100% 一致同时证明「本规则没有改动任何已提交证据内容」。

## 回滚 / 修复

- **回滚**：删除 `.gitattributes` 末尾新增的这一行并提交即可；blob 未变，无需其它操作。
- **旧工作树/克隆若已漂移**：`git checkout -- <paths>`（或 `git checkout -- docs/development-evidence`）
  从 blob 恢复；判定用 `git hash-object --no-filters <path>` 与 `git cat-file blob HEAD:<path>` 的 sha256 比对，
  不要只看 `git status`（autocrlf 归一化会让漂移不可见）。

## 纪律

- 纯配置：未用 GPU、未跑产品/参考测试、未改判据/容差/参考/fixtures；
- 只追加 `.gitattributes` 一行 + 本证据目录；未 `git add --renormalize`、未触发任何 blob 重写、
  未改 `.ps1` 规则、未改动 `.git/info/attributes`（该文件已存在，见「效果范围」）；
- `normify` 检查：`json(receipt.json/task_id)` + `unchanged src/, tests/, fixtures/, docs/evidence/`；
  门禁 fmt/clippy/clippy-cuda/unit 全过（无代码改动，unit 数不变）。

## 证据修订记录（2026-11-05，`normify fix`）

QA 复核（`.gitattributes` 卡 PASS）拎出两条**执行证据事实错误**，经 `normify fix` 修正；**规则本身
（`.gitattributes` 追加行）与已入库 blob 均未改动**，本修订只改本证据目录的 README/receipt/manifest：

- **O1**：本 README 原写 `.git/info/attributes`「不存在/未创建」——实际该文件 2026-10-30 起已存在
  （3 条 guards `-text` 规则，见「效果范围」）；已更正为「已存在、本次未改动」。
- **O2**：`manifest.txt` 头原写 `repo-relative path`——条目实际是**证据目录内相对路径**；已把表头改为
  `evidence-dir-relative path` 并加一行 `# base dir: docs/development-evidence/repo-evidence-gitattributes-rule`。
- **O3/O4**（记录性/提交信息 cosmetic）：按 Chief 指示**不修**，仅在此说明。
- 复核锚点：`git show ce4a7d18 -- .gitattributes` 仍为 1 insertion；本修订提交只改
  `docs/development-evidence/repo-evidence-gitattributes-rule/` 下 README/receipt/manifest。

## 证据文件

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `attr/worktree-attr-before.txt` / `attr/worktree-attr-after.txt` | 主工作树属性前后逐行输出 |
| `gitattributes-diff.txt` | `git diff -- .gitattributes` 原文（1 行新增） |
| `verify/clone-compare.json` | fresh clone before/after 逐文件 sha256 对比与命令 |
| `verify/verify_fresh_checkout.py` | 可复跑的验证脚本 |
| `manifest.txt` | 本目录 sha256 + bytes + **证据目录内相对路径**（base dir 见表头，不自列） |
