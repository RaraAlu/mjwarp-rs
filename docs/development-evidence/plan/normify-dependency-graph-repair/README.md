# normify 依赖图修复（死依赖与占位伞卡；2026-11-11）

- 任务：`2026-11-11-normify-dependency-graph-repair`（owner Arch，planning）
- 性质：**只改 `.normify` 任务记录**（依赖/note）。**不改**任何卡的 acceptance / scope / status / owner，不 start / 不 verified 任何占位卡；不碰判据/参考/spec/`fixtures/`/`src/`/`tests/` 与 W0 冻结输入（perworld 三脚本、`scene-a-light/scene.xml`、spec、S-3 材料、guards 清单）；零 GPU、不采集、不开窗。
- 依据：Chief 2026-11-11 对治理提案 4 点全部批准（依赖改指收口卡 / 占位卡加 note / T4 drop+追加 / Linux 政策编码为依赖）。

## 1. 前后统计

| 指标 | before | after |
| --- | --- | --- |
| 开放任务（proposed+backlog） | 58 | 58 |
| 依赖边总数 | 241 | 293 |
| **依赖 abandoned 卡的边** | **65** | **0** |
| 含 abandoned 依赖的开放任务 | **17** | **0** |

修复动作：**改指 62 条**死边 → 追加 **108** 条 verified 后继依赖；**waive 2 条**（a06-p10，代码/测试已交付无卡）；**T4 1 条**（drop 死边 + 追加 `2026-10-27-t4-historical-capture-rebaseline-decision`）；**伞卡下游改指 5 条**（r06-p8 / r10-p8 → 已验证家族收口卡）；**Linux 政策依赖 8 条**（2026-10-24-linux-batch-retest）；**占位卡 note 53 张**（阶段占位统一措辞 45 + deferred 变体 8）。

## 2. 修法口径（本卡写死）

1. **死依赖 → 已 verified 后继**：逐条取该 abandoned 卡自身 `放弃：…；依据：<卡 id…>` 里的 verified 卡，不改卡面语义；62 条逐条依据见 `artifacts/dependency-diff.tsv` / `.md`。
2. **例外 a06-p10（执行图）**：其依据是 `src/runtime/launcher/graph.rs` + `tests/launcher_graph.rs`（无对应卡）→ 在 p10 / p11 上 **drop 该依赖 + note 记依据**。
3. **例外 t4-artifacts**：用户 2026-10-23 裁决 B（T4 暂停、历史捕获不重基线、恢复需另立任务），无 verified 后继 → `next-t4-numeric` **drop 死边 + 追加 `2026-10-27-t4-historical-capture-rebaseline-decision`**，使「等用户 A/B/C」成为图上真实阻塞。
4. **占位伞卡不强行 verified**：r06-p8 / r10-p8 的家族收口卡已验证（`2026-11-04-render-r06-flex-closeout`、`2026-11-04-render-r05-r09-r10-family-closeout`），只把**下游**对这些占位卡的依赖改指收口卡；占位卡本身保持 proposed + note。
5. **Linux 政策编码**：`p1`/`d02`/`d03`/`d03-final`/`p11`/`layers-linux`/`np03-dual-platform`/`np04-performance` 追加依赖 `2026-10-24-linux-batch-retest`（用户 2026-10-24：Windows 全量完成后再集中回测）。
6. **占位卡 note**：统一措辞「阶段占位：无机器检查项，不参与就绪判定；真实验收由已验证收口卡/后续可执行切片承担」；对 8 张非阶段占位的 deferred/backlog 卡用变体「无机器检查项，不参与就绪判定；立项时须补机器检查项，真实验收由后续可执行切片承担」（避免把真实待办误标为阶段占位）。
7. **可追溯**：被改依赖的 17 个任务在 note 追加一行指向本目录；前后对照表入 `artifacts/`。

## 3. 17 个受影响任务的修后结论

