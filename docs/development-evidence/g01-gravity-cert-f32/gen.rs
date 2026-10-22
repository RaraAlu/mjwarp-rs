// 离线重力位证书生成器（不参与构建、无第三方依赖）。
//
// 目的：用户 2026-10-22 裁决 W-D4 口径第 1 项——证书期望值按上游同样的
// f32 逐步舍入口径产生，而不是精确有理值只在最后舍入一次。
//
// 独立性：
// - 只使用测试输入（模型拓扑/位置、gravity/body_mass/body_gravcomp 参数批）
//   与上游公式；不读取任何 GPU 实际输出，不回读产品快照作为期望。
// - 同时用精确有理算术复算旧证书，验证本生成器对原证书口径的复现。
//
// 上游公式来源（71da24d956378a87a703b6e1442b13aec0c4ac29）：
// - target/mjwarp-parallel/upstream/mujoco_warp/_src/smooth.py：
//   _subtree_com_init / _subtree_com_acc / _subtree_div / _cdof。
// - 仓库冻结 kernel src/physics/gravity_compensation/cuda.rs
//   （gravity_compensation_compute；来源注释 passive.py/support.py）。
//
// 编译与运行（PowerShell）：
//   rustc -O docs/development-evidence/g01-gravity-cert-f32/gen.rs -o target/g01-cert/gen.exe
//   target/g01-cert/gen.exe docs/development-evidence/g01-gravity-cert-f32
//
// f32 语义假设（与审计 2026-10-22-g01-numerics-audit 一致）：
// - mul/add/sub/div 均为 round-to-nearest-even 的独立 f32 指令；
// - 该 kernel 的 PTX 无 fma（审计已统计 fma.rn.f32 = 0）；
// - 原子加在单贡献者时等价一次写入；多贡献者按 body 升序模拟，
//   并在 GPU 测试中逐位复核（本生成器不读取该结果）。

use std::fmt::Write as _;
use std::ops::{Add, Div, Mul, Sub};

const MASS_ROWS: [[f32; 5]; 3] = [
    [0.0, 1.0, 2.0, 4.0, 8.0],
    [0.0, 2.0, 1.0, 0.5, 4.0],
    [0.0, 0.5, 4.0, 2.0, 1.0],
];
const GRAVITY_ROWS: [[f32; 3]; 2] = [[0.0, -1.0, 0.0], [0.0, -0.5, 0.0]];
const GRAVCOMP_ROWS: [[f32; 5]; 5] = [
    [0.0, 1.0, 1.0, 1.0, 1.0],
    [0.0, 0.5, 1.0, 2.0, 0.25],
    [0.0, 2.0, 0.5, 0.25, 2.0],
    [0.0, 1.0, 0.25, 0.5, 1.0],
    [0.0, 0.25, 2.0, 1.0, 0.5],
];
/// multi_body_model(): body_parentid/body_pos/body_ipos，body0 为世界体。
const PARENT: [usize; 5] = [0, 0, 1, 2, 0];
const BODY_POS_X: [f32; 5] = [0.0, 0.0, 2.0, 4.0, -2.0];
const BODY_IPOS_X: [f32; 5] = [0.0, 1.0, 1.0, 1.0, 1.0];
/// 四个 hinge（jnt_type=3）绕 z 轴，jnt_pos 全零。
const JNT_BODY: [usize; 4] = [1, 2, 3, 4];
const WORLDS: usize = 7;
/// 旧证书（精确有理值，最后单次舍入）：用于口径变更对照与生成器自检。
const LEGACY_EXPECTED: [[f32; 4]; WORLDS] = [
    [35.0, 22.0, 4.0, 8.0],
    [5.5, 3.0, 0.5, 0.5],
    [10.5, 4.5, 0.5, 2.0],
    [8.25, 5.25, 1.0, 4.0],
    [10.0, 4.5, 0.5, 2.0],
    [13.25, 7.0, 1.0, 0.5],
    [62.5, 42.0, 8.0, 2.0],
];

#[derive(Clone, Copy, Debug)]
struct Ratio {
    n: i128,
    d: i128,
}

