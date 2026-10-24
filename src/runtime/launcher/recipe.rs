//! W02：编译配方、用途与指纹。
//! 配方只描述编译选项，不调用编译器。
//! 物理配方取值必须与现行L1路径一致。

use super::LauncherError;

/// 配方用途。
/// 物理内核只接受`Physics`；渲染内核只接受`Render`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecipePurpose {
    /// 产品物理内核。
    Physics,
    /// 产品渲染内核（S7a 起）。
    Render,
    /// 纯参考与诊断内核；结果不得反馈物理路径。
    Diagnostic,
    /// P1路线探针。
    Probe,
}

impl RecipePurpose {
    fn tag(self) -> &'static str {
        match self {
            Self::Physics => "physics",
            Self::Render => "render",
            Self::Diagnostic => "diagnostic",
            Self::Probe => "probe",
        }
    }
}

/// C++语言标准。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CppStd {
    Cpp17,
}

impl CppStd {
    fn flag(self) -> &'static str {
        match self {
            Self::Cpp17 => "--std=c++17",
        }
    }
}

/// NVRTC目标；`Compute(70)`即`compute_70`虚拟架构。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArchTarget {
    Compute(u32),
}

impl ArchTarget {
    pub(super) fn flag(self) -> String {
        match self {
            Self::Compute(sm) => format!("--gpu-architecture=compute_{sm}"),
        }
    }
}

/// 生成PTX后的精度检查。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrecisionGate {
    /// 现行`check_physics_ptx`：拒绝非f32物理运算。
    F32Physics,
    /// 渲染内核f32检查：由`render::kernel`的渲染门禁执行，允许W15头文件自带的
    /// `_svd_config<double>`两条常量声明（不是f32运算），其余非f32仍拒绝。
    F32Render,
    /// 不做精度检查；仅限诊断与探针。
    Unchecked,
}

/// 一份完整编译配方。
/// 字段私有；只能使用下列常量。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompileRecipe {
    name: &'static str,
    purpose: RecipePurpose,
    fmad: bool,
    fast_math: bool,
    std: CppStd,
    arch: ArchTarget,
    defines: &'static [(&'static str, &'static str)],
    gate: PrecisionGate,
}

impl CompileRecipe {
    /// 冻结Warp 1.15物理配方。
    /// 与`transfer::kernel`现行选项逐项一致。
    pub const WARP_1_15_PHYSICS: Self = Self {
        name: "warp-1.15-physics",
        purpose: RecipePurpose::Physics,
        fmad: true,
        fast_math: false,
        std: CppStd::Cpp17,
        arch: ArchTarget::Compute(70),
        defines: &[],
        gate: PrecisionGate::F32Physics,
    };

    /// 诊断配方：编译选项同物理配方，不做f32检查。
    /// 对应现行`compile_checked(.., false)`。
    pub const WARP_1_15_DIAGNOSTIC: Self = Self {
        name: "warp-1.15-diagnostic",
        purpose: RecipePurpose::Diagnostic,
        fmad: true,
        fast_math: false,
        std: CppStd::Cpp17,
        arch: ArchTarget::Compute(70),
        defines: &[],
        gate: PrecisionGate::Unchecked,
    };

    /// 冻结Warp 1.15渲染配方（S7a）。
    ///
    /// 编译选项与物理配方一致（fmad开、fast-math关、C++17、compute_70）；f32门禁由
    /// `render::kernel` 的渲染门禁执行：W15 头树（经 `builtin.h`）固定带两条
    /// `_svd_config<double>` 常量声明，渲染门禁精确放行这两条，其余非 f32 仍拒绝。
    pub const WARP_1_15_RENDER: Self = Self {
        name: "warp-1.15-render",
        purpose: RecipePurpose::Render,
        fmad: true,
        fast_math: false,
        std: CppStd::Cpp17,
        arch: ArchTarget::Compute(70),
        defines: &[],
        gate: PrecisionGate::F32Render,
    };

