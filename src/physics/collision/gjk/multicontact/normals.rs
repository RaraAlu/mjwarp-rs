//! 上游 `collision_gjk` 的多接触法线/面恢复：box、mesh、cylinder。

use super::super::geom_type;
use super::super::math::{
    cross3, dot3, mul_mat3_vec, normalize3, scale3, sub3, transpose_mul_mat3_vec,
};
use super::super::{CYLINDER_COS_16, CYLINDER_SIN_16, FACE_TOL, Geom, MeshView};
use super::feature::{intersect1, intersect2};

/// 法线/索引/端点三元组输出（容量按上游 `nmeshdegmax = max(deg,3)`，这里放宽到 8）。
#[derive(Default)]
pub(crate) struct Normals {
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<i32>,
    pub endverts: Vec<[f32; 3]>,
}

fn blank(len: usize) -> Vec<[f32; 3]> {
    vec![[0.0; 3]; len]
}

/// 上游 `_mesh_normals`。
pub(crate) fn mesh_normals(
    feature_dim: usize,
    feature_index: [i32; 3],
    mat: &[f32; 9],
    vertadr: i32,
    polyadr: i32,
    mesh: &MeshView<'_>,
) -> Normals {
    let mut out = Normals::default();
    let v1 = feature_index[0];
    let v2 = feature_index[1];
    let v3 = feature_index[2];
    let vertadr = vertadr as usize;
    let polyadr = polyadr as usize;

    if feature_dim == 3 {
        let v1_adr = mesh.polymapadr[vertadr + v1 as usize];
        let v1_num = mesh.polymapnum[vertadr + v1 as usize];
        let v2_adr = mesh.polymapadr[vertadr + v2 as usize];
        let v2_num = mesh.polymapnum[vertadr + v2 as usize];
        let v3_adr = mesh.polymapadr[vertadr + v3 as usize];
        let v3_num = mesh.polymapnum[vertadr + v3 as usize];

        let (n, edgeset) = intersect1(
            mesh.polymap,
            mesh.polymap,
            v1_adr as usize,
            v2_adr as usize,
            v1_num as usize,
            v2_num as usize,
        );
        if n == 0 {
            return out;
        }
        let (n, faceset) = intersect2(&edgeset, mesh.polymap, v3_adr as usize, n, v3_num as usize);
        if n == 0 {
            return out;
        }
        // 三个顶点确定唯一面。
        out.normals.push(mul_mat3_vec(
            mat,
            mesh.polynormal[polyadr + faceset[0] as usize],
        ));
        out.indices.push(faceset[0]);
        return out;
    }

    if feature_dim == 2 {
        let v1_adr = mesh.polymapadr[vertadr + v1 as usize];
        let v1_num = mesh.polymapnum[vertadr + v1 as usize];
        let v2_adr = mesh.polymapadr[vertadr + v2 as usize];
        let v2_num = mesh.polymapnum[vertadr + v2 as usize];

        // 两顶点确定一条边，最多两个面。
        let (n, edgeset) = intersect1(
            mesh.polymap,
            mesh.polymap,
            v1_adr as usize,
            v2_adr as usize,
            v1_num as usize,
            v2_num as usize,
        );
        if n == 0 {
            return out;
        }
        for i in 0..n {
            out.normals.push(mul_mat3_vec(
                mat,
                mesh.polynormal[polyadr + edgeset[i] as usize],
            ));
            out.indices.push(edgeset[i]);
        }
        return out;
    }

    if feature_dim == 1 {
        let v1_adr = mesh.polymapadr[vertadr + v1 as usize] as usize;
        let v1_num = mesh.polymapnum[vertadr + v1 as usize] as usize;
        for i in 0..v1_num {
            let index = mesh.polymap[v1_adr + i];
            out.normals
                .push(mul_mat3_vec(mat, mesh.polynormal[polyadr + index as usize]));
            out.indices.push(index);
        }
        return out;
    }
    out
}

