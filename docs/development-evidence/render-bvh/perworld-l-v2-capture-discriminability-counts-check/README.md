# QA 复核：capture.json §2.4 判别力计数卡（`311b5a84`；零 GPU）

- 任务：`2026-11-08-render-perworld-l-v2-capture-discriminability-counts-check`（owner QA、p79、P8-渲染与BVH）
- 复核对象：Backend 卡 `2026-11-08-render-perworld-l-v2-capture-discriminability-counts`，提交 **`311b5a84`**（parent `b0556af1`）
- 依据：已签 spec v3 `bb064fe0…`/39,379 B **§2.4**（capture.json 判别力计数）与 G-L0 `readout_c1_c2.py`（`4a054aba…`/16,711 B）
- 基线：复核开始时 HEAD `569e636c`、树 clean、锁 FREE（期间 Backend 短暂持锁改实体卡，我停写等待后继续）；**只读被复核内容 + 只写本卡证据；零 GPU、不采集、不开窗**

## 0. 结论

**PASS（0 阻断）。可进 S-2 材料前置。**

- §2.4 的 5 组计数、两套 C1 计数（bitwise + magnitude）、C2 depth/seg、两个诊断项**字段与结构齐全且键名逐字对齐**；相机归属写死、无多余冒充。
- 计数公式**逐位复刻** readout（同一 unpack、同一 f32→f64 diff、同一 per-pixel 比较）；用 pilot 保留 npz 独立复算，全部 8 个量 × 2 对 world 与 `readout.json` 及卡内 artifact 逐项一致。
- 计数在**两遍采集逐位一致校验之后**计算（数组相等 + npz 字节哈希相等之后）；helper 仅定义在 `perworld_capture_meta.py`，主脚本 1 import + 1 call；行数 490/379 ≤500，无新模块文件。
- 默认行为不变（dry-run 仅 `/recipe/script/{sha256,bytes}` 变、unexpected=0、不建 out_dir；`--worlds 1` 两版 exit 1）；模块自检 **16/16**。
- 边界干净（只两脚本 + 证据）；被测 manifest 11/11；四门禁通过：fmt/clippy/clippy-cuda exit 0、unit **1387/0/402**（本卡自身 run 记录 1386/0/402；1387 来自随后 `bab01507` 新增的一条测试）。

## 1. 字段与结构（spec §2.4 逐项）

`capture.json` 顶层新增（`discriminability_counts` 返回值平铺）：

| 字段 | 结构 | 本参考的期望形态 |
| --- | --- | --- |
| `c1_rgb_changed_px` | `{"0->1": {"bitwise": N, "magnitude": M}, "1->2": {...}}` | cam-persp 两套计数 |
| `c2_depth_changed_px` | `{"0->1": N, "1->2": N}` | cam-hinge depth 位型变化像素 |
| `c2_seg_changed_px` | `{"0->1": N, "1->2": N}` | cam-hinge seg pair 变化像素 |
| `campersp_depth_seg_changed_px` | `{"0->1": {"depth": N, "seg": N}, ...}` | 诊断项（只记录） |
| `camhinge_rgb_changed_px` | `{"0->1": {"bitwise": N, "magnitude": M}, ...}` | 诊断项（只记录） |

- 名称与 spec §2.4 逐字一致（`c1_rgb_changed_px` / `c2_depth_changed_px` / `c2_seg_changed_px` / `campersp_depth_seg_changed_px` / `camhinge_rgb_changed_px`）；两对 world 键 `0->1`/`1->2` 齐全。
- 相机归属写死：C1 只取 `rgb_*_cam0`（cam-persp）、C2 只取 `depth/seg_*_cam1`（cam-hinge）；诊断项 cam-persp depth/seg + cam-hinge rgb，互不冒充。
- `verify_capture_metadata` 增加结构校验：5 组键必须恰为 `["0->1","1->2"]`；`c1_rgb_changed_px`/`camhinge_rgb_changed_px` 每对必须恰为 `["bitwise","magnitude"]`；`campersp_depth_seg_changed_px` 每对必须恰为 `["depth","seg"]`。

## 2. 口径与独立复算（pilot 保留 npz；npz 在 `target/`、非入库）

公式（与 `readout_c1_c2.py` 逐位同式，无自创等价式）：

- `magnitude = count_nonzero(np.abs(x.astype(f64) - y.astype(f64)).max(axis=-1) >= 1/255)`
- `bitwise = count_nonzero(np.any(x.view(int32) != y.view(int32), axis=-1))`
- `c2_depth = count_nonzero(d0.view(int32) != d1.view(int32))`
- `c2_seg = count_nonzero(np.any(s0 != s1, axis=-1))`

