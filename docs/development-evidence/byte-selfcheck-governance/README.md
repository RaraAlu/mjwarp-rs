# 字节级自查脚本（BOM / 编码 / 注释外一致）

**任务**：`2026-10-27-byte-selfcheck-bom-encoding`（owner Arch，Chief 2026-10-27 建议）。
**触发事实**：T4 债务 B 落盘提交 `c8753bdb` 标称「仅注释改动」，实际把 `scripts/test-windows-g01.ps1` 的
UTF-8 BOM 去掉了（21907 B → 无 BOM），由 `9d8db76c` 恢复（21910 B）。仅注释改动也可能改字节，
不能靠自审偶发发现。

## 1. 用法

```text
node scripts/byte-selfcheck.mjs --base <rev> [--paths a,b] [--comments-only] [--allow-bom-change] [--repo <path>]
node scripts/byte-selfcheck.mjs --self-test
```

- 默认取 `git diff --name-only <base>` + `git diff --cached` + 未跟踪文件作为检查集；`--paths` 可指定。
- 检查：① 严格 UTF-8 可解码（二进制含 NUL 自动跳过）；② **BOM 状态相对 base 不得变化**；③ `--comments-only`
  时「去注释行（`#`/`//`）与空行后的行序列」必须与 base 完全一致。
- 退出码：`0` 通过；`1` 违规（打印文件与具体原因，如 `BOM 状态变化`、`注释外内容变化`）；`2` 用法错误。

## 2. 自测与真实对照（本次实测）

| 用例 | 命令 | 结果 |
| --- | --- | --- |
| 内置 self-test | `--self-test` | **5/5 PASS**（BOM 有/无识别、仅注释新增不变、注释外多一行失败、非法 UTF-8 被拒） |
| 正例（B 落盘前的 base） | `--base 0e39bf17 --paths scripts/test-windows-g01.ps1` | exit 0（BOM 保留） |
| 正例（注释类改动） | `--base 0e39bf17 --comments-only --paths scripts/test-windows-g01.ps1` | exit 0（注释外一致） |
| 反例（缺陷提交） | `--base c8753bdb --paths scripts/test-windows-g01.ps1` | **exit 1**，报 `BOM 状态变化（base=无 current=有）` |
| 覆盖开关 | 同上加 `--allow-bom-change` | exit 0（显式允许编码变更） |

## 3. 流程接入

`docs/development-evidence/scheduling-registration-checklist.md` 新增 **§7.2**：改动 `.ps1`/`.psm1`，或任何声称
「只改注释/只改文档文字」的代码/脚本文件，提交前必须跑本脚本；「声明只改注释」必须用 `--comments-only` 证明。

## 4. 边界

- 纯提交前自查工具：不改产品代码、判据、容差、窗口数据；BOM 口径以 base 提交为基准，不自造编码策略。
- 与 hook 的关系是「提前自查」而非替代。
