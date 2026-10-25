//! 上游 `ccd_hfield_kernel` 的宿主实现：子网格棱柱扫描 + `ccd()` + 0..3 接触选择。

use super::super::gjk::math::{
    add3, cross3, dot3, length3, make_frame, mul_mat3_vec, scale3, sub3,
};
use super::super::gjk::{CcdOptions, Geom, ccd, geom_type};
use super::{HfieldData, filter::hfield_filter};

/// 上游 `MJ_MAXCONPAIR`。
pub const MJ_MAXCONPAIR: usize = 50;
/// 上游 `MIN_DIST_TO_NEXT_CONTACT`。
const MIN_DIST_TO_NEXT_CONTACT: f32 = 1.0e-3;
/// 上游 `MJ_MAXVAL`。
const MJ_MAXVAL: f32 = 1.0e10;
/// hfield geom1 恒为单位姿态。
const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// 一个 hfield 接触（世界系）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HfieldContact {
    pub dist: f32,
    pub pos: [f32; 3],
    pub frame: [f32; 9],
}

/// hfield 单对输出；`overflow` 对应上游 `OverflowType.HFIELD`。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HfieldOutput {
    pub contacts: Vec<HfieldContact>,
    pub overflow: bool,
    /// 通过筛选进入 `ccd()` 的棱柱数。
    pub prism_count: usize,
}

/// 单对输入；`geom2` 的位姿必须已变换到 hfield 局部帧。
#[derive(Clone, Copy)]
pub struct HfieldInput<'a> {
    pub hfield: &'a HfieldData,
    pub geom2: Geom<'a>,
    pub geomtype2: i32,
    /// `geom_rbound`（世界批口径）。
    pub other_rbound: f32,
    /// 两侧 `geom_margin` 之和。
    pub margin: f32,
    /// 两侧 `geom_gap` 之和。
    pub gap: f32,
    pub options: CcdOptions,
    /// 世界 <- hfield 的旋转矩阵（行优先）。
    pub hf_mat: [f32; 9],
    /// 世界 <- hfield 的平移。
    pub hf_pos: [f32; 3],
}

/// 上游相对旋转 `transpose(a) * b` 由 `relmat::upstream_relative_matrix` 提供（测试用；
/// 产品路径把 geom2 变换到 hfield 局部帧由调用方完成）。旧版此处自带 `bᵀa` 构造，已删除。
fn sum6(prism: &[[f32; 3]; 6]) -> [f32; 3] {
    let mut total = [0.0_f32; 3];
    for vertex in prism {
        total = add3(total, *vertex);
    }
    total
}

fn write_contact(output: &mut HfieldOutput, dist: f32, pos: [f32; 3], normal: [f32; 3]) {
    output.contacts.push(HfieldContact {
        dist,
        pos,
        frame: make_frame(normal),
    });
}

