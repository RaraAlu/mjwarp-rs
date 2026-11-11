# NP5-01a C-4 复核（QA）：C-3 低项闭环 + C 批 pose/batch/sleep 63 站点 source 级

- 复核对象：卡 `2026-11-11-np5-01a-c4-pose-batch-sleep-source-level`；内容提交 **`f503c9df`**（登记 `0909f1f8`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 1 观察）**。

## 1. 第一节：C-3 的 1 低 + 观察闭环

- **① veto 块禁用否定窗口 ✅**：`policy_scan.py` = **16,532 B / `21a895b6…`**；`--self-test` **34/34 PASS**。我构造实测：
  - **veto 块 + 紧邻断言**：`{S}\nwithdrawn: never supersede\n结果位级等价\n{E}` → **命中**（换行布局）；同一行布局 → **命中**；远距断言 → 命中；
  - **veto 块 + 被否定断言**：`{S}\nwithdrawn: never supersede\n不提供位级等价\n{E}` → **命中**（保守方向，否定窗口在 veto 块内被禁用）；
  - **不误伤**：肯定声明块（`撤回`）内普通否定句 → **0 命中**；肯定声明块内普通断言 → 豁免；未标记断言 → 命中；`supersedes:` 块 → 豁免。
  - docstring 已写明 veto 与否定窗口的交互规则。
- **② `.gitignore` ✅**：根 `.gitignore` 新增 `__pycache__/`（第 24 行）与 `*.pyc`（第 25 行）并带注释；**仓内其它文件零改动**（本提交另含 `.normify/modules/mjwarp/index.md` 模块索引刷新，属 normify 簿记）。
- **③ `errata-0001`**：保持转发状态（C-3 `errata_pending` → 收尾 addenda），本片未改场表历史。

## 2. 第二节：C-4 pose/batch/sleep 63 站点

- **站点集合 ✅**：63/63，与 `sites.json` 的 4 行并集**逐 id 相等**（camlight 17 / field_batch 17 / sleep_wake 15 / attached 14），无缺/无多/无重复。
- **直方图 ✅**：`{identical 21, n/a 42}` = 63；**累计 443**（380+63，交集为空）；**C 批 52+68+43+63 = 226 站点全部收口**。
- **唯一 f64 接触点表述合规 ✅**：`field_batch:…宿主打包…:intermediate:1`（`wrap_type==2` 的 `scale = 1.0/wrap_prm`）：
  - `type_mapping.double_in_product: true` + `double_precision_parity`：「上游 `io.py:1038` 在 numpy float64 里算 `1.0/wrap_prm` 再存 f32 warp 数组；产品同为 float64 除法→f32 存储」；
  - notes：「两侧同为 float64 除法 → f32 存储；**降精度路径对称**；未做位级测试」；
  - 我逐行核对两处锚点：上游 `io.py:1038` = `m.wrap_pulley_scale[...] = 1.0 / mjm.wrap_prm[padr]`、产品 `pack_batches.rs:162` = `scale = 1.0 / fields.wrap_prm[i];` —— **路径对称、非产品单方面升精度**，表述合规。
- **锚点/字段 ✅**：全量解析 **0 错误**（38 条 file:line + 25 条 `no-reference-expression`，均 n/a 带 reason）；必填字段 63/63。
- **证据/门禁 ✅**：`policy_scan` 对 C-4 evidence/README **0 命中**；manifest **4/4**（539 B）；selfcheck 16 项全 PASS；门禁 1,406/0/402；UTF-8/LF 无 BOM/CR/NUL；无覆盖率/通过率表述。
- **证据哈希与派工值一致 ✅**：evidence `dac78b2d…`/89,151、README `b50571ab…`/6,754、receipt `20acea7a…`/4,547、manifest `35feb2c2…`/539、selfcheck `c886e579…`/1,278。

## 3. 发现

### 观察
- **O1**：本提交除 C-4 证据与工具外，还包含 `.gitignore`（已批写域）与 `.normify/modules/mjwarp/index.md` 的模块索引刷新（normify `done` 簿记）——均属预期，不构成写域越界；f64 parity 表述合规（路径对称、非升精度）。

## 4. 非声明

本复核对 C-3 低项闭环与 C-4 站点/分类/锚点/f64 表述/卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
