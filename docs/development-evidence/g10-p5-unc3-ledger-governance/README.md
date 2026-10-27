# UNC-3 台账更新（B3 分支片收口）+ B3 证据 QA 观察勘误

**任务**：`2026-10-26-g10-p5-unc3-ledger-branch-closure`（owner Arch）。
**基准**：B3 实现 `e221e6c2`（verified）、QA 复核 `856b7835`（pass / 阻断 0 / 观察 6）。**纯文档与证据文字治理，不改 `src/`、`tests/`、`fixtures/`、判据、容差、期望或任何已验证数字。**

## 1. UNC-3 更新（前 → 后）

| 项 | 本次更新前 | 本次更新后 |
| --- | --- | --- |
| 组合层 | 13/13 | 13/13（不变） |
| 分支层（已覆盖） | B1/B2 覆盖的 4 类 | **+ B3：graph hillclimb、`dist==0` fallback、投影 clamp、cap、corner、non-parallel det，以及 swap 几何顺序** |
| 分支层（未覆盖） | 8 类 | **3 类**：`plane_cylinder` degenerate x 轴回退、`plane_ellipsoid` 非上游退化保护、`plane-box` 其余 5 角点（含 plane-capsule/cylinder 其余候选未 detected） |
| 债务 | sphere-box/capsule-box/box-box 分支级未标注 | 同上 **+ `capsule_capsule` 平行轴分支**在融合后端（det≈2.6e-12 → 非平行）未被 device 触发 |
| status | partial | **partial（不变）** |

**改动文件**
- `docs/development-evidence/g10-p5-acceptance-map/README.md`：§6 分支层清单改写（已覆盖 6 项 + swap；剩 3 类；债务补 capsule 平行轴）；capsule-capsule 表行措辞按融合后端实际分支修正；证据指针补 B3 与 QA。
- `docs/development-evidence/g10-p5-acceptance-map/map.json`：`uncovered[UNC-3]` 的 `measured`/`evidence_paths`/`reason`/`impact` 与 `summary.note` 同步；status 仍 `partial`，其余 requirement 未动。

## 2. QA B3 观察收口（6 条，纯文字/边界）

| 观察 | 级别 | 处理 |
| --- | --- | --- |
| O1 | 一般 | B3 README 的「13 组合位级无回退」改为**「判据内无回退」**并注明 B1/B2 只输出 `failures=0`、PTX 已变（3786439→3786510），不能先验排除 ULP 级差异 |
| O2 | 建议 | 边界说明：CPU 自检不经过 `run_case` geom 解析路径，`device_touched=false` 恒真，只能兜住参考/形状类缺陷 |
| O3 | 一般 | 执行脚本已入库为**逐字节相同**的证据副本（`9cff9b31…` / `6b0eba49…`，见 B3 `scripts/sha256.json`）；工作树中未跟踪的执行副本已清理 |
| O4 | 建议 | 去除 B3 README 的 2 个 BEL 控制字符；重复的 `## 9` 标题改 `## 11` |
| O5 | 建议 | 标注 precheck `axis_dot=0.0` 为理想化口径；按参考 `geom_xmat` f32 实算 ≈-7.57e-9、det ≈2.073600335e-4，分支判定不受影响 |
| O6 | 建议 | 说明 attempt1 branches 二进制未归档；「修复只碰 tests/」由 panic 日志 + 提交内容 + dirty 计数支持，不作二进制级对照 |

B3 证据 `receipt.json` 增加 `post_qa_closure` 字段记录以上映射；B3 README 新增 §12 收口节。

## 3. 证据指针

- 实现：`docs/development-evidence/g10-p5-primitive-branch-coverage/`（§7.4 attempt2、§12 QA 收口、参考 `a8407a90…`）。
- 复核：`docs/development-evidence/g10-p5-primitive-branch-coverage-independent-check/`（pass）。
- 台账：`docs/development-evidence/g10-p5-acceptance-map/{README.md,map.json}` §6 / UNC-3。

## 4. 边界

- 只更新证据映射与文字；不改任何判据/容差/期望/参考；不改已 verified 数字与结论。
- UNC-3 仍 `partial`：不宣称全路径 / 全分支 / 跨平台；精度与等价性结论仅限已列 case 与已批准判据内。

## 5. S5 第三次台账收口（2026-10-27；S4 分支收口验证窗 + QA `efccbf63` pass 后）

**任务**：`2026-10-27-unc3-ledger-closure-s5`（owner Arch）。基准：S3 捕获 `43ec5676`（QA `68ca6fa5` pass / 观察 10，收口 `2d4a5f37`+`e4713472`）、S2 静态映射 `c257bc0b`、S4 实现 `b3e56a5f`、S4 验证窗 `055d0c8a`（QA `efccbf63` pass / 0 阻断 / 观察 5）。机器可读回执：`s5-receipt.json`（本目录）。

| 项 | 第二次收口后 | S5 收口后 |
| --- | --- | --- |
| UNC-3 分支层（已覆盖） | B1/B2/B3 覆盖的 6 类 + swap | **+ U1/U3/U4/U5/U6/U9 六项「本窗验收覆盖」**（S4 窗，判据内、零放宽） |
| UNC-3 分支层（边界项） | U2/U7/U8 三类未覆盖 | **U2「不可达 / 非上游防御」；U7/U8「仅限制存在」**；U8 补「设备 1 vs 上游 4 = 真实功能差距」（backlog，不 start） |
| 容量限制 | 未单列 | **单列两条**：`G5_MC_MESH_DEG_MAX=10`、`G5_MC_MESH_FACE_MAX=16`（vs 上游 `nmeshdegmax`/`npolygonmax` 动态容量） |
| UNC-1 | mesh-mesh 已接；一般 mesh/hfield-capsule 未覆盖 | **收窄**：nmesh>1 仅 1 例、一般拓扑未穷举、degree>10/polygon>16 仅限制存在、mesh 边/顶点分支层仍静态；**hfield-capsule 计入已覆盖（单 case 275/275 bit）** |
| status | partial | **partial（不变）** |

**数字口径（QA S4 O1/O2 已吸收）**：case 级比较 **33**（multicontact 3 + launcher 4 + primitive 3 + plane 6 + sphere_capsule 4 + branches 6+4 + M2 2 + M3 1）+ stage2 **17 dump**；「13 组合」为旧口径、已作废；launcher 一律写 4。**回归对象 = M3 窗口冻结二进制**（8 个 `target/m3-freeze`，构建 01:06；M3 提交 `f1e5cda1` 在冻结之后，含 O7 + 测试拆分）。

**runner 改进项（O3–O5，本片不改 runner）**：`facts.txt` 的 `guard_aggregate` 字典串、guard sampler 写文件 BOM、stage2 日志控制台转码。

**改动文件**：`g10-p5-acceptance-map/README.md`（§1 两条 + 新增 §6.2/§7.3）、`g10-p5-acceptance-map/map.json`（UNC-1/UNC-3 增 `s5_update` + evidence_paths + summary note）、`mesh-multicontact-mesh-summary/README.md`（§6 两条 + 新增 §10）、本目录 `README.md` + `s5-receipt.json`。

**边界**：只改文字与映射，不改已 verified 数字/判据/容差/参考/期望值；不开 GPU、不编译、不改代码；不写「窗口通过」「精度达标」「上游等价」「双平台」「任何分支全量覆盖」。