/// 上游 `ccd_hfield_kernel` 的宿主版：`geom1` 为 hfield（棱柱六顶点），`geom2` 为对侧几何。
#[allow(clippy::needless_range_loop)]
pub fn hfield_pair_contacts(input: &HfieldInput<'_>) -> HfieldOutput {
    let hfield = input.hfield;
    let mut geom2 = input.geom2;
    geom2.margin = 0.0;
    let Some(bounds) = hfield_filter(
        hfield,
        &geom2,
        input.geomtype2,
        input.other_rbound,
        input.margin,
    ) else {
        return HfieldOutput::default();
    };
    geom2.margin = input.margin;

    let size = hfield.size;
    let nrow = hfield.nrow;
    let ncol = hfield.ncol;
    let x_scale = 0.5 * (ncol as f32 - 1.0) / size[0];
    let y_scale = 0.5 * (nrow as f32 - 1.0) / size[1];
    let cmin = (((bounds.xmin + size[0]) * x_scale).floor() as i32).max(0);
    let cmax = (((bounds.xmax + size[0]) * x_scale).ceil() as i32).min(ncol - 1);
    let rmin = (((bounds.ymin + size[1]) * y_scale).floor() as i32).max(0);
    let rmax = (((bounds.ymax + size[1]) * y_scale).ceil() as i32).min(nrow - 1);

    let dx = 2.0 * size[0] / (ncol as f32 - 1.0);
    let dy = 2.0 * size[1] / (nrow as f32 - 1.0);
    let dr = [1_i32, 0_i32];

    let mut prism = [[0.0_f32; 3]; 6];
    prism[0][2] = -size[3];
    prism[1][2] = -size[3];
    prism[2][2] = -size[3];

    let mut cached_dist = [0.0_f32; MJ_MAXCONPAIR];
    let mut cached_pos = [[0.0_f32; 3]; MJ_MAXCONPAIR];
    let mut cached_normal = [[0.0_f32; 3]; MJ_MAXCONPAIR];
    let mut min_dist = MJ_MAXVAL;
    let mut min_normal = [MJ_MAXVAL; 3];
    let mut min_pos = [MJ_MAXVAL; 3];
    let mut min_id: i32 = -1;
    let mut count = 0_usize;
    let mut overflow = false;
    let hfield_geom = Geom::simple([0.0; 3], IDENTITY, [0.0; 3], 0.0);

    'scan: for row in rmin..rmax {
        // 预置本行前两个顶点（行首列）。
        for init_index in 0..2 {
            let x = dx * cmin as f32 - size[0];
            let y = dy * (row + dr[init_index]) as f32 - size[1];
            let z = hfield.height(row + dr[init_index], cmin) * size[2] + input.margin;
            prism[0] = prism[1];
            prism[1] = prism[2];
            prism[3] = prism[4];
            prism[4] = prism[5];
            prism[2][0] = x;
            prism[5][0] = x;
            prism[2][1] = y;
            prism[5][1] = y;
            prism[5][2] = z;
        }
        for column in (cmin + 1)..=cmax {
            for i in 0..2 {
                if count >= MJ_MAXCONPAIR {
                    overflow = true;
                    break 'scan;
                }
                // 追加本列顶点。
                let x = dx * column as f32 - size[0];
                let y = dy * (row + dr[i]) as f32 - size[1];
                let z = hfield.height(row + dr[i], column) * size[2] + input.margin;
                prism[0] = prism[1];
                prism[1] = prism[2];
                prism[3] = prism[4];
                prism[4] = prism[5];
                prism[2][0] = x;
                prism[5][0] = x;
                prism[2][1] = y;
                prism[5][1] = y;
                prism[5][2] = z;

                // prism 高度测试。
                if prism[3][2] < bounds.zmin
                    && prism[4][2] < bounds.zmin
                    && prism[5][2] < bounds.zmin
                {
                    continue;
                }

                let mut geom1 = hfield_geom;
                geom1.polyvert = prism;
                let center = scale3(sum6(&prism), 1.0 / 6.0);
                let result = ccd(
                    &geom1,
                    &geom2,
                    geom_type::HFIELD,
                    input.geomtype2,
                    center,
                    geom2.pos,
                    input.options,
                );
                // 分离棱柱跳过（ccd 对分离返回正距离）。
                if result.ncontact == 0 || result.dist >= input.margin + input.gap {
                    continue;
                }
                let pos_local = scale3(add3(result.x1, result.x2), 0.5);
                let pos = add3(mul_mat3_vec(&input.hf_mat, pos_local), input.hf_pos);
                let frame_local = make_frame(sub3(result.x1, result.x2));
                let normal_local = [frame_local[0], frame_local[1], frame_local[2]];
                let normal = mul_mat3_vec(&input.hf_mat, normal_local);
                cached_dist[count] = result.dist;
                cached_pos[count] = pos;
                cached_normal[count] = normal;
                if result.dist < min_dist {
                    min_dist = result.dist;
                    min_normal = normal;
                    min_pos = pos;
                    min_id = count as i32;
                }
                count += 1;
            }
        }
    }

    let mut output = HfieldOutput {
        contacts: Vec::with_capacity(4),
        overflow,
        prism_count: count,
    };
    if count == 0 {
        return output;
    }
    let min_id = min_id as usize;
    write_contact(&mut output, cached_dist[min_id], min_pos, min_normal);

    // contact 1：离最小距离接触点最远的接触。
    let mut id1: i32 = -1;
    let mut dist1 = -MJ_MAXVAL;
    for index in 0..count {
        if index == min_id {
            continue;
        }
        let dist = length3(sub3(cached_pos[index], min_pos));
        if dist > dist1 {
            id1 = index as i32;
            dist1 = dist;
        }
    }
    if id1 == -1 || (dist1 > 0.0 && dist1 < MIN_DIST_TO_NEXT_CONTACT) {
        return output;
    }
    let pos1 = cached_pos[id1 as usize];
    write_contact(
        &mut output,
        cached_dist[id1 as usize],
        pos1,
        cached_normal[id1 as usize],
    );

    // contact 2：离 min_pos→pos1 直线最远的接触。
    let dist_min1 = cross3(min_normal, sub3(min_pos, pos1));
    let mut id2: i32 = -1;
    let mut dist_12 = -MJ_MAXVAL;
    for index in 0..count {
        if index == min_id || index == id1 as usize {
            continue;
        }
        let dist = dot3(sub3(cached_pos[index], min_pos), dist_min1).abs();
        if dist > dist_12 {
            id2 = index as i32;
            dist_12 = dist;
        }
    }
    if id2 == -1 || (dist_12 > 0.0 && dist_12 < MIN_DIST_TO_NEXT_CONTACT) {
        return output;
    }
    let pos2 = cached_pos[id2 as usize];
    write_contact(
        &mut output,
        cached_dist[id2 as usize],
        pos2,
        cached_normal[id2 as usize],
    );

    // contact 3：离另一条三角边最远的接触。
    let vec_min2 = cross3(min_normal, sub3(min_pos, pos2));
    let vec_12 = cross3(min_normal, sub3(pos1, pos2));
    let mut id3: i32 = -1;
    let mut dist3 = -MJ_MAXVAL;
    for index in 0..count {
        if index == min_id || index == id1 as usize || index == id2 as usize {
            continue;
        }
        let point = cached_pos[index];
        let dist =
            dot3(sub3(point, min_pos), vec_min2).abs() + dot3(sub3(pos1, point), vec_12).abs();
        if dist > dist3 {
            id3 = index as i32;
            dist3 = dist;
        }
    }
    if id3 == -1 || (dist3 > 0.0 && dist3 < MIN_DIST_TO_NEXT_CONTACT) {
        return output;
    }
    write_contact(
        &mut output,
        cached_dist[id3 as usize],
        cached_pos[id3 as usize],
        cached_normal[id3 as usize],
    );
    output
}
