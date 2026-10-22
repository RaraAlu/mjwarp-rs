//! NVRTC 编译封装（与产品同一 cudarc/工具链；只编译不跑内核）。

use crate::extract::sha256_hex;
use cudarc::nvrtc::{compile_ptx_with_opts, CompileOptions};

pub struct CompileSpec {
    pub name: String,
    pub source: String,
    pub options: Vec<String>,
    pub fmad: bool,
    pub fast_math: bool,
}

pub struct Compiled {
    pub ptx: String,
    pub source_sha256: String,
    pub ptx_sha256: String,
    pub options: Vec<String>,
}

pub fn compile(spec: &CompileSpec) -> Result<Compiled, String> {
    let ptx = compile_ptx_with_opts(
        &spec.source,
        CompileOptions {
            fmad: Some(spec.fmad),
            use_fast_math: Some(spec.fast_math),
            options: spec.options.clone(),
            name: Some(spec.name.clone()),
            ..Default::default()
        },
    )
    .map_err(|error| format!("NVRTC({}): {error:?}", spec.name))?;
    let text = ptx.to_src();
    Ok(Compiled {
        source_sha256: sha256_hex(spec.source.as_bytes()),
        ptx_sha256: sha256_hex(text.as_bytes()),
        options: spec.options.clone(),
        ptx: text,
    })
}
