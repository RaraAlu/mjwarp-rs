# 证据/fixture 行尾（EOL）口径统一 —— A 方案：pre-gate 工作树规范化

卡：`2026-11-04-governance-evidence-eol-normalization`（owner Arch，Chief 2026-11-04 批准 A 方案，无需用户签）。
脚本：`scripts/validate/normalize-evidence-eol.ps1`（ASCII-only，Windows PowerShell 5.1 解析 0 错）。

## 1. 问题：`checks.unchanged` 会被行尾假阳性咬

`normify` 0.7 的 `checks.unchanged` 按**工作树原始字节**与 `HEAD` blob 逐字节比对。本机 `core.autocrlf=true`，而 `.gitattributes` 只给部分路径 pin 了 `text eol=lf`，于是 `docs/evidence/`、`docs/development-evidence/`、`fixtures/` 下大量历史文件在检出时被展开成 CRLF：`git status` 看起来**干净**（git 的 clean 过滤器把 CRLF 归一回去），但工作树原始字节与 blob 不同。后果是任何覆盖这些路径的 `unchanged` 检查随机报「受保护路径已改动」。

两个实证触发实例：

1. **S3a 收口**（`54402b9` 前后）：`unchanged = docs/evidence/ + fixtures/` 报受保护路径已改动，当时靠一次**临时手写**工作树行尾规范化才过闸——没有脚本、没有计数、没有自证，属救火。
2. **p63 NP02 采集**（`25bd3479`）：同一坑在 `.ps1` 产物上再现（`core.autocrlf=true` 下新检出即 CRLF），QA 只能用 `.gitattributes` 的 `docs/evidence/**/new-kernels-2026-11/** -text -diff` 做**局部**保护，治标不治本。

## 2. 方案：A（工作树规范化，零 blob 变化）

只把「与 `HEAD` blob 仅行尾不同」的文件按 `HEAD` 字节重写；**不产生任何 git blob 变化、不改提交内容**。

- **不做 B 方案**（`git add --renormalize` 改 blob）：那会改动以 sha256+bytes 冻结的既有证据字节，属「改证据」范围，**需要用户签**，本次明确不做。
- **也不靠 `.gitattributes` 大范围 pin `eol=lf`**：那会让未来 `git add` 对证据文件做 CRLF→LF 归一，等价于改变未来入库的字节，仍落在 B 方案的签名边界内。

### 实现要点

| 要点 | 做法 |
| --- | --- |
| 只碰 EOL 差异 | 对每个 size 不一致的文件：读字节 → 去掉所有 `0x0D` → `git hash-object` 与 `HEAD` blob sha 比对；**相等才写回**，不等则只列报告、不动文件（真实内容改动） |
| 快路径 | 一次 `git ls-tree -r -z HEAD` 拿 blob sha，一次 `git cat-file --batch-check` 拿 blob size；**行尾差异必然改变字节数**，所以 size 相等直接跳过（13,197 个文件里 11,970 个走这条），首跑 76.3 s（旧逐文件哈希实现 >4 min 未跑完） |
| 安全阀 | 默认拒绝在脏工作树（有 tracked 改动）上运行并列出清单 → `exit 2`；`-DryRun` 只报不写；`-Force` 仅用于确认「脏的只是 EOL」 |
| 索引 stat 刷新 | `core.autocrlf=true` 让 index 记住 CRLF（smudged）size，写回 LF 后内容等于 blob 但仍显示 ` M`。脚本按 200 个一批 `git add`，**只刷新 stat/oid 缓存**：clean 过滤器 LF→LF，blob 不变（post-check 逐文件重哈希 + `git ls-files -s` 指纹不变双向验证） |
| 自证 | 写后逐文件 `git hash-object` 必须等于 `HEAD` blob；`git status --porcelain --untracked-files=no` 必须为空；不等或脏 → `exit 1` |
| 幂等 | 二次运行 `stale-eol = 0` |
| 退出码 | `0` 正常 / `1` 发现真实内容差异（列出、不改） / `2` 脏树拒绝 |

## 3. 实测（2026-11-04 06:5x，HEAD 含 S3b 拆卡登记之后）

扫描面：`docs/evidence/`、`docs/development-evidence/`、`fixtures/` 共 **13,197** 个 tracked 文件。

| 指标 | 首跑（写） | 二次跑（幂等验证） |
| --- | --- | --- |
| size 相等（快路径跳过） | 11,970 | 13,197 |
| stale-eol（仅行尾不同） | **1,227** | **0** |
| 已规范化 | 1,227 | 0 |
| 真实内容差异（列出、未动） | 0 | 0 |
| 工作树缺失 | 0 | 0 |
| 耗时 | 76.3 s | 3.0 s |

分布：`fixtures/` 36 个（559 个 tracked 中），`docs/evidence/` + `docs/development-evidence/` 1,191 个。

**零 blob 变化自证（首跑前后）**：

- `git ls-files -s` 全量指纹 sha256：`A1D7E44AC0BE37871E83433F2F9C0E1EE7A50EFAF33F60CF814E2A3F178C7A97`（前后一致）；
- `git diff --cached --numstat`：0 行（没有任何内容进入暂存区）；
- post-check：**1,227/1,227** 文件 `git hash-object` 等于其 `HEAD` blob；
- 首跑后 `git status --porcelain --untracked-files=no`：0 条。

**安全阀实测**：在 QA 正持锁改 S3c 证据（8 条 dirty）时跑 `-DryRun`，脚本 `exit 2` 并列出 8 条 dirty；`-Force -DryRun` 下把 QA 正在写的 3 个 S3c 证据文件判为 `real-content-diff` 并**原样不动**（这正是「真实内容改动只列不动」的行为证明）。

## 4. 使用口径（门禁前）

```powershell
# 1) 先看一眼会不会动东西
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/normalize-evidence-eol.ps1 -DryRun

# 2) 正式跑（脏树会被拒绝，exit 2）
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/normalize-evidence-eol.ps1

# 3) 再跑门禁 / normify gate
```

何时必须跑：

- 任何带 `checks.unchanged` 覆盖 `docs/evidence/`、`docs/development-evidence/`、`fixtures/` 的卡，**在 `normify gate` 之前**；
- 任何 `git checkout` / `git reset` / 分支切换之后（检出会把旧文件重新展开成 CRLF，1,227 个文件会再次 stale——这是 `core.autocrlf=true` 的固有行为，不是脚本缺陷）。

边界：不改判据、不改容差、不改产品代码；`-Force` 只用于「我确认脏的只有 EOL」；零 GPU。

## 5. 遗留项

- 根治方案是「把证据路径统一 pin `text eol=lf` 或统一改 blob」，两者都会改变未来入库/冻结字节，属用户签名范围，本次**不做**，只记录在案。
- 脚本首跑会重写 1,227 个工作树文件（mtime 变化、内容等于 `HEAD` blob），对使用者可见的只有「`git status` 由假脏变真干净」；如果哪天误在脏树上 `-Force`，脚本会把「CR 剥离后恰好等于 `HEAD` blob」的文件写回——这类文件在语义上就是 EOL-only，不会吞掉真实改动（真实改动必然 CR 剥离后仍不等于 blob，会被列出而不是写回）。
