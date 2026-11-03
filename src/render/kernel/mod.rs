//! 渲染内核的编译精度门禁与入口（渲染/BVH S7a）。
//!
//! 只做编译期 f32 门禁与发射入口；像素内核属 S7b。渲染内核必须经
//! [`launch::compile_render_artifact`] / [`launch::RenderKernel::compile`] 编译，
//! 这两个入口是唯一路径，都会执行 [`check_render_ptx`]（`PrecisionGate::F32Render`）。
//!
//! ## 为什么渲染门禁与物理门禁不同
//!
//! 渲染内核必须包含 W15 原样复制的 `builtin.h`（`array.h`/`bvh.h`/`texture.h` 都包含
//! 它），而 `builtin.h` 会带入 `svd.h`。W15 的 `svd.h` 固定声明
//! `_svd_config<double>` 的两条 `static constexpr double` 成员，NVRTC 会为它们生成
//! 两条 `.global .align 8 .f64` 常量：
//!
//! ```text
//! .global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;
//! .global .align 8 .f64 _ZN2wp11_svd_configIdE17QR_GIVENS_EPSILONE = 0d3D719799812DEA11;
//! ```
//!
//! 它们是 W15 头文件自带的**死数据常量**，不被任何 f32 运算使用，也无法通过写内核源码
//! 避免（`builtin.h` 是无条件包含）。因此渲染门禁**精确放行且仅放行这两条**
//! （符号名 + 字面量 + 行尾逐字节匹配、各只允许一次），这是精确放行而不是容差放宽：
//! 其他任何 f64 全局声明、f64 算术、f16/bf16/tf32 一律拒绝。
//!
//! `.reg .f64` 仍按物理门禁口径：只接受 CUDA-12.8 `sinf`/`cosf` 整数角归约的
//! `cvt.rn.f64.s64` + `mul.rn.f64`(0d3BF921FB54442D19) + `cvt.rn.f32.f64` 三元组。

pub mod cast;
pub mod launch;
pub mod megakernel;
mod megakernel_aa;
mod megakernel_arrays;
pub mod output;
pub mod per_world;
pub mod renderer;
mod world_arrays;
pub mod world_camera;

pub use megakernel::MegakernelRenderer;
pub use megakernel_arrays::{LightDataPose, light_data_poses};
pub use per_world::{reject_multi_sample, render_per_world};
pub use renderer::DepthSegRenderer;

use crate::runtime::launcher::{
    BlockRule, LaunchDims, LauncherError, PrecisionGate, RenderArg, RenderArgKind, RenderElem,
    RenderScalarKind, RenderSignature,
};

/// S7b1 深度/分割内核的入口名。
pub const DEPTH_SEG_ENTRY: &str = "render_depth_seg";

/// S7b2 全签名渲染内核的入口名（81 槽，S7a `MEGAKERNEL_SLOTS` 冻结签名）。
pub const MEGAKERNEL_ENTRY: &str = "render_megakernel";

/// S7b2 全签名渲染内核源码：S2b `scene.cu`（只读复用）+ S3 mesh 数学副本
/// （`render_math.cu`，含球面对齐）+ 公共部分（`render_common.cu`）+ R09 阴影
/// （`render_shadow.cu`，方案 A 复用 first-hit `cast_ray`）+ 着色
/// （`render_shade.cu`）+ 81 槽内核本体（`render_megakernel.cu`）。
///
/// 拼接在编译期完成，产物键覆盖全部片段；不改 `src/render/bvh/**`。
pub const MEGAKERNEL_SOURCE: &str = concat!(
    include_str!("../bvh/cuda/scene.cu"),
    include_str!("cuda/render_math.cu"),
    include_str!("cuda/render_common.cu"),
    include_str!("cuda/render_shadow.cu"),
    include_str!("cuda/render_shade.cu"),
    include_str!("cuda/render_megakernel.cu"),
);

/// S7b3 AA 解析内核的入口名（`samples_per_pixel > 1` 时每帧在全部 sample 之后发射一次）。
pub const AA_RESOLVE_ENTRY: &str = "aa_resolve";

/// S7b3 AA 解析内核源码：独立拼接（不含 scene.cu / 着色链），只依赖 `builtin.h`
/// 与自带的 pack 等价实现（见该文件头注释）。
pub const AA_RESOLVE_SOURCE: &str = include_str!("cuda/aa_resolve.cu");

/// `aa_resolve` 的 4 槽签名（1 bounds + 3 数组）：`aa_accum`(vec3)、`rgb_out`(u32)、
/// `inv_n`(1 元素 f32；S7a 标量槽只有 bool/i32/u64，f32 走数组)。
pub static AA_RESOLVE_SLOTS: [RenderArgKind; 4] = {
    use RenderArgKind as K;
    use RenderElem as E;
    [
        K::Bounds2D,
        K::Array(E::Vec3),
        K::Array(E::U32),
        K::Array(E::F32),
    ]
};

