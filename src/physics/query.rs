//! G28 查询阶段切片的宿主侧实现：原语射线查询 `ray`/`rays`。
//!
//! 对齐冻结上游 `mujoco_warp.ray/rays`（U039/U040；G28；P7/P8）的
//! 原语子集：plane/sphere/capsule/ellipsoid/cylinder/box 六个原语
//! 的射线求交，输出最近命中距离、geom 编号与法线；无命中返回
//! 距离 -1、geom 编号 -1、法线零。求交公式逐分支对齐上游
//! `ray_geom`（`ray_plane`/`ray_sphere`/`ray_capsule`/`ray_ellipsoid`/
//! `ray_cylinder`/`ray_box` 与 `_ray_map`/`_ray_quad`）。
//!
//! 本片不做 BVH、不做 geomgroup/flg_static/bodyexclude 过滤、
//! 不支持 mesh/hfield/flex/SDF；`jac`（U072）与 `contact_force`（U070）
//! 依赖运动学链与碰撞/约束子系统，不在本片。
//!
//! 未验收边界：非原语 geom、过滤参数、批量世界广播（多世界）、
//! 设备内核与完整 G28 均未交付。

use std::error::Error;
use std::fmt;

/// 原语 geom 类型（对齐 `mjtGeom`）。
pub const GEOM_PLANE: i32 = 0;
pub const GEOM_SPHERE: i32 = 2;
pub const GEOM_CAPSULE: i32 = 3;
pub const GEOM_ELLIPSOID: i32 = 4;
pub const GEOM_CYLINDER: i32 = 5;
pub const GEOM_BOX: i32 = 6;
/// 上游 `mjMINVAL` 与 `mjMAXVAL`。
const MJ_MINVAL: f32 = 1.0e-15;
const MJ_MAXVAL: f32 = 1.0e10;

/// 原语查询错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryError {
    UnsupportedGeom {
        index: usize,
        geom_type: i32,
    },
    DegenerateSize {
        index: usize,
    },
    InvalidLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedGeom { index, geom_type } => {
                write!(formatter, "不支持的原语类型：geom[{index}]={geom_type}")
            }
            Self::DegenerateSize { index } => {
                write!(formatter, "退化尺寸：geom[{index}] 的 size 含 0")
            }
            Self::InvalidLength {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "长度不匹配：{field} 期望 {expected}，实际 {actual}"
            ),
        }
    }
}

impl Error for QueryError {}

/// 冻结 geom 位姿与尺寸（世界坐标系；`mat` 行主序 3×3）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeomPrimitive {
    pub geom_type: i32,
    pub pos: [f32; 3],
    pub mat: [f32; 9],
    pub size: [f32; 3],
}

/// 射线：起点与方向（世界坐标系）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub pnt: [f32; 3],
    pub vec: [f32; 3],
}

/// 查询结果：最近命中距离、geom 编号、世界坐标法线。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    dist: f32,
    geom_id: i32,
    normal: [f32; 3],
}

impl RayHit {
    pub fn dist(&self) -> f32 {
        self.dist
    }

    pub fn geom_id(&self) -> i32 {
        self.geom_id
    }

    pub fn normal(&self) -> [f32; 3] {
        self.normal
    }

    fn miss() -> Self {
        Self {
            dist: -1.0,
            geom_id: -1,
            normal: [0.0; 3],
        }
    }
}

/// 单条射线对所有 geom 求最近命中（`rays` 的单射线形式）。
pub fn ray(geoms: &[GeomPrimitive], query: Ray) -> Result<RayHit, QueryError> {
    let mut hit = RayHit::miss();
    let mut best = MJ_MAXVAL;
    for (index, geom) in geoms.iter().enumerate() {
        let (dist, normal) = ray_geom(geom, query).map_err(|error| match error {
            QueryError::UnsupportedGeom { .. } => QueryError::UnsupportedGeom {
                index,
                geom_type: geom.geom_type,
            },
            QueryError::DegenerateSize { .. } => QueryError::DegenerateSize { index },
            QueryError::InvalidLength {
                field,
                expected,
                actual,
            } => QueryError::InvalidLength {
                field,
                expected,
                actual,
            },
        })?;
        if dist >= 0.0 && dist < best {
            best = dist;
            hit = RayHit {
                dist,
                geom_id: index as i32,
                normal,
            };
        }
    }
    Ok(hit)
}

/// 批量射线查询；每条射线独立求最近命中，顺序与输入一致。
pub fn rays(geoms: &[GeomPrimitive], queries: &[Ray]) -> Result<Vec<RayHit>, QueryError> {
    queries.iter().map(|query| ray(geoms, *query)).collect()
}

/// 单个 geom 的求交：返回（距离，世界法线）；无命中返回（-1，零向量）。
pub fn ray_geom(geom: &GeomPrimitive, query: Ray) -> Result<(f32, [f32; 3]), QueryError> {
    let required: &[usize] = match geom.geom_type {
        GEOM_PLANE => &[],
        GEOM_SPHERE => &[0],
        GEOM_CAPSULE | GEOM_CYLINDER => &[0, 1],
        GEOM_ELLIPSOID | GEOM_BOX => &[0, 1, 2],
        other => {
            return Err(QueryError::UnsupportedGeom {
                index: 0,
                geom_type: other,
            });
        }
    };
    if required.iter().any(|axis| geom.size[*axis] == 0.0) {
        return Err(QueryError::DegenerateSize { index: 0 });
    }
    let zero = [0.0_f32; 3];
    let result = match geom.geom_type {
        GEOM_PLANE => ray_plane(geom, query),
        GEOM_SPHERE => ray_sphere(geom.pos, geom.size[0] * geom.size[0], query),
        GEOM_CAPSULE => ray_capsule(geom, query),
        GEOM_ELLIPSOID => ray_ellipsoid(geom, query),
        GEOM_CYLINDER => ray_cylinder(geom, query),
        GEOM_BOX => ray_box(geom, query),
        other => {
            return Err(QueryError::UnsupportedGeom {
                index: 0,
                geom_type: other,
            });
        }
    };
    Ok(if result.0 < 0.0 {
        (result.0, zero)
    } else {
        result
    })
}

fn ray_plane(geom: &GeomPrimitive, query: Ray) -> (f32, [f32; 3]) {
    let (lpnt, lvec) = ray_map(geom, query);
    if lvec[2] > -MJ_MINVAL {
        return (-1.0, [0.0; 3]);
    }
    let x = -lpnt[2] / lvec[2];
    if x < 0.0 {
        return (-1.0, [0.0; 3]);
    }
    let p = [lpnt[0] + x * lvec[0], lpnt[1] + x * lvec[1]];
    let inside_x = geom.size[0] <= 0.0 || p[0].abs() <= geom.size[0];
    let inside_y = geom.size[1] <= 0.0 || p[1].abs() <= geom.size[1];
    if inside_x && inside_y {
        let normal = [geom.mat[2], geom.mat[5], geom.mat[8]];
        (x, normal)
    } else {
        (-1.0, [0.0; 3])
    }
}

fn ray_sphere(pos: [f32; 3], dist_sqr: f32, query: Ray) -> (f32, [f32; 3]) {
    let dif = sub(query.pnt, pos);
    let a = dot(query.vec, query.vec);
    let b = dot(query.vec, dif);
    let c = dot(dif, dif) - dist_sqr;
    let (sol, _) = ray_quad(a, b, c);
    if sol >= 0.0 {
        let s = add(query.pnt, scale(query.vec, sol));
        (sol, normalize(sub(s, pos)))
    } else {
        (-1.0, [0.0; 3])
    }
}

