# R06 柔体渲染家族收口（plan.json 九片标 delivered + 家族汇总）

任务：`2026-11-04-render-r06-flex-closeout`（owner Arch）。日期：2026-11-04（空闲段）。
**边界：证据与排程收口，零 GPU、零 `src/`/`tests/`/`fixtures/` 改动、不改判据/容差/参考、不回改已 verified 证据。**

## 1. 一句话结论

R06 柔体渲染家族 S0 预检 → S3c 全部 verified；验收判定以 **attempt6 单次设备验证窗（VALID_ROUND）** 为准——
**除用户签名 allowlist 的 1 个像素外全部 bit-first**。本次收口只做证据归纳与 plan.json 记账，不新增判定。

**写死口径：单帧位级成立；多帧演化未覆盖。**
即：同一 GPU / 同一 recipe / 同一编译形态下，产品 flex 渲染对冻结 `scene-flex` 参考的单帧输出在内层判据内；
「相邻时钟步位姿变化 → 渲染随之变化且与参考一致」的多帧演化**没有判据也没有参考**，本家族不作任何动画一致性声明。
多帧缺口登记为跟踪卡 `2026-11-04-render-r06-flex-multiframe`（backlog，不排期、不开窗）。

## 2. 家族交付面（plan.json `slices` 新增九片）

| 片 | 内容 | 实现卡 | 关键提交 | 证据 |
| --- | --- | --- | --- | --- |
| R06-S0 | 预检：81 槽 flex 形参对表 + authoring 环境恢复 | `2026-11-03-render-r06-flex-precheck` | `7e8ec8c8` | `docs/development-evidence/render-bvh/r06-flex-precheck/` |
| R06-S1a | 零行为结构拆分 `megakernel_args.rs` | `2026-11-03-render-r06-s1a-megakernel-args-split` | `0ac566a5` | 提交 + 门禁记录（无独立证据目录） |
| R06-S2 | 宿主通道：9 数组 + `bvh_nflexgeom` + 两处 2-D 形状 | `2026-11-03-render-r06-s2-flex-host-channel` | `7c0bb8f9` | `……/r06-s2-flex-host-channel/` |
| R06-S2b | 数据源：原生 additive（32 B / 72 B）+ 只读视图 + T4 身份重钉 | `2026-11-04-render-r06-s2b-flex-datasource-t4-merged` | `1f199737`、`88e19c44` | `……/r06-flex-datasource/` |
| R06-S2c | shell/elem 数据层 + 拓扑校验入口顺序修复 | `2026-11-04-render-r06-s2c-flex-shell-data` | `df6d74af`、`b4a7e827` | `……/r06-s2c-flex-shell/` |
| R06-S3pre | `FlexHostInputs` 组装 + `render_with_flex` 真实调用方 | `2026-11-04-render-r06-s3pre-flexhost-wiring` | `0b82c676` | `……/r06-s3pre-flexhost-wiring/` |
| R06-S3a | 设备 1-D 命中链（`render_flex.cu` cast/BVH/seg） | `2026-11-04-render-r06-s3a-flex-device` | `544d02b9`、`735dcc6f` | `……/r06-s3a-flex-device/` |
| R06-S3b | 参考采集 + 设备验证窗（attempt6）+ 收口 | `2026-11-04-render-r06-s3b-closure` | `71ec7c83`、`d35cab68`、`3406f1ca`、`7a48205d`、`db082561`、`8aec9619` | `……/r06-s3b-flex-validate/`、`……/r06-s3b-closure/` |
| R06-S3c | 2-D/3-D flex 设备接线（mesh BVH 入 MeshDevice blob） | `2026-11-04-render-r06-s3c-flex-mesh-device` | `181eaa7a`、`e9e198cd`、`ddd06292`、`5d52ebeb` | `……/r06-s3c-flex-mesh-device/` |

## 3. 验收判据（沿用已签口径，不重议）

- **主体**：`rgb` / `depth` / `seg` 对冻结参考 **bit-first**（rgb ≤1 LSB 且 beyond_1_lsb=0；depth ≤4 ULP 且 beyond4=0；seg 精确相等）。
- **唯一放宽（用户 2026-11-04 22:14 签名，`3406f1ca`）**：仅 `cam2 / (x=134,y=128) / flexid0` 允许 `ULP ≤ 4` **且 `beyond4 = 0`**；
  `beyond4 ≠ 0` 即失败，不得以「仍在 4 ULP 内」放行；范围不扩大、不写成一般容差。
