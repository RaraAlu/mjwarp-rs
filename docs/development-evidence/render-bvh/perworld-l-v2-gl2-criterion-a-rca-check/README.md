# QA 复核：G-L2 判据 A 只读 RCA 卡（`63cf7cc0`）

- 复核卡：`2026-11-11-render-perworld-l-v2-gl2-criterion-a-rca-check`（QA）
- 被复核：`2026-11-11-render-perworld-l-v2-gl2-criterion-a-rca`（Arch，`63cf7cc0`；README `23d592a1…`/9,084 B、manifest 5/5）
- 性质：只读复核，零 GPU、零重跑；只写本卡与本卡证据。
- 结论：**PASS（0 阻断，1 条信息级观察）**。RCA 记录如实可复核；不改变 G-L2 判定（未通过），不宣称修复生效、不建议放宽。

## 1. 八项内容（逐项核对）

1. **归属纠正**：`seg` 第二分量是 MuJoCo object type `mjOBJ_GEOM`(5)，非 geom 类型；真实命中 `geom_id 3` = `geom_type=7`(MESH)、`mat 0` = mat-red（`mat_rgba=[0.9,0.1,0.1,1]`、`mat_texid_row` 一致）；并把 diag 的 `geom_type` 命名误读列入 errata ✅
2. **方法与 depth 校验**：dump sha `68b3f78e…`/51,388 B、G-L1 参考、model-315、pinned upstream `71da24d9`、warp 头；f32 重放 + `depth = t×(−dir.z)` 与 dump 位型比对（2/5 相同、3/5 差 1 ULP，属重放收缩序；产品/参考 depth 位型本身全等）✅
3. **唯一站点两侧对照**：上游 `normalize(mat @ cross)`（`ray.py:727-729`）vs 我方 `normalize(mat @ normalize(cross))`（`render_math.cu:239` + `render_common.cu:200`）✅
4. **5 px 机理链**：face 2、顶点法线 dot 0.4664/1.0000/0.5726 → 2 拒 1 留、pre-normal 1 ULP → 着色法线 1 ULP → 高光 `pow(ndoth,76.8)` 放大 → rgb 量化边界翻 1 字节；解释「只 mesh / 只 cam1 / G 通道也翻 / depth-seg 全等 / legacy 全绿」✅
5. **H1 否证**：差异可逐行指出；helper 收缩序可把 depth 复现到 ≤1 ULP；纯编译链差异无法解释只 mesh 链 ✅
6. **修复方向与影响面**：一行级改法；内核身份变 ⇒ 重冻 12 exe + 新 PTX 基线 + 全窗重跑；判据/参考/spec/签名材料不变 ✅
7. **风险登记**：woop 未实现上游 `U==0||V==0||W==0` double 重算分支（已在源码注释登记）✅
8. **非声明**：不宣称 G-L2 通过/判据满足/上游等价/渲染数值 ✅

## 2. artifacts 与数字一致

- `pixel-dump-excerpt.json`：**5 条 rgb 失配全量**（w0/cam1 px 13268(148,41)、31154(114,97)；w1/cam1 px 27360(160,85)、28614(134,89)、39492(132,123)），产品/参考字节与 README 表逐字节相同；6 个 frame 的 depth/seg 统计全 0（`depth_mismatches`/`seg_mismatches` 为空），与 README「depth/seg 全等」一致 ✅
- `replay-output.txt`：5 px 全 hit **face 2**，depth 2/5 位型相同 + 3/5 差 1 ULP，pre-normal ULP `[0,1,1]`，final-normal ULP ≤1，顶点法线 dot 与 README 相同 ✅
- `replay_mesh_normal.py`：纯 `json/struct/pathlib/numpy`，f32 模拟，只读 dump/npz/model，**不写文件、无 GPU**（静态可核）✅

## 3. 边界 / 门禁 / 卫生

- `63cf7cc0` 仅改 RCA 卡 JSON + 新增证据目录（README/receipt/manifest + 3 artifacts）+ 4 门禁 runs；`src/`/`tests/`/`fixtures/`/`scripts/` **各 0 diff**；G-L2 与诊断卡证据目录 0 ✅
- manifest **5/5 独立重算全对**（README `23d592a1…`/9,084 B 等）；四门禁 runs 全 `passed=true`，unit **1387/0/402** ✅

## 4. 观察项（信息级）

- **O1：「唯一站点」的适用范围**。站点对照在**渲染 megakernel 与 depth/seg 两条消费链**上成立（`mesh_query_ray` 全仓仅 `render_common.cu:194`、`render.cu:225` 两处调用，已逐一核对）。但另有**同构站点** `src/render/bvh/cuda/mesh.cu`（`mesh_bvh_ray`）：L290 `*out_normal = normalize3(min_normal)` + L357 `normal = normalize3(mat_vec(xmat, n_local))`，与本次 RCA 的「多归一化一次」同类；该入口是公开 API（`render::bvh::MeshBvhQuery`），并有自有回归 `render_mesh_bvh`（G-L2 中 exit 0）。建议把「唯一站点」限定为「G-L2 判据 A 渲染链上唯一站点」，并把 mesh_bvh_ray 同构点登记为后续 H2-class 复查项（不阻断 RCA 结论）。

## 5. 非声明

本复核只核 RCA 记录的如实性；不宣称修复生效、不宣称 G-L2 通过或判据满足、不宣称与上游等价；不改任何判据/阈值/参考/相机/recipe/冻结件。