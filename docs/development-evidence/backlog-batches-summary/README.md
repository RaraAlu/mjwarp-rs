# 批次 1–4 跨批次收口审计与汇总

任务：`2026-10-23-backlog-batches-closure-audit`。日期：2026-10-23。执行：Backend。
性质：**只读证据/文档审计**（不改产品源码、测试、容差、fixtures，不 start 任何路线图任务）。
机读版本：同目录 `summary.json`。

## 1. 范围与方法

批次 1–4 按 Chief 派单的 ready 路线图任务逐条交付，共 **20 条交付 + 4 条顺延**。
本审计读取：20 份交付回执（`docs/development-evidence/changes/<task-id>.json`）、
四批证据目录（`backlog-batch1/2/3/4`）、`.normify/changes/<task-id>.json` 任务状态与
`.normify/runs/<task-id>/` 门禁记录，做目录/回执/门禁三方一致性校验，产出汇总。

## 2. 批次统计

| 批次 | 证据目录 | 交付 | 顺延 |
| --- | --- | --- | --- |
| 批次 1 | `docs/development-evidence/backlog-batch1/` | 8 | 0 |
| 批次 2 | `docs/development-evidence/backlog-batch2/` | 7 | 1（r01-p8） |
| 批次 3 | `docs/development-evidence/backlog-batch3/` | 4 | 1（a10-p11） |
| 批次 4 | `docs/development-evidence/backlog-batch4/` | 1 | 2（g10-p5、m10-p10） |
| **合计** | | **20** | **4** |

## 3. 已交付 20 条（提交号 / 登记号 / 核心验证数字）

证据目录均为 `<批次目录>/<task-id>/`（README + authoring.json；个别见回执）。

### 批次 1（G27/M08/G05/G28，8 条）

| 任务 | 提交 / 登记 | 核心验证数字 |
| --- | --- | --- |
| `g27-p2` 固定常量 | `70b36b7` / `2354645` | 逐元素位型一致（0 位差，上游 5 次运行同一位型）；原生 f64 2.98e-07；独立 f64 复算 0 |
| `m08-p3` 世界掩码局部重置 | `e63b296` / `1309506` | 7/7 用例 time/qpos/qvel 位型一致（容差 0）；原生 0；掩码隔离与关键帧越界跳过 |
| `g05-p4` 执行器力 | `9850ee0` / `f421045` | act_dot/actuator_force/qfrc_actuator 位型一致；原生 9.54e-08；限幅与非法类型拒绝 |
| `g28-p7` 原语射线查询 | `8764ced` / `2d8d35a` | geomid 9/9 一致；距离/法线 ≤1e-6；原生 1.68e-07/1.83e-07；六种原语覆盖 |
| `g27-p7` qpos0 依赖常量 | `a1fa5b5` / `383aaa0` | 两案例×8 组常量位型一致；原生 1.19e-07；动态重算匹配上游 |
| `m08-p7` 扩展重置 | `b92e55d` / `2fd6bf2` | 6 用例×9 字段位型一致；原生 0；掩码/关键帧/mocap 语义断言 |
| `g05-p7` 关节传动 | `e7a7859` / `f8dfef3` | transmission 逐位一致；执行器力链逐位一致；原生 9.54e-08 |
| `g28-p8` 批量射线 | `00b8b60` / `1ed0f7a` | 2 世界×5 射线 geomid 一致；原生 1.19e-08；无命中 -1/零法线 |

### 批次 2（G07/G16/G20/G21/G23/A08/A10，7 条）