impl Ratio {
    fn new(n: i128, d: i128) -> Self {
        assert!(d != 0, "zero denominator");
        let g = gcd(n.unsigned_abs(), d.unsigned_abs()).max(1) as i128;
        let (n, d) = (n / g, d / g);
        if d < 0 { Self { n: -n, d: -d } } else { Self { n, d } }
    }
    fn int(n: i128) -> Self {
        Self { n, d: 1 }
    }
    fn add(self, other: Self) -> Self {
        Self::new(self.n * other.d + other.n * self.d, self.d * other.d)
    }
    fn sub(self, other: Self) -> Self {
        Self::new(self.n * other.d - other.n * self.d, self.d * other.d)
    }
    fn mul(self, other: Self) -> Self {
        Self::new(self.n * other.n, self.d * other.d)
    }
    fn div(self, other: Self) -> Self {
        Self::new(self.n * other.d, self.d * other.n)
    }
    fn f64(self) -> f64 {
        self.n as f64 / self.d as f64
    }
    fn f32(self) -> f32 {
        self.f64() as f32
    }
    fn text(self) -> String {
        if self.d == 1 {
            format!("{}", self.n)
        } else {
            format!("{}/{}", self.n, self.d)
        }
    }
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// f64 值的精确二进制有理表示（所有输入都是有限 f32，可精确展开）。
fn exact(x: f64) -> Ratio {
    if x == 0.0 {
        return Ratio::int(0);
    }
    let bits = x.to_bits();
    let sign = if bits >> 63 == 1 { -1 } else { 1 };
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = (bits & ((1u64 << 52) - 1)) as i128;
    let (mantissa, shift) = if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction + (1i128 << 52), exponent - 1075)
    };
    if shift >= 0 {
        Ratio::new(sign * mantissa * (1i128 << shift), 1)
    } else {
        Ratio::new(sign * mantissa, 1i128 << (-shift))
    }
}

fn fr(x: f32) -> Ratio {
    exact(f64::from(x))
}

fn bits(x: f32) -> String {
    format!("\"0x{:08x}\"", x.to_bits())
}