- **与 D8 分开登记**：D8（`a1cc22ae`，cam2 321 px、beyond4 297、maxULP 259）是 f32 收缩序修复前的缺口诊断；
  `d35cab68` 后收敛到 1 px。两组数字**不混算、不叠加**。
- **B 类前提**：冻结参考 6 文件、同 GPU（RTX 4070 Ti SUPER / driver 596.36 / SM89）、同 recipe、同编译形态（attempt6 窗口内 canonical rebuild 产物）。
  任一前提失效 → 需重跑重判，本汇总不自动继承。
- **跨轮明细闭环**：`git diff d35cab68..8c15ad75 -- src/` 为空（渲染器源码未变）；attempt6 两跑 sha 与 r2/D8 采集轮逐位相同；
  该 px 明细 r2 已有（bits `0x400015ba` vs 参考 `0x400015b7`、ULP=3、beyond4=0、seg 双方 `[0,9]`）。

## 4. 验证窗六项（attempt6，`7a48205d`）

| 项 | 数字 |
| --- | --- |
| 窗口规则 | 一次 canonical rebuild（强制 clean 后 14.25 s）→ 同刻记录指纹 → `freeze -Attempt 2` → 冻结后零编译 |
| 冻结指纹 | test exe `e48c7268…` / **107,854,859 B**；产品 bin `c5014d83…` / 85,041,741 B |
| 守卫 | `GUARDS_OK=16/16`（2 exes + 14 inputs）、`attempt=2 frozen`、aggregate_all `c5263d33…`、aggregate_inputs `ec6fa222…` |
| monitor | 60/60 连续干净，util 2–7%，free_min 14,414 ≥ floor 11,688 |
| 实体体 | `running 1 test → 1 passed`（1.41 s），17 filtered out 属正常过滤 |
| 两跑 sha | rgb `d84ecaed…`、depth `98292153…`、seg `c4be9853…`，两跑相同 |
| INVALID 检查 | 冻结后零编译/零 relink，`attempt=2 status=frozen`，live/stash 2/2、inputs 14/14 |

## 5. 未覆盖 / 已知限制（如实登记）

1. **多帧演化未覆盖**（本汇总写死；跟踪卡 `2026-11-04-render-r06-flex-multiframe`，不排期）。
2. **flex `z2vec` 近平行负方向分支潜伏缺陷**：产品 `quat[3]` 未清，当前路径未触发；跟踪卡 `2026-11-04-render-r06-flex-z2vec-parallel-branch`（p40）。
3. **flex-vs-flex 容差内平局语义**：未改引擎语义（`min-d` 偏离方案已被否）；r4 占位 `2026-11-04-render-r06-s3b-flex-tie-order-r4` 仅在用户改选时启用。
4. **attempt5 FAILED_ROUND**（`6e9aa585`）定性为**判据实现缺口**（用例断言严于签名 allowlist），不是数值红；证据保留，不重试。
5. 运行时逐相机/allowlist 明细打印缺口已由 test-only 只读增强 `a0118271` 补齐（零 GPU）。
6. **不覆盖**：上游完全等价、Linux/T4、性能、更高 spp、多 world 下的 flex（单 world 家族内验证）。

## 6. 非声明

- 不声明「动画与几何一致性」整体成立（只有单帧位级证据）；
- 不声明与上游完全等价、不声明 parity 通过；
- 不声明 flex 渲染整体可用、不声明对外发布依据；
- 不声明 R06 家族之外的任何渲染能力。

## 7. plan.json 记账

- `slices` 新增 `R06-S0/S1a/S2/S2b/S2c/S3pre/S3a/S3b/S3c` 九条（`status=delivered` + `delivered_commits` + `delivered_note`）。
- `delivery_tracking.rows` 新增 10 行（九片 + S3b QA 复核行），`updated_at` 推进到 2026-11-04。
- `scheduling_notes` 新增 `N14`（家族交付面、窗口六项、签名 allowlist、D8 分开、已知限制、非声明）。
- **`roadmap-r06-p8` 保持 `proposed` 不关**：P8 家族节点由 `roadmap-r01-p8` 门控，roadmap 节点不承载实现。

## 8. 复跑与引用

本汇总不含新数字，全部引用已 verified 证据（见 §2 表）。复算入口：

```powershell
# 签名/判据文本
.\docs\development-evidence\render-bvh\r06-s3b-closure\README.md
# attempt6 原始记录
.\docs\development-evidence\render-bvh\r06-s3b-flex-attempt6\README.md
# 家族证据目录
Get-ChildItem .\docs\development-evidence\render-bvh\r06-*
```