fn ray_capsule(geom: &GeomPrimitive, query: Ray) -> (f32, [f32; 3]) {
    let size = geom.size;
    let ssz = size[0] + size[1];
    let (dist_sphere, _) = ray_sphere(geom.pos, ssz * ssz, query);
    if dist_sphere < 0.0 {
        return (-1.0, [0.0; 3]);
    }
    let (lpnt, lvec) = ray_map(geom, query);
    let mut x = -1.0_f32;
    let sq_size0 = size[0] * size[0];
    let mut a = lvec[0] * lvec[0] + lvec[1] * lvec[1];
    let mut b = lvec[0] * lpnt[0] + lvec[1] * lpnt[1];
    let mut c = lpnt[0] * lpnt[0] + lpnt[1] * lpnt[1] - sq_size0;
    let (sol, _) = ray_quad(a, b, c);
    let mut part = 0_i32;
    if sol >= 0.0 && (lpnt[2] + sol * lvec[2]).abs() <= size[1] {
        x = sol;
    }
    let mut ldif = [lpnt[0], lpnt[1], lpnt[2] - size[1]];
    a += lvec[2] * lvec[2];
    b = dot(lvec, ldif);
    c = dot(ldif, ldif) - sq_size0;
    let (_, xx) = ray_quad(a, b, c);
    for value in xx {
        if value >= 0.0 && lpnt[2] + value * lvec[2] >= size[1] && (x < 0.0 || value < x) {
            x = value;
            part = 1;
        }
    }
    ldif = [ldif[0], ldif[1], lpnt[2] + size[1]];
    b = dot(lvec, ldif);
    c = dot(ldif, ldif) - sq_size0;
    let (_, xx) = ray_quad(a, b, c);
    for value in xx {
        if value >= 0.0 && lpnt[2] + value * lvec[2] <= -size[1] && (x < 0.0 || value < x) {
            x = value;
            part = -1;
        }
    }
    if x < 0.0 {
        return (-1.0, [0.0; 3]);
    }
    let normal_z = if part == 0 {
        0.0
    } else {
        lpnt[2] + lvec[2] * x - size[1] * part as f32
    };
    let normal = [lpnt[0] + lvec[0] * x, lpnt[1] + lvec[1] * x, normal_z];
    (x, rotate(geom.mat, normalize(normal)))
}

fn ray_ellipsoid(geom: &GeomPrimitive, query: Ray) -> (f32, [f32; 3]) {
    let (lpnt, lvec) = ray_map(geom, query);
    let s = [
        safe_div(1.0, geom.size[0] * geom.size[0]),
        safe_div(1.0, geom.size[1] * geom.size[1]),
        safe_div(1.0, geom.size[2] * geom.size[2]),
    ];
    let slvec = mul(s, lvec);
    let a = dot(slvec, lvec);
    let b = dot(slvec, lpnt);
    let c = dot(mul(s, lpnt), lpnt) - 1.0;
    let (sol, _) = ray_quad(a, b, c);
    if sol >= 0.0 {
        let l = add(lpnt, scale(lvec, sol));
        let normal = rotate(geom.mat, normalize(mul(s, l)));
        (sol, normal)
    } else {
        (-1.0, [0.0; 3])
    }
}

fn ray_cylinder(geom: &GeomPrimitive, query: Ray) -> (f32, [f32; 3]) {
    let size = geom.size;
    let (dist_sphere, _) = ray_sphere(geom.pos, size[0] * size[0] + size[1] * size[1], query);
    if dist_sphere < 0.0 {
        return (-1.0, [0.0; 3]);
    }
    let (lpnt, lvec) = ray_map(geom, query);
    let mut x = -1.0_f32;
    let mut part = 0_i32;
    if lvec[2].abs() > MJ_MINVAL {
        for side in [-1.0_f32, 1.0_f32] {
            let sol = (side * size[1] - lpnt[2]) / lvec[2];
            if sol >= 0.0 {
                let p = [lpnt[0] + sol * lvec[0], lpnt[1] + sol * lvec[1]];
                if (p[0] * p[0] + p[1] * p[1]) <= size[0] * size[0] && (x < 0.0 || sol < x) {
                    x = sol;
                    part = side as i32;
                }
            }
        }
    }
    let a = lvec[0] * lvec[0] + lvec[1] * lvec[1];
    let b = lvec[0] * lpnt[0] + lvec[1] * lpnt[1];
    let c = lpnt[0] * lpnt[0] + lpnt[1] * lpnt[1] - size[0] * size[0];
    let (sol, _) = ray_quad(a, b, c);
    if sol >= 0.0 && (lpnt[2] + sol * lvec[2]).abs() <= size[1] && (x < 0.0 || sol < x) {
        x = sol;
        part = 0;
    }
    if x < 0.0 {
        return (-1.0, [0.0; 3]);
    }
    let normal = if part == 0 {
        let mut normal = [lpnt[0] + lvec[0] * x, lpnt[1] + lvec[1] * x, 0.0];
        normal = normalize(normal);
        normal
    } else {
        [0.0, 0.0, part as f32]
    };
    (x, rotate(geom.mat, normal))
}