/// 上游 `_mesh_edge_normals`。
pub(crate) fn mesh_edge_normals(
    dim: usize,
    mat: &[f32; 9],
    pos: [f32; 3],
    vertadr: i32,
    polyadr: i32,
    mesh: &MeshView<'_>,
    v1: [f32; 3],
    v2: [f32; 3],
    v1i: i32,
) -> Normals {
    let mut out = Normals::default();

    if dim == 2 {
        out.endverts.push(v2);
        out.normals.push(normalize3(sub3(v2, v1)));
        return out;
    }

    if dim == 1 {
        let vertadr = vertadr as usize;
        let polyadr = polyadr as usize;
        let v1_adr = mesh.polymapadr[vertadr + v1i as usize] as usize;
        let v1_num = mesh.polymapnum[vertadr + v1i as usize] as usize;

        out.normals = blank(v1_num);
        out.endverts = blank(v1_num);

        // 遍历包含顶点 v1 的所有面。
        for i in 0..v1_num {
            let idx = mesh.polymap[v1_adr + i];
            let adr = mesh.polyvertadr[polyadr + idx as usize] as usize;
            let nvert = mesh.polyvertnum[polyadr + idx as usize] as usize;
            // 找多边形中的前一个顶点形成边。
            for j in 0..nvert {
                if mesh.polyvert[adr + j] == v1i {
                    let k = if j == 0 { nvert - 1 } else { j - 1 };
                    let end = super::super::math::add3(
                        mul_mat3_vec(mat, mesh.vert[vertadr + mesh.polyvert[adr + k] as usize]),
                        pos,
                    );
                    out.endverts[i] = end;
                    out.normals[i] = normalize3(sub3(end, v1));
                }
            }
        }
        return out;
    }
    out
}

