// G10@P5 G5：接触写入、nacon/naccd 容量截断与 OverflowType 汇总设备版。
// Adapted from frozen MJWarp `collision_core.py::write_contact` 与
// `collision_convex.py::eval_ccd_write_contact` / `ccd_hfield_kernel`, revision 71da24d.
// Apache-2.0.
//
// 上游实测口径（`io.py`）：`nacon` 是**全局**单计数器（`nacon: array(1)`），接触数组行全局编号
// （长度 `naconmax`），world 归属靠 `contact_worldid` 字段，**没有逐 world 行分段**
// （`naconmax` 总行 = `nconmax * nworld` 只是默认换算）。`naccdmax` 同样是全局总量，
// `nccd` 计数按 `geomgeomid` 分组、容量判定 `ccdid >= naccdmax`。
#pragma once
#include "hfield.cu"
#include "multicontact.cu"

// 上游 `ContactType.CONSTRAINT` / `ContactType.SENSOR`。
#define G5_CONTACT_TYPE_CONSTRAINT (1u << 0)
#define G5_CONTACT_TYPE_SENSOR (1u << 1)
// 上游 `vec5` 宽度（friction/solimp）。
#define G5_CONTACT_VEC5 5
// 上游 `wp.vec2` 宽度（solref/solreffriction）。
#define G5_CONTACT_VEC2 2

// 上游 `contact_*` 输出数组视图（全局行编号，容量由调用方给出）。
struct d_contact_out {
  float* dist;
  v3* pos;
  // [naconmax][9]，行优先 3×3 frame。
  float* frame;
  float* includemargin;
  float* friction;        // [naconmax][5]
  float* solref;          // [naconmax][2]
  float* solreffriction;  // [naconmax][2]
  float* solimp;          // [naconmax][5]
  int* dim;
  int* geom;              // [naconmax][2]
  // [naconmax][efc_address_width]；初始写 -1，真实 efc 编号由约束管线填。
  int* efc_address;
  int* worldid;
  int* type;
  int* geomcollisionid;
  float* adhesion;
  // 上游 `nacon`：全局检测到的接触数（可超过 naconmax，用于诊断/传感器）。
  int* nacon;
  int efc_address_width;
};

// 上游 `contact_params` / `contact_material_params` 的单对结果。
struct d_contact_params {
  // 上游 `contact_margin_gap`：pair 覆盖或两侧 geom 之和。
  float margin;
  float gap;
  int condim;
  float friction[G5_CONTACT_VEC5];
  float solref[G5_CONTACT_VEC2];
  float solreffriction[G5_CONTACT_VEC2];
  float solimp[G5_CONTACT_VEC5];
  float adhesion;
  int geoms[2];
  // 上游 `pairid`：`pairid[0]` 为 pair 匹配号，`pairid[1]` 为 collision sensor 号。
  int pairid[2];
  int worldid;
};

// 上游 `write_contact`：原子分配行号；只有 `cid < naconmax` 才写数组（不越界写）。
// 返回 1 表示 active 接触（上游语义），0 表示不活跃/被截断。
__device__ __forceinline__ int g5_write_contact(const d_contact_params& params, int naconmax,
                                                int id_, float dist_in, v3 pos_in,
                                                const float* frame_in, d_contact_out* out) {
  bool active = (dist_in < params.margin) ||
                ((params.adhesion != 0.0f) && (dist_in < params.margin + params.gap));
  bool detected = dist_in < params.margin + params.gap;
  int condim = ((params.adhesion != 0.0f) && (dist_in >= params.margin)) ? 1 : params.condim;

  // 上游：跳过不检测的 pair（非 collision sensor 且 pairid[0] == -2）。
  if (((params.pairid[0] == -2) || !detected) && (params.pairid[1] == -1)) {
    return 0;
  }

  unsigned contact_type = 0u;
  if (params.pairid[0] >= -1 && detected) contact_type |= G5_CONTACT_TYPE_CONSTRAINT;
  if (params.pairid[1] >= 0) contact_type |= G5_CONTACT_TYPE_SENSOR;

  // 上游先 atomic_add 再判容量：超容量时计数仍递增，行不写。
  int cid = atomicAdd(out->nacon, 1);
  if (cid < naconmax) {
    out->dist[cid] = dist_in;
    out->pos[cid] = pos_in;
    for (int k = 0; k < 9; ++k) out->frame[cid * 9 + k] = frame_in[k];
    out->geom[cid * 2] = params.geoms[0];
    out->geom[cid * 2 + 1] = params.geoms[1];
    out->worldid[cid] = params.worldid;
    out->includemargin[cid] = params.margin;
    out->dim[cid] = condim;
    for (int k = 0; k < G5_CONTACT_VEC5; ++k) {
      out->friction[cid * G5_CONTACT_VEC5 + k] = params.friction[k];
      out->solimp[cid * G5_CONTACT_VEC5 + k] = params.solimp[k];
    }
    for (int k = 0; k < G5_CONTACT_VEC2; ++k) {
      out->solref[cid * G5_CONTACT_VEC2 + k] = params.solref[k];
      out->solreffriction[cid * G5_CONTACT_VEC2 + k] = params.solreffriction[k];
    }
    out->adhesion[cid] = params.adhesion;
    out->type[cid] = (int)contact_type;
    // 上游 `contact_geomcollisionid_out[cid] = id_`（对内的接触序号）。
    out->geomcollisionid[cid] = id_;
    for (int k = 0; k < out->efc_address_width; ++k) {
      out->efc_address[cid * out->efc_address_width + k] = -1;
    }
    return active ? 1 : 0;
  }
  return 0;
}

