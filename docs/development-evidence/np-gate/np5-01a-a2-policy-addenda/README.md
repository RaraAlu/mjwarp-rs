# NP5-01a A-2 政策违规撤回（卡 A2-POL；append-only；零 GPU）

- 任务：`2026-11-11-np5-01a-a2-policy-addenda`（owner Backend；p95；P8-渲染与BVH）。
- 来源：QA 对 A-2 复核 PASS + **一般项 G1（政策违规）**：`evidence.json` 中有正面「位级·等价/位级·一致」表述
  （`crb:rigid_mass_matrix:accumulation:2` 的 `why_engine_semantic`；`crb:rounding-note:1` 与
  `kinematics:rounding-note:1` 的 `label`/`product_anchor.expression`）；低 L1：A-2 selfcheck 禁词表缺失
  致 `forbidden_phrases_hit=[]` 误报。

## 1. 处置

- **append-only 撤回**：`np-gate/np5-01a-a2/addenda.md` 逐条撤回 **5 处**正面位级表述，改为
  「同一 f32 加法、次序不同；**未做位级测试、不提供位级保证**」；A-2 `evidence.json`/`README`/`receipt`
  字节不改，manifest 重算（列入 addenda）。
- **声明**：63 站点分类不变（n/a 32 / identical 27 / engine-semantic 4）；`bit_level_guarantee:false` 仍成立。
- **禁词表补强**：新增 target 工具 `policy_scan.py`（`位级·等价`/`位级·一致`/`位级相同`/`位级相等`/
  `bit-level equivalent(ce)`/`bit-level equality`/`bitwise equal`；否定前缀（不得/未/不/无/非/禁止/not/no/never）
  豁免）；并入 `evidence_schema.py` 的 `validate_evidence()`，后续片 finalizer 统一调用。
- **自检重跑**（含 addendum 扫描）：
  - A-2 `evidence.json`：policy_scan 命中 **5** 处（= 已撤回的 5 处，均为历史字节；以 addendum 为准）
  - A-2 `addenda.md`：命中 **0**（期望 0）
  - `policy_scan.py` 自身：命中 **0**（期望 0；自测用短语均带否定前缀或分散组合）
  - 其余新证据文件：0 命中（见 `logs/selfcheck.txt`）

## 2. 工具与哈希

- `target/render-bvh/gl2-ptx-diff/np5-01a-tools/policy_scan.py`（工作材料，非证据）
- A-2 addenda：`np-gate/np5-01a-a2/addenda.md` `0ba40a0bf907b465fd21b81cb064be04e57162f5346b75fb3a6a09b8a457ee62` / 2030 B
- A-2 manifest：`np-gate/np5-01a-a2/manifest.txt` `9109897e8dc45047169925042379cbe3f4c0e705faabf04a5030aad66fba880b` / 661 B

## 3. 非声明

不宣称 NP01 通过/精度完成/上游等价/双平台/位级·等价；不改判据/参考/容差；零 GPU、零窗。
