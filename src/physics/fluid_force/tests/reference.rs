//! 流体冻结参考实现与数值工具。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn mul(rotation: &[f32], value: [f32; 3]) -> [f32; 3] {
    [
        rotation[0] * value[0] + rotation[1] * value[1] + rotation[2] * value[2],
        rotation[3] * value[0] + rotation[4] * value[1] + rotation[5] * value[2],
        rotation[6] * value[0] + rotation[7] * value[1] + rotation[8] * value[2],
    ]
}

#[cfg(feature = "cuda-probe")]
pub(super) fn mul_transpose(rotation: &[f32], value: [f32; 3]) -> [f32; 3] {
    [
        rotation[0] * value[0] + rotation[3] * value[1] + rotation[6] * value[2],
        rotation[1] * value[0] + rotation[4] * value[1] + rotation[7] * value[2],
        rotation[2] * value[0] + rotation[5] * value[1] + rotation[8] * value[2],
    ]
}

#[cfg(feature = "cuda-probe")]
pub(super) fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

#[cfg(feature = "cuda-probe")]
type FluidSources<'a> = (&'a [f32], &'a [f32], &'a [f32], Option<&'a [f32]>);

#[cfg(feature = "cuda-probe")]
pub(super) fn reference(
    model: &AttachedModelInput,
    world: usize,
    sources: FluidSources<'_>,
    mass: &[f32],
    inertia: &[f32],
    input: &FluidForceInput,
) -> (Vec<f32>, Vec<f32>) {
    let kinematic = model.rigid().kinematics();
    let inertial = model.rigid().fields();
    let (nb, nj, nv) = (kinematic.nbody(), kinematic.njnt(), kinematic.nv());
    let ng = model.ngeom();
    let rigid_stride = 28 * nb + 6 * nj;
    let com_stride = 14 * nb + 6 * nv;
    let velocity_stride = 6 * nb + 6 * nv;
    let attached_stride = 12 * (ng + model.nsite());
    let rigid = &sources.0[4 + world * rigid_stride..];
    let com = &sources.1[4 + world * com_stride..];
    let velocity = &sources.2[4 + world * velocity_stride..];
    let attached = sources
        .3
        .map(|values| &values[4 + world * attached_stride..]);
    let wind = input.wind.world(world);
    let density = input.density.world(world)[0];
    let viscosity = input.viscosity.world(world)[0];
    let xipos = &rigid[16 * nb..19 * nb];
    let ximat = &rigid[19 * nb..28 * nb];
    let subtree_com = &com[nb..4 * nb];
    let cdof = &com[14 * nb..14 * nb + 6 * nv];
    let cvel = &velocity[..6 * nb];
    let parent = &kinematic.fields().body_parentid;
    let mut roots = vec![0usize; nb];
    for (body, slot) in roots.iter_mut().enumerate().skip(1) {
        let mut root = body;
        while parent[root] != 0 {
            root = parent[root] as usize;
        }
        *slot = root;
    }
    let mut body_force = vec![0.0f32; 6 * nb];
    for body in 1..nb {
        if mass[body] < 1.0e-15 {
            continue;
        }
        let rotation = &ximat[9 * body..9 * body + 9];
        let angular = [cvel[6 * body], cvel[6 * body + 1], cvel[6 * body + 2]];
        let linear = [cvel[6 * body + 3], cvel[6 * body + 4], cvel[6 * body + 5]];
        let offset = [
            xipos[3 * body] - subtree_com[3 * roots[body]],
            xipos[3 * body + 1] - subtree_com[3 * roots[body] + 1],
            xipos[3 * body + 2] - subtree_com[3 * roots[body] + 2],
        ];
        let offset_cross = cross(offset, angular);
        let linear_com = [
            linear[0] - offset_cross[0],
            linear[1] - offset_cross[1],
            linear[2] - offset_cross[2],
        ];
        if input.body_fluid_ellipsoid[body] {
            let attached = attached.expect("ellipsoid reference requires attached frames");
            let geom_xpos = &attached[..3 * ng];
            let geom_xmat = &attached[3 * ng..12 * ng];
            let sizes = input.geom_size.world(world);
            let mut force_global = [0.0f32; 3];
            let mut torque_global = [0.0f32; 3];
            for geom in 0..ng {
                if model.fields().geom_bodyid[geom] as usize != body {
                    continue;
                }
                let fluid = &input.geom_fluid[12 * geom..12 * geom + 12];
                let coefficient = fluid[0];
                if coefficient <= 0.0 {
                    continue;
                }
                let size = &sizes[3 * geom..3 * geom + 3];
                let semiaxes = match input.geom_type[geom] {
                    2 => [size[0], size[0], size[0]],
                    3 => [size[0], size[0], size[1] + size[0]],
                    5 => [size[0], size[0], size[1]],
                    _ => [size[0], size[1], size[2]],
                };
                let rotation = &geom_xmat[9 * geom..9 * geom + 9];
                let position = &geom_xpos[3 * geom..3 * geom + 3];
                let point_offset = [
                    position[0] - xipos[3 * body],
                    position[1] - xipos[3 * body + 1],
                    position[2] - xipos[3 * body + 2],
                ];
                let point_cross = cross(angular, point_offset);
                let linear_point = [
                    linear_com[0] + point_cross[0],
                    linear_com[1] + point_cross[1],
                    linear_com[2] + point_cross[2],
                ];
                let lang = mul_transpose(rotation, angular);
                let mut llin = mul_transpose(rotation, linear_point);
                if wind[0] != 0.0 || wind[1] != 0.0 || wind[2] != 0.0 {
                    let local_wind = mul_transpose(rotation, [wind[0], wind[1], wind[2]]);
                    for index in 0..3 {
                        llin[index] -= local_wind[index];
                    }
                }
                let mut local_force = [0.0f32; 3];
                let mut local_torque = [0.0f32; 3];
                if density > 0.0 {
                    let virtual_linear = [
                        density * fluid[6] * llin[0],
                        density * fluid[7] * llin[1],
                        density * fluid[8] * llin[2],
                    ];
                    let virtual_angular = [
                        density * fluid[9] * lang[0],
                        density * fluid[10] * lang[1],
                        density * fluid[11] * lang[2],
                    ];
                    let added_force = cross(virtual_linear, lang);
                    let linear_torque = cross(virtual_linear, llin);
                    let angular_torque = cross(virtual_angular, lang);
                    for index in 0..3 {
                        local_force[index] += added_force[index];
                        local_torque[index] += linear_torque[index] + angular_torque[index];
                    }
                }
                #[allow(clippy::excessive_precision)]
                // 冻结源码的逐位十进制展开；只豁免本项，禁止缩短。
                const FOUR_THIRDS_PI: f32 = 4.18879032135009765625;
                #[allow(clippy::excessive_precision)]
                const EIGHT_FIFTEENTHS_PI: f32 = 1.6755161285400390625;
                #[allow(clippy::excessive_precision)]
                const TWO_THIRDS: f32 = 0.666666686534881591796875;
                #[allow(clippy::excessive_precision)]
                const THREE_PI: f32 = 9.424777984619140625;
                let volume = FOUR_THIRDS_PI * semiaxes[0] * semiaxes[1] * semiaxes[2];
                let maximum = semiaxes[0].max(semiaxes[1]).max(semiaxes[2]);
                let minimum = semiaxes[0].min(semiaxes[1]).min(semiaxes[2]);
                let middle = semiaxes[0] + semiaxes[1] + semiaxes[2] - maximum - minimum;
                let maximum_area = std::f32::consts::PI * maximum * middle;
                let speed = (llin[0] * llin[0] + llin[1] * llin[1] + llin[2] * llin[2]).sqrt();
                let inverse_speed = if speed > 1.0e-15 { 1.0 / speed } else { 0.0 };
                let unit = [
                    llin[0] * inverse_speed,
                    llin[1] * inverse_speed,
                    llin[2] * inverse_speed,
                ];
                let inverse_maximum = if maximum > 1.0e-15 {
                    1.0 / maximum
                } else {
                    0.0
                };
                let scaled = [
                    semiaxes[0] * inverse_maximum,
                    semiaxes[1] * inverse_maximum,
                    semiaxes[2] * inverse_maximum,
                ];
                let mut magnus = cross(lang, llin);
                let magnus_scale = fluid[5] * density * volume;
                for value in &mut magnus {
                    *value *= magnus_scale;
                }
                let s12 = scaled[1] * scaled[2];
                let s20 = scaled[2] * scaled[0];
                let s01 = scaled[0] * scaled[1];
                let s12_sq = s12 * s12;
                let s20_sq = s20 * s20;
                let s01_sq = s01 * s01;
                let unit_sq = [unit[0] * unit[0], unit[1] * unit[1], unit[2] * unit[2]];
                let projection_numerator =
                    s12_sq * unit_sq[0] + s20_sq * unit_sq[1] + s01_sq * unit_sq[2];
                let inverse_projection = if projection_numerator > 1.0e-15 {
                    1.0 / projection_numerator
                } else {
                    0.0
                };
                let projection = [
                    s12_sq * inverse_projection,
                    s20_sq * inverse_projection,
                    s01_sq * inverse_projection,
                ];
                let projection_denominator = projection[0] * projection[0] * unit[0] * unit[0]
                    + projection[1] * projection[1] * unit[1] * unit[1]
                    + projection[2] * projection[2] * unit[2] * unit[2];
                let projected_area = std::f32::consts::PI
                    * maximum
                    * maximum
                    * (projection_numerator * projection_denominator).sqrt();
                let cosine = if projection_denominator > 1.0e-15 {
                    1.0 / projection_denominator
                } else {
                    0.0
                };
                let normal = [
                    projection[0] * unit[0],
                    projection[1] * unit[1],
                    projection[2] * unit[2],
                ];
                let mut kutta = [0.0f32; 3];
                if density > 0.0 && fluid[4] != 0.0 && speed > 1.0e-15 {
                    let mut circulation = cross(normal, llin);
                    let scale = fluid[4] * density * cosine * projected_area;
                    for value in &mut circulation {
                        *value *= scale;
                    }
                    kutta = cross(circulation, llin);
                }
                let equivalent_diameter = TWO_THIRDS * (semiaxes[0] + semiaxes[1] + semiaxes[2]);
                let linear_viscosity = THREE_PI * equivalent_diameter;
                let angular_viscosity = std::f32::consts::PI
                    * equivalent_diameter
                    * equivalent_diameter
                    * equivalent_diameter;
                let power4 = |value: f32| {
                    let square = value * value;
                    square * square
                };
                let maximum_moment = EIGHT_FIFTEENTHS_PI * middle * power4(maximum);
                let moment = |direction: usize| {
                    let d0 = semiaxes[direction];
                    let d1 = semiaxes[(direction + 1) % 3];
                    let d2 = semiaxes[(direction + 2) % 3];
                    EIGHT_FIFTEENTHS_PI * d0 * power4(d1.max(d2))
                };
                let moments = [moment(0), moment(1), moment(2)];
                let viscous_moment = [
                    lang[0] * (fluid[3] * moments[0] + fluid[2] * (maximum_moment - moments[0])),
                    lang[1] * (fluid[3] * moments[1] + fluid[2] * (maximum_moment - moments[1])),
                    lang[2] * (fluid[3] * moments[2] + fluid[2] * (maximum_moment - moments[2])),
                ];
                let linear_drag = viscosity * linear_viscosity
                    + density
                        * speed
                        * (projected_area * fluid[1] + fluid[2] * (maximum_area - projected_area));
                let angular_drag = viscosity * angular_viscosity
                    + density
                        * (viscous_moment[0] * viscous_moment[0]
                            + viscous_moment[1] * viscous_moment[1]
                            + viscous_moment[2] * viscous_moment[2])
                            .sqrt();
                for index in 0..3 {
                    local_torque[index] -= angular_drag * lang[index];
                    local_force[index] += magnus[index] + kutta[index] - linear_drag * llin[index];
                    local_torque[index] *= coefficient;
                    local_force[index] *= coefficient;
                }
                let force = mul(rotation, local_force);
                let torque = mul(rotation, local_torque);
                for index in 0..3 {
                    force_global[index] += force[index];
                    torque_global[index] += torque[index];
                }
            }
            body_force[6 * body..6 * body + 3].copy_from_slice(&force_global);
            body_force[6 * body + 3..6 * body + 6].copy_from_slice(&torque_global);
            continue;
        }
        let lang = mul_transpose(rotation, angular);
        let mut llin = mul_transpose(rotation, linear_com);
        if wind[0] != 0.0 || wind[1] != 0.0 || wind[2] != 0.0 {
            let local_wind = mul_transpose(rotation, [wind[0], wind[1], wind[2]]);
            for index in 0..3 {
                llin[index] -= local_wind[index];
            }
        }
        let body_inertia = &inertia[3 * body..3 * body + 3];
        let scale = 6.0 / mass[body];
        let box0 =
            ((body_inertia[1] + body_inertia[2] - body_inertia[0]).max(1.0e-15) * scale).sqrt();
        let box1 =
            ((body_inertia[0] + body_inertia[2] - body_inertia[1]).max(1.0e-15) * scale).sqrt();
        let box2 =
            ((body_inertia[0] + body_inertia[1] - body_inertia[2]).max(1.0e-15) * scale).sqrt();
        let mut local_force = [0.0f32; 3];
        let mut local_torque = [0.0f32; 3];
        if viscosity > 0.0 {
            let diam = (box0 + box1 + box2) / 3.0;
            let diam_pow3 = diam.powf(3.0);
            for index in 0..3 {
                local_torque[index] =
                    ((-lang[index] * diam_pow3) * std::f32::consts::PI) * viscosity;
                local_force[index] =
                    (((-3.0 * llin[index]) * diam) * std::f32::consts::PI) * viscosity;
            }
        }
        if density > 0.0 {
            local_force[0] -= 0.5 * density * box1 * box2 * llin[0].abs() * llin[0];
            local_force[1] -= 0.5 * density * box0 * box2 * llin[1].abs() * llin[1];
            local_force[2] -= 0.5 * density * box0 * box1 * llin[2].abs() * llin[2];
            let scale = density / 64.0;
            let powers = [box0.powf(4.0), box1.powf(4.0), box2.powf(4.0)];
            local_torque[0] -= box0 * (powers[1] + powers[2]) * lang[0].abs() * lang[0] * scale;
            local_torque[1] -= box1 * (powers[0] + powers[2]) * lang[1].abs() * lang[1] * scale;
            local_torque[2] -= box2 * (powers[0] + powers[1]) * lang[2].abs() * lang[2] * scale;
        }
        let force = mul(rotation, local_force);
        let torque = mul(rotation, local_torque);
        body_force[6 * body..6 * body + 3].copy_from_slice(&force);
        body_force[6 * body + 3..6 * body + 6].copy_from_slice(&torque);
    }

    let mut qfrc = vec![0.0f32; nv];
    for dof in 0..nv {
        let vector = &cdof[6 * dof..6 * dof + 6];
        let jacobian = [
            vector[3], vector[4], vector[5], vector[0], vector[1], vector[2],
        ];
        let dof_body = inertial.dof_bodyid[dof];
        let mut accum = 0.0f32;
        for body in dof_body as usize..nb {
            let force = &body_force[6 * body..6 * body + 6];
            if force.iter().all(|&value| value == 0.0) {
                continue;
            }
            let mut cursor = body as i32;
            while cursor != 0 && cursor != dof_body {
                cursor = parent[cursor as usize];
            }
            if cursor == 0 {
                continue;
            }
            let offset = [
                xipos[3 * body] - subtree_com[3 * roots[body]],
                xipos[3 * body + 1] - subtree_com[3 * roots[body] + 1],
                xipos[3 * body + 2] - subtree_com[3 * roots[body] + 2],
            ];
            let cross_term = cross([vector[0], vector[1], vector[2]], offset);
            let mut dot6 = 0.0f32;
            for index in 0..6 {
                dot6 += jacobian[index] * force[index];
            }
            let dot3 =
                cross_term[0] * force[0] + cross_term[1] * force[1] + cross_term[2] * force[2];
            accum += dot6 + dot3;
        }
        qfrc[dof] = accum;
    }
    (body_force, qfrc)
}