// 上游 `nccd`/`naccdmax` 容量门：按 `geomgeomid` 分组原子申请 EPA 槽；
// 超容量只置 `OverflowType.CCD`，不写接触（不越界使用 EPA 缓冲）。
__device__ __forceinline__ bool g5_claim_ccd_slot(int geomgeomid, int* nccd, int naccdmax,
                                                  int worldid, unsigned* overflow_out) {
  int ccdid = atomicAdd(nccd + geomgeomid, 1);
  if (ccdid >= naccdmax) {
    atomicOr(overflow_out + worldid, G5_OVERFLOW_CCD);
    return false;
  }
  return true;
}

// 上游 `eval_ccd_write_contact` 的单对输入。
struct d_contact_pair_input {
  // 世界系几何视图（内部做原点平移）。
  d_geom geom1;
  d_geom geom2;
  int geomtype1;
  int geomtype2;
  // `opt.ccd_tolerance`、`opt.ccd_iterations`、`epa_iterations`。
  float tolerance;
  int gjk_iterations;
  int epa_iterations;
  // 上游 `pairid[1] >= 0`：collision sensor 对（cutoff=1e32、跳过 gap 早退）。
  bool collision_sensor;
  // 上游 `naccdmax` 与 `nccd[geomgeomid]` 计数器（全局容量）。
  int naccdmax;
  int geomgeomid;
  // `m.opt.warn_overflow & OverflowType.*`；设备侧不打印，只汇总位。
  unsigned* overflow_out;
};

// 上游 `eval_ccd_write_contact`：GJK → （需 EPA 时申请 naccd 槽）EPA
// → gap 早退 → margin 修正 → （可选的 multicontact）→ frame/witness → 逐接触写入。
// `use_multiccd` 对应上游 kernel 构建期的静态特化（不同入口传不同值）。
__device__ __forceinline__ bool g5_mc_supported_type(int geom_type) {
  return geom_type == G5_BOX || geom_type == G5_MESH || geom_type == G5_CYLINDER;
}