/// S7b1 渲染内核源码：S2b `scene.cu`（primitive 数学，只读复用）+ S3 mesh 数学的
/// 逐式复制（`cuda/render_math.cu`）+ 组合/内核本体（`cuda/render.cu`）。
///
/// 拼接在编译期完成：NVRTC 只看到一份源码，产物键覆盖三段内容；不改 `src/render/bvh/**`。
pub const DEPTH_SEG_SOURCE: &str = concat!(
    include_str!("../bvh/cuda/scene.cu"),
    include_str!("cuda/render_math.cu"),
    include_str!("cuda/render.cu"),
);

/// `render_depth_seg` 的 32 槽签名（1 bounds + 24 数组 + 7 标量），顺序为上游
/// `_render_megakernel` 子集槽序（证据给「子集槽 → 上游槽索引」映射；81 槽全签名迁移归 S7b2）。
pub static DEPTH_SEG_SLOTS: [RenderArgKind; 32] = {
    use RenderArgKind as K;
    use RenderElem as E;
    use RenderScalarKind as S;
    [
        K::Bounds2D,
        K::Array(E::I32),
        K::Array(E::F32),
        K::Array(E::I32),
        K::Array(E::F32),
        K::Array(E::Vec2I),
        K::Array(E::I32),
        K::Array(E::Vec4),
        K::Array(E::I32),
        K::Array(E::Vec4),
        K::Array(E::I32),
        K::Array(E::F32),
        K::Array(E::Vec2),
        K::Array(E::Vec4),
        K::Array(E::Vec3),
        K::Array(E::Mat33),
        K::Array(E::Vec2I),
        K::Array(E::I32),
        K::Array(E::I32),
        K::Array(E::Bool),
        K::Array(E::Bool),
        K::Array(E::Vec3),
        K::Array(E::Vec3),
        K::Array(E::F32),
        K::Array(E::Vec2I),
        K::Scalar(S::I32),
        K::Scalar(S::I32),
        K::Scalar(S::I32),
        K::Scalar(S::I32),
        K::Scalar(S::I32),
        K::Scalar(S::Bool),
        K::Scalar(S::Bool),
    ]
};

/// S7b1 深度/分割渲染内核：编译经 A′ 渲染门禁（`compile_render_artifact`），无旁路。
pub struct DepthSegKernel {
    kernel: launch::RenderKernel,
}

impl DepthSegKernel {
    /// 编译 + PTX 门禁 + 加载入口（需要 NVIDIA 驱动与 NVRTC）。
    pub fn compile(session: &crate::runtime::TransferSession) -> Result<Self, LauncherError> {
        let signature =
            RenderSignature::new(DEPTH_SEG_ENTRY, BlockRule::WarpGrid, &DEPTH_SEG_SLOTS)?;
        let kernel = launch::RenderKernel::compile(
            session,
            DEPTH_SEG_ENTRY,
            DEPTH_SEG_SOURCE,
            signature,
            DEPTH_SEG_ENTRY,
        )?;
        Ok(Self { kernel })
    }

    /// 已编译内核（PTX/哈希与模块缓存证据）。
    pub fn kernel(&self) -> &launch::RenderKernel {
        &self.kernel
    }

    /// 按签名校验后同步发射。
    pub fn launch(
        &self,
        session: &crate::runtime::TransferSession,
        args: &[RenderArg],
        dims: LaunchDims,
    ) -> Result<(), LauncherError> {
        self.kernel.launch(session, args, dims)
    }
}

/// 渲染门禁的精度标签（写进回执与证据）。
pub const RENDER_PRECISION_GATE: &str = "f32-render";

/// W15 `svd.h` 自带的 `_svd_config<double>` 常量声明（精确放行清单，仅此两条）。
pub const SVD_DOUBLE_CONSTANTS: [&str; 2] = [
    ".global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;",
    ".global .align 8 .f64 _ZN2wp11_svd_configIdE17QR_GIVENS_EPSILONE = 0d3D719799812DEA11;",
];

/// CUDA-12.8 `sinf`/`cosf` 整数角归约常量（与物理门禁一致）。
const SIN_COS_REDUCTION_CONSTANT: &str = "0d3BF921FB54442D19";

/// `WARP_1_15_RENDER` 的门禁必须是 `F32Render`，且只能由本模块的入口编译。
pub fn render_recipe_gate() -> PrecisionGate {
    crate::runtime::launcher::CompileRecipe::WARP_1_15_RENDER.gate()
}

