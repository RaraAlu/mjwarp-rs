# G-L2 实体：两跑 sha 打印补齐（测试侧 println；断言/判据零改动；零 GPU）

- 任务：`2026-11-11-render-perworld-l-v2-two-run-sha-print`（owner Backend；p75；P8-渲染与BVH）。
  来源：attempt 4 canonical 窗口归档的 backlog R1——「两跑 `first_sha`/`second_sha` 只有 `assert_eq!`、`--nocapture`
  日志不给具体 sha 值」；另按 Chief 要求确认 B/world 隔离/C 数值的打印覆盖。
- 边界：**只改测试源（打印）与证据**——断言/判据/阈值/参考/相机/recipe 零改动；无 `.cu`、无 `src/` ⇒ 内核身份不变；
  零 GPU、零窗口、不重冻 guards。

## 1. 改动（`tests/render_per_world_light.rs`，+3 行）

仅在两跑 sha 断言前补打印（`sha256_frames` 返回 `[String; 3]`，元素为 `{:x}` 的 rgb/depth/seg 十六进制串）：

```diff
     let second_sha = light::sha256_frames(&light::rendered_frames(&context, &resolved));
+    // backlog R1：attempt-4 日志只有断言、无具体 sha 值；这里把两跑十六进制值直接落日志（`--nocapture` 可读）。
+    println!("two_run_sha first={first_sha:?} second={second_sha:?}");
+    println!("two_run_sha equal={}", first_sha == second_sha);
     assert_eq!(
         first_sha, second_sha,
         "两跑 rgb/depth/seg 整块 sha 必须相等"
     );
```

- 文件 494 → **497** 物理行（< 500 棘轮）；`git diff` 仅 3 个新增行、0 删除/修改；断言与逻辑逐行未动。
- 完整 diff：`logs/diff-stat.txt`。

## 2. 打印覆盖审计（`logs/prints-audit.txt`）

权威运行样本 = attempt 4 canonical 窗口的实体 stdout（已归档，未重跑）：
`perworld-l-v2-validate-window-g-l2/attempts/attempt4/window/run/exe-logs/render_per_world_light.stdout.log`
（`9f132559…`/1,837 B）。逐项核对：

| 项 | 现状（attempt4 stdout） | 本卡动作 |
| --- | --- | --- |
| C 数值 + verdict | `c1_rgb_changed=[38285, 40644] c2_depth_changed=[35828, 33883] c2_seg_changed=[16041, 16754] verdict=PassMargin` ✅ | 确认，不改 |
| C diag / capture 交叉核对 | `diag campersp depth=[0, 0] seg=[0, 0]; camhinge rgb=[50363, 52329]`、`capture.json C1 交叉核对 = Compared` ✅ | 确认，不改 |
| A 逐帧统计 | 6 × `A world<W> cam<C>: FrameStats { ... }` ✅ | 确认，不改 |
| B `LightStats` 全字段 | `B: LightStats { worlds: 3, lights: 3, xpos_mismatched: 0, xdir_mismatched: 0 }`（`light.rs:278` `derive(Debug)` 含全部 4 字段）✅ | 确认，不改 |
| world 隔离矩阵 | `isolation changed=[[false, false], [true, true], [false, false]]` ✅ | 确认，不改 |
| **两跑 sha 具体值** | **缺失**（只有 `assert_eq!`） | **本卡新增 `two_run_sha first=…/second=…/equal=…`** |
| pilot C 数值（`#[ignore]` 人工用例） | `pilot C1=[38285, 40644] C2 depth=[35828, 33883] seg=[16041, 16754]` ✅ | 确认，不改 |

## 3. 预期日志形态（下一渲染窗口 `--nocapture` 直接可见）

```text
two_run_sha first=["<rgb sha256 hex>", "<depth sha256 hex>", "<seg sha256 hex>"] second=[...] 
two_run_sha equal=true
```

- 本卡**未执行** `#[ignore]` 实体用例（需要 GPU 驱动 + G-L1 采集参考，属渲染 GPU 窗口），因此只验证到「编译通过 +
  源码格式确定」；真实 sha 数值与 `equal=true` 待下一次渲染窗口观测（届时日志与 attempt4 一样可直接引用具体值）。
- 这是**打印补齐**，不改变任何断言：两跑 sha 相等与否的判定仍只由既有 `assert_eq!` 承担。

## 4. 测试 exe 身份与重冻含义（`logs/identity-measurements.txt`）

| 项 | sha256 | bytes |
| --- | --- | --- |
| attempt-4 冻结实体 exe（窗口基线，**已 drift**：P10/z2vec 后源码变化） | `73245bbf175bf10c3332b6f3180ae6bc600a6db5fadb55e873ebfdd69a0d2542` | 92,590,602 |
| 本卡前（HEAD 源、逆暂存重建，warm cache） | `6908c5a2bce75ec44d9f92bab64895020efe57679b85e82f3970a8d74bf5147d` | 92,630,818 |
| **本卡后（最终 on-disk）** | **`55d0c32ceaa8d87ec8183637281f79ac7eaff42166831a55ca587ad60f1ee354`** | 92,631,489 |

- 归因干净：先 `git stash` 掉本卡改动重建 HEAD 源基线，再恢复重建，delta = **+671 B**（= 1 条注释 + 2 条 println 的
  字符串/代码增量），不是 z2vec 等库变化。
- **测量注意（观察项）**：本环境同源重复构建的 exe sha 不逐字节可复现（3 次同源构建得到 `4ad790c0…`/`198aa439…`/
  `55d0c32c…`，bytes 相同）；因此 exe 身份以**冻结产物自身**为准，不能靠重建复现（不影响既有窗口有效性口径，
  但记入本证据供后续冻结流程参考）。
- **重冻含义**：`render_per_world_light` 是 12 个 guards 输入之一，其身份已变 ⇒ **后续渲染 GPU 窗口必须重新冻结
  guards/exe**；本卡不重冻、不占窗。guards JSON `94478b3d…`/43,729 B 本次只读未变。

## 5. 验证

- 构建：`cargo test --locked --features cuda-probe --test render_per_world_light --no-run` ⇒ exit 0。
- 四门禁 fmt/clippy/clippy-cuda/unit（unit 1,406 过 / 0 败 / 402 ignored，与卡 C 后基线一致）；`normify check` → done。
- 断言零改动证据：`git diff` 仅 +3 行（comment + 2 println），无删除/修改；判据/阈值/参考/相机/recipe 未触碰。
- 边界：`unchanged src/`、`fixtures/`、`scripts/`；`git diff -- '*.cu'` 空；内核身份不变。

## 6. 非声明

不宣称设备端行为变化/渲染数值变化/等价；本卡未执行实体用例（需 GPU 窗口），真实 sha 值待下一窗口观测；
不改判据/参考/容差/相机/recipe；不重冻 guards、不占窗口。