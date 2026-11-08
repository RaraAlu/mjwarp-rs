# perworld L v2 场景/相机改卡复核（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-scene-camera-check`（owner QA）。日期：2026-11-08。
复核目标：Backend 卡 `2026-11-08-render-perworld-l-v2-scene-camera`（verified；登记 `c6dd10df` → start `ce5f44a3` → 实现 **`3d546fbe`**）。
复核时 HEAD = `e3aea21f`（QA 签名留证复核提交；`3d546fbe..HEAD` 未触及 fixture/属性/被复核证据）。
边界：只读核验 + 只写本卡证据目录；零 GPU、不采集、不开窗、不改 fixture/`.gitattributes`/被复核证据。

**结论：PASS-with-observations（0 阻断；1 条低观察 + 2 条信息项）。**

## 1. scene.xml 字节与文本（独立重算）

| 项 | 实测 | 判定 |
| --- | --- | --- |
| 改后工作树 sha256 | `4b354054116ddb81c6209ed10c021d133ea30d6fd82c1a6f866bafff9f625716` | ✅ |
| 改后 `3d546fbe:` blob sha256 | 同上（blob `f4376874…`） | ✅ |
| 改后 bytes / 行数 | 5,134 B / 86 行（LF，CR=0，TAB=0，无 BOM） | ✅ |
| 改前 sha256 / bytes / 行数 | `4db66e2c…` / 4,987 B / 84 行（`ce5f44a3:` blob 独立重算） | ✅ |
| diff 面 | `ce5f44a3..3d546fbe` 该文件 **2 增 / 0 删**（+147 B） | ✅ |
| 插入位置 | `cam-persp` 之后、`cam-track` 之前 | ✅ |
| 插入文本 | 与 spec v3 §2.1 写死值逐字一致（pos `-0.0925 0.4981 0.8296`、quat `-0.211088 -0.149763 0.558924 0.787792`、fovy `30`、resolution `320 240`） | ✅ |
| 预测核对 | Chief 预测 `4b354054…`/5,134 B/86 行，实算逐项一致 | ✅ |

## 2. `.gitattributes`（仅 +1 行）

- diff：仅新增 `fixtures/render-bvh/scene-a-light/** text eol=lf`（L289，位于既有 `scene-flex` LF 规则 L288 之后、`*.npz -text -diff` L296 之前），其余行未动 ✅。
- `git check-attr text eol -- fixtures/render-bvh/scene-a-light/scene.xml` → **`set` / `lf`** ✅。
- `git ls-files --eol` → `i/lf w/lf attr/text eol=lf` ✅；工作树 LF ✅。
- 不越界：`fixtures/render-bvh/scene-a/scene.xml` 仍 `text: unspecified / eol: unspecified` ✅；该子树下 `.npz` 仍 `text: unset`（后续规则优先，二进制不被改写）✅；当前该子树唯一已跟踪文件就是 `scene.xml`（`git ls-files` 复核）✅。

## 3. MuJoCo CPU 解析（pinned venv 3.15.0，独立复算）

改前模型用 `git show ce5f44a3:` 的副本（`target/qa-scenecheck/old-scene.xml`，sha 校验 `4db66e2c…`）载入对照：

| 项 | 实测 |
| --- | --- |
| `ncam` | **3** |
| `cam_mode` | **`[0, 0, 3]`** |
| `cam_fovy` | **`[73.73979461915735, 30.0, 40.0]`** |
| `cam_resolution` | **`[[320,240],[320,240],[240,180]]`** |
| `cam_sensorsize`（cam-hinge） | `[0, 0]`（走 fovy 路径） |
| `cam_intrinsic`（cam-hinge） | `[0.01, 0.01, 0, 0]`（模型默认） |
| `cam_pos` / `cam_quat`（cam-hinge） | 与插入值一致（float32 精度内） |
| 相机名顺序 | `cam-persp` / `cam-hinge` / `cam-track` |
| **属性逐项比对** | 比对 **581 个模型属性**：差异 **27 项**，全部为相机数组（16）+ 名字表（`name_camadr`/`name_lightadr`/`name_matadr`/`name_meshadr`/`name_texadr`/`names_map`）+ 标量 {`ncam` 2→3、`nbuffer` 108272→108656、`nnames` 193→203、`nnames_map` 48→50、`signature`} |
| **非相机物理数组** | **逐位不变（0 差异）** ✅ |

