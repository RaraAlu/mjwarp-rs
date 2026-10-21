//! 独立 f64 相机/光源 oracle。
//! 仅供测试使用，不提供 CPU 引擎回退。

use mjwarp_rs::{
    model::{CamLightFields, CamLightParameter as F, CamLightParameters},
    physics::KinematicsSnapshot,
};

fn vector(v: &[f32]) -> [f64; 3] {
    [f64::from(v[0]), f64::from(v[1]), f64::from(v[2])]
}
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = v.iter().map(|v| v * v).sum::<f64>().sqrt();
    if n > 0.0 { v.map(|v| v / n) } else { [0.0; 3] }
}
fn rotate(q: &[f32], v: [f64; 3]) -> [f64; 3] {
    let q: Vec<f64> = q.iter().map(|&q| f64::from(q)).collect();
    let u = [q[1], q[2], q[3]];
    let dot: f64 = u.iter().zip(v).map(|(u, v)| u * v).sum();
    let square = q[0] * q[0] - u.iter().map(|u| u * u).sum::<f64>();
    let c = cross(u, v);
    std::array::from_fn(|i| square * v[i] + 2.0 * dot * u[i] + 2.0 * q[0] * c[i])
}
pub(super) fn shared(fields: &CamLightFields, f: F) -> &[f32] {
    match f {
        F::CamPos => &fields.cam_pos,
        F::CamQuat => &fields.cam_quat,
        F::CamPoscom0 => &fields.cam_poscom0,
        F::CamPos0 => &fields.cam_pos0,
        F::CamMat0 => &fields.cam_mat0,
        F::LightPos => &fields.light_pos,
        F::LightDir => &fields.light_dir,
        F::LightPoscom0 => &fields.light_poscom0,
        F::LightPos0 => &fields.light_pos0,
        F::LightDir0 => &fields.light_dir0,
    }
}
pub(super) fn oracle(
    out: &KinematicsSnapshot,
    w: usize,
    fields: &CamLightFields,
    p: &CamLightParameters,
) -> Vec<f64> {
    let r = out.rigid().world(w).unwrap();
    let com = out.com().world(w).unwrap();
    let row = |f| p.get(f).map_or_else(|| shared(fields, f), |b| b.world(w));
    let mut cx = vec![];
    let mut cm = vec![];
    let mut lx = vec![];
    let mut ld = vec![];
    for (c, &b) in fields.cam_bodyid.iter().enumerate() {
        let b = b as usize;
        let mode = fields.cam_mode[c];
        let t = fields.cam_targetbodyid[c];
        let mut pos = add(
            vector(&r.xpos[3 * b..]),
            rotate(&r.xquat[4 * b..], vector(&row(F::CamPos)[3 * c..])),
        );
        if mode == 1 {
            pos = add(vector(&r.xpos[3 * b..]), vector(&row(F::CamPos0)[3 * c..]));
        }
        if mode == 2 {
            pos = add(
                vector(&com.subtree_com[3 * b..]),
                vector(&row(F::CamPoscom0)[3 * c..]),
            );
        }
        cx.extend(pos);
        if mode == 1 || mode == 2 {
            cm.extend(
                row(F::CamMat0)[9 * c..9 * c + 9]
                    .iter()
                    .map(|&v| f64::from(v)),
            );
        } else if (mode == 3 || mode == 4) && t >= 0 {
            let target = vector(if mode == 3 {
                &r.xpos[3 * t as usize..]
            } else {
                &com.subtree_com[3 * t as usize..]
            });
            let z = unit(sub(pos, target));
            let x = unit(cross([0.0, 0.0, 1.0], z));
            let y = unit(cross(z, x));
            for i in 0..3 {
                cm.extend([x[i], y[i], z[i]]);
            }
        } else {
            // Rotate each local camera basis through local then body frames.
            let cols: [[f64; 3]; 3] = std::array::from_fn(|i| {
                let mut axis = [0.0; 3];
                axis[i] = 1.0;
                rotate(&r.xquat[4 * b..], rotate(&row(F::CamQuat)[4 * c..], axis))
            });
            for ((x, y), z) in cols[0].into_iter().zip(cols[1]).zip(cols[2]) {
                cm.extend([x, y, z]);
            }
        }
    }
    for (l, &b) in fields.light_bodyid.iter().enumerate() {
        let b = b as usize;
        let mode = fields.light_mode[l];
        let t = fields.light_targetbodyid[l];
        let mut pos = add(
            vector(&r.xpos[3 * b..]),
            rotate(&r.xquat[4 * b..], vector(&row(F::LightPos)[3 * l..])),
        );
        if mode == 1 {
            pos = add(
                vector(&r.xpos[3 * b..]),
                vector(&row(F::LightPos0)[3 * l..]),
            );
        }
        if mode == 2 {
            pos = add(
                vector(&com.subtree_com[3 * b..]),
                vector(&row(F::LightPoscom0)[3 * l..]),
            );
        }
        lx.extend(pos);
        let dir = if mode == 1 || mode == 2 {
            vector(&row(F::LightDir0)[3 * l..])
        } else if (mode == 3 || mode == 4) && t >= 0 {
            sub(
                vector(if mode == 3 {
                    &r.xpos[3 * t as usize..]
                } else {
                    &com.subtree_com[3 * t as usize..]
                }),
                pos,
            )
        } else {
            rotate(&r.xquat[4 * b..], vector(&row(F::LightDir)[3 * l..]))
        };
        ld.extend(if (mode == 3 || mode == 4) && t < 0 {
            dir
        } else {
            unit(dir)
        });
    }
    [cx, cm, lx, ld].concat()
}