    /// P1探针配方；对应`cache.rs`的`fmad=false;fastmath=false`。
    pub const P1_PROBE: Self = Self {
        name: "p1-probe",
        purpose: RecipePurpose::Probe,
        fmad: false,
        fast_math: false,
        std: CppStd::Cpp17,
        arch: ArchTarget::Compute(70),
        defines: &[],
        gate: PrecisionGate::Unchecked,
    };

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn purpose(&self) -> RecipePurpose {
        self.purpose
    }

    pub fn fmad(&self) -> bool {
        self.fmad
    }

    pub fn fast_math(&self) -> bool {
        self.fast_math
    }

    pub fn gate(&self) -> PrecisionGate {
        self.gate
    }

    pub fn arch(&self) -> ArchTarget {
        self.arch
    }

    /// NVRTC附加选项；顺序与现行L1路径一致。
    pub fn nvrtc_options(&self) -> Vec<String> {
        let mut options = vec![self.std.flag().to_owned(), self.arch.flag()];
        options.extend(
            self.defines
                .iter()
                .map(|(key, value)| format!("-D{key}={value}")),
        );
        options
    }

    /// 稳定指纹；任一字段变化都会改变指纹。
    pub fn fingerprint(&self) -> String {
        let defines: Vec<String> = self
            .defines
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        format!(
            "recipe-v1;name={};purpose={};fmad={};fast-math={};std={};arch={};gate={};defines={}",
            self.name,
            self.purpose.tag(),
            u8::from(self.fmad),
            u8::from(self.fast_math),
            self.std.flag(),
            self.arch.flag(),
            match self.gate {
                PrecisionGate::F32Physics => "f32-physics",
                PrecisionGate::F32Render => "f32-render",
                PrecisionGate::Unchecked => "unchecked",
            },
            defines.join(",")
        )
    }

    /// 校验配方自身的用途约束。
    pub fn validate(&self) -> Result<(), LauncherError> {
        match self.purpose {
            RecipePurpose::Physics => {
                if self.gate != PrecisionGate::F32Physics {
                    return Err(LauncherError::InvalidRecipe("物理配方必须启用f32检查"));
                }
                if self.fast_math {
                    return Err(LauncherError::InvalidRecipe("物理配方禁止fast-math"));
                }
            }
            RecipePurpose::Render => {
                if self.gate != PrecisionGate::F32Render {
                    return Err(LauncherError::InvalidRecipe("渲染配方必须启用渲染f32检查"));
                }
                if self.fast_math {
                    return Err(LauncherError::InvalidRecipe("渲染配方禁止fast-math"));
                }
            }
            RecipePurpose::Diagnostic | RecipePurpose::Probe => {}
        }
        for (key, value) in self.defines {
            let valid = |text: &str| {
                !text.is_empty() && text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            };
            if !valid(key) || !valid(value) {
                return Err(LauncherError::InvalidRecipe("宏名或取值非法"));
            }
        }
        Ok(())
    }