| 任务 | 提交 / 登记 | 核心验证数字 |
| --- | --- | --- |
| `g07-p5` 宽相碰撞 | `12151dc` / `5820fde` | 47 用例候选多集合逐案相等；NXN/SAP 集合一致；原生 27/27 包含、边距 6/6 |
| `g16-p6` Euler/RK4 | `9692c92` / `2ad0475` | euler 64/72、RK4 67/72 位型，max abs 5.96e-08；原生 1.19e-07/6.26e-06；一/四阶收敛 |
| `g20-p7` 逆向动力学 | `a10aea5` / `074a736` | qfrc_inverse vs 上游 1.91e-06；原生 2.62e-06；正逆回代 2.62e-06（≤1e-5） |
| `g21-p7` 平滑速度导数 | `cef3357` / `90736a7` | 18/18 逐位一致（0）；差分 1.55e-06 / 原生 4.67e-08 |
| `g23-p7` 传感器 | `34bbd05` / `0ba71e6` | sensordata 1.19e-07；原生 5.96e-08；cutoff 与阶段单元断言 |
| `a08-p10` 零拷贝 | `55a6cb8` / `68f5529` | 视图指针 == 原缓冲 + offset；跨流/未完成/代数/越界 5 类拒绝；5 测试 |
| `a10-p10` 记录与性能工具 | `3757567` / `e84620b` | 阶段聚合/无头计时/状态与报告 JSON 往返 + 批次 2 API 复核；5 测试 |

### 批次 3（G08/G18/G24/M10，4 条）

| 任务 | 提交 / 登记 | 核心验证数字 |
| --- | --- | --- |
| `g08-p5` 基本几何窄相 | `2269b04` / `6ebe9ac` | 候选 5 条一致；接触数 5=上游=native 之和；字段 7.45e-09、原生 5.96e-08 |
| `g18-p6` 离散积分 | `9239fa5` / `88b3593` | 单步位型 18/22、max abs 5.96e-08；原生 5.96e-08；与 euler 分支逐位等价 |
| `g24-p7` 历史与延迟 | `9cdf5de` / `9d9e3e5` | 4 场景缓冲字逐位一致；51 读取值逐位一致（max abs 0）；重置/原生 2.68e-09 |
| `m10-p7` Rust 回调契约 | `b9cd236` / `c315f49` | 7 槽位/Stage 1/2/3 与冻结上游一致；时序/门控/写入字段断言；7 测试 |

### 批次 4（G09，1 条）

| 任务 | 提交 / 登记 | 核心验证数字 |
| --- | --- | --- |
| `g09-p5` 凸体（plane-box 多接触） | `c757e3d` / `18b3fb5` | 15 接触 vs 上游 5.96e-08、原生 1.19e-07；接触数 4/2/1/0/4/4/0 逐世界一致 |

## 4. 顺延 4 条（均保持 proposed、不 start、无交付声明）

| 任务 | 裁决 | 阻塞项 | 恢复条件 |
| --- | --- | --- | --- |
| `r01-p8` 渲染上下文 | A 顺延 | 渲染/BVH 子系统缺失（BVH 构建/重拟合、资产上传、CUDA 渲染内核、相机/采样参数） | 渲染能力具备后（批次3或独立渲染计划）重评 |
| `a10-p11` 记录与性能 P11 | A 顺延 | Linux RTX5080 访问（Linux deferred）、真实 GPU 事件计时/显存统计、清洁部署、T4+Linux 双平台准入；P10 宿主侧已覆盖 | Linux 与完整 GPU 引擎具备后重评 |
| `g10-p5` 地形与 SDF | A 顺延 | 高度场/SDF 资产管线与碰撞实现缺失（src 无 hfield/SDF/mesh；上游 hfield→CONVEX/GJK、SDF→collision_sdf） | 地形资产管线具备后重评，或与 r01 合并 |
| `m10-p10` Rust 回调 P10 | A 顺延 | P10 图重放/缓存失效/性能依赖 forward/step 或图集成；上游无可冻结 callback-graph 语义 | 图重放能力具备后重评，或与 m10-p7 合并收口 |

## 5. 一致性校验结论（本次实测）

- 四批证据目录数量与索引一致：batch1=8、batch2=7、batch3=4、batch4=1；无错位。
- 20 条交付：状态均 `verified`；证据目录存在；回执存在且 `/task_id` 一致；回执内
  证据路径均指向本批目录且文件存在。
