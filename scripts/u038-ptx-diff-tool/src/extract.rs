//! 只读提取：产品 CUDA 常量、上游生成 CUDA 函数、哈希。

use sha2::{Digest, Sha256};
use std::path::Path;

pub fn read_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// 读取 Rust 源里的 `const NAME: &str = r#"..."#;` 原始字符串字面量。
pub fn raw_const(text: &str, name: &str) -> Result<String, String> {
    let marker = format!("const {name}: &str = r#\"");
    let start = text
        .find(&marker)
        .ok_or_else(|| format!("缺少常量 {name}"))?
        + marker.len();
    let rest = &text[start..];
    let end = rest
        .find("\"#;")
        .ok_or_else(|| format!("常量 {name} 未闭合"))?;
    Ok(rest[..end].to_string())
}

/// 从 `signature` 起始处截取到函数体闭合大括号（含）。
pub fn function_block<'a>(text: &'a str, signature: &str) -> Result<&'a str, String> {
    let start = text
        .find(signature)
        .ok_or_else(|| format!("缺少函数签名 {signature}"))?;
    let brace = start
        + text[start..]
            .find('{')
            .ok_or_else(|| format!("{signature} 无函数体"))?;
    let mut depth = 0usize;
    for (offset, ch) in text[brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(&text[start..brace + offset + 1]);
                }
            }
            _ => {}
        }
    }
    Err(format!("{signature} 函数体未闭合"))
}

/// 提取 `[from, to)` 之间的子串；`to` 用另一段已提取文本的结束位置。
pub fn span_between<'a>(text: &'a str, from: &str, through: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or_else(|| format!("缺少起始标记 {from}"))?;
    let through_offset = text
        .find(through)
        .ok_or_else(|| format!("缺少结束标记 {through}"))?;
    let end = through_offset + through.len();
    if end <= start {
        return Err(format!("范围非法 {from}..{through}"));
    }
    Ok(&text[start..end])
}

/// 文件存在性与大小/哈希摘要。
pub fn file_identity(path: &Path) -> Result<(u64, String), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok((bytes.len() as u64, sha256_hex(&bytes)))
}
