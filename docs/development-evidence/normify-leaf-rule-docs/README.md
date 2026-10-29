# normify 结构规则口径文档：`structure/leaf-too-coarse`

任务：`2026-10-30-normify-leaf-rule-docs`（Arch；治理/文档；只写文档与台账，不改 normify 二进制）
承接：P1 复核 **O5**（该规则仓库内无文档）＋ 154 复核 **O9**（告警文字「或超220行」与实测计数不符）
测量日期：2026-10-30（HEAD 见 `materials/measured-coarse.txt`）

## 1. 规则实际口径（源码依据）

出处：`D:\Rust\normify-rs\src\validate.rs` 叶子分支（L109-126）：

```text
leaf  = 该模块没有任何子模块（id 前缀无后代）
span  = max over module.source of ((s.end_line or s.line) - s.line)   // 记录的行区间长度
        （s 没有 line/end_line 时为 0）
coarse += 1  if module.source.len() >= 2 || span >= 220
```

要点（容易误读的三处）：

1. **`220` 判的不是文件长度**，而是模块 `[[source]]` 记录里 `line`/`end_line` 的行区间跨度；本仓目前没有任何模块记录行号 ⇒ `span` 恒为 0 ⇒ **`span >= 220` 分支今天从不触发**（死分支）。
2. 实际生效判据 = **`[[source]]` 条数 ≥ 2**（注意是条目数，包含 `scripts/`、`docs/` 等非 `src/` 源；不是「文件数」也不是「行数」）。
3. 告警是**一条聚合**：`W structure/leaf-too-coarse <N> 个叶子可能过大（多文件或超220行），建议拆分` —— `N` 是计数，不是告警条数。

## 2. 实测（2026-10-30，HEAD 见材料）

| 项 | 值 |
| --- | --- |
| 模块数 / 叶子数 | **71 / 63** |
| 触发行数 `coarse=N` | **39** |
| 其中「≥2 源」 | **39**（= 全部） |
| 其中「span ≥ 220」 | **0**（全仓无模块记录 `line`/`end_line`） |
| 单源叶但文件 LF ≥ 220（**不计入**） | **16** |

⇒ 告警文字「多文件或超220行」与实现不符：多文件判据成立（39），但「超220行」既不按文件长度（16 个单源 >220 行不计），其 span 分支当前也永不触发。

明细与完整列表：`materials/measured-coarse.txt`（含 39 个 coarse 叶与 16 个单源 >220 叶的逐项数据）。

## 3. 修正选项（改共享工具须用户批，本片不实施）

| 选项 | 内容 | 备注 |
| --- | --- | --- |
| A | 只改**告警文字**：把「多文件或超220行」改为「源条目 ≥2」；保留 span 分支但注明「仅当模块记录行号时生效」 | 最小改动，需改 normify 二进制（共享工具） |
| B | 让 span 分支真正可用：在渲染/校验时按 source 路径统计文件行数（或要求模块记录 line/end_line） | 会改规则语义与全部模块的告警数量，需用户批 |
| C | 维持现状，仅本文档说明 | 现状：QA 每次都要按数据反推 |

本片只出文档（选项 C 的说明），A/B 由后续治理片决定。

## 4. 154 复核观察处置台账（P1 O5 / 154 QA O1–O9）

| 项 | 内容 | 处置 |
| --- | --- | --- |
| P1 O5 | `structure/leaf-too-coarse` 精确规则无文档 | **本片覆盖**（§1-§3） |
| 154 O1 | 工作区证据 CRLF 是 `core.autocrlf=true` 检出产物，blob 全 LF | 不处置（以 committed blob 为准） |
| 154 O2 | 行数口径：README 的 Σ(newline+1) vs LF 行数 | 后续证据注明口径（render split 步 2 已写 LF/Σ+1=差文件数） |
| 154 O3 | `module-counts-after.txt` 的 apis 列只计 `rpc:`（合计 392 vs receipt 398，漏 6 条 file API） | 后续证据注明「rpc-only」口径 |
| 154 O4 | 该 log 67 行 vs 68 模块（零源容器 `mjwarp.physics` 无行） | 记录级接受；log 头注明「零源容器不列」 |
| 154 O5 | `validate-intermediate.txt` 捕获不完整（手抄截断） | 记录级接受；今后用重定向捕获 |
| 154 O6 | 主卡证据在否定句里字面出现术语 5 处 | 记录级接受（non-claim，check 通过） |
| 154 O7 | 主卡无 manifest | 记录级接受（已用 blob 对账替代） |
| 154 O8 | `--no-run` 日志混入 PowerShell stderr/mojibake | 后续用 `cmd /c` 或 `--message-format` 过滤 |
| 154 O9 | 告警文字「或超220行」与实测不符 | **本片覆盖**（§1-§3 数据与解释） |

## 5. 复现方式（只读）

```text
# 计数与分类（本片材料生成方式）
Get-ChildItem .normify/modules -Recurse -Filter *.md   # 解析每个模块的 [[source]] 条数与 id
# 叶子 = id 没有任何后代；coarse = 源条目 >= 2 或 span >= 220
normify validate      # 交叉核对 W structure/leaf-too-coarse 的 N
```

输出见 `materials/measured-coarse.txt`；`N=39` 与 `normify validate` 一致。
