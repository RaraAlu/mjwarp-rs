//! L2 冻结 PTX 来源与摘要准入。
//! L2诊断只加载上游生成的PTX。
//! 摘要核验沿用L2既有常量，不重新签发。
//! 本模块只接纳、不发射；发射见`warp`。

use sha2::{Digest, Sha256};

use super::LauncherError;

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

/// 已通过摘要核验的冻结PTX来源。
#[derive(Clone, Debug)]
pub struct FrozenPtx {
    name: &'static str,
    ptx: String,
    sha256: String,
}

impl FrozenPtx {
    /// 从PTX文本建立来源并核验调用方给定的摘要。
    /// 文本不得为空、不得含NUL。
    pub fn from_ptx(
        name: &'static str,
        ptx: &str,
        expected_sha256: &str,
    ) -> Result<Self, LauncherError> {
        if !valid_name(name) {
            return Err(LauncherError::InvalidSource("冻结PTX名称非法"));
        }
        if ptx.is_empty() || ptx.contains('\0') {
            return Err(LauncherError::InvalidSource("冻结PTX为空或含NUL"));
        }
        let sha256 = digest(ptx.as_bytes());
        if sha256 != expected_sha256 {
            return Err(LauncherError::InvalidSource("冻结PTX摘要不符"));
        }
        Ok(Self {
            name,
            ptx: ptx.to_owned(),
            sha256,
        })
    }

    /// 从PTX文件字节建立来源。
    /// 文件末尾允许一个NUL填充，摘要按原始字节核验。
    pub fn from_file_bytes(
        name: &'static str,
        bytes: &[u8],
        expected_sha256: &str,
    ) -> Result<Self, LauncherError> {
        if !valid_name(name) {
            return Err(LauncherError::InvalidSource("冻结PTX名称非法"));
        }
        let sha256 = digest(bytes);
        if sha256 != expected_sha256 {
            return Err(LauncherError::InvalidSource("冻结PTX摘要不符"));
        }
        let text = bytes.strip_suffix(&[0]).unwrap_or(bytes);
        if text.contains(&0) {
            return Err(LauncherError::InvalidSource("冻结PTX含内嵌NUL"));
        }
        let ptx = std::str::from_utf8(text).map_err(|error| {
            LauncherError::InvalidSource(if error.valid_up_to() == 0 {
                "冻结PTX不是UTF-8"
            } else {
                "冻结PTX非UTF-8尾部"
            })
        })?;
        if ptx.is_empty() {
            return Err(LauncherError::InvalidSource("冻结PTX为空"));
        }
        Ok(Self {
            name,
            ptx: ptx.to_owned(),
            sha256,
        })
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// 已剥离末尾NUL的PTX文本。
    pub fn ptx(&self) -> &str {
        &self.ptx
    }

    /// 原始字节摘要；作为模块缓存键。
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}