/// 上游 `_box_normals2`：由碰撞法线反推 box 面法线。
fn box_normals2(mat: &[f32; 9], n: [f32; 3]) -> Normals {
    let mut out = Normals::default();
    let local_n = normalize3(transpose_mul_mat3_vec(mat, n));
    let face_normals = [
        [1.0_f32, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
    ];
    for (i, face) in face_normals.iter().enumerate() {
        if dot3(local_n, *face) > FACE_TOL {
            out.normals.push(mul_mat3_vec(mat, *face));
            out.indices.push(i as i32);
            return out;
        }
    }
    out
}

fn bit(value: i32, mask: i32) -> bool {
    value & mask != 0
}

/// 上游 `_box_normals`。
pub(crate) fn box_normals(
    feature_dim: usize,
    feature_index: [i32; 3],
    mat: &[f32; 9],
    dir: [f32; 3],
) -> Normals {
    let mut out = Normals::default();
    let v1 = feature_index[0];
    let v2 = feature_index[1];
    let v3 = feature_index[2];

    if feature_dim == 3 {
        let x = if bit(v1, 1) && bit(v2, 1) && bit(v3, 1) {
            1.0_f32
        } else if !bit(v1, 1) && !bit(v2, 1) && !bit(v3, 1) {
            -1.0
        } else {
            0.0
        };
        let y = if bit(v1, 2) && bit(v2, 2) && bit(v3, 2) {
            1.0_f32
        } else if !bit(v1, 2) && !bit(v2, 2) && !bit(v3, 2) {
            -1.0
        } else {
            0.0
        };
        let z = if bit(v1, 4) && bit(v2, 4) && bit(v3, 4) {
            1.0_f32
        } else if !bit(v1, 4) && !bit(v2, 4) && !bit(v3, 4) {
            -1.0
        } else {
            0.0
        };
        out.normals.push(mul_mat3_vec(mat, [x, y, z]));
        let sgn = x + y + z;
        if x != 0.0 {
            out.indices.push(0);
        }
        if y != 0.0 {
            out.indices.push(2);
        }
        if z != 0.0 {
            out.indices.push(4);
        }
        if sgn == -1.0 {
            if let Some(first) = out.indices.first_mut() {
                *first += 1;
            }
        }
        if out.indices.len() == 1 {
            return out;
        }
        return box_normals2(mat, dir);
    }

    if feature_dim == 2 {
        let x = if bit(v1, 1) && bit(v2, 1) {
            1.0_f32
        } else if !bit(v1, 1) && !bit(v2, 1) {
            -1.0
        } else {
            0.0
        };
        let y = if bit(v1, 2) && bit(v2, 2) {
            1.0_f32
        } else if !bit(v1, 2) && !bit(v2, 2) {
            -1.0
        } else {
            0.0
        };
        let z = if bit(v1, 4) && bit(v2, 4) {
            1.0_f32
        } else if !bit(v1, 4) && !bit(v2, 4) {
            -1.0
        } else {
            0.0
        };
        if x != 0.0 {
            out.normals.push(mul_mat3_vec(mat, [x, 0.0, 0.0]));
            out.indices.push(if x > 0.0 { 0 } else { 1 });
        }
        if y != 0.0 {
            out.normals.push(mul_mat3_vec(mat, [0.0, y, 0.0]));
            out.indices.push(if y > 0.0 { 2 } else { 3 });
        }
        if z != 0.0 {
            out.normals.push(mul_mat3_vec(mat, [0.0, 0.0, z]));
            out.indices.push(if z > 0.0 { 4 } else { 5 });
        }
        let count = out.normals.len();
        if count == 1 || count == 2 {
            return out;
        }
        return box_normals2(mat, dir);
    }

    if feature_dim == 1 {
        let x = if bit(v1, 1) { 1.0_f32 } else { -1.0 };
        let y = if bit(v1, 2) { 1.0_f32 } else { -1.0 };
        let z = if bit(v1, 4) { 1.0_f32 } else { -1.0 };
        out.normals.push(mul_mat3_vec(mat, [x, 0.0, 0.0]));
        out.normals.push(mul_mat3_vec(mat, [0.0, y, 0.0]));
        out.normals.push(mul_mat3_vec(mat, [0.0, 0.0, z]));
        out.indices.push(if x > 0.0 { 0 } else { 1 });
        out.indices.push(if y > 0.0 { 2 } else { 3 });
        out.indices.push(if z > 0.0 { 4 } else { 5 });
        return out;
    }
    out
}

/// 上游 `_box_edge_normals`。
pub(crate) fn box_edge_normals(
    dim: usize,
    mat: &[f32; 9],
    pos: [f32; 3],
    size: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    v1i: i32,
) -> Normals {
    let mut out = Normals::default();
    if dim == 2 {
        out.endverts.push(v2);
        out.normals.push(normalize3(sub3(v2, v1)));
        return out;
    }
    if dim == 1 {
        let x = if bit(v1i, 1) { size[0] } else { -size[0] };
        let y = if bit(v1i, 2) { size[1] } else { -size[1] };
        let z = if bit(v1i, 4) { size[2] } else { -size[2] };

        let end0 = super::super::math::add3(mul_mat3_vec(mat, [-x, y, z]), pos);
        out.endverts.push(end0);
        out.normals.push(normalize3(sub3(end0, v1)));

        let end1 = super::super::math::add3(mul_mat3_vec(mat, [x, -y, z]), pos);
        out.endverts.push(end1);
        out.normals.push(normalize3(sub3(end1, v1)));

        let end2 = super::super::math::add3(mul_mat3_vec(mat, [x, y, -z]), pos);
        out.endverts.push(end2);
        out.normals.push(normalize3(sub3(end2, v1)));
        return out;
    }
    out
}

/// 上游 `_cylinder_normals`。
pub(crate) fn cylinder_normals(
    feature_dim: usize,
    feature_index: [i32; 3],
    mat: &[f32; 9],
) -> Normals {
    let mut out = Normals::default();
    if feature_dim == 1 {
        let sgn = if feature_index[0] != 0 { -1.0_f32 } else { 1.0 };
        // mat[:, 2] 为 z 轴列。
        out.normals.push([mat[2] * sgn, mat[5] * sgn, mat[8] * sgn]);
        out.indices.push(feature_index[0]);
        return out;
    }
    out
}

/// 上游 `_cylinder_edge_normals`。
pub(crate) fn cylinder_edge_normals(
    dim: usize,
    mat: &[f32; 9],
    size: [f32; 3],
    v: [f32; 3],
    v1i: i32,
) -> Normals {
    let mut out = Normals::default();
    if dim == 1 || dim == 2 {
        let sgn = if v1i != 0 { 1.0_f32 } else { -1.0 };
        let res = [mat[2] * sgn, mat[5] * sgn, mat[8] * sgn];
        out.normals.push(res);
        let d = scale3(res, 2.0 * size[1]);
        out.endverts.push(super::super::math::add3(v, d));
        return out;
    }
    out
}

/// 上游 `_box_face`：按面索引恢复 4 个世界顶点。
pub(crate) fn box_face(mat: &[f32; 9], pos: [f32; 3], size: [f32; 3], idx: i32) -> Vec<[f32; 3]> {
    let add = |values: [f32; 3]| super::super::math::add3(mul_mat3_vec(mat, values), pos);
    match idx {
        0 => vec![
            add([size[0], size[1], size[2]]),
            add([size[0], size[1], -size[2]]),
            add([size[0], -size[1], -size[2]]),
            add([size[0], -size[1], size[2]]),
        ],
        1 => vec![
            add([-size[0], size[1], -size[2]]),
            add([-size[0], size[1], size[2]]),
            add([-size[0], -size[1], size[2]]),
            add([-size[0], -size[1], -size[2]]),
        ],
        2 => vec![
            add([-size[0], size[1], -size[2]]),
            add([size[0], size[1], -size[2]]),
            add([size[0], size[1], size[2]]),
            add([-size[0], size[1], size[2]]),
        ],
        3 => vec![
            add([-size[0], -size[1], size[2]]),
            add([size[0], -size[1], size[2]]),
            add([size[0], -size[1], -size[2]]),
            add([-size[0], -size[1], -size[2]]),
        ],
        4 => vec![
            add([-size[0], size[1], size[2]]),
            add([size[0], size[1], size[2]]),
            add([size[0], -size[1], size[2]]),
            add([-size[0], -size[1], size[2]]),
        ],
        5 => vec![
            add([size[0], size[1], -size[2]]),
            add([-size[0], size[1], -size[2]]),
            add([-size[0], -size[1], -size[2]]),
            add([size[0], -size[1], -size[2]]),
        ],
        _ => Vec::new(),
    }
}

/// 上游 `_mesh_face`：按面索引恢复多边形（顶点倒序）。
pub(crate) fn mesh_face(
    mat: &[f32; 9],
    pos: [f32; 3],
    vertadr: i32,
    polyadr: i32,
    mesh: &MeshView<'_>,
    idx: i32,
) -> Vec<[f32; 3]> {
    let vertadr = vertadr as usize;
    let polyadr = polyadr as usize;
    let adr = mesh.polyvertadr[polyadr + idx as usize] as usize;
    let nvert = mesh.polyvertnum[polyadr + idx as usize] as usize;
    let mut out = Vec::with_capacity(nvert);
    for i in (0..nvert).rev() {
        let v = mesh.vert[vertadr + mesh.polyvert[adr + i] as usize];
        out.push(super::super::math::add3(mul_mat3_vec(mat, v), pos));
    }
    out
}

/// 上游 `_cylinder_face`：16 边形端面。
pub(crate) fn cylinder_face(
    mat: &[f32; 9],
    pos: [f32; 3],
    size: [f32; 3],
    idx: i32,
) -> Vec<[f32; 3]> {
    let sgn = if idx != 0 { -1.0_f32 } else { 1.0 };
    let center = super::super::math::add3(scale3([mat[2], mat[5], mat[8]], sgn * size[1]), pos);
    let col0 = scale3([mat[0], mat[3], mat[6]], size[0]);
    let col1 = scale3([mat[1], mat[4], mat[7]], size[0] * sgn);
    let mut out = Vec::with_capacity(16);
    for i in 0..16 {
        let a = scale3(col0, CYLINDER_COS_16[i]);
        let b = scale3(col1, CYLINDER_SIN_16[i]);
        out.push(super::super::math::add3(sub3(a, b), center));
    }
    out
}

/// 上游 `_box_face`/`_mesh_face`/`_cylinder_face` 分派。
pub(crate) fn feature_face(geomtype: i32, geom: &Geom<'_>, idx: i32) -> Vec<[f32; 3]> {
    match geomtype {
        geom_type::BOX => box_face(&geom.rot, geom.pos, geom.size, idx),
        geom_type::MESH => match geom.mesh {
            Some(mesh) => mesh_face(&geom.rot, geom.pos, geom.vertadr, mesh.polyadr, mesh, idx),
            None => Vec::new(),
        },
        geom_type::CYLINDER => cylinder_face(&geom.rot, geom.pos, geom.size, idx),
        _ => Vec::new(),
    }
}

/// 上游 `_box_face` 需要的叉积（保持导入一致性）。
#[allow(dead_code)]
fn _keep_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    cross3(a, b)
}
