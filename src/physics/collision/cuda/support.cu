// G10@P5 G5：支持点函数设备版，与 host `gjk/support.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once
#include "geom.cu"

#define G5_PLANE 0
#define G5_HFIELD 1
#define G5_SPHERE 2
#define G5_CAPSULE 3
#define G5_ELLIPSOID 4
#define G5_CYLINDER 5
#define G5_BOX 6
#define G5_MESH 7
#define G5_SDF 8
#define G5_FLEX 105
#define G5_TRIANGLE 999

#define G5_FLOAT_MIN (-1.0e30f)
#define G5_MINVAL 1.0e-15f

__device__ __forceinline__ void support_mesh(const d_geom& geom, const d_mesh& mesh,
                                             v3 local_dir, d_support_point* point) {
  unsigned vertadr = (unsigned)(geom.vertadr > 0 ? geom.vertadr : 0);
  unsigned vertnum = (unsigned)(geom.vertnum > 0 ? geom.vertnum : 0);
  float max_dist = G5_FLOAT_MIN;
  if (geom.graphadr == -1 || geom.vertnum < 10) {
    if (geom.index > -1) {
      unsigned index = (unsigned)geom.index;
      point->cached_index = geom.index;
      max_dist = v3_dot(v3_make(mesh.vert[index * 3], mesh.vert[index * 3 + 1],
                                mesh.vert[index * 3 + 2]),
                        local_dir);
      point->point = v3_make(mesh.vert[index * 3], mesh.vert[index * 3 + 1],
                             mesh.vert[index * 3 + 2]);
    }
    for (unsigned i = 0; i < vertnum; ++i) {
      unsigned base = (vertadr + i) * 3;
      v3 vert = v3_make(mesh.vert[base], mesh.vert[base + 1], mesh.vert[base + 2]);
      float dist = v3_dot(vert, local_dir);
      if (dist > max_dist) {
        max_dist = dist;
        point->point = vert;
        point->cached_index = (int)(vertadr + i);
      }
    }
    point->vertex_index = point->cached_index - geom.vertadr;
  } else {
    unsigned graphadr = (unsigned)(geom.graphadr > 0 ? geom.graphadr : 0);
    unsigned numvert = (unsigned)mesh.graph[graphadr];
    unsigned vert_edgeadr = graphadr + 2;
    unsigned vert_globalid = graphadr + 2 + numvert;
    unsigned edge_localid = graphadr + 2 + 2 * numvert;
    int prev = -1;
    unsigned imax = geom.index > -1 ? (unsigned)geom.index : 0u;
    {
      unsigned idx = (unsigned)mesh.graph[vert_globalid + imax];
      unsigned base = (vertadr + idx) * 3;
      max_dist = v3_dot(local_dir, v3_make(mesh.vert[base], mesh.vert[base + 1],
                                           mesh.vert[base + 2]));
    }
    while ((int)imax != prev) {
      prev = (int)imax;
      unsigned i = (unsigned)mesh.graph[vert_edgeadr + imax];
      int subidx = mesh.graph[edge_localid + i];
      while (subidx >= 0) {
        unsigned idx = (unsigned)mesh.graph[vert_globalid + (unsigned)subidx];
        unsigned base = (vertadr + idx) * 3;
        v3 vert = v3_make(mesh.vert[base], mesh.vert[base + 1], mesh.vert[base + 2]);
        float dist = v3_dot(local_dir, vert);
        if (dist > max_dist) {
          imax = (unsigned)subidx;
          max_dist = dist;
        }
        i += 1;
        subidx = mesh.graph[edge_localid + i];
      }
    }
    point->cached_index = (int)imax;
    point->vertex_index = mesh.graph[vert_globalid + imax];
    unsigned base = (vertadr + (unsigned)point->vertex_index) * 3;
    point->point = v3_make(mesh.vert[base], mesh.vert[base + 1], mesh.vert[base + 2]);
  }
  point->point = v3_add(mul_mat3_vec(geom.rot, point->point), geom.pos);
}