fn json_num(x: f32) -> String {
    // f32 十进制足够还原 bits；同时总是给出 bits 字段，避免文本歧义。
    format!("{}", x)
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn main() {
    let out_dir = std::env::args().nth(1).unwrap_or_else(|| {
        "docs/development-evidence/g01-gravity-cert-f32".to_string()
    });
    let roots = roots();
    let ancestors = ancestors(&roots);
    let (xpos, xipos) = positions();
    let mut worlds_json = Vec::new();
    let mut new_bits_rows = Vec::new();
    let mut legacy_bits_rows = Vec::new();
    let mut differences = Vec::new();
    for world in 0..WORLDS {
        let masses = MASS_ROWS[world % 3];
        let gravity = GRAVITY_ROWS[world % 2];
        let gravcomp = GRAVCOMP_ROWS[world % 5];
        let sub = subtree_com(&roots, &xipos, &masses);
        let cdof = cdof(&roots, &sub, &xpos);
        let mut slots = Vec::new();
        let mut new_bits = [0u32; 4];
        let mut legacy_bits = [0u32; 4];
        for dof in 0..4 {
            let mut contributions = Vec::new();
            let mut acc = 0.0f32;
            let mut exact_sum = Ratio::int(0);
            for body in 1..5 {
                if !ancestors[body][dof] {
                    continue;
                }
                let gc = gravcomp[body];
                let force = [
                    f32::mul(f32::mul(-gravity[0], masses[body]), gc),
                    f32::mul(f32::mul(-gravity[1], masses[body]), gc),
                    f32::mul(f32::mul(-gravity[2], masses[body]), gc),
                ];
                let exact_force = [
                    fr(-gravity[0]).mul(fr(masses[body])).mul(fr(gc)),
                    fr(-gravity[1]).mul(fr(masses[body])).mul(fr(gc)),
                    fr(-gravity[2]).mul(fr(masses[body])).mul(fr(gc)),
                ];
                let root = roots[body];
                let offset = [
                    f32::sub(xipos[body][0], sub[root].f32[0]),
                    f32::sub(xipos[body][1], sub[root].f32[1]),
                    f32::sub(xipos[body][2], sub[root].f32[2]),
                ];
                let offset_exact = [
                    exact(f64::from(xipos[body][0])).sub(sub[root].exact[0]),
                    exact(f64::from(xipos[body][1])).sub(sub[root].exact[1]),
                    exact(f64::from(xipos[body][2])).sub(sub[root].exact[2]),
                ];
                let [c0, c1, c2, c3, c4, c5] = cdof[dof].f32;
                let jac = [
                    f32::add(c3, f32::sub(f32::mul(c1, offset[2]), f32::mul(c2, offset[1]))),
                    f32::add(c4, f32::sub(f32::mul(c2, offset[0]), f32::mul(c0, offset[2]))),
                    f32::add(c5, f32::sub(f32::mul(c0, offset[1]), f32::mul(c1, offset[0]))),
                ];
                let ce = &cdof[dof].exact;
                let jac_exact = [
                    ce[3].add(ce[1].mul(offset_exact[2]).sub(ce[2].mul(offset_exact[1]))),
                    ce[4].add(ce[2].mul(offset_exact[0]).sub(ce[0].mul(offset_exact[2]))),
                    ce[5].add(ce[0].mul(offset_exact[1]).sub(ce[1].mul(offset_exact[0]))),
                ];
                let product = f32::add(
                    f32::add(f32::mul(jac[0], force[0]), f32::mul(jac[1], force[1])),
                    f32::mul(jac[2], force[2]),
                );
                let exact_product = jac_exact[0]
                    .mul(exact_force[0])
                    .add(jac_exact[1].mul(exact_force[1]))
                    .add(jac_exact[2].mul(exact_force[2]));
                acc = f32::add(acc, product);
                exact_sum = exact_sum.add(exact_product);
                contributions.push(format!(
                    "{{\"body\":{body},\"gravcomp_f32\":{},\"force_f32\":[{},{},{}],\"force_bits\":[{},{},{}],\"force_exact\":[{},{},{}],\"offset_f32\":[{},{},{}],\"offset_bits\":[{},{},{}],\"offset_exact\":[{},{},{}],\"jacobian_f32\":[{},{},{}],\"jacobian_bits\":[{},{},{}],\"jacobian_exact\":[{},{},{}],\"product_f32\":{},\"product_bits\":{},\"product_exact\":{}}}",
                    json_num(gravcomp[body]),
                    json_num(force[0]), json_num(force[1]), json_num(force[2]),
                    bits(force[0]), bits(force[1]), bits(force[2]),
                    json_str(&exact_force[0].text()), json_str(&exact_force[1].text()), json_str(&exact_force[2].text()),
                    json_num(offset[0]), json_num(offset[1]), json_num(offset[2]),
                    bits(offset[0]), bits(offset[1]), bits(offset[2]),
                    json_str(&offset_exact[0].text()), json_str(&offset_exact[1].text()), json_str(&offset_exact[2].text()),
                    json_num(jac[0]), json_num(jac[1]), json_num(jac[2]),
                    bits(jac[0]), bits(jac[1]), bits(jac[2]),
                    json_str(&jac_exact[0].text()), json_str(&jac_exact[1].text()), json_str(&jac_exact[2].text()),
                    json_num(product), bits(product), json_str(&exact_product.text())
                ));
            }
            let legacy = exact_sum.f32();
            new_bits[dof] = acc.to_bits();
            legacy_bits[dof] = legacy.to_bits();
            if acc.to_bits() != legacy.to_bits() {
                differences.push(format!(
                    "{{\"world\":{world},\"dof\":{dof},\"legacy_bits\":{},\"legacy_value\":{},\"new_bits\":{},\"new_value\":{},\"ulp\":{},\"exact\":{}}}",
                    bits(legacy), json_num(legacy), bits(acc), json_num(acc),
                    legacy.to_bits().abs_diff(acc.to_bits()), json_str(&exact_sum.text())
                ));
            }
            slots.push(format!(
                "{{\"dof\":{dof},\"contributors\":[{}],\"accumulation_bits\":[{}],\"accumulation_f32\":{},\"expected_bits\":{},\"expected_f32\":{},\"legacy_exact\":{},\"legacy_bits\":{},\"legacy_f32\":{},\"ulp_difference\":{}}}",
                contributions.join(","),
                bits(acc),
                json_num(acc),
                bits(acc),
                json_num(acc),
                json_str(&exact_sum.text()),
                bits(legacy),
                json_num(legacy),
                legacy.to_bits().abs_diff(acc.to_bits())
            ));
        }
        new_bits_rows.push(new_bits);
        legacy_bits_rows.push(legacy_bits);
        let sub_json = sub
            .iter()
            .map(|entry| {
                format!(
                    "{{\"node\":{},\"exact\":{},\"f32\":{},\"bits\":{}}}",
                    entry.node,
                    json_str(&entry.exact[0].text()),
                    json_num(entry.f32[0]),
                    bits(entry.f32[0])
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let cdof_json = cdof
            .iter()
            .enumerate()
            .map(|(dof, entry)| {
                format!(
                    "{{\"dof\":{dof},\"axis_f32\":[{},{},{}],\"xanchor_f32\":{},\"offset_exact\":{},\"offset_f32\":{},\"offset_bits\":{},\"cdof_f32\":[{},{},{},{},{},{}],\"cdof_bits\":[{},{},{},{},{},{}],\"linear_exact\":[{}]}}",
                    json_num(0.0), json_num(0.0), json_num(1.0),
                    json_num(entry.anchor),
                    json_str(&entry.offset_exact.text()),
                    json_num(entry.offset_f32),
                    bits(entry.offset_f32),
                    json_num(entry.f32[0]), json_num(entry.f32[1]), json_num(entry.f32[2]),
                    json_num(entry.f32[3]), json_num(entry.f32[4]), json_num(entry.f32[5]),
                    bits(entry.f32[0]), bits(entry.f32[1]), bits(entry.f32[2]),
                    bits(entry.f32[3]), bits(entry.f32[4]), bits(entry.f32[5]),
                    entry.exact[3..].iter().map(|v| json_str(&v.text())).collect::<Vec<_>>().join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        worlds_json.push(format!(
            "{{\"world\":{world},\"gravity_f32\":[{},{},{}],\"body_mass_f32\":[{},{},{},{}],\"body_gravcomp_f32\":[{},{},{},{}],\"subtree_com\":[{sub_json}],\"cdof\":[{cdof_json}],\"slots\":[{}]}}",
            json_num(gravity[0]), json_num(gravity[1]), json_num(gravity[2]),
            json_num(masses[1]), json_num(masses[2]), json_num(masses[3]), json_num(masses[4]),
            json_num(gravcomp[1]), json_num(gravcomp[2]), json_num(gravcomp[3]), json_num(gravcomp[4]),
            slots.join(",")
        ));
    }
    // 自检：精确有理复算必须逐位复现旧证书（否则说明公式推导有误）。
    let mut legacy_mismatches = Vec::new();
    for world in 0..WORLDS {
        for dof in 0..4 {
            let expected = LEGACY_EXPECTED[world][dof].to_bits();
            if legacy_bits_rows[world][dof] != expected {
                legacy_mismatches.push(format!(
                    "world={world} dof={dof} derived={} legacy={}",
                    bits(f32::from_bits(legacy_bits_rows[world][dof])),
                    bits(LEGACY_EXPECTED[world][dof])
                ));
            }
        }
    }
    if !legacy_mismatches.is_empty() {
        eprintln!("旧证书复现失败：{legacy_mismatches:#?}");
        std::process::exit(1);
    }
    let certificate = format!(
        "{{\n\"schema\":1,\n\"task_id\":{},\n\"baseline_upstream_revision\":\"71da24d956378a87a703b6e1442b13aec0c4ac29\",\n\"generator\":\"docs/development-evidence/g01-gravity-cert-f32/gen.rs\",\n\"semantics\":\"upstream-formula + stepwise f32 rounding; independent of GPU outputs\",\n\"f32_rules\":[\"mul/add/sub/div are separate round-to-nearest-even f32 ops\",\"this kernel PTX has no fma\",\"atomic accumulation simulated in ascending body order for multiple contributors\"],\n\"upstream_sources\":[\"target/mjwarp-parallel/upstream/mujoco_warp/_src/smooth.py: _subtree_com_init/_subtree_com_acc/_subtree_div/_cdof\",\"src/physics/gravity_compensation/cuda.rs: gravity_compensation_compute\"],\n\"model\":{{\"body_parentid\":[0,0,1,2,0],\"body_pos_x\":[0,0,2,4,-2],\"body_ipos_x\":[0,1,1,1,1],\"root\":[0,1,1,1,4],\"jnt_bodyid\":[1,2,3,4],\"jnt_type\":[\"hinge\",\"hinge\",\"hinge\",\"hinge\"],\"jnt_axis\":[\"z\",\"z\",\"z\",\"z\"]}},\n\"parameters\":{{\"gravity_period\":2,\"body_mass_period\":3,\"body_gravcomp_period\":5}},\n\"expected_bits\":{},\n\"legacy_expected_bits\":{},\n\"differences\":[{}],\n\"worlds\":[{}]\n}}\n",
        json_str("2026-10-22-g01-gravity-cert-f32-regen"),
        bits_rows(&new_bits_rows),
        bits_rows(&legacy_bits_rows),
        differences.join(","),
        worlds_json.join(",")
    );
    std::fs::create_dir_all(&out_dir).expect("create output directory");
    std::fs::write(format!("{out_dir}/certificate.json"), &certificate).expect("write certificate");
    let legacy = format!(
        "{{\n\"schema\":1,\n\"task_id\":{},\n\"archive_of\":\"exact-rational certificate (pre-2026-10-22)\",\n\"reason\":\"user decided 2026-10-22 (W-D4 option 1): regenerate using stepwise f32 semantics; exact certificate cannot match upstream f32 rounding\",\n\"legacy_expected\":{},\n\"legacy_bits\":{},\n\"new_expected_bits\":{},\n\"differences\":[{}]\n}}\n",
        json_str("2026-10-22-g01-gravity-cert-f32-regen"),
        legacy_values(),
        bits_rows(&legacy_bits_rows),
        bits_rows(&new_bits_rows),
        differences.join(",")
    );
    std::fs::write(format!("{out_dir}/legacy-exact-certificate.json"), &legacy)
        .expect("write legacy archive");
    let mut rows = String::new();
    for world in 0..WORLDS {
        let new = new_bits_rows[world];
        let old = legacy_bits_rows[world];
        let _ = writeln!(
            rows,
            "| {world} | {} | {} | {} |",
            new.iter().map(|b| format!("0x{b:08x}")).collect::<Vec<_>>().join(" "),
            old.iter().map(|b| format!("0x{b:08x}")).collect::<Vec<_>>().join(" "),
            if new == old { "一致" } else { "口径差异" }
        );
    }
    let report = format!(
        "# G01 重力位证书（f32 逐步口径）\n\n任务：`2026-10-22-g01-gravity-cert-f32-regen`。\n由 `gen.rs` 独立生成；输入只用测试模型与参数批，不读取 GPU 输出。\n\n| world | f32 逐步期望 bits | 旧精确证书 bits | 对比 |\n| --- | --- | --- | --- |\n{rows}\n差异槽：{} 处（见 certificate.json 的 differences）。\n\n生成命令：\n\n```powershell\nrustc -O docs/development-evidence/g01-gravity-cert-f32/gen.rs -o target/g01-cert/gen.exe\n& target/g01-cert/gen.exe docs/development-evidence/g01-gravity-cert-f32\n```\n\n逐步舍入点见 `certificate.json`：`subtree_com`（init 乘法、树序累加、除以子树质量）、`cdof`（com−anchor 减法、cross 乘减）、每个贡献的 force/offset/jacobian/product、以及累加；同时给出精确有理值与 f32 bits。\n",
        differences.len()
    );
    std::fs::write(format!("{out_dir}/CERTIFICATE.md"), report).expect("write certificate md");
    println!(
        "wrote {out_dir}/certificate.json, legacy-exact-certificate.json, CERTIFICATE.md; differences={}",
        differences.len()
    );
}

struct SubtreeEntry {
    node: usize,
    exact: [Ratio; 3],
    f32: [f32; 3],
}

struct CdofEntry {
    anchor: f32,
    offset_exact: Ratio,
    offset_f32: f32,
    f32: [f32; 6],
    exact: [Ratio; 6],
}

fn roots() -> [usize; 5] {
    let mut roots = [0usize; 5];
    for body in 1..5 {
        let mut root = body;
        while PARENT[root] != 0 {
            root = PARENT[root];
        }
        roots[body] = root;
    }
    roots
}

fn ancestors(roots: &[usize; 5]) -> [[bool; 4]; 5] {
    let mut result = [[false; 4]; 5];
    for body in 1..5 {
        let mut chain = Vec::new();
        let mut cursor = body;
        loop {
            chain.push(cursor);
            if PARENT[cursor] == 0 {
                break;
            }
            cursor = PARENT[cursor];
        }
        for dof in 0..4 {
            result[body][dof] = chain.contains(&JNT_BODY[dof]);
        }
    }
    let _ = roots;
    result
}

fn positions() -> ([[f32; 3]; 5], [[f32; 3]; 5]) {
    let mut xpos = [[0.0f32; 3]; 5];
    let mut xipos = [[0.0f32; 3]; 5];
    for body in 1..5 {
        for axis in 0..3 {
            let position = if axis == 0 { BODY_POS_X[body] } else { 0.0 };
            let ipos = if axis == 0 { BODY_IPOS_X[body] } else { 0.0 };
            xpos[body][axis] = f32::add(xpos[PARENT[body]][axis], position);
            xipos[body][axis] = f32::add(xpos[body][axis], ipos);
        }
    }
    (xpos, xipos)
}

fn subtree_com(_roots: &[usize; 5], xipos: &[[f32; 3]; 5], masses: &[f32; 5]) -> [SubtreeEntry; 5] {
    let mut entries = std::array::from_fn(|node| SubtreeEntry {
        node,
        exact: [Ratio::int(0); 3],
        f32: [0.0; 3],
    });
    let mut weighted = [[0.0f32; 3]; 5];
    let mut exact_weighted = [[Ratio::int(0); 3]; 5];
    let mut mass_sum = [0.0f32; 5];
    let mut exact_mass = [Ratio::int(0); 5];
    // _subtree_com_init：逐体 mass*xipos（f32 乘法）。
    for body in 1..5 {
        for axis in 0..3 {
            weighted[body][axis] = f32::mul(xipos[body][axis], masses[body]);
            exact_weighted[body][axis] =
                exact(f64::from(xipos[body][axis])).mul(fr(masses[body]));
        }
        mass_sum[body] = masses[body];
        exact_mass[body] = fr(masses[body]);
    }
    // _subtree_com_acc：子体先加到父体（body_tree 逆序）；本链为 body3→body2→body1。
    for body in (2..5).rev() {
        let parent = PARENT[body];
        for axis in 0..3 {
            weighted[parent][axis] = f32::add(weighted[parent][axis], weighted[body][axis]);
            exact_weighted[parent][axis] =
                exact_weighted[parent][axis].add(exact_weighted[body][axis]);
        }
        mass_sum[parent] = f32::add(mass_sum[parent], mass_sum[body]);
        exact_mass[parent] = exact_mass[parent].add(exact_mass[body]);
    }
    // _subtree_div：除以子树质量（f32 除法）。
    for node in 1..5 {
        for axis in 0..3 {
            entries[node].f32[axis] = f32::div(weighted[node][axis], mass_sum[node]);
            entries[node].exact[axis] = exact_weighted[node][axis].div(exact_mass[node]);
        }
    }
    entries
}

fn cdof(roots: &[usize; 5], subtree: &[SubtreeEntry; 5], xpos: &[[f32; 3]; 5]) -> [CdofEntry; 4] {
    std::array::from_fn(|dof| {
        let body = JNT_BODY[dof];
        let root = roots[body];
        // _cdof 使用关节锚点 xanchor（jnt_pos=0 时等于体全局位置），不是 xipos。
        let anchor = xpos[body][0];
        let offset_f32 = f32::sub(subtree[root].f32[0], anchor);
        let offset_exact = subtree[root].exact[0].sub(exact(f64::from(anchor)));
        // _cdof：angular = xaxis；linear = cross(xaxis, offset)，f32 乘减。
        // axis=(0,0,1)、offset 只有 x 分量：cross=(0*0-1*0, 1*ox-0*0, 0*0-0*ox)，
        // 每步都是精确 f32 运算，只有 offset 的减法产生舍入。
        let mut value = [0.0f32; 6];
        value[2] = 1.0;
        value[4] = f32::sub(f32::mul(1.0, offset_f32), f32::mul(0.0, 0.0));
        let mut exact = [Ratio::int(0); 6];
        exact[0] = Ratio::int(0);
        exact[1] = Ratio::int(0);
        exact[2] = Ratio::int(1);
        exact[3] = Ratio::int(0);
        exact[4] = offset_exact;
        exact[5] = Ratio::int(0);
        CdofEntry {
            anchor,
            offset_exact,
            offset_f32,
            f32: value,
            exact,
        }
    })
}

fn bits_rows(rows: &[[u32; 4]]) -> String {
    format!(
        "[{}]",
        rows.iter()
            .map(|row| format!(
                "[{}]",
                row.iter()
                    .map(|b| format!("\"0x{b:08x}\""))
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn legacy_values() -> String {
    format!(
        "[{}]",
        LEGACY_EXPECTED
            .iter()
            .map(|row| format!(
                "[{}]",
                row.iter()
                    .map(|v| json_num(*v))
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}
