# R09-S2 复核观察收口（文档面）+ 登记 R09-S2b 解锁片

任务：`2026-10-28-render-bvh-r09-s2-obs-closure`（P8 渲染与 BVH；纯文档治理片，无 GPU 窗口）
来源：QA 独立复核 `2026-10-28-render-bvh-r09-s2-check`（verdict **pass / 阻断 0 / 观察 8**；提交 `023eafdd` + `66c0fa43`）
被复核实现：`6b5d6dc6` + `1ad2f268`（R09-S2 内核光照阴影实现）
本片口径：**不改**代码、判据、容差、参考、夹具、窗口数字；不重开窗口、不重跑 GPU。

## 1. 8 条观察的处置表

| # | 级 | 观察 | 处置 | 落地位置 |
| --- | --- | --- | --- | --- |
| O1 | 严重 | 阴影分支在产品路径**暂不可达**：`megakernel.rs:97-99` 仍前置拒绝 `use_shadows=true`（S7b3 既有策略，本片未改）、第 40 槽恒 false；两条既有 `#[ignore]` GPU 用例（`render_kernel_flags.rs` L412-435、`render_kernel_rgb.rs` L209-224）仍以「拒绝」为契约 | **不阻断 + 留痕 + 后续片** | 实现片 README §6 / receipt `host_guard_prerequisite`；新卡 `2026-10-28-render-bvh-r09-s2-host-guard-unlock` |
| O2 | 一般 | `tests/render_lighting_kernel.rs` 实测 144 行，证据写 145 | **fix** | 实现片 README §3 与 receipt `changes[].added` 改 144 |
| O3 | 一般 | 5 条单测偏结构/字符串级；接线（`shading_words()→scale_constants[2]`、第 40 槽取值）无 CPU 测试 | **accept 延后** | 并入 `…-r09-s2-host-guard-unlock` 片（新增 in-crate 单测） |
| O4 | 建议 | `render_megakernel.cu:15` 头注释仍写 `[znear, zfar, unused]` | **accept 延后** | 随 unlock 片改（改 `.cu` 会变 `MEGAKERNEL_SOURCE`/PTX sha，与已冻证据面冲突） |
| O5 | 建议 | 实现片 manifest 混用相对基准（证据目录 / 仓库根） | **accept + 规则化** | 实现片 manifest 头注写明解析顺序与基准规则 |
| O6 | 建议 | `tests-and-lint.log` 保留 rustfmt 修前中间态 | **accept** | 历史日志，最终树 fmt 通过（QA 复跑）；不改日志 |
| O7 | 建议 | 卡面 acceptance#1「拆分核验」本片无拆分对象 | **accept** | 实现片 README §6 补「本片无拆分/移动对象；贴线文件零改动」 |
| O8 | 建议 | 实现片 receipt `head=5cc6a3d5` 是证据时点 HEAD（提交前），非最终树绑定 | **accept + 字段口径** | receipt 增 `head_note` 说明语义 |

## 2. O1 的排程结论（本片最重要的一条）

R09-S2 的卡面契约是「内核实现 + 单测 + 零数值结论」，交付的是 **分支实现就位、产品路径待解锁**，不是「阴影已可用」。但要让 R09-S3b 验证窗真正从产品入口发射阴影，必须先解锁：

1. 移除 `megakernel.rs` 的 `use_shadows=true` 前置拒绝；
2. 把两条既有 `#[ignore]` GPU 断言由「期望拒绝」改为「期望接受」（仍 `#[ignore]`，窗口内跑，不得删或降级）；
3. 更正 `cuda/render_megakernel.cu:15` 头注释；
4. 加 in-crate 单测锁 `resolved.shadow_light_fraction → scale_constants[2]` 与第 40 槽取值。

→ 已登记 `2026-10-28-render-bvh-r09-s2-host-guard-unlock`（proposed，owner Backend，depends_on R09-S2），**必须在 R09-S3b 窗之前完成**。切片顺序改为 **R09-S2 → R09-S2b → R09-S3a 捕获窗 → R09-S3b 验证窗**。

## 3. manifest 基准更正（EOL 教训）

实现片 `manifest.txt` 原先把源码行按**工作树**字节记录；在 `core.autocrlf=true` 下工作树是 CRLF，而入库产物（committed blob）是 LF，两者 sha256/bytes 不同。本片把源码行改为按 **committed blob（LF）** 记录，并在 manifest 头注写明：

- 解析顺序：先证据目录相对路径，再仓库根相对路径；
- 基准：外部被复核文件一律取 `HEAD` 的 committed blob（LF），不是工作树字节；
- 背景：与既有 EOL 债务（任务 `2026-10-28-repo-fixture-eol-attributes`、R08 复核 O8）同类，属证据面被 EOL 影响的一个实例。

## 4. plan.json 同步

- `R09-S2` 切片 note 补：host 前置拒绝与两条断言仍以拒绝为契约，本片不解锁；解锁归属 R09-S2b；
- 新增切片 `R09-S2b`（host-guard-unlock，status proposed，depends_on R09-S2）；
- 顺序写明 **S2 → S2b → S3a 捕获 → S3b 窗口**，防止窗口空跑。

## 5. 边界

- 零代码改动：`src/**`、`tests/**`、`.cu`、`fixtures/**`、`Cargo*` 全部未动；
- 不写「已解锁 / 已可用 / 已覆盖 / 窗口通过 / 精度达标 / 上游等价」；
- 不宣称任何验收通过；不改任何窗口数字与 attempt 记账。

## 6. 门禁与提交

- `normify gate 2026-10-28-render-bvh-r09-s2-obs-closure`（绑最终树）：见 receipt 与 `.normify/runs/2026-10-28-render-bvh-r09-s2-obs-closure/`；
- 提交显式列路径（禁 `git add -A`）。