__device__ __forceinline__ d_support_point support(const d_geom& geom, int geomtype,
                                                   v3 dir) {
  d_support_point point;
  point.point = v3_make(0.0f, 0.0f, 0.0f);
  point.cached_index = -1;
  point.vertex_index = 0;

  if (geomtype == G5_SPHERE) {
    point.point = v3_add(geom.pos, v3_scale(dir, geom.size.x + 0.5f * geom.margin));
    return point;
  }

  v3 local_dir = transpose_mul_mat3_vec(geom.rot, dir);

  if (geomtype == G5_BOX) {
    v3 tmp = v3_sign(local_dir);
    v3 res = v3_cw_mul(tmp, geom.size);
    point.point = v3_add(mul_mat3_vec(geom.rot, res), geom.pos);
    int vertex_index = 0;
    if (tmp.x > 0.0f) vertex_index += 1;
    if (tmp.y > 0.0f) vertex_index += 2;
    if (tmp.z > 0.0f) vertex_index += 4;
    point.vertex_index = vertex_index;
  } else if (geomtype == G5_CAPSULE) {
    v3 res = v3_scale(local_dir, geom.size.x);
    res.z += local_dir.z < 0.0f ? -geom.size.y : geom.size.y;
    point.point = v3_add(mul_mat3_vec(geom.rot, res), geom.pos);
  } else if (geomtype == G5_ELLIPSOID) {
    v3 res = v3_normalize(v3_cw_mul(local_dir, geom.size));
    res = v3_cw_mul(res, geom.size);
    point.point = v3_add(mul_mat3_vec(geom.rot, res), geom.pos);
  } else if (geomtype == G5_CYLINDER) {
    v3 res = v3_make(0.0f, 0.0f, 0.0f);
    float d = sqrtf(local_dir.x * local_dir.x + local_dir.y * local_dir.y);
    if (d > G5_MINVAL) {
      float scl = geom.size.x / d;
      res.x = local_dir.x * scl;
      res.y = local_dir.y * scl;
    }
    bool top = local_dir.z >= 0.0f;
    res.z = top ? geom.size.y : -geom.size.y;
    point.vertex_index = top ? 0 : 1;
    point.point = v3_add(mul_mat3_vec(geom.rot, res), geom.pos);
  } else if (geomtype == G5_MESH) {
    support_mesh(geom, geom.mesh, local_dir, &point);
  } else if (geomtype == G5_HFIELD) {
    point.vertex_index = dir.z < 0.0f ? -2 : -3;
    float max_dist = G5_FLOAT_MIN;
    for (int slot = 0; slot < 6; ++slot) {
      v3 vert = v3_make(geom.polyvert[slot * 3], geom.polyvert[slot * 3 + 1],
                        geom.polyvert[slot * 3 + 2]);
      float dist = v3_dot(vert, dir);
      if (dist > max_dist) {
        max_dist = dist;
        point.point = vert;
      }
    }
  } else if (geomtype == G5_TRIANGLE) {
    v3 t1 = v3_make(geom.rot[0], geom.rot[1], geom.rot[2]);
    v3 t2 = v3_make(geom.rot[3], geom.rot[4], geom.rot[5]);
    v3 t3 = v3_make(geom.rot[6], geom.rot[7], geom.rot[8]);
    float d1 = v3_dot(t1, dir);
    float d2 = v3_dot(t2, dir);
    float d3 = v3_dot(t3, dir);
    point.point = (d1 > d2 && d1 > d3) ? t1 : (d2 > d3 ? t2 : t3);
  } else if (geomtype == G5_FLEX) {
    v3 p0 = v3_make(geom.polyvert[0], geom.polyvert[1], geom.polyvert[2]);
    v3 p1 = v3_make(geom.polyvert[3], geom.polyvert[4], geom.polyvert[5]);
    v3 p2 = v3_make(geom.polyvert[6], geom.polyvert[7], geom.polyvert[8]);
    v3 p3 = v3_make(geom.polyvert[9], geom.polyvert[10], geom.polyvert[11]);
    float d0 = v3_dot(p0, dir);
    float d1 = v3_dot(p1, dir);
    float d2 = v3_dot(p2, dir);
    float d3 = v3_dot(p3, dir);
    point.point = (d0 > d1 && d0 > d2 && d0 > d3)
                      ? p0
                      : (d1 > d2 && d1 > d3 ? p1 : (d2 > d3 ? p2 : p3));
  }

  if (geom.margin > 0.0f) {
    point.point = v3_add(point.point, v3_scale(dir, 0.5f * geom.margin));
  }
  return point;
}

__device__ __forceinline__ void support_pair(const d_geom& geom1, const d_geom& geom2,
                                             int geomtype1, int geomtype2, v3 dir,
                                             d_support_point* p1,
                                             d_support_point* p2) {
  *p1 = support(geom1, geomtype1, dir);
  *p2 = support(geom2, geomtype2, v3_neg(dir));
}

__device__ __forceinline__ bool discrete_geoms(int type1, int type2) {
  bool d1 = type1 == G5_MESH || type1 == G5_BOX || type1 == G5_HFIELD;
  bool d2 = type2 == G5_MESH || type2 == G5_BOX || type2 == G5_HFIELD;
  return d1 && d2;
}