    /// 内核按用途准入配方；不符即拒绝，不降级。
    pub fn admit(&self, required: RecipePurpose) -> Result<(), LauncherError> {
        self.validate()?;
        if self.purpose != required {
            return Err(LauncherError::RecipePurpose {
                expected: required,
                actual: self.purpose,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 直接读取现行源码，防止配方常量与L1选项漂移。
    const L1_KERNEL: &str = include_str!("../transfer/kernel.rs");
    const L3_CACHE: &str = include_str!("../cache.rs");

    #[test]
    fn physics_recipe_mirrors_current_l1_options() {
        let recipe = CompileRecipe::WARP_1_15_PHYSICS;
        assert!(L1_KERNEL.contains("const PHYSICS_FMAD: bool = true;"));
        assert!(L1_KERNEL.contains("use_fast_math: Some(false),"));
        assert!(L1_KERNEL.contains(
            r#"options: vec!["--std=c++17".into(), "--gpu-architecture=compute_70".into()],"#
        ));
        assert!(recipe.fmad());
        assert!(!recipe.fast_math());
        assert_eq!(
            recipe.nvrtc_options(),
            ["--std=c++17", "--gpu-architecture=compute_70"]
        );
    }

    #[test]
    fn diagnostic_recipe_differs_from_physics_only_in_purpose_and_gate() {
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        let diagnostic = CompileRecipe::WARP_1_15_DIAGNOSTIC;
        // 现行L1以同一选项编译，仅以此开关跳过f32检查。
        assert!(L1_KERNEL.contains("require_f32: bool,"));
        assert_eq!(diagnostic.nvrtc_options(), physics.nvrtc_options());
        assert_eq!(diagnostic.fmad(), physics.fmad());
        assert_eq!(diagnostic.fast_math(), physics.fast_math());
        assert_eq!(diagnostic.gate(), PrecisionGate::Unchecked);
    }

    #[test]
    fn render_recipe_is_frozen_and_admits_only_render() {
        let render = CompileRecipe::WARP_1_15_RENDER;
        render.validate().unwrap();
        assert_eq!(render.name(), "warp-1.15-render");
        assert_eq!(render.purpose(), RecipePurpose::Render);
        assert!(render.fmad());
        assert!(!render.fast_math());
        assert_eq!(render.gate(), PrecisionGate::F32Render);
        // 编译选项与物理配方逐项一致（只有用途与门禁不同）。
        assert_eq!(
            render.nvrtc_options(),
            CompileRecipe::WARP_1_15_PHYSICS.nvrtc_options()
        );
        assert!(render.admit(RecipePurpose::Render).is_ok());
        assert!(matches!(
            render.admit(RecipePurpose::Physics),
            Err(LauncherError::RecipePurpose { .. })
        ));
        assert!(matches!(
            CompileRecipe::WARP_1_15_DIAGNOSTIC.admit(RecipePurpose::Render),
            Err(LauncherError::RecipePurpose { .. })
        ));
        // 渲染配方不得降级到无检查或 fast-math。
        let mut invalid = CompileRecipe::WARP_1_15_RENDER;
        invalid.gate = PrecisionGate::Unchecked;
        assert!(matches!(
            invalid.validate(),
            Err(LauncherError::InvalidRecipe(_))
        ));
        let mut invalid = CompileRecipe::WARP_1_15_RENDER;
        invalid.fast_math = true;
        assert!(invalid.validate().is_err());
        // 指纹区分用途/门禁。
        assert_ne!(
            render.fingerprint(),
            CompileRecipe::WARP_1_15_PHYSICS.fingerprint()
        );
        assert!(render.fingerprint().contains("purpose=render"));
        assert!(render.fingerprint().contains("gate=f32-render"));
    }

    #[test]
    fn probe_recipe_mirrors_cache_recipe_flags() {
        assert!(L3_CACHE.contains("fmad=false;fastmath=false"));
        let probe = CompileRecipe::P1_PROBE;
        assert!(!probe.fmad());
        assert!(!probe.fast_math());
    }

    #[test]
    fn rejects_invalid_physics_recipes() {
        let mut recipe = CompileRecipe::WARP_1_15_PHYSICS;
        recipe.gate = PrecisionGate::Unchecked;
        assert!(matches!(
            recipe.validate(),
            Err(LauncherError::InvalidRecipe(_))
        ));
        let mut recipe = CompileRecipe::WARP_1_15_PHYSICS;
        recipe.fast_math = true;
        assert!(matches!(
            recipe.admit(RecipePurpose::Physics),
            Err(LauncherError::InvalidRecipe(_))
        ));
        let mut recipe = CompileRecipe::WARP_1_15_DIAGNOSTIC;
        recipe.defines = &[("BAD KEY", "1")];
        assert!(recipe.validate().is_err());
    }

    #[test]
    fn defines_enter_options_and_fingerprint() {
        let mut recipe = CompileRecipe::WARP_1_15_DIAGNOSTIC;
        let before = recipe.fingerprint();
        recipe.defines = &[("WP_TILE", "64")];
        assert!(recipe.validate().is_ok());
        assert_eq!(recipe.nvrtc_options().last().unwrap(), "-DWP_TILE=64");
        assert_ne!(recipe.fingerprint(), before);
    }
}
