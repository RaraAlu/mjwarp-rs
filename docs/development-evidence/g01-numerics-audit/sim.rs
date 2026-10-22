// 离线诊断：按 gravity_compensation_compute 的表达式顺序做 f32 模拟，
// 并用精确有理算术复算证书，定位 4 ulp 差异来源。
// 不参与构建、不修改产品源码；只用于审计 G01 重力位证书的 2 个槽。
fn fl(x: f64) -> f32 { x as f32 }

fn main() {
    // world=1/world=4 的输入（body_mass 周期 3、gravity 周期 2；COM 与 cdof 取自诊断报告）。
    let subtree_root: f32 = 2.4285714626312256; // body1 子树 COM 的 f32 值
    let cdof2: f32 = 1.0; // 角 z
    let cdof0: f32 = 0.0;
    let cdof4: f32 = -3.5714285373687744; // 线 y
    let oz: f32 = 0.0;
    let gravity1: f32 = -0.5;
    let mass: f32 = 0.5;
    let gravcomp: f32 = 2.0;
    let gpu_actual: f32 = f32::from_bits(0x3efffffc);
    let expected: f32 = 0.5;

    println!("== 精确有理证书 ==");
    // body1 子树 = body1(2, x=1) + body2(1, x=3) + body3(0.5, x=7)。
    let com = (2.0 * 1.0 + 1.0 * 3.0 + 0.5 * 7.0) / 3.5; // 17/7
    let cdof4_exact = -25.0 / 7.0;
    let ox_exact = 7.0 - com; // 32/7
    let jy_exact = cdof4_exact + ox_exact; // 1
    let result_exact = jy_exact * 0.5; // 0.5
    println!("subtree_com=17/7={com:.16}  cdof4=-25/7={cdof4_exact:.16}");
    println!("ox=32/7={ox_exact:.16}  jy={jy_exact:.16}  result={result_exact:.16} (bits {:#010x})", expected.to_bits());

    println!("== f32 逐步模拟 ==");
    let xipos3: f32 = 7.0; // 体惯性位置（world 位置 6 + ipos 1）
    let ox = xipos3 - subtree_root;
    let fy = ((-gravity1) * mass) * gravcomp;
    let jy_no_fma = (cdof4 + (cdof2 * ox - cdof0 * oz)) as f32;
    let jy_fma = cdof2.mul_add(ox, (-cdof0).mul_add(oz, cdof4));
    let jy_exact_from_f32 = fl(f64::from(cdof4) + (f64::from(cdof2) * f64::from(ox) - f64::from(cdof0) * f64::from(oz)));
    for (jlabel, jy) in [("no_fma", jy_no_fma), ("fma", jy_fma), ("exact1rn", jy_exact_from_f32)] {
        let res_mul = jy * fy;
        let res_fma = jy.mul_add(fy, 0.0);
        for (rlabel, res) in [("res_mul", res_mul), ("res_fma", res_fma)] {
            println!(
                "ox={ox:.10e} jy[{jlabel}]={jy:.10e} fy={fy:.10e} {rlabel}={res:.10e} bits={:#010x} gpu={:#010x} expected={:#010x} match_gpu={} match_expected={}",
                res.to_bits(),
                gpu_actual.to_bits(),
                expected.to_bits(),
                res.to_bits() == gpu_actual.to_bits(),
                res.to_bits() == expected.to_bits()
            );
        }
    }
    println!("f32 输入与精确值的差：com {:.3e}  cdof4 {:.3e}  ox {:.3e}",
        f64::from(subtree_root) - com,
        f64::from(cdof4) - cdof4_exact,
        f64::from(ox) - ox_exact);
}
