# M0a QA 观察收口（O1 / O3 / O4 / O6）— 证据文字与编码勘误

**任务**：`2026-10-26-mesh-multicontact-m0a-observation-closure-r2`（owner Arch）。
**依据**：M0a 独立复核 **pass（阻断 0 / 观察 6）**，收口提交 `5d07b4fe`；M0a 实现 `097c54b9`。**纯证据勘误：不改任何数字、判据、容差、期望、ABI 布局或冒烟指纹。**

## 1. 逐条处理

| 观察 | 级别 | 处理 |
| --- | --- | --- |
| O1 | 一般 | `README §7.1` 裸跑失败表述按实测更正：失败点在依赖编译阶段（`dlltool.exe` 缺失 → `getrandom`/`windows-sys` 编译失败），**未进入 build.rs 阶段**；删除「env 未设 → build.rs panic」的未经证据支持表述。「需 `MJWARP_MUJOCO_ROOT` + msvc 工具链」的结论不变 |
| O2 | 一般 | **折入 M2 acceptance**（提交 `9ccd9042`）：补 `sum(polynum)==nmeshpoly`、polyadr/polyvert/polymap 区间与负值规则单测；empty-sidecar 测试显式断言 `mesh_poly_args` 为 null |
| O3 | 建议 | `README §1` 行数改实测（`product.cu` **426**、`abi.rs` **337**）；§5.1/§5.2 顺序整理为「5.1 探针同步 → 5.2 旧 case 兼容」，正文引用同步 |
| O4 | 建议 | 日志转 UTF-8 无 BOM：**实测 6 个 `.log` 为 UTF-16LE**（`all-features-bare-fail`、`pre/post-drift-recapture`、`pre/post-identity-live`、`post-smoke-final`），其余 3 个 `.txt` 本就是 UTF-8；已全部转为 UTF-8，内容逐字不变，git 不再视为二进制 |
| O5 | 建议 | **折入 M2 acceptance**（同 `9ccd9042`）：为冻结探针 `ProbeArgs`（496B/23 指针）加 static_assert/测试锁定 |
| O6 | 建议 | `README §7.2` 增边界说明：size-gate 拆分（plane 测试 503→**450** + `tests/support/primitive_plane_cases.rs` 63 行）为 tests-only；「ABI/冒烟不变」由**提交后树实测**确认（`238969 / ec2e7acd… / 3786520 / entries 6`、unit 980/0/372），单 commit 内「拆分前后」不可独立还原，如实记录 |

**O4 口径更正**：QA 原文「9 个 `.log` 为 UTF-16LE」实测为 **6 个**（另 3 个 `.txt` 为 UTF-8）；转换覆盖全部实际 UTF-16 文件，此更正一并留档。

## 2. 影响面

- 改动仅 `docs/development-evidence/mesh-multicontact-m0a/`（README + 6 个日志重编码）与本回执目录。
- `src/`、`tests/`、`fixtures/`、`Cargo.*` 零改动；ABI 504 布局、sidecar 72B、冒烟指纹、门禁数字均未变。

## 3. 证据指针

- M0a 交付：`docs/development-evidence/mesh-multicontact-m0a/`（README、receipt、logs）。
- M0a 复核：`docs/development-evidence/mesh-multicontact-m0a-independent-check/`（pass）。
- M2 acceptance（O2/O5 折入）：`.normify/changes/2026-10-26-mesh-multicontact-m2-convex-vs-mesh.json`。

## 4. 边界

- 不改已 verified 数字与结论；不宣称 mesh 已支持；null/非 null sidecar 的发射级行为仍留 M2 验证窗。