| 量 | 0→1 | 1→2 | 与 readout.json / 卡内 artifact |
| --- | --- | --- | --- |
| C1 magnitude | **38,285** | **40,644** | 一致 |
| C1 bitwise | **43,604** | **41,815** | 一致 |
| C2 depth | **35,828** | **33,883** | 一致 |
| C2 seg | **16,041** | **16,754** | 一致 |
| 诊断 cam-persp depth / seg | 0 / 0 | 0 / 0 | 一致 |
| 诊断 cam-hinge rgb magnitude | **50,363** | **52,329** | 一致 |
| 诊断 cam-hinge rgb bitwise | **54,380** | **54,250** | 一致 |

`matches_readout_json = true`、`matches_card_artifact = true`、`unexpected = 0`（`artifacts/counts-recompute.json`；pilot npz sha 与读到的 `readout.json` sha `7138f238…` 一并记录）。

## 3. 写入时机 / 实现 / 规模

- 时序（主脚本）：`first/second` 采集 → 数组逐位相等校验（L433-438）→ npz 字节哈希相等校验（L447-448）→ `discriminability_counts(first["arrays"])`（**L453**）→ 组装 capture.json → 写入 → `verify_capture_metadata(capture)`（L475）。即计数在**确定性证明之后**计算，取自 first run（与第二跑逐位一致的同一份数据）。
- `discriminability_counts` 仅定义于 `scripts/perworld_capture_meta.py`（L169）；主脚本恰 1 处 import（L53）+ 1 处调用（L453）。
- 行数（git-blob 换行口径）：`capture_render_perworld.py` **490**、`perworld_capture_meta.py` **379**，均 ≤500；未新建模块文件（diff 只两脚本）。

## 4. 默认行为与自检

- 默认 dry-run 与显式 light-scene dry-run（HEAD vs 工作树逐叶对照）：`unexpected=0`，仅 `/recipe/script/{sha256,bytes}` 变；`cameras` 3 条（persp/hinge/track）、`cam_active_fixed_only=[true,true,false]`、`legacy_manifest_entries=112`、不创建 out_dir。
- `--worlds 1`：两版 exit 1、stderr 逐字相同、不建 out_dir（行为不变）。
- 独立复跑 `python -B scripts/perworld_capture_meta.py` → **SELFCHECK PASS 16/16**（新增 `discriminability-positive` 与 `discriminability-negative-missing-array`）；`artifacts/selfcheck-rerun.log`。
- 我另行实跑默认 light-scene dry-run：exit 0、未建 out_dir；dry-run 计划中**不含**计数键（dry-run 行为未被改变）。

## 5. 边界 / 证据 / 门禁

- `311b5a84`（parent `b0556af1`）只改 `scripts/capture_render_perworld.py`、`scripts/perworld_capture_meta.py` + 本卡证据 + 卡/runs；`fixtures/`、`src/`、`tests/`、spec 目录 **零 diff**。
- 被测证据目录 manifest **11/11** 独立重算；编码卫生（UTF-8/LF、无 BOM、裸 CR·TAB 0）。
- 四门禁：fmt exit 0、clippy exit 0、clippy-cuda exit 0、unit **1387 passed / 0 failed / 402 ignored**（当前 HEAD；`311b5a84` 自身 run 为 1386/0/402，+1 来自随后 `bab01507`）。

## 6. 观察项（不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 信息 | `verify_capture_metadata` 在 `capture.json` **写入之后**调用（L472-475）；计数结构异常时窗口以非零退出但文件已落盘 | 现有关键校验流程一致，非本卡缺陷；如要更严可先 verify 再写盘 |
| O2 | 低 | 结构校验只查 `c2_depth/c2_seg` 的 pair 键，未校验其值类型（int）；`campersp` 的 depth/seg 值类型亦未查 | 可在 `verify_capture_metadata` 加两行 `int` 校验（非阻断） |
| O3 | 信息 | 实体 `check_capture_c1` 与本结构不匹配属**预期**：本卡按 §2.4 输出字典形态（正确），实体侧适配由另卡 `bab01507` 处理（已落库） | 不需本卡改动 |

## 7. 非声明

本卡只核 `capture.json` 的计数记录与 CPU 自证一致性：**不宣称**采集有效性、不宣称判别力恢复、不宣称任何渲染数值结论；真实计数以 G-L1 采集窗产物为准。pilot npz 与 readout.json 均在 `target/`、不入库。