__device__ __forceinline__ void g5_ccd_pair_write(const d_contact_pair_input& in,
                                                  const d_contact_params& params, int naconmax,
                                                  bool use_multiccd, int* nccd, d_polytope* pt,
                                                  d_contact_out* out) {
  d_geom geom1 = in.geom1;
  d_geom geom2 = in.geom2;
  geom1.margin = params.margin;
  geom2.margin = params.margin;

  float cutoff = in.collision_sensor ? 1.0e32f : params.gap;

  // 上游：原点平移到 geom1 位置（平移不变性下的数值精度）。
  v3 origin = geom1.pos;
  geom1.pos = v3_make(0.0f, 0.0f, 0.0f);
  geom2.pos = v3_sub(geom2.pos, origin);

  d_gjk_phase phase;
  g5_gjk_phase(in.tolerance, cutoff, in.gjk_iterations, geom1, geom2, in.geomtype1,
               in.geomtype2, geom1.pos, geom2.pos, &phase);

  float dist = phase.dist;
  int ncontact = phase.ncontact;
  v3 w1 = phase.x1;
  v3 w2 = phase.x2;
  int multiccd_idx = -1;
  bool has_polytope = false;

  if (phase.needs_epa) {
    if (!g5_claim_ccd_slot(in.geomgeomid, nccd, in.naccdmax, params.worldid,
                           in.overflow_out)) {
      return;
    }
    d_epa_phase epa;
    g5_epa_phase(in.tolerance, in.epa_iterations, phase.result, phase.geom1, phase.geom2,
                 in.geomtype1, in.geomtype2, pt, &epa);
    dist = epa.dist;
    ncontact = epa.ncontact;
    w1 = epa.x1;
    w2 = epa.x2;
    multiccd_idx = epa.multiccd_idx;
    has_polytope = epa.has_polytope;
    if (epa.overflow != 0u) atomicOr(in.overflow_out + params.worldid, epa.overflow);
    // 上游 `eval_ccd_write_contact` 只汇总 EPA 阶段的溢出位（`EPA_HORIZON`）；
    // `CCD` 位只在上面 `nccd >= naccdmax` 的真实容量截断时置，不由 `FLOAT_MAX` 推断。
  }

  if (dist >= params.gap && !in.collision_sensor) return;

  // 上游：CCD 在 margin 膨胀形状上工作，dist 修正回真实表面距离。
  dist += params.margin;

  // 上游 multicontact 分支：`(use_multiccd or (BOX,BOX))` 且两侧类型都在
  // {BOX, MESH, CYLINDER}；`multiccd_idx > -1` 时用 EPA 最终穿透面生成 <=4 个接触。
  int ncontacts = ncontact;
  v3 witness1[4];
  v3 witness2[4];
  float dists[4];
  witness1[0] = w1;
  witness2[0] = w2;
  dists[0] = dist;
  bool want_multi = use_multiccd ||
                    (in.geomtype1 == G5_BOX && in.geomtype2 == G5_BOX);
  if (want_multi && has_polytope && multiccd_idx > -1 &&
      g5_mc_supported_type(in.geomtype1) && g5_mc_supported_type(in.geomtype2)) {
    // 上游 convex_narrowphase 的 scratch 容量上界：npolygonmax=20（A1，2026-10-31；含 cylinder）、
    // nmeshdegmax 取 10 的安全上界（本片只实现 box/cylinder/mesh 分支）。
    v3 polygon[40];
    v3 clipped[40];
    v3 pnormal[20];
    float pdist[20];
    int idx1[10];
    int idx2[10];
    v3 n1[10];
    v3 n2[10];
    v3 endvert[10];
    v3 face1[20];
    v3 face2[20];
    g5_mc_result mc = g5_multicontact(polygon, clipped, pnormal, pdist, idx1, idx2, n1, n2,
                                      endvert, face1, face2, pt->vert, pt->vert_index,
                                      pt->face[multiccd_idx], w1, w2, phase.geom1, phase.geom2,
                                      in.geomtype1, in.geomtype2);
    if (mc.n >= 1) {
      ncontacts = mc.n;
      for (int i = 0; i < 4; ++i) {
        witness1[i] = mc.witness1[i];
        witness2[i] = mc.witness2[i];
        dists[i] = mc.dists[i];
      }
    }
  }

  // 上游：重叠时 witness1 越过 witness2，方向按符号取反（frame 只算一次，全部接触共用）。
  float frame[9];
  if (dist <= params.margin) {
    make_frame(v3_sub(witness1[0], witness2[0]), frame);
  } else {
    make_frame(v3_sub(witness2[0], witness1[0]), frame);
  }

  // 上游 `for i in range(ncollision)`：n>1 时用 multicontact 的 dists[i]，否则用 EPA 的 dist；
  // 位置统一为 `0.5*(witness1[i]+witness2[i])+origin`，`id_` = 接触序号。
  for (int i = 0; i < ncontacts; ++i) {
    float contact_dist = (ncontacts > 1) ? dists[i] : dist;
    v3 pos = v3_add(v3_scale(v3_add(witness1[i], witness2[i]), 0.5f), origin);
    g5_write_contact(params, naconmax, i, contact_dist, pos, frame, out);
  }
}

// hfield 路径的接触写入：上游 `ccd_hfield_kernel` 在选好 0..3 个接触后逐个
// `write_contact`（`id_` = 0..3）；溢出位（HFIELD/CCD/EPA_HORIZON）在写接触前汇总。
__device__ __forceinline__ void g5_hfield_write_contacts(const d_hfield_result& result,
                                                         const d_contact_params& params,
                                                         int naconmax, unsigned* overflow_out,
                                                         d_contact_out* out) {
  if (result.overflow != 0u) atomicOr(overflow_out + params.worldid, result.overflow);
  for (int i = 0; i < result.ncontact; ++i) {
    g5_write_contact(params, naconmax, i, result.contact[i].dist, result.contact[i].pos,
                     result.contact[i].frame, out);
  }
}
