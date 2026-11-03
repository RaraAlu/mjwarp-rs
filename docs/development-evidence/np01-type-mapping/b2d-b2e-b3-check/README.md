# NP01 B2d/B2e/B3 逐字段表 QA 独立复核（10 张表 + host 覆盖门禁）

任务：`2026-11-04-precision-np01-b2d-b2e-b3-check`（复核卡，owner QA）。被复核提交：B2d `4ec69165`、B2e `638383f2`、B3 `e886fec8`。复核时点：2026-11-04T04:49+08:00，HEAD `6d5069cc`，rustc 1.99.0 / cargo 1.99.0（Windows）。零 GPU；未改产品代码、判据/容差、清单；只写本复核记录与卡状态。

## 结论

**PASS-with-observations**（无阻断、无表内容错误；10/10 张表抽查通过）。门禁机制在「缺表」「row_id 冒充产品行」「`PENDING_ROWS` 清空后产品行缺表」三种情形下均按预期失败；但发现「多余/重复表、文件名与 `row_id` 不一致」不设防（观察 1）。NP01 仍未验收，本复核不表示类型映射验收通过。

## 一、逐表抽查（每张表 ≥2 条断言回源码核对）

| 表 | 抽查断言 | 源码证据（file:line） | 结论 |
| --- | --- | --- | --- |
| `applied_force`（B2d） | ①双哨符：meta 侧 int `-131072` 与 geometry/ft 侧 `-131072.0f`，含首部 4 元素与尾部 4 元素；②delta 累加 `accum += force_dot6(jac,f)+force_dot3(cross,f)` 顺序；③publish 强制 `__fadd_rn` | `src/physics/applied_force.rs:245`（两型哨符同句，`meta[4+ml+j]`/`geometry[4+gl+j]` 尾部段）、`:270`（累加式）、`:276`（`add?__fadd_rn(current[4+i],delta[4+i]):delta[4+i]`）；`force_dot6` 定义 `:254` | PASS |
| `fluid_force`（B2d） | ①双哨符段（`raw` int 与 `p/rigid/com/velocity/attached` float，各含首 4 与尾 4）；②`FF_FOUR_THIRDS_PI` 为 f32 字面量；③椭球体积乘链与 `ff_mul_t` 行展开 | `src/physics/fluid_force/cuda.rs:28-30`（三段哨符校验）、`:10`（`4.18879032135009765625f`）、`:57`（`volume=FF_FOUR_THIRDS_PI*...`）、`:22`（`ff_mul_t` 三行展开） | PASS |
| `kinetic_energy`（B2d） | ①32 lane 片形状 `products[(ENERGY_NV+31)/32]` 与越界补 0；②warp 归约 `16→8→4→2→1`；③`staged[4+2w]=current` 直通、`staged[5+2w]=0.5f*sum` | `src/physics/kinetic_energy/cuda.rs:19`、`:21`（`products[r]=a*b`，越界 `a=b=0`）、`:25`（`__ballot_sync`）、`:28`（`offset=16;offset>0;offset/=2`）、`:30`（两槽写出） | PASS |
| `attached`（B2e） | ①编译源拼接 `KINEMATICS_CUDA + "\n" + ATTACHED_CUDA`；②宿主哨符 `-131072.0` 后 `[4, guarded-4)` 填 `f32::NAN`；③段 stride `12*(ng+ns)` | `src/physics/attached.rs:155`（拼接行）、`:152-153`（fill）、`:159`（`check_device_values` 回读校验）、`:230`/`:303`（`12*ng+12*ns` / `12*(ng+ns)`） | PASS |
| `camlight`（B2e） | ①`light_normalize`：`len>0.0f` 才逐分量除、否则 `{0,0,0}`（kEps=0）；②`look_at` 列装入顺序 `[x.x,y.x,z.x, x.y,y.y,z.y, x.z,y.z,z.z]`；③段 stride `12*nc+6*nl` | `src/physics/camlight.rs:109-112`（normalize）、`:116-125`（look_at 的 `mat[0..9]` 顺序）、`:141`/`:177`（stride）、`:149-150`（cmat 直搬分支） | PASS |
| `field_batch`（B2e） | ①`field_parameter = p + desc[0] + (u64)(world % desc[1]) * width`（整型取模+宽乘）；②打包长度 `checked_add` 限 i32；③唯一 f64 接触点 `1.0 / wrap_prm[i]` | `src/physics/resident/pack_batches.rs:217-218`（设备取数式）、`:50`（`checked_add`）、`:162`（`scale = 1.0 / fields.wrap_prm[i]`，写入 f32 槽） | PASS |
| `sleep_wake`（B2e） | ①`wakeval` 初值 `-11`；②asleep 链写回带越界/回环上限（`next<0||next>=ntree`、`current==tree`）；③`awake[n]=count; awake[n+1]=0; awake[n+2]=0` | `src/physics/sleep.rs:108`（`int any_awake=0,wakeval=-11`）、`:89-95`（链式写回与两处截断）、`:121-122`（`awake[i]=asleep[i]<0`、计数与 n+1/n+2 置 0） | PASS |
| `gjk_host`（B3） | ①`dot3` 三独立乘 + 左结合两加（非融合）；②`xnorm=(dot3(x_k,x_k)).sqrt()`、终止判据 `|xnorm_prev-xnorm|<MINVAL`；③`s3d` 的 `m_det=c41+c42+c43+c44` 顺序；④host gjk 目录内 `mul_add`/`f64` 0 命中 | `src/physics/collision/gjk/math.rs:15-17`（dot3 函数体；全文件 `mul_add`=0、`f64`=0）、`core.rs:96`（xnorm）、`:100`（终止）、`simplex.rs:84`（m_det 顺序）、`:298`（`linear_combine`）、`polytope.rs:69`（刻意不用 FMA/f64 的注释） | PASS |
| `ccd_host`（B3） | ①原点平移到 `geom1.pos`（平移不变性）；②sensor 对 cutoff 取 `1.0e32_f32`，否则 `gap`；③`pos = add3([0.5*sum[i]], origin)` 先加后乘 0.5 再平移；④`dist -= margin1+margin2`、`full_margin=size+0.5*orig_margin`、`cutoff += full_margin1+full_margin2` | `src/physics/collision/gjk/contact.rs:118`（origin）、`:111-115`（cutoff 分支）、`:246`（pos）、`:235`/`:237`（frame 方向选择）；`phase.rs:139`（dist 减）、`:174`（shrink 的 `size+0.5*orig_margin`）、`:186`（cutoff 加） | PASS |
| `epa_host`（B3） | ①`face_norm2` 显式两次舍入 `head=r0*r0 → tail=r1*r1+head → norm2=r2*r2+tail`（刻意不写 `mul_add`）；②`lower2` 取最小 `face_norm2`、`lower=sqrt(lower2)`、归一化前上界 `upper_k=dot3(face_pr,w)/lower`、`upper2=upper*upper`；③horizon 判定 `dot3(face_pr[i],w)-face_norm2[i] > 1e-10` | `src/physics/collision/gjk/polytope.rs:69`（注释：不用 FMA helper、不引 f64、不写 `mul_add`）、`:70-72`（head/tail/norm2 三步）；`epa.rs:79-81`（lower2）、`:96`（sqrt）、`:108`（upper_k）、`:111`（upper2）、`:151`（1e-10 判定） | PASS |