/// 渲染 PTX 门禁错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderPtxError {
    /// 缺少声明的入口符号。
    MissingEntry { entry: &'static str },
    /// 出现被禁止的精度 token。
    ForbiddenToken {
        line: usize,
        token: &'static str,
        code: String,
    },
    /// `.reg .f64` 存在但没有合法的 sin/cos 归约三元组解释。
    UnaccountedF64Register { line: usize },
    /// f64 全局声明不是清单里的那两条（符号或字面量不符，或多出第三条）。
    UnexpectedF64Global { line: usize, code: String },
    /// 清单内的常量声明重复出现。
    DuplicateSvdConstant { line: usize, code: String },
    /// 清单内的常量声明出现在非 `.global` 行。
    MisplacedSvdConstant { line: usize, code: String },
}

impl std::fmt::Display for RenderPtxError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEntry { entry } => write!(formatter, "缺少入口 {entry}"),
            Self::ForbiddenToken { line, token, code } => {
                write!(formatter, "第{line}行禁止{token}：{code}")
            }
            Self::UnaccountedF64Register { line } => {
                write!(formatter, "第{line}行的.reg .f64没有sin/cos归约解释")
            }
            Self::UnexpectedF64Global { line, code } => {
                write!(formatter, "第{line}行f64全局声明不在放行清单：{code}")
            }
            Self::DuplicateSvdConstant { line, code } => {
                write!(formatter, "第{line}行放行常量重复：{code}")
            }
            Self::MisplacedSvdConstant { line, code } => {
                write!(formatter, "第{line}行放行常量位置错误：{code}")
            }
        }
    }
}

impl std::error::Error for RenderPtxError {}