| 任务 | 死依赖 | 改指/waive | 修后仍阻塞（真开） | 结论 |
| --- | --- | --- | --- | --- |
| `2026-10-11-roadmap-a12-p3` | 1 | 2026-10-07-inertial-model-fields<br>2026-10-07-native-model-admission<br>2026-10-16-done-g08-input<br>2026-10-18-layers-m4 | 2026-10-11-roadmap-p1 | 修后仍阻塞 |
| `2026-10-11-roadmap-d02` | 2 | 2026-10-07-native-model-admission<br>2026-10-11-two-reference-gates<br>2026-10-24-linux-batch-retest | 2026-10-24-linux-batch-retest | 修后仍阻塞 |
| `2026-10-11-roadmap-d03` | 2 | 2026-10-07-native-model-admission<br>2026-10-11-two-reference-gates<br>2026-10-24-linux-batch-retest | 2026-10-24-linux-batch-retest | 修后仍阻塞 |
| `2026-10-11-roadmap-d03-final` | 3 | 2026-10-23-w15-verbatim-headers<br>2026-10-06-cubecl-dual-probe<br>2026-10-22-split-ptx-identity<br>2026-10-06-windows-external-resources<br>2026-10-06-windows-kernel-capabilities<br>2026-10-24-linux-batch-retest | 2026-10-11-roadmap-d03<br>2026-10-24-linux-batch-retest | 修后仍阻塞 |
| `2026-10-11-roadmap-g19-p4` | 2 | 2026-10-07-batched-device-transfers<br>2026-10-18-layers-m3<br>2026-10-18-layers-m5<br>2026-10-22-layers-a1-lifecycle-wiring | — | 依赖就绪（占位卡，不 start） |
| `2026-10-11-roadmap-m11-p3` | 1 | 2026-10-07-inertial-model-fields<br>2026-10-07-native-model-admission<br>2026-10-16-done-g08-input<br>2026-10-18-layers-m4 | 2026-10-11-roadmap-p1 | 修后仍阻塞 |
| `2026-10-11-roadmap-p1` | 5 | 2026-10-11-roadmap-p0-01<br>2026-10-11-roadmap-p0-07<br>2026-10-11-roadmap-p0-08<br>2026-10-23-w14-tile-launcher<br>2026-10-23-w15-verbatim-headers<br>2026-10-06-windows-resource-leases<br>2026-10-06-windows-external-resources<br>2026-10-23-w14-shared-memory-device-limit<br>2026-10-18-layers-m2<br>2026-10-24-linux-batch-retest | 2026-10-24-linux-batch-retest | 修后仍阻塞 |
| `2026-10-11-roadmap-p10` | 7 | 2026-10-07-batched-device-transfers<br>2026-10-16-done-g08-input<br>2026-10-18-layers-m5<br>2026-10-18-layers-m2<br>2026-10-18-layers-a1 | 2026-10-11-roadmap-g25-p10<br>2026-10-11-roadmap-m10-p10<br>2026-10-11-roadmap-r10-p10 | 修后仍阻塞 |
| `2026-10-11-roadmap-p11` | 24（+伞卡/Linux） | 2026-10-20-layers-m3-rigid-kinematics<br>2026-10-16-done-t4-numeric<br>2026-10-23-final-acceptance-rerun<br>2026-10-19-layers-m3-rne-bias<br>2026-10-19-layers-m3-rne-cacc<br>2026-10-19-layers-m3-rne-cfrc<br>2026-10-19-layers-m3-rne-qfrc<br>2026-10-16-done-crb-dense<br>2026-10-22-g03-matrix-chain-alignment<br>2026-10-19-layers-m3-rigid-mass-solve<br>2026-10-16-done-sparse<br>2026-10-16-done-passive-combine<br>2026-10-16-done-passive-producers<br>2026-10-19-layers-m3-smooth-force-compute<br>2026-10-07-resident-fixed-tendon<br>2026-10-07-resident-spatial-tendon<br>2026-10-07-resident-geom-tendon<br>2026-10-16-next-g26-complete<br>2026-10-07-native-model-admission<br>2026-10-18-layers-m1<br>2026-10-07-inertial-model-fields<br>2026-10-16-done-g08-input<br>2026-10-07-batched-device-transfers<br>2026-10-18-layers-m3<br>2026-10-19-layers-m3-velocity-position-g01-correction<br>2026-10-23-w15-verbatim-headers<br>2026-10-23-w14-tile-launcher<br>2026-10-06-windows-resource-leases<br>2026-10-06-windows-external-resources<br>2026-10-23-w14-shared-memory-device-limit<br>2026-10-18-layers-m5<br>2026-10-18-layers-m2<br>2026-10-18-layers-a1<br>2026-10-11-roadmap-p0-01<br>2026-10-11-roadmap-p0-07<br>2026-10-11-roadmap-p0-08<br>2026-10-22-layers-a1-lifecycle-wiring<br>2026-11-04-render-r06-flex-closeout<br>2026-10-24-linux-batch-retest | 2026-10-11-roadmap-g10-p5<br>2026-10-11-roadmap-g11-p7<br>2026-10-11-roadmap-g12-p6<br>2026-10-11-roadmap-g13-p6<br>2026-10-11-roadmap-g14-p6<br>2026-10-11-roadmap-g15-p6<br>2026-10-11-roadmap-g17-p6<br>2026-10-11-roadmap-g19-p6<br>2026-10-11-roadmap-g22-p7<br>2026-10-11-roadmap-g25-p10<br>2026-10-11-roadmap-m03-p8<br>2026-10-11-roadmap-m10-p10<br>2026-10-11-roadmap-m11-p11<br>2026-10-11-roadmap-r07-p8<br>2026-10-11-roadmap-r10-p10<br>2026-10-11-roadmap-a11-p11<br>2026-10-11-roadmap-a12-p11<br>2026-10-11-roadmap-p1<br>2026-10-11-roadmap-p4<br>2026-10-11-roadmap-p5<br>2026-10-11-roadmap-p6<br>2026-10-11-roadmap-p7<br>2026-10-11-roadmap-p8<br>2026-10-11-roadmap-p9<br>2026-10-11-roadmap-p10<br>2026-10-11-roadmap-d02<br>2026-10-11-roadmap-d03-final<br>2026-10-11-roadmap-np01<br>2026-10-11-roadmap-np02<br>2026-10-11-roadmap-np03<br>2026-10-11-roadmap-np04<br>2026-10-18-layers-linux<br>2026-10-24-linux-batch-retest | 修后仍阻塞 |
| `2026-10-11-roadmap-p4` | 6 | 2026-10-20-layers-m3-rigid-kinematics<br>2026-10-16-done-t4-numeric<br>2026-10-23-final-acceptance-rerun<br>2026-10-19-layers-m3-rne-bias<br>2026-10-19-layers-m3-rne-cacc<br>2026-10-19-layers-m3-rne-cfrc<br>2026-10-19-layers-m3-rne-qfrc<br>2026-10-16-done-crb-dense<br>2026-10-22-g03-matrix-chain-alignment<br>2026-10-19-layers-m3-rigid-mass-solve<br>2026-10-16-done-sparse<br>2026-10-16-done-passive-combine<br>2026-10-16-done-passive-producers<br>2026-10-19-layers-m3-smooth-force-compute<br>2026-10-07-resident-fixed-tendon<br>2026-10-07-resident-spatial-tendon<br>2026-10-07-resident-geom-tendon<br>2026-10-16-next-g26-complete | 2026-10-11-roadmap-g19-p4 | 修后仍阻塞 |
| `2026-10-11-roadmap-p6` | 1 | 2026-10-22-g03-matrix-chain-alignment<br>2026-10-19-layers-m3-rigid-mass-solve<br>2026-10-16-done-sparse | 2026-10-11-roadmap-g12-p6<br>2026-10-11-roadmap-g13-p6<br>2026-10-11-roadmap-g14-p6<br>2026-10-11-roadmap-g15-p6<br>2026-10-11-roadmap-g17-p6<br>2026-10-11-roadmap-g19-p6 | 修后仍阻塞 |
| `2026-10-11-roadmap-p7` | 4 | 2026-10-07-resident-fixed-tendon<br>2026-10-07-resident-spatial-tendon<br>2026-10-07-resident-geom-tendon<br>2026-10-16-next-g26-complete<br>2026-10-18-layers-m4<br>2026-10-16-done-g08-input | 2026-10-11-roadmap-g11-p7<br>2026-10-11-roadmap-g22-p7<br>2026-10-11-roadmap-g25-p7 | 修后仍阻塞 |
| `2026-10-11-roadmap-p8` | 0（+伞卡/Linux） | 2026-11-04-render-r06-flex-closeout | 2026-10-11-roadmap-p8-base<br>2026-10-11-roadmap-m03-p8 | 修后仍阻塞 |
| `2026-10-11-roadmap-p8-base` | 1（+伞卡/Linux） | 2026-10-07-batched-device-transfers<br>2026-11-04-render-r05-r09-r10-family-closeout | 2026-10-11-roadmap-r07-p8 | 修后仍阻塞 |
| `2026-10-11-roadmap-p8-entry` | 2 | 2026-10-07-batched-device-transfers<br>2026-10-18-layers-m3<br>2026-10-18-layers-m5<br>2026-10-22-layers-a1-lifecycle-wiring | — | 依赖就绪（占位卡，不 start） |
| `2026-10-11-roadmap-p9` | 2 | 2026-10-07-batched-device-transfers<br>2026-10-18-layers-m3<br>2026-10-18-layers-a1 | 2026-10-11-roadmap-p9-incremental | 修后仍阻塞 |
| `2026-10-11-roadmap-p9-incremental` | 1 | 2026-10-11-two-reference-gates | — | 依赖就绪（占位卡，不 start） |
| `2026-10-11-roadmap-r10-p10` | 0（+伞卡/Linux） | 2026-11-04-render-r05-r09-r10-family-closeout | — | 依赖就绪（占位卡，不 start） |
| `2026-10-11-roadmap-r10-p8` | 0（+伞卡/Linux） | 2026-11-04-render-r06-flex-closeout | 2026-10-11-roadmap-r07-p8 | 修后仍阻塞 |
| `2026-10-16-next-t4-numeric` | 1 | 2026-10-27-t4-historical-capture-rebaseline-decision | 2026-10-27-t4-historical-capture-rebaseline-decision | 修后仍阻塞 |
| `2026-10-18-layers-linux` | 0 | 2026-10-24-linux-batch-retest | 2026-10-24-linux-batch-retest | 修后仍阻塞 |
| `2026-10-23-np03-dual-platform` | 0 | 2026-10-24-linux-batch-retest | 2026-10-24-linux-batch-retest | 修后仍阻塞 |
| `2026-10-23-np04-performance` | 0 | 2026-10-24-linux-batch-retest | 2026-10-24-linux-batch-retest | 修后仍阻塞 |

