//! M1发射边界的静态PTX证据。
//! 样本清单固定；不做全局扫描计数。
//! 静态结果只证明首参数大小与对齐。
//! 字段顺序与运行语义留给M4实测。

/// 一个冻结上游内核样本。
pub struct BoundsSample {
    /// 相对仓库根的`.cu`路径。
    pub cu: &'static str,
    /// 相对仓库根的`.ptx`路径。
    pub ptx: &'static str,
    pub cu_sha256: &'static str,
    pub ptx_sha256: &'static str,
    pub entry: &'static str,
    /// `launch_bounds_t<N>`的`N`。
    pub dims: u8,
}

const CU: &str = "fixtures/gravity-upstream-abi/generated/module.cu";
const PTX: &str = "fixtures/gravity-upstream-abi/generated/module.ptx";
// 与`receipt.json`及`warp_gravity.rs`登记值一致。
const CU_SHA256: &str = "dd9338e9f38feb767273c3e1a445aea54bee3dee7d899076363202a4d26e865d";
const PTX_SHA256: &str = "d06154da63f64118e8fdb19c0b646a59c5b485d580f4716e5edea97a57c1dd70";

const fn sample(entry: &'static str, dims: u8) -> BoundsSample {
    BoundsSample {
        cu: CU,
        ptx: PTX,
        cu_sha256: CU_SHA256,
        ptx_sha256: PTX_SHA256,
        entry,
        dims,
    }
}

/// 该产物中全部1维内核，另加2维与3维对照各一。
/// 3维对照即L2实际运行的重力入口。
pub const SAMPLES: &[BoundsSample] = &[
    sample("_qfrc_adhesion_8d86bfc5_cuda_kernel_forward", 1),
    sample("_eff_contact_build_f1a6f426_cuda_kernel_forward", 1),
    sample("_eff_contact_force_3892154f_cuda_kernel_forward", 1),
    sample(
        "_spring_damper_flexedge_passive_ade4181f_cuda_kernel_forward",
        2,
    ),
    sample("_gravity_force_7c63ac28_cuda_kernel_forward", 3),
];

pub fn read(path: &str) -> Vec<u8> {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read(&full).unwrap_or_else(|error| panic!("{}: {error}", full.display()))
}

/// 从`.cu`读取入口首参数的`launch_bounds_t<N>`。
pub fn cu_bounds_dims(source: &str, entry: &str) -> Option<u8> {
    let start = source.find(&format!("void {entry}("))? + entry.len() + 6;
    let rest = source[start..].trim_start();
    let rest = rest.strip_prefix("wp::launch_bounds_t<")?;
    let (digits, _) = rest.split_once('>')?;
    digits.parse().ok()
}

/// PTX入口参数的声明形式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PtxParam {
    /// `.param .align A .b8 name[B]`：按值结构体。
    Bytes { align: u64, bytes: u64 },
    /// `.param .u8 name`等：按值标量。
    Scalar { bits: u32 },
}

/// 读取入口全部参数声明。
/// 遇到未知形式即返回`None`，不做猜测。
pub fn ptx_params(ptx: &str, entry: &str) -> Option<Vec<PtxParam>> {
    let start = ptx.find(&format!(".entry {entry}("))?;
    let mut params = Vec::new();
    for line in ptx[start..].lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with(')') {
            return Some(params);
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        let param = match words.as_slice() {
            [".param", ".align", align, ".b8", name] => {
                let (_, bytes) = name.trim_end_matches(',').split_once('[')?;
                PtxParam::Bytes {
                    align: align.parse().ok()?,
                    bytes: bytes.strip_suffix(']')?.parse().ok()?,
                }
            }
            [".param", kind, _name] => {
                let bits = kind.trim_start_matches(".").get(1..)?.parse().ok()?;
                if !matches!(&kind[..2], ".u" | ".s" | ".f" | ".b") {
                    return None;
                }
                PtxParam::Scalar { bits }
            }
            _ => return None,
        };
        params.push(param);
    }
    None
}