fn ray_box(geom: &GeomPrimitive, query: Ray) -> (f32, [f32; 3]) {
    let size = geom.size;
    let (dist_sphere, _) = ray_sphere(geom.pos, dot(size, size), query);
    if dist_sphere < 0.0 {
        return (-1.0, [0.0; 3]);
    }
    let (lpnt, lvec) = ray_map(geom, query);
    let mut x = -1.0_f32;
    let mut face_axis: i32 = -1;
    let mut face_side = 0.0_f32;
    for axis in 0..3 {
        if lvec[axis].abs() > MJ_MINVAL {
            for side in [-1.0_f32, 1.0_f32] {
                let sol = (side * size[axis] - lpnt[axis]) / lvec[axis];
                if sol >= 0.0 {
                    let id0 = if axis == 0 { 1 } else { 0 };
                    let id1 = if axis == 2 { 1 } else { 2 };
                    let p0 = lpnt[id0] + sol * lvec[id0];
                    let p1 = lpnt[id1] + sol * lvec[id1];
                    if p0.abs() <= size[id0] && p1.abs() <= size[id1] && (x < 0.0 || sol < x) {
                        x = sol;
                        face_axis = axis as i32;
                        face_side = side;
                    }
                }
            }
        }
    }
    if x < 0.0 {
        return (-1.0, [0.0; 3]);
    }
    let mut normal = [0.0_f32; 3];
    normal[face_axis as usize] = face_side;
    (x, rotate(geom.mat, normal))
}

fn ray_map(geom: &GeomPrimitive, query: Ray) -> ([f32; 3], [f32; 3]) {
    let delta = sub(query.pnt, geom.pos);
    let transpose = [
        geom.mat[0],
        geom.mat[3],
        geom.mat[6],
        geom.mat[1],
        geom.mat[4],
        geom.mat[7],
        geom.mat[2],
        geom.mat[5],
        geom.mat[8],
    ];
    (rotate(transpose, delta), rotate(transpose, query.vec))
}

fn ray_quad(a: f32, b: f32, c: f32) -> (f32, [f32; 2]) {
    let det = b * b - a * c;
    if det < MJ_MINVAL {
        return (-1.0, [-1.0, -1.0]);
    }
    let det = det.sqrt();
    let den = safe_div(1.0, a);
    let x0 = (-b - det) * den;
    let x1 = (-b + det) * den;
    if x0 >= 0.0 {
        (x0, [x0, x1])
    } else if x1 >= 0.0 {
        (x1, [x0, x1])
    } else {
        (-1.0, [x0, x1])
    }
}

fn safe_div(x: f32, y: f32) -> f32 {
    x / if y != 0.0 { y } else { MJ_MINVAL }
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = dot(v, v).sqrt();
    scale(v, 1.0 / length)
}

/// 行主序 3×3 矩阵乘向量。
fn rotate(mat: [f32; 9], v: [f32; 3]) -> [f32; 3] {
    [
        mat[0] * v[0] + mat[1] * v[1] + mat[2] * v[2],
        mat[3] * v[0] + mat[4] * v[1] + mat[5] * v[2],
        mat[6] * v[0] + mat[7] * v[1] + mat[8] * v[2],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(v: [f32; 3], factor: f32) -> [f32; 3] {
    [v[0] * factor, v[1] * factor, v[2] * factor]
}

fn mul(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

mod batch;
pub use batch::{WorldGeoms, rays_batch};

#[cfg(test)]
mod tests;
