# 勘误 grep 复查报告（2026-10-28，卡 `2026-10-28-render-bvh-r09-s3b-evidence-erratum`）

## 1. 目的与范围

- 目的：复核 `r09-s3b-shadow-validation` 证据目录中 **attempt1/attempt2 产物 sha 引用**与 **采样样本数（161/160）** 是否有混用。
- 范围：`docs/development-evidence/render-bvh/r09-s3b-shadow-validation/`（全量递归）。
- 方法：逐文件 UTF-8 读取、逐行正则命中（PowerShell）。

## 2. sha 类复查（六个 sha：`7ba67015|96c781dd|49f157b7|b704b9bb|603df98b|c2269841`）

### 2.1 修复前命中（5 处）

| 文件:行 | 命中 sha | 归属 | 判定 |
| --- | --- | --- | --- |
| `manifest.txt:19` | `96c781dd…` | attempt2 | **正确**（`windows/attempt2/entity-results.csv`，1 135 B） |
| `manifest.txt:20` | `7ba67015…` | attempt2 | **正确**（`windows/attempt2/facts.txt`，1 907 B） |
| `manifest.txt:21` | `49f157b7…` | attempt2 | **正确**（`windows/attempt2/guard.log`，81 B） |
| `README.md:57` | `7ba67015/96c781dd/49f157b7` | attempt2 | **错**（写在 attempt1 产物段）→ 已修正 |
| `receipt.json:54` | `7ba67015/96c781dd/49f157b7` | attempt2 | **错**（写在 attempt1 `raw_products`）→ 已修正 |

### 2.2 修复后命中（8 处，全部正确引用或勘误明示记录）

| 文件:行 | 命中 | 判定 |
| --- | --- | --- |
| `manifest.txt:19-21` | attempt2 三个 sha | 正确（归属 `windows/attempt2/`） |
| `README.md:57` | attempt1 三个正确 sha | 正确（已修正） |
| `README.md:158-159` | attempt2 错值 + attempt1 正确值 | 正确（§10 勘误小节**明示记录**，非引用错误） |
| `receipt.json:54` | attempt1 三个正确 sha | 正确（已修正） |
| `receipt.json:222-223` | attempt2 错值 + attempt1 正确值 | 正确（`errata` 字段**明示记录**，非引用错误） |

### 2.3 原件复算（权威值）

attempt1（`target/s3b-validation-window/attempt1/`，20 文件、原样未改）：

| 文件 | sha256 | bytes |
| --- | --- | --- |
| `facts.txt` | `b704b9bb7d69411b2b5c50315960518a5c1519d9cf0c071315918076c9a0f227` | 1 921 |
| `entity-results.csv` | `603df98b6c9b4f6245735d050178ba8ca9259aa3b440a8e5a6b26ae07c9e7677` | 1 139 |
| `guard.log` | `c2269841a855aa5774933861f8aa18a25774bf872fdd88eeb307d4d6676de01c` | 81 |

attempt2（`target/s3b-validation-window/attempt2/`，19 文件、原样未改）：

| 文件 | sha256 | bytes |
| --- | --- | --- |
| `facts.txt` | `7ba670158150159d99fc7dfbc2e54201aa76d4af5dcb36076650efc7a93c6b4a` | 1 907 |
| `entity-results.csv` | `96c781dd709e9b7378cd6b405cbc108abe0129a1966d73ce2cfd281ffc8e67c1` | 1 135 |
| `guard.log` | `49f157b7d44e35a4672c751debe3249e8f274048b108a638b21dba090c48c922` | 81 |

## 3. 样本数复查（`161|160` 上下文命中）

| 文件:行 | 命中 | 归属 | 判定 |
| --- | --- | --- | --- |
| `README.md:69` | 采样 **160** 样本 | attempt2 | **正确**（已修正，原误写 161） |
| `README.md:104` | 采样 **161** 样本 | attempt2-r2 | 正确 |
| `README.md:165` | 161 错值 + 正确说明 | 勘误 §10 **明示记录** | 正确（非引用错误） |
| `receipt.json:69` | `"samples": 160`（peak 2 849） | attempt2 | **正确**（已修正；另加 `samples_note`） |
| `receipt.json:114` | `"samples": 161`（peak 2 822） | attempt2-r2 | 正确 |
| `receipt.json:235` | 161→160 说明 | `errata` **明示记录** | 正确（非引用错误） |
| `windows/attempt2/facts.txt:34` | `samples=160` | 窗口原件 | 正确 |
| `windows/attempt2/guard_aggregate.json:2` | `"samples": 160` | 窗口原件 | 正确 |
| `windows/attempt2-r2/facts.txt:34` | `samples=161` | 窗口原件 | 正确 |
| `windows/attempt2-r2/guard_aggregate.json:2` | `"samples": 161` | 窗口原件 | 正确 |

## 4. 结论

**除已修正的 2 处 sha 引用（README §5.1 / receipt `attempt1.raw_products`）与 1 处样本数（README §5.2 attempt2 161→160）外，无其它错配。**
修复后：错值仅出现在勘误说明（README §10 / receipt `errata`）中作为「错在哪」的明示记录；所有正式引用行均指向正确归属。

## 5. 其它追加勘误的事实核对

- **deps 快照条数**：`target/s3b-validation-window/attempt2-build/final-freeze/deps-snapshots.txt` 实有 3 条——`20:31:29.658`（预采，GPU util 21% 不达标，舍去）、`20:32:58.615`、`20:34:00.728`（后两条间隔 62.113 s、count 1826/1826、max write 同为 `20:26:42.825`）；README §5.3 已按此改写，三行原文保留。
- **`runner exit 10` provenance**：attempt2 产物 `guard.log` 只有 `PRECHECK_OK` 与 `GUARDS_OK=56/56`（缺 `WINDOW_RUN_DONE`），`facts.txt` 有 `failures=shadows_acceptance_missing:render_kernel_flags;render_kernel_rgb`；**退出码 10 为调用方 PowerShell 进程返回值，产物内不可独立复算**，README §5.2 / receipt 已如实写明。

## 6. 影响面声明

- 本次勘误**仅修正 sha 引用、样本数表述、deps 快照口径与退出码 provenance 说明**；**不涉及任何数值、判据、容差、参考、夹具、窗口事实**。
- `windows/` 与 `target/` 原始产物零改动。
- S3b 卡 `2026-10-28-render-bvh-r09-s3b-shadow-validation` **不 reopen**（保持 verified 时间线），以本勘误卡留痕。
