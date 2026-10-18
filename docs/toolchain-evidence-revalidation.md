# 证据复核与六项诊断治理

日期：2026-10-18。
任务：`2026-10-18-toolchain-evidence-revalidation`。
基线提交：`25186895d187aa8a9a4068e09e828a340163589d`。
平台：Windows，RTX 4070 Ti SUPER，驱动596.36。
工具链：rustc 1.99.0，NVRTC 12.8。
本项不覆盖NP01至NP04。
本项不构成产品精度验收。

## 结论

六项暂缓诊断已经清零。
19项宿主失败中18项通过。
缓存集合一项保留失败。
旧证据先归档，再更新身份。
映射变化处已重建并实测。
GPU结果与内核摘要前后一致。
50条历史记录中45条完成复验。
另5条因既有GPU失败未复验。

| 门禁 | 基线 | 当前 |
| --- | ---: | ---: |
| 默认`clippy --all-targets -D warnings` | 4处暂缓项 | 0处 |
| `cuda-probe`同上 | 3处暂缓项，另4处被遮蔽 | 4处，均在范围外 |
| 默认宿主测试 | 303过、0败 | 303过、0败 |
| `cuda-probe`宿主测试 | 437过、19败 | 455过、1败 |
| GPU程序 | 68个，27项既有失败 | 68个，同样27项 |
| 内核摘要 | 279组 | 279组，逐项相同 |

`cuda-probe`全局静态门禁仍未转绿。
四项集成诊断归续接任务。

## 摘要口径

源码身份采用整文件SHA-256。
计算前先把CRLF换成LF。
口径与各守卫实现一致。
选择器自身摘要按原字节计算。
重力包也按原字节计算。

## 漂移源头

五份源码各有一个旧版本。
差异从固定版本算到HEAD。

| 源码 | 固定于 | 差异 | 判定 |
| --- | --- | --- | --- |
| `src/physics/resident.rs` | `791bcc3` | +477/-1，新增借用接口与测试 | 映射成立 |
| `src/physics/mod.rs` | `af7f887` | +71/-7，模块导出与旋转分组 | 映射变化 |
| `src/runtime/transfer.rs` | `e9173d7` | +5，同会话判定 | 映射成立 |
| `src/io/inertial.rs` | `a08ce06` | +5，稀疏子模块导出 | 映射成立 |
| `src/model/inertial.rs` | `a08ce06` | +3，稀疏子模块导出 | 映射成立 |

`resident.rs`没有改动内核字符串。
它也没有改动打包器与类型。
临时换摘要后，15项映射检查通过。
剩余3项来自旋转分组修复。

## 映射重建

### 旋转步骤

`8a5bac0`改写了刚体`rotate`。
旧锚点因此不再存在。
新写法逐算子对应上游。

| 上游`rot_vec_quat` | 产品`rotate` |
| --- | --- |
| `wp.dot(u, vec) * u`，`mul(s,a)=a[i]*s` | `scale(u,dot_v(u,v))` |
| `2.0 * (...)` | `scale(...,2.0f)` |
| `s * s - wp.dot(u, u)` | `q.w*q.w-dot_v(u,u)` |
| `2.0 * s * wp.cross(u, vec)` | `scale(cross_v(u,v),2.0f*q.w)` |

上游依据为Warp 1.15.0的`vec.h`。
`dot`与`cross`的展开次序相同。
两份清单的分类改为`same-scalar`。
锚点改为新写法的三行。
测试常量同步更新。
机器码融合本项未逐条比较。

### 零四元数溯源

原检查要求整文件还原到`595d57…`。
旋转修复后该还原不可达。
现改为两段已测溯源。

1. 逆变换旋转修复。
   整文件须等于`c74aede`版本。
   摘要为`71ee8dd5…`。
2. 再逆变换零四元数分支。
   刚体模板须等于历史捕获。
   该捕获的PTX为`7d1db8eb…`。

第二段只比较刚体模板。
`c74aede`与`af7f887`之间只差模块导出。
模板在两版之间逐字相同。
逆变换模块拒绝其他漂移。

## 六项诊断

| 位置 | 规则 | 处置 |
| --- | --- | --- |
| `gravity_compensation.rs:108` | `needless_range_loop` | 改用`iter_mut().enumerate().skip(1)` |
| `gravity_compensation.rs:646` | `unused_imports` | 两项导入改为`cuda-probe`条件 |
| `gravity_compensation.rs:686` | `dead_code` | 辅助模型改为`cuda-probe`条件 |
| `gravity_compensation.rs:836` | `useless_vec` | 改用定长数组 |
| `resident.rs:1655` | `dead_code` | 一项改条件编译，一项窄范围豁免 |
| `resident.rs:5718` | `needless_range_loop` | 改用`enumerate` |

