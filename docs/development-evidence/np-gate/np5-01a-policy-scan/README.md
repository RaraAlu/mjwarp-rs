# NP5-01a policy_scan 工具入仓（卡 POL-SCAN；零 GPU）

- 任务：`2026-11-11-np5-01a-policy-scan-landing`（owner Backend；p95；P8-渲染与BVH）。
- 目的：按 QA/Chief 要求，把修复后的禁词扫描器落到仓库可复核路径（与 RE-0 脚本同口径），
  供后续片 finalizer/自检引用；避免「工具只在 target、版本不可复核」。

## 1. 工具

- 路径：`scripts/np-gate/policy_scan.py`（sha256 `59765f71f273e469d42923523fd405509ec22520b48b75f4348153dac8b527ee` / 4559 B）。
- 头注：**NOT in the 93 guards inputs; adding it to guards requires a user re-sign**（未触碰 guards/freeze script/既有采集脚本）。
- 能力：分隔符归一化匹配（空白/中点/连字符/下划线/斜杠/句点去掉后查核心词；核心词同规则归一化）；
  否定豁免（不得/未/不/无/非/禁止/not/no/never；不同/不仅/不但 先剔除以免误判）；
  `--self-test` **12 条**构造用例（分隔符与 bit-level 变体必须命中；「不写位级…」「不提供位级保证」仍豁免）。
- 局限（docstring 明记）：18 字符否定窗口为启发式，近距否定可豁免、远距否定可能漏判；扫描器是护栏，
  须配人工复核。

## 2. 自测与使用

- `python scripts/np-gate/policy_scan.py --self-test` ⇒ **12/12 PASS**（本卡实跑，见 `logs/selfcheck.txt`）。
- 用法：`python scripts/np-gate/policy_scan.py <全部 .json/.md 证据文件>`；0 命中才 PASS。
- 本卡落地源为 `target/render-bvh/gl2-ptx-diff/card-18/policy_scan_staged.py`（与落库文件逐字节相同）。

## 3. 非声明

不宣称 NP01 通过/精度完成/位级保证；不改判据/参考/容差；零 GPU、零窗。
