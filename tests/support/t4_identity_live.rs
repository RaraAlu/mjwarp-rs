//! T4 身份活体校验支撑：默认特性、纯 CPU、无 GPU/NVRTC/外部依赖。
//!
//! 口径（与既有 `t4_rigid_capture` / `t4_evidence` 一致）：
//! - 单文件身份 = `sha256(LF 归一化字节)`（CRLF → LF）；
//! - `src/physics/mod.rs` 是 **physics_core 组合摘要**：锚点
//!   `docs/development-evidence/t4-resident-anchors/physics.rs` 的 5 个 `include_str!`
//!   内容按 `\n` 连接后做 LF 归一化再 sha256（顺序：mod / com_position / device_checks /
//!   history / kinematics）；
//! - 期望值只来自 `fixtures/precision/t4-current-kernels.json`，本模块不复制任何期望值。
//!
//! 反向判别 seam：`T4_IDENTITY_LIVE_FIXTURE` 环境变量可覆盖 fixture 路径（测试专用，
//! 用于在临时副本上篡改期望并验证测试确实失败；默认不设置时用仓库内 fixture）。

use std::path::{Path, PathBuf};

/// fixture `product_sources` 的规范路径集合（20 条）。
pub const EXPECTED_PATHS: [&str; 20] = [
    "fixtures/mjwarp-mass-solve/manifest.json",
    "src/physics/attached.rs",
    "src/physics/camlight.rs",
    "src/physics/com_velocity.rs",
    "src/physics/fixed_tendon.rs",
    "src/physics/flex_edge.rs",
    "src/physics/flex_face.rs",
    "src/physics/flex_hessian.rs",
    "src/physics/flex.rs",
    "src/physics/mass_matrix.rs",
    "src/physics/mass_solve.rs",
    "src/physics/mod.rs",
    "src/physics/resident.rs",
    "src/physics/rne_bias.rs",
    "src/physics/sleep.rs",
    "src/physics/spatial_tendon.rs",
    "src/physics/tendon_wrap.rs",
    "src/physics/tendon.rs",
    "src/runtime/transfer.rs",
    "src/runtime/transfer/kernel.rs",
];

/// physics_core 组合摘要的 5 个成员（锚点 `include_str!` 顺序）。
pub const PHYSICS_CORE_PATHS: [&str; 5] = [
    "src/physics/mod.rs",
    "src/physics/com_position.rs",
    "src/physics/device_checks.rs",
    "src/physics/history.rs",
    "src/physics/kinematics.rs",
];

/// fixture 中一条 `product_sources` 记录。
#[derive(Clone, Debug)]
pub struct Entry {
    pub path: String,
    pub sha256: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 测试专用 seam：`T4_IDENTITY_LIVE_FIXTURE` 覆盖 fixture 路径。
pub fn fixture_path() -> PathBuf {
    std::env::var_os("T4_IDENTITY_LIVE_FIXTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("fixtures/precision/t4-current-kernels.json"))
}

/// 读取并解析 fixture 的 `product_sources`（失败即 panic，带路径与原因）。
pub fn load_fixture() -> Vec<Entry> {
    let path = fixture_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读取 T4 身份 fixture 失败 {}: {error}", path.display()));
    parse_product_sources(&text)
        .unwrap_or_else(|error| panic!("解析 T4 身份 fixture 失败 {}: {error}", path.display()))
}

/// 在 fixture JSON 文本中定位 `product_sources` 数组并解出 `{path, sha256}` 列表。
pub fn parse_product_sources(text: &str) -> Result<Vec<Entry>, String> {
    let key = text
        .find("\"product_sources\"")
        .ok_or_else(|| "缺少 product_sources 键".to_string())?;
    let open = key
        + text[key..]
            .find('[')
            .ok_or_else(|| "product_sources 后缺少数组".to_string())?;
    let bytes = text.as_bytes();
    let mut index = open + 1;
    let mut entries = Vec::new();
    loop {
        skip_whitespace(bytes, &mut index);
        match bytes.get(index) {
            Some(b']') => break,
            Some(b',') => index += 1,
            Some(b'{') => {
                let (entry, next) = parse_entry(bytes, index)?;
                entries.push(entry);
                index = next;
            }
            Some(byte) => {
                return Err(format!(
                    "product_sources 数组位置 {index} 出现意外字节 0x{byte:02x}"
                ));
            }
            None => return Err("product_sources 数组未闭合".to_string()),
        }
    }
    Ok(entries)
}

fn parse_entry(bytes: &[u8], start: usize) -> Result<(Entry, usize), String> {
    let mut index = start + 1;
    let mut path: Option<String> = None;
    let mut sha256: Option<String> = None;
    loop {
        skip_whitespace(bytes, &mut index);
        match bytes.get(index) {
            Some(b'}') => {
                index += 1;
                break;
            }
            Some(b',') => index += 1,
            Some(b'"') => {
                let (key, next) = parse_string(bytes, index)?;
                index = next;
                skip_whitespace(bytes, &mut index);
                if bytes.get(index) != Some(&b':') {
                    return Err(format!("键 {key} 后缺少冒号（位置 {index}）"));
                }
                index += 1;
                skip_whitespace(bytes, &mut index);
                if key == "path" || key == "sha256" {
                    let (value, next) = parse_string(bytes, index)?;
                    index = next;
                    if key == "path" {
                        path = Some(value);
                    } else {
                        sha256 = Some(value);
                    }
                } else {
                    index = skip_value(bytes, index)?;
                }
            }
            Some(_) => return Err(format!("对象位置 {index} 出现意外字节")),
            None => return Err("product_sources 对象未闭合".to_string()),
        }
    }
    let path = path.ok_or_else(|| "product_sources 条目缺少 path".to_string())?;
    let sha256 = sha256.ok_or_else(|| "product_sources 条目缺少 sha256".to_string())?;
    Ok((Entry { path, sha256 }, index))
}

fn skip_whitespace(bytes: &[u8], index: &mut usize) {
    while matches!(bytes.get(*index), Some(b' ' | b'\n' | b'\r' | b'\t')) {
        *index += 1;
    }
}

fn parse_string(bytes: &[u8], start: usize) -> Result<(String, usize), String> {
    if bytes.get(start) != Some(&b'"') {
        return Err(format!("位置 {start} 不是字符串起点"));
    }
    let mut out = String::new();
    let mut index = start + 1;
    loop {
        match bytes.get(index) {
            Some(b'"') => return Ok((out, index + 1)),
            Some(b'\\') => {
                index += 1;
                let escaped = *bytes.get(index).ok_or("字符串转义未闭合")?;
                out.push(match escaped {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'/' => '/',
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    other => return Err(format!("不支持的转义 \\{}", other as char)),
                });
                index += 1;
            }
            Some(byte) => {
                out.push(*byte as char);
                index += 1;
            }
            None => return Err("字符串未闭合".to_string()),
        }
    }
}

fn skip_value(bytes: &[u8], start: usize) -> Result<usize, String> {
    match bytes.get(start) {
        Some(b'"') => parse_string(bytes, start).map(|(_, next)| next),
        Some(b'{') | Some(b'[') => {
            let open = bytes[start];
            let close = if open == b'{' { b'}' } else { b']' };
            let mut depth = 0usize;
            let mut index = start;
            while index < bytes.len() {
                match bytes[index] {
                    b'"' => {
                        let (_, next) = parse_string(bytes, index)?;
                        index = next;
                        continue;
                    }
                    byte if byte == open => depth += 1,
                    byte if byte == close => {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(index + 1);
                        }
                    }
                    _ => {}
                }
                index += 1;
            }
            Err("嵌套值未闭合".to_string())
        }
        Some(_) => {
            let mut index = start;
            while matches!(bytes.get(index), Some(byte) if !matches!(byte, b',' | b'}' | b']')) {
                index += 1;
            }
            Ok(index)
        }
        None => Err("值缺失".to_string()),
    }
}

/// CRLF → LF 字节归一化（与 `str::replace("\r\n", "\n")` 等价）。
pub fn normalize_lf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
            out.push(b'\n');
            index += 2;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    out
}