补充：host 表的 B 类前提（「host 两次舍入」与 device `fma.rn` 是两套 recipe，位级对照须声明同 recipe/同编译形态/同 GPU）已在 `gjk_host`/`ccd_host`/`epa_host` 的 `rounding_notes` 与 `judgment_premise` 中如实声明；`epa_host.face_norm2` 的 head→tail→norm2 与源码逐行一致（复核卡提示的易错点确认写对）。

## 二、门禁有效性（变异实验，2026-11-04 实跑）

基线：`cargo test --locked --test precision_type_gate` **5 passed / 0 failed**（含两条覆盖断言）。

| 变异 | 操作 | 观测结果 |
| --- | --- | --- |
| A 缺表 | 把 `host-tables/epa_host.json` 移出目录 | **FAIL**：`以下 host 站点没有逐字段表（不允许静默缺表）：["epa_host"]`（tests:491-494） |
| B row_id 冒充产品行 | 把 `field-tables/kinetic_energy.json` 复制进 `host-tables/` | **FAIL**：`kinetic_energy.json 的 row_id=kinetic_energy 不在 inventory.hostSiteRegistration.hostCoverage 里`（allowed 集合校验，tests:311-314） |
| C 多表（重复同 row_id） | 复制 `gjk_host.json` → `gjk_host_duplicate.json` | **PASS（未拦截）**：host 测试 `covered` 不去重、不校验表数（tests:466-489） |
| D PENDING_ROWS 清空后缺产品表 | 移出 `field-tables/tendon_wrap.json` | **FAIL**：`以下清单行既没有逐字段表、也不在待批清单里（不允许静默缺表）：["tendon_wrap"]`（tests:422-431；`PENDING_ROWS` 为空 tests:357） |
| E 文件名与 row_id 错配 | 互换 `gjk_host.json`/`ccd_host.json` 的 `row_id` | **PASS（未拦截）**：只校验 row_id 集合覆盖，不校验 `file_stem == row_id` |
| F 字段表目录多余重复表 | 复制 `kinetic_energy.json` → `kinetic_energy_duplicate.json` | **PASS（未拦截）**：field 测试虽 `dedup()`（tests:410），但不校验表数/多余文件 |

