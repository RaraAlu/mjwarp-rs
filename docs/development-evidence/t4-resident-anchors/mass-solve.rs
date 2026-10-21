// 质量求解模块根拆分后的源码组合锚点。
// 顺序固定：模块根在前，随后是承载同步内核常量的 cuda 子模块。
// 该锚点只服务内核模板解析；产品身份哈希在身份重钉任务中处理。
concat!(
    include_str!("../../../src/physics/mass_solve.rs"),
    "\n",
    include_str!("../../../src/physics/mass_solve/cuda.rs"),
)