- 20 条交付的 fmt/clippy/unit 门禁记录在 `.normify/runs/<task-id>/` 中 **passed**（exit=0）。
- 4 条顺延：任务保持 `proposed`、无证据目录、无交付回执；note 含顺延/裁决 A/阻塞项
  与恢复条件（r01-p8 以「顺延到渲染能力具备后」表述）。
- **unit 数字口径**：`.normify/gates.toml [gate.unit] = cargo test --locked`（全目标）。
  每任务记录为其最近一次门禁值（部分任务因回执修正重跑，数值可能晚于交付时点）；
  收口实测 **478 过 0 败**。不得用 `--lib` 单独数字替代门禁口径。
- 批次 4 复核两项「一般」问题已修复并留痕：g09-p5 退化尺寸校验落空、回执 model_sha256 登记错误，由小修任务 `2026-10-23-g09-validation-gap-fix`（`1388124`）修复，g09 以 `normify fix`（`6c6f798`）记录；修复后数字不变。`n- `normify validate`：错误 0（1 条既有结构告警）；工作区干净。

## 6. Windows 范围内可执行 roadmap 项已清空

按当前引擎与平台能力，**Windows 范围内可执行的 roadmap 项已清空**：批次 1–4 交付完
20 条 ready 路线图任务后，剩余 ready 项均为顺延/保留项。剩余工作依赖：
1. **缺失子系统**：渲染/BVH（r01）、地形/SDF 资产管线（g10）、图重放/缓存失效（m10-p10）；
2. **性能平台**：真实 GPU 事件计时/显存统计与 T4+Linux 双平台准入（a10-p11、NP04）；
3. **用户裁决**：W13 nvJitLink 许可/部署、Linux deferred 平台。

## 7. 复核发现并已修（批次 4）`n`n- **退化尺寸校验落空**：g09 acceptance 要求退化输入显式拒绝，原窄相只校验数组长度。已补 `narrowphase/validate.rs` 的 `validate_geom_sizes` 与 `InvalidGeomSize`，覆盖 plane/sphere/box 的 0/负/NaN/±Inf 拒绝测试与 box-box 候选拒绝。`n- **回执 sha256 登记错误**：g09 回执 `model_sha256` 误填 git blob `63743a8a…`，已改为文件 sha256 `a78b1547…`。`n- 修复任务 `2026-10-23-g09-validation-gap-fix`（`1388124`），g09 经 `6c6f798` 追加修正记录；15 接触/4-2-1-0/5.960e-08/1.192e-07 数字不变。`n`n## 8. 跨批次未验收边界

- 所有交付均为「当前代码布局下最小可交付切片」，均未宣称完整矩阵族验收；
  各任务 README 的「未验收边界」为权威清单。
- 物理族余量：G08 其余几何对/CCD/约束、G18 有效度量与约束求解、G24 三次插值与
  forward 调度、M10 回调执行与阶段调度、G09 box-box/GJK/EPA/mesh/CCD。
- 平台/性能：GPU 内核与图执行、真实批次性能门槛与显存统计（NP04）、
  Linux/T4 双平台验收未做。
- 精度：已交付路径全链 f32；但完整精度硬指标（NP01–NP04 全量）仍未验收。

## 9. 复现

```powershell
# 批次目录与回执抽查
Get-ChildItem docs/development-evidence/backlog-batch1,backlog-batch2,backlog-batch3,backlog-batch4 -Directory
Get-ChildItem docs/development-evidence/changes | Measure-Object
# 门禁记录抽查（passed=true）
Get-Content .normify/runs/2026-10-11-roadmap-g09-p5/unit.json
# 机读汇总
Get-Content docs/development-evidence/backlog-batches-summary/summary.json -Raw
```

审计脚本仅离线只读使用，未纳入产品构建/测试/门禁；产品测试不启动 Python。