结论：**缺表、row_id 冒充产品行、`PENDING_ROWS` 清空后缺产品表都会红**；但「多余/重复表」「文件名↔row_id 不一致」两种污染不设防，README §十 第 118 行「每条必须在 `host-tables/` 下有同名 `row_id` 表」的措辞比实现更严。建议后续（不阻塞本批）收紧：`covered` 去重后 `len == host_sites.len()`、`file_stem() == row_id`；field 侧同样断言表数等于清单行数。

## 三、边界核对

`git show --name-only` 三片提交：B2d 6 文件、B2e 7 文件、B3 6 文件；命中 `src/`、`include/warp-headers/`、`fixtures/` 的路径 **0 个**。三片 `checks` 均为 `unchanged`（src/、include/warp-headers/、fixtures/），`normify validate` 复核时错误 0。表内 `evidence` 路径（如 `tests/gjk_epa.rs`、`g10-p5-gjk-epa/upstream-mjwarp-f32.json`）为仓内既有真实路径。

## 四、口径与时间戳

- README 第 7 行段：「**NP01 仍未验收**——30/30 条产品路径 + 3/3 条 host 站点均有逐字段表（证据面齐全），但 g10-p5 E 指标仍未定稿、host/device 逐表达式位级对照未补；不宣称通过」；第 123 行边界「不宣称 NP01 通过」。无通过性表述。
- B3 卡 note 有「验收仍待：E 指标未定稿、host/device 逐表达式位级对照未补」；**B2d/B2e 卡 note 未写该句**（其 acceptance 有「明确写 NP01 未验收」条款，README 已写）→ 观察 2。
- 时间戳订正：`80932319`/`e5112047`/`6d5069cc` 各只改 1 张 verified 卡（追加 addenda，内容与注册提交一致：`3b301673` 本地 04:39:22、`53054a82` 本地 04:43:02）；复核卡是 proposed 阶段由 `d8c3ff6b` 直接改 `created_at` 字段（真实登记时刻 04:44:34），非 addendum → 观察 4。

## 五、发现分类

**阻断 0 / 需修 0 / 观察 5**

1. **观察 1（门禁强度）**：两条覆盖断言对「重复/多余表」「文件名↔row_id 不一致」不设防（变异 C/E/F 均通过），README §十 L118 措辞夸大；建议补表数与文件名一致性断言。
2. **观察 2（口径落地）**：B2d/B2e 卡 note 缺「NP01 仍未验收（E 指标未定稿、host/device 逐表达式位级对照缺）」；建议 `normify fix` 追加 addendum（不改冻结字段）。
3. **观察 3（表结构一致性）**：`gjk_host` 的 `source.kernels` 列了 `linear_combine`，但表内无独立 kernel 段（并入 `gjk` 段的 accumulations），其定义在 `simplex.rs:298`（`source.files` 已含该文件）→ 建议表内加一句位置说明。
4. **观察 4（措辞）**：「四张 NP01 卡用 normify fix 追加 addenda」实际为 3 张 verified 卡 addenda + 1 张 proposed 复核卡字段订正。
5. **观察 5（机器门禁记录）**：三片卡 checks 仅 `unchanged`，acceptance 提到的 `precision_type_gate`/unit 全绿无 `.normify/runs` 记录；QA 已独立复跑 `precision_type_gate` 5/5，unit 计数引最近记录 run（R06-S2 卡 `unit.json`：1276 passed / 0 failed / 398 ignored，含此前 S1a 基线 1268 + 本次 B2d/B2e/B3 无新增用例）。不属于本批复核缺陷，记录供后续卡参考。

## 六、边界与复现

- 零 GPU、零窗口；未改产品代码/测试/判据/容差/fixtures；本文件与卡状态是唯一写入。
- 复现：`cargo test --locked --test precision_type_gate`（5 用例）；变异实验按第二节表格逐条做（改/移表 → 跑对应覆盖用例 → `git checkout --` 恢复，已确认恢复后 `git status` 干净）。
- 不宣称 NP01 通过、不宣称类型映射率达标、不宣称与上游等价。
