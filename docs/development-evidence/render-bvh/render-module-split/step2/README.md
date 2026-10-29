# mjwarp.render 结构反转 · 步 2（53 源归位，容器只留 `mod.rs` 门面）

任务：`2026-10-30-render-module-split-step2`（Arch；P8；案 2 步 2）
前置：`2026-10-30-render-module-split`（步 1，verified；QA verdict pass）
基线：`revision.before = 996188fbb2f1fcf9f3e31d15af9dd70e5e4352dd`

本片**纯搬迁**：只改 module md 的 `[[source]]` 归属，磁盘文件零移动；`src/**`、`tests/**`、`fixtures/**`、`Cargo.*` 零改动；**API / deps 一条未动**；`.normify/layouts/mjwarp/render.toml` 未改（3 个直接子模块不变）。

## 1. 源分配（53 源全部有主）

| 目标 | 源数 | 内容 | LF 行数 |
| --- | --- | --- | --- |
| `mjwarp.render.kernel` | **22** | `src/render/kernel/**` 19 + `camera.rs` + `rays.rs` + `camera/cuda/build_rays.cu` | **6,945** |
| `mjwarp.render.bvh` | **11** | `src/render/bvh/**` | **3,662** |
| `mjwarp.render.context` | **19** | `src/render/context*` 16 + `texture.rs` + `texture/cuda.rs` + `texture/sample.cu` | **5,086** |
| `mjwarp.render`（容器） | **1** | `src/render/mod.rs`（模块门面） | **385** |
| 合计 | **53** | — | **16,078** |

**行数口径（QA O2）**：主表用 **LF 行数**（`ReadAllLines`/按 `\n` 计），合计 **16,078**；交叉口径 **Σ(LF+1) = 16,131**（差 **53** = 文件数），与步 1 README 的 16,131 对齐——两个口径都对，差别只是「文件末尾换行是否计入」。

归属理由：`rays.rs` 以 `include_str!("camera/cuda/build_rays.cu")` + `SynchronousKernel` 发射射线 ⇒ camera 三件归 kernel；`texture/cuda.rs` 以 `include_str!("sample.cu")` 建纹理对象/采样 ⇒ texture 三件归 context；`mod.rs` 是模块门面（`pub mod bvh/camera/context/kernel/rays/texture` + 跨叶 re-export），不属任一叶 ⇒ 留容器。

## 2. 口径更新与偏差（三条，按 Chief 要求写死）

1. **口径更新**：步 1 README 写的「步 2 清零（容器剩 7 源）」由本片更新为「**容器保留 1 个门面源 `src/render/mod.rs`**」。先例：根容器 `mjwarp` 持 `src/lib.rs` 等 8 个源、`mjwarp.runtime` 容器持 `src/runtime/mod.rs` 等。
2. **API 归属 ≠ 源码位置**：`mod.rs` 留容器 ⇒ `OutputChannels` / `ImageOutput` / `ImageBatchLayout` 三个 API 仍在 `mjwarp.render.context` 叶声明、实现符号在容器源文件。**本片不消除、只点名**；API/deps 一条未动（容器 0 API / 0 依赖，三叶 1/2/8 API 与 4/0/2 依赖与步 1 相同）。
3. **纯搬迁**：本片只改 `[[source]]` 归属；`git diff --stat -- src` 为空（实测 0 行）；卡面 `unchanged(paths=["src/","tests/","fixtures/"])` 机器检查通过。

## 3. 机器断言（实测）

| 断言 | 结果 | 证据 |
| --- | --- | --- |
| 路径集合：4 个模块 == 基线 53 条 | **total=53 / distinct=53 / duplicate=0 / missing=0 / extra=0 / set_equal=true** | `materials/verify-pathset.txt` |
| 纯搬迁（本任务未动受保护路径） | `unchanged` 检查通过；`git diff --stat -- src` 为空 | 卡面检查 + 本文 §2.3 |
| validate | 错误 **0**、警告 **5** 行、`structure/leaf-too-coarse` 实测 **39** | `logs/validate-final.txt` |
| 计数不变 | 模块 **71** / 叶子 **63** / API 键 **398**（重复 0）/ deps **6** | 本片实测 |
| 布局 | `.normify/layouts/mjwarp/render.toml` 未改（3 子模块） | `git diff` 无该文件 |

中间态（`in_progress`）另有一条 `evidence/fingerprint-drift 4 个模块指纹已过期`：三叶指纹为 64 位 hex 占位（先写占位才能 `module source --add`），`done` 刷新真值后消失。

## 4. impact 落位（实测）

| 路径 | 模块 |
| --- | --- |
| `src/render/kernel/megakernel.rs` | `mjwarp.render.kernel`（22 源/1 API） |
| `src/render/bvh/refit.rs` | `mjwarp.render.bvh`（11 源/2 API） |
| `src/render/context/lifecycle.rs` | `mjwarp.render.context`（19 源/8 API） |
| `src/render/camera.rs` | `mjwarp.render.kernel` |
| `src/render/texture.rs` | `mjwarp.render.context` |
| `src/render/mod.rs` | `mjwarp.render`（容器，1 源/0 API） |

## 5. 门禁

fmt / clippy / clippy-cuda / unit 全过（unit 1106 过 / 0 败）；`cargo test --locked --no-run` exit=0 且日志含 `Finished`。日志见 `logs/gate-*.txt`、`logs/no-run.txt`。

## 6. 未做

- 不宣称「架构完成」或「全部模块已拆分」：`mod.rs` 仍属容器门面，三个 API 的「归属≠源码位置」偏差保留（如需消除要另立片决定 `mod.rs` 归属）。
- 不改任何判据/容差/参考；不跑 GPU 窗口。
