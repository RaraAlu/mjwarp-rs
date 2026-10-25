//! 上游 `collision_gjk.multicontact()`：由 EPA 面恢复 ≤4 个接触 witness。

mod clip;
mod feature;
mod normals;

use clip::polygon_clip;
use feature::{aligned_face_edge, aligned_faces, feature_dim, set_edge};
use normals::{
    Normals, box_edge_normals, box_normals, cylinder_edge_normals, cylinder_normals, feature_face,
    mesh_edge_normals, mesh_normals,
};

use super::geom_type;
use super::math::{neg3, sub3};
use super::{Geom, Polytope};

/// `multicontact()` 的返回。
pub(crate) struct MultiContact {
    pub ncontact: usize,
    pub witness1: [[f32; 3]; 4],
    pub witness2: [[f32; 3]; 4],
    pub dists: [f32; 4],
}

/// 单接触回退（无可恢复的多点 feature）。
fn single(witness1: [f32; 3], witness2: [f32; 3]) -> MultiContact {
    let mut out = MultiContact {
        ncontact: 1,
        witness1: [[0.0; 3]; 4],
        witness2: [[0.0; 3]; 4],
        dists: [0.0; 4],
    };
    out.witness1[0] = witness1;
    out.witness2[0] = witness2;
    out
}

/// 面法线分派。
fn face_normals(
    geomtype: i32,
    feature_dim: usize,
    feature_index: [i32; 3],
    geom: &Geom<'_>,
    dir: [f32; 3],
) -> Normals {
    match geomtype {
        geom_type::BOX => box_normals(feature_dim, feature_index, &geom.rot, dir),
        geom_type::MESH => match geom.mesh {
            Some(mesh) => mesh_normals(
                feature_dim,
                feature_index,
                &geom.rot,
                geom.vertadr,
                mesh.polyadr,
                mesh,
            ),
            None => Normals::default(),
        },
        geom_type::CYLINDER => cylinder_normals(feature_dim, feature_index, &geom.rot),
        _ => Normals::default(),
    }
}

/// 边法线分派。
fn edge_normals(
    geomtype: i32,
    feature_dim: usize,
    feature_index: [i32; 3],
    feature_vertex: [[f32; 3]; 3],
    geom: &Geom<'_>,
) -> Normals {
    match geomtype {
        geom_type::BOX => box_edge_normals(
            feature_dim,
            &geom.rot,
            geom.pos,
            geom.size,
            feature_vertex[0],
            feature_vertex[1],
            feature_index[0],
        ),
        geom_type::MESH => match geom.mesh {
            Some(mesh) => mesh_edge_normals(
                feature_dim,
                &geom.rot,
                geom.pos,
                geom.vertadr,
                mesh.polyadr,
                mesh,
                feature_vertex[0],
                feature_vertex[1],
                feature_index[0],
            ),
            None => Normals::default(),
        },
        geom_type::CYLINDER => cylinder_edge_normals(
            feature_dim,
            &geom.rot,
            geom.size,
            feature_vertex[0],
            feature_index[0],
        ),
        _ => Normals::default(),
    }
}

/// 上游 `multicontact()`。
pub(crate) fn multicontact(
    pt: &Polytope,
    epa_face: u32,
    x1: [f32; 3],
    x2: [f32; 3],
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    geomtype1: i32,
    geomtype2: i32,
) -> MultiContact {
    let face = super::polytope::get_face_verts(epa_face);

    // 两个 geom 的 feature 维度。
    let (nface1, feature_index1, feature_vertex1) = feature_dim(face, &pt.vert_index, &pt.vert, 0);
    let (nface2, feature_index2, feature_vertex2) = feature_dim(face, &pt.vert_index, &pt.vert, 1);

    let dir = sub3(x2, x1);
    let dir_neg = neg3(dir);

    let mut normals1 = face_normals(geomtype1, nface1, feature_index1, geom1, dir_neg);
    let mut normals2 = face_normals(geomtype2, nface2, feature_index2, geom2, dir);

    let mut is_edge_contact_geom1 = false;
    let mut is_edge_contact_geom2 = false;

    let (mut found, mut res) = aligned_faces(
        &normals1.normals,
        normals1.normals.len(),
        &normals2.normals,
        normals2.normals.len(),
    );

    if !found {
        // 边-面碰撞。
        if nface1 < 3 && nface1 <= nface2 {
            normals1 = edge_normals(geomtype1, nface1, feature_index1, feature_vertex1, geom1);
            let (found2, res2) = aligned_face_edge(
                &normals1.normals,
                normals1.normals.len(),
                &normals2.normals,
                normals2.normals.len(),
                dir,
            );
            if !found2 {
                return single(x1, x2);
            }
            is_edge_contact_geom1 = true;
            res = res2;
            found = true;
        } else if nface2 < 3 {
            normals2 = edge_normals(geomtype2, nface2, feature_index2, feature_vertex2, geom2);
            let (found2, res2) = aligned_face_edge(
                &normals2.normals,
                normals2.normals.len(),
                &normals1.normals,
                normals1.normals.len(),
                dir_neg,
            );
            if !found2 {
                return single(x1, x2);
            }
            is_edge_contact_geom2 = true;
            res = res2;
            found = true;
        } else {
            // 无多接触。
            return single(x1, x2);
        }
    }
    let _ = found;

    let i = res[0];
    let j = res[1];

    // 恢复 geom1 的匹配边或面。
    let face1: Vec<[f32; 3]> = if is_edge_contact_geom1 {
        set_edge(&pt.vert, &normals1.endverts, face[0], i, 0)
    } else {
        let ind = if is_edge_contact_geom2 {
            normals1.indices[j]
        } else {
            normals1.indices[i]
        };
        feature_face(geomtype1, geom1, ind)
    };

    // 恢复 geom2 的匹配边或面。
    let face2: Vec<[f32; 3]> = if is_edge_contact_geom2 {
        set_edge(&pt.vert, &normals2.endverts, face[0], i, 1)
    } else {
        feature_face(geomtype2, geom2, normals2.indices[j])
    };

    // geom1 是边：裁剪 face2 与 face1。
    if is_edge_contact_geom1 {
        let clip = polygon_clip(
            &face2,
            face2.len(),
            &face1,
            face1.len(),
            normals2.normals[j],
            neg3(normals2.normals[j]),
        );
        // 上游把裁剪双方对调后返回，这里再翻回来。
        return MultiContact {
            ncontact: clip.ncontact,
            witness1: clip.witness2,
            witness2: clip.witness1,
            dists: clip.dists,
        };
    }

    // geom2 是边：裁剪 face2 与 face1。
    if is_edge_contact_geom2 {
        let clip = polygon_clip(
            &face1,
            face1.len(),
            &face2,
            face2.len(),
            normals1.normals[j],
            neg3(normals1.normals[j]),
        );
        return MultiContact {
            ncontact: clip.ncontact,
            witness1: clip.witness1,
            witness2: clip.witness2,
            dists: clip.dists,
        };
    }

    // 面-面碰撞。
    let clip = polygon_clip(
        &face1,
        face1.len(),
        &face2,
        face2.len(),
        normals1.normals[i],
        normals2.normals[j],
    );
    MultiContact {
        ncontact: clip.ncontact,
        witness1: clip.witness1,
        witness2: clip.witness2,
        dists: clip.dists,
    }
}