/// 读取仓库内文件并按 LF 归一化后返回字节。
fn normalized_file(path: &str) -> Result<Vec<u8>, String> {
    let full = root().join(path);
    let bytes =
        std::fs::read(&full).map_err(|error| format!("{} 读取失败: {error}", full.display()))?;
    Ok(normalize_lf(&bytes))
}

/// physics_core 组合摘要：5 文件 LF 归一化后以 `\n` 连接再 sha256。
pub fn physics_core_digest() -> Result<String, String> {
    let mut combined: Vec<u8> = Vec::new();
    for (position, path) in PHYSICS_CORE_PATHS.iter().enumerate() {
        if position > 0 {
            combined.push(b'\n');
        }
        combined.extend_from_slice(&normalized_file(path)?);
    }
    Ok(sha256_hex(&combined))
}

/// 某条 `product_sources` 路径的活体摘要。
pub fn live_digest(path: &str) -> Result<String, String> {
    if path == "src/physics/mod.rs" {
        physics_core_digest()
    } else {
        normalized_file(path).map(|bytes| sha256_hex(&bytes))
    }
}

/// 逐条活体比对；返回全部差异（空 = 全部一致）。
pub fn verify_all(entries: &[Entry]) -> Vec<String> {
    let mut mismatches = Vec::new();
    for entry in entries {
        match live_digest(&entry.path) {
            Ok(actual) if actual == entry.sha256 => {}
            Ok(actual) => mismatches.push(format!(
                "{}: expected={} actual={}",
                entry.path, entry.sha256, actual
            )),
            Err(error) => mismatches.push(format!("{}: 无法计算活体摘要（{error}）", entry.path)),
        }
    }
    mismatches
}

/// SHA-256（FIPS 180-4），返回小写十六进制。
pub fn sha256_hex(input: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let mut message = input.to_vec();
    let bits = (input.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bits.to_be_bytes());
    for chunk in message.as_chunks::<64>().0 {
        let mut schedule = [0u32; 64];
        for (index, word) in schedule.iter_mut().take(16).enumerate() {
            let base = index * 4;
            *word = u32::from_be_bytes([
                chunk[base],
                chunk[base + 1],
                chunk[base + 2],
                chunk[base + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = schedule[index - 15].rotate_right(7)
                ^ schedule[index - 15].rotate_right(18)
                ^ (schedule[index - 15] >> 3);
            let s1 = schedule[index - 2].rotate_right(17)
                ^ schedule[index - 2].rotate_right(19)
                ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }
        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(schedule[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }
    let mut output = String::with_capacity(64);
    for word in state {
        output.push_str(&format!("{word:08x}"));
    }
    output
}

/// 供测试与证据复用的相对路径展示。
pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}
