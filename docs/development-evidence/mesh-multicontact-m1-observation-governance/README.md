# M1 QA 观察 O1–O5 收口（证据文字勘误）

任务 `2026-10-26-mesh-multicontact-m1-observation-closure`。依据 M1 独立复核 **pass（阻断 0 / 观察 5）**，QA 收口 `c1016aa7`；实现 `35192320`。**纯证据勘误，不改参考 JSON、不改任何数字与判据。**

| 观察 | 级别 | 处理 |
| --- | --- | --- |
| O1 | 一般 | `dispatch-audit.md` / `README`：「primitive 表无 mesh」→「目标组合不在 primitive 表（L48-70 只含 plane/sphere/capsule/box；L53 的 `(PLANE,MESH)` 属 plane-mesh、不在目标组合内）」，结论不变 |
| O2 | 一般 | baseline `compute-apps` 条目数 34 → 实测 **31**（README + `receipt.raw_entry_count`）；「无本任务进程」结论不变 |
| O3 | 建议 | README 增「脚本归档的复核链」：`byte_identical_after_copy` 在执行副本删除后不可独立复核；实际链 = 参考 `provenance.authoring_script_sha256` 与 `scripts/sha256.json` 的归档副本 sha 互相印证 |
| O4 | 建议 | README mesh-cylinder `pos` z 由 `0.095` 改为 f32 实测 `0.0949999988` |
| O5 | 建议 | README 采样摘要加「近阈值」注记：`util_max=94%`、`util_streak_max=2` 接近 80%×3 守卫线，本片未 abort（规则正确） |

影响面：只改 M1 证据三个文件；`src/`、`tests/`、`fixtures/`、`Cargo.*` 零改动；参考 JSON 内容与 sha `c83baffe…` 不动。

边界：不宣称 mesh 已支持；device 对照仍留 M2。