> 说明：独立脚本第一次分类把 `names_map` 误标为 unexpected（脚本白名单缺该项），复核后确认其为名字表簿记项，与 Backend 列的「预期变化」一致；最终 `UNEXPECTED_COUNT=0`。

## 4. dry-run 前后对照（独立复跑）

- 装置：改前副本放 `target/qa-scenecheck/old-scene.xml`（同 out-root/out-dir/tag，两侧 `--dry-run` exit 0）。
- 叶子差异 **13 条**，全部可归入预期类：`/recipe/scene_xml/{path,sha256,bytes}`、`/environment/scene_xml/{path,sha256,bytes}`、`/environment/camera_modes`、`/cam_active_fixed_only`、`/model_static/cameras`、`/light_pose_precheck/scene_xml`、`/paths/effective_scene`。
- **唯一「非卡内」差异**是 `/command/argv` 与 `/command/rendered`——因我把改前副本放在 `target/` 路径（测试装置本身），非 fixture 改动导致；Backend 的 before/after 日志取自同一仓库路径，故其 diff 不含该项。
- 关键值：`model_static.cameras = [[0,0,'cam-persp'], [1,0,'cam-hinge'], [2,3,'cam-track']]` ✅；`cam_active_fixed_only = [true,true,false]` ✅；`legacy_manifest_entries = 112`（改前=改后）✅；默认 dry-run `legacy = 107` ✅。

## 5. 旧 sha 与 diff 面

- `4db66e2c` 在历史证据中的 occurrence = **8**（`light-prep/`、`light-prep-check/` 共 7 文件，本提交未改）✅；本卡登记 JSON 1 行 2 处（新增引用，非回改）✅。
- `c6dd10df..3d546fbe` 改动面：`.gitattributes`、`fixtures/render-bvh/scene-a-light/scene.xml`、卡 JSON、`.normify/modules/mjwarp/index.md`（`normify done` 刷新，1/1）、4 个 runs JSON、证据目录 ✅。
- `fixtures/render-bvh/scene-a/`、`src/`、`tests/`、`scripts/` **零 diff** ✅；无采集/开窗/CUDA 产物 ✅。

## 6. 证据与门禁

- 被复核证据 `docs/development-evidence/render-bvh/perworld-l-v2-scene-camera/`：manifest **15/15** 自核、`# base dir:` 在位、不自列；全部文件 UTF-8/LF/无 BOM/裸 CR·TAB 0 ✅。
- README/receipt/artifacts 与我的独立复算一致（sha/bytes/行数、mjcf 摘要、dry-run 分类、EOL、边界、非声明）✅。
- 四门禁独立复跑：fmt / clippy / clippy-cuda 通过，**unit 1359 过 / 0 败**（记录见本卡 `.normify/runs/`）。

## 7. 观察项（按严重度）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低 | `artifacts/old-sha-references.txt` 头部计数与 HEAD 实况不符：写作「total matching lines in HEAD: 8（historical 8, this card registration JSON: 0）」，但该 txt 正文已列出本卡登记 JSON 行；`artifacts/scene-identity.json` 的 `old_sha_registration_refs: 0` 同源过期。实况：历史证据 8 处 + 本卡登记 JSON 1 行（2 处）+ 卡自身证据 5 处 = **15 处**。README §4 与 receipt（`card_registration_refs: 1`）写法正确，**结论与「未回改」判定不受影响** | 后续 fix 轮把两个 artifact 的计数头改为「历史 8 + 本卡登记 1 行 + 卡自身证据」并重算 manifest；不改结论 |
| O2 | 信息 | 新 `.gitattributes` 规则覆盖整个 `scene-a-light/**` 子树（当前仅 `scene.xml`；后续采集产出的 `capture.json` 等文本将 enforce LF，`.npz` 由后续 `-text` 规则保护） | 知悉即可；采集窗需注意 `capture.json` 将以 LF 入仓 |
| O3 | 信息 | 首次提交被钩子以「模块 mjwarp 指纹未刷新」拦下，补 stage `.normify/modules/mjwarp/index.md` 后同提交成功；该文件已在本提交 diff 中（1/1） | 无残留；no action |

## 8. 非宣称

- 本复核只判「fixture 与属性是否按已签 spec v3 §2.1 正确落地」，**不**宣称判别力恢复、采集是否有效、per-world 灯验收、与上游实现等价、任何渲染数值结论。
- 不构成 G-L0/G-L1/G-L2 启动或通过；pilot/采集窗/验证窗另卡。
- 未改 fixture、`.gitattributes`、被复核证据与既有卡状态。