/// 渲染 f32 门禁：只放行 W15 的两条 `_svd_config<double>` 常量声明。
pub fn check_render_ptx(ptx: &str, entry: &'static str) -> Result<(), RenderPtxError> {
    let lines: Vec<&str> = ptx
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code).trim())
        .filter(|line| !line.is_empty())
        .collect();
    let operands = |line: &str| -> Vec<String> {
        line.split(|c: char| c.is_ascii_whitespace() || c == ',' || c == ';')
            .filter(|word| !word.is_empty())
            .map(str::to_owned)
            .collect()
    };
    let mut allowed_seen = [false; 2];
    let mut reduction_count = 0;
    let mut declarations: Option<usize> = None;
    let mut i = 0;
    while i < lines.len() {
        let code = lines[i];
        let line = i + 1;
        if code.starts_with(".reg .f64 ") || code.starts_with(".reg .f64\t") {
            declarations.get_or_insert(line);
            i += 1;
            continue;
        }
        let a = operands(code);
        if a.len() == 3 && a[0] == "cvt.rn.f64.s64" && i + 2 < lines.len() {
            let b = operands(lines[i + 1]);
            let c = operands(lines[i + 2]);
            if b.len() == 4
                && matches!(b[0].as_str(), "mul.rn.f64" | "mul.f64")
                && b[2] == a[1]
                && b[3] == SIN_COS_REDUCTION_CONSTANT
                && c.len() == 3
                && c[0] == "cvt.rn.f32.f64"
                && c[2] == b[1]
            {
                reduction_count += 1;
                i += 3;
                continue;
            }
        }
        if code.contains(".f64 ") || code.contains(".f64;") || code.ends_with(".f64") {
            // f64 只允许两种形态：精确放行的两条常量声明，或 sin/cos 归约被消费的行。
            if code.starts_with(".global") {
                if let Some(index) = SVD_DOUBLE_CONSTANTS
                    .iter()
                    .position(|allowed| *allowed == code)
                {
                    if allowed_seen[index] {
                        return Err(RenderPtxError::DuplicateSvdConstant {
                            line,
                            code: code.to_owned(),
                        });
                    }
                    allowed_seen[index] = true;
                    i += 1;
                    continue;
                }
                return Err(RenderPtxError::UnexpectedF64Global {
                    line,
                    code: code.to_owned(),
                });
            }
            if SVD_DOUBLE_CONSTANTS.contains(&code) {
                return Err(RenderPtxError::MisplacedSvdConstant {
                    line,
                    code: code.to_owned(),
                });
            }
            return Err(RenderPtxError::UnexpectedF64Global {
                line,
                code: code.to_owned(),
            });
        }
        for token in code
            .split_whitespace()
            .flat_map(|word| word.split('.').skip(1))
        {
            let token = token.trim_matches(|c: char| !c.is_ascii_alphanumeric());
            if matches!(token, "f16" | "f16x2" | "bf16" | "bf16x2" | "tf32") {
                return Err(RenderPtxError::ForbiddenToken {
                    line,
                    token: "f16",
                    code: code.to_owned(),
                });
            }
        }
        i += 1;
    }
    if let Some(line) = declarations
        && reduction_count == 0
    {
        return Err(RenderPtxError::UnaccountedF64Register { line });
    }
    if !ptx.contains(&format!(".visible .entry {entry}")) {
        return Err(RenderPtxError::MissingEntry { entry });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOTH: &str = concat!(
        ".global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;\n",
        ".global .align 8 .f64 _ZN2wp11_svd_configIdE17QR_GIVENS_EPSILONE = 0d3D719799812DEA11;\n",
    );

    fn ptx(body: &str) -> String {
        format!(".visible .entry probe_kernel() {{\n{body}\n}}\n")
    }

    fn check(body: &str) -> Result<(), RenderPtxError> {
        check_render_ptx(&ptx(body), "probe_kernel")
    }

    #[test]
    fn accepts_exactly_the_two_w15_constants() {
        assert_eq!(check(BOTH), Ok(()));
        assert_eq!(check_render_ptx(&ptx(""), "probe_kernel"), Ok(()));
        assert_eq!(
            check(
                ".global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;"
            ),
            Ok(())
        );
    }

    #[test]
    fn rejects_a_third_unknown_f64_global() {
        let body = format!(
            "{BOTH}.global .align 8 .f64 _ZN2wp11_other_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;\n"
        );
        assert!(matches!(
            check(&body),
            Err(RenderPtxError::UnexpectedF64Global { .. })
        ));
    }

    #[test]
    fn rejects_symbol_off_by_one_character() {
        let body =
            ".global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILON = 0d3D719799812DEA11;";
        assert!(matches!(
            check(body),
            Err(RenderPtxError::UnexpectedF64Global { .. })
        ));
    }

    #[test]
    fn rejects_changed_literal() {
        let body =
            ".global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEB11;";
        assert!(matches!(
            check(body),
            Err(RenderPtxError::UnexpectedF64Global { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_or_misplaced_allowlisted_constant() {
        let duplicated = format!("{BOTH}{}", SVD_DOUBLE_CONSTANTS[0]);
        assert!(matches!(
            check(&duplicated),
            Err(RenderPtxError::DuplicateSvdConstant { .. })
        ));
        let misplaced =
            ".const .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;";
        assert!(matches!(
            check(misplaced),
            Err(RenderPtxError::UnexpectedF64Global { .. })
        ));
    }

    #[test]
    fn rejects_f64_registers_without_sin_cos_reduction() {
        assert!(matches!(
            check(".reg .f64 %rd1;"),
            Err(RenderPtxError::UnaccountedF64Register { .. })
        ));
        let reduction = concat!(
            ".reg .f64 %rd1;\n",
            "cvt.rn.f64.s64 %fd1, %rd1;\n",
            "mul.rn.f64 %fd2, %fd1, 0d3BF921FB54442D19;\n",
            "cvt.rn.f32.f64 %f1, %fd2;\n",
        );
        assert_eq!(check(reduction), Ok(()));
    }

    #[test]
    fn rejects_f64_arithmetic_f16_and_missing_entry() {
        assert!(matches!(
            check("mul.f64 %fd1, %fd2, %fd3;"),
            Err(RenderPtxError::UnexpectedF64Global { .. })
        ));
        assert!(matches!(
            check(".reg .f16 %h<4>;"),
            Err(RenderPtxError::ForbiddenToken { .. })
        ));
        assert!(matches!(
            check_render_ptx(
                ".global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;",
                "probe_kernel"
            ),
            Err(RenderPtxError::MissingEntry { .. })
        ));
    }

    #[test]
    fn depth_seg_kernel_has_expected_abi_and_reuses_s2b_math() {
        assert_eq!(DEPTH_SEG_SLOTS.len(), 32);
        assert_eq!(DEPTH_SEG_SLOTS[0], RenderArgKind::Bounds2D);
        let arrays = DEPTH_SEG_SLOTS
            .iter()
            .filter(|slot| matches!(slot, RenderArgKind::Array(_)))
            .count();
        let scalars = DEPTH_SEG_SLOTS
            .iter()
            .filter(|slot| matches!(slot, RenderArgKind::Scalar(_)))
            .count();
        assert_eq!((arrays, scalars), (24, 7), "渲染 ABI 槽位种类");
        assert!(DEPTH_SEG_SOURCE.contains("extern \"C\" __global__ void render_depth_seg("));
        // S2b primitive 数学与 S3 mesh 数学都在同一编译单元内（复用而非手抄漂移）。
        assert!(DEPTH_SEG_SOURCE.contains("scene_bvh_ray"));
        assert!(DEPTH_SEG_SOURCE.contains("mesh_query_ray"));
        assert!(DEPTH_SEG_SOURCE.contains("value[1] = 5;"));
    }

    #[test]
    fn render_recipe_uses_f32_render_gate() {
        assert_eq!(render_recipe_gate(), PrecisionGate::F32Render);
        assert_eq!(RENDER_PRECISION_GATE, "f32-render");
        assert_eq!(SVD_DOUBLE_CONSTANTS.len(), 2);
    }
}