## 4. 修后 ready 列表（proposed，deps 全 verified）

- `2026-10-11-roadmap-g10-p5`
- `2026-10-11-roadmap-m03-p8`
- `2026-10-11-roadmap-m10-p10`
- `2026-10-11-roadmap-r06-p8`
- `2026-10-11-roadmap-r07-p8`
- `2026-10-11-roadmap-r10-p10`
- `2026-11-04-render-r06-s3b-flex-tie-order-r4`

> 说明：normify 的 ready 只看「依赖全 verified」，不读 note。上述 7 张里 **6 张是 checks=0 的占位卡**（g10-p5 / r06-p8 / r07-p8 / m03-p8 / m10-p10 / r10-p10），按本卡 note 口径**不参与就绪判定、不 start**；**真可执行的是 tie-order-r4**（等用户②裁决）。若要把假 ready 从列表里彻底移除，需要允许把这 6 张占位卡的 status 改为 backlog（超出本次「不改 status」边界，未做，待裁决）。

## 5. 完成判据与边界

- **达成**：`normify status` **0 个开放任务依赖 abandoned**；0 错误 0 警告；verified 数不因本卡异常变化（本卡自身 +1 verified 属正常）；`unchanged src/, fixtures/, scripts/, tests/`。
- **未做（按边界）**：未改任何 status/owner/acceptance/scope；未 start 任何占位卡；未新登记 A2/S1 等可执行切片（属 W2/W3 派工）；未动 W0/G-L2 冻结输入。
- **非声明**：本卡只是排程图修复，不构成任何阶段验收、不改变任何判据/参考，也不宣称任何功能完成。