`passive_sleep_context`唯一调用者在范围外。
该调用者本身带同样豁免。
本项没有修改内核字符串。
本项没有修改编译选项。

## 重力证据包

改码前已归档原包。
改码后包内两项源码摘要更新。
测试内常量同步更新。
随后真实执行一次GPU链。

| 对比项 | `gpu-2` | `gpu-3` |
| --- | --- | --- |
| 比较值 | 1134 | 1134 |
| 位差 | 0 | 0 |
| 五份原始位 | — | 逐项相同 |
| 实际PTX入口 | 8 | 8，摘要相同 |
| 耗时 | — | 4.6秒 |

旧`gpu-2`保持原样。
新证据位于`gpu-3`。

## GPU对比

候选状态包含两受保护文件。
68个程序逐个执行`--ignored`。
记录器捕获每次编译摘要。

| 对比项 | 治理基线 | 候选 |
| --- | ---: | ---: |
| 程序数 | 68 | 68 |
| 计数、失败名或退出码不同 | — | 0 |
| 内核摘要 | 279 | 279，逐项相同 |
| 既有失败 | 27项，10个程序 | 相同27项 |
| 累计耗时 | 341.1秒 | 369.1秒 |
| 构建耗时 | — | 12.8秒 |

耗时只作记录，不作性能结论。
27项失败未转为通过。
失败日志另存到`gpu-candidate-logs/`。

## 50条历史记录

旧证据文件保持原样。
继任关系见`evidence-succession.json`。
14个源文件的内核字符串未变。
各组测试在当前源码上实际执行。

| 状态 | 条数 | 说明 |
| --- | ---: | --- |
| 已复验 | 45 | 组内宿主与GPU测试全部通过 |
| 未复验 | 5 | applied-force四条，resident-mixed-mass一条 |

未复验项对应既有GPU失败。
失败名与治理基线相同。

## 清洁检出

工作区设置`core.autocrlf=true`。
临时提交不进入分支。

| 检出 | 失败 |
| --- | ---: |
| HEAD | 30 |
| 候选，未补换行属性 | 20 |
| 候选，补换行属性后 | 16 |

本项补充四条LF属性。
它们覆盖两份L2源码与d04输入。
剩余16项缺少忽略目录内容。
缺少的是`target/mjwarp-parallel`。
该目录含冻结上游与Warp头文件。
清洁部署仍须先准备该目录。

## 缓存集合

该项保留失败。
外层三份文件与清单吻合。
内层`1.15.0/1.15.0/`另有三份。
内层字节与外层不同。
内层注释指向`upstream`检出。
外层注释指向`venv`安装。
来源仍未查清。
本项没有删除或扩充清单。

## 转为历史的记录

以下记录固定了旧选择器摘要：

- `fixtures/precision/g01-input-audit.json`
- `fixtures/precision/g01-packing-routes-audit.json`
- `docs/evidence/t4-g03-retest/flex-internal/audit.json`
- `docs/evidence/t4-g03-retest/flex-matrices/audit.json`
- `docs/evidence/kinetic-energy/precall-source.json`

没有测试强制它们。
本项保持原样，不再改写。

## 验证

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| 五个目标与重力链宿主测试 | 18项恢复，缓存一项失败 |
| 默认`cargo test --locked --no-fail-fast` | 303过、0败 |
| `cuda-probe`同上 | 455过、1败、237忽略 |
| 默认`clippy --all-targets -D warnings` | 0处 |
| `cuda-probe`同上 | 4处，均在范围外 |
| 重力非零链GPU | 1134项、0位差 |
| 68个GPU程序 | 与基线一致 |
| 清洁检出宿主测试 | 16项缺失依赖 |

## 未覆盖项

- 缓存集合来源未查清。
- 清洁部署须预置忽略目录。
- 四项集成诊断归续接任务。
- 5条历史记录未复验。
- release配置未运行。
- Linux未运行。
- 旋转融合未逐条比较机器码。
- NP01至NP04未验收。

## 证据文件

位于`docs/development-evidence/toolchain-evidence-revalidation/`：

| 文件 | 内容 |
| --- | --- |
| `archive/` | 改前选择器与重力包原字节 |
| `impact-inventory.json` | 漂移源、差异与临时实测 |
| `review-19.json` | 19项逐项复核表 |
| `evidence-succession.json` | 50条记录继任关系 |
| `baseline-host-19.log` | HEAD上的19项失败 |
| `probe-repin-head-host-19.log` | 临时换摘要实测 |
| `candidate-host-*.log` | 候选宿主测试 |
| `candidate-clippy-*.log` | 候选静态检查 |
| `gravity-gpu-3.log` | 重力链GPU执行 |
| `gpu-candidate-*` | GPU结果、摘要与失败日志 |
| `clean-checkout-*.log` | 三次清洁检出 |
