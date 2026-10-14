// Reuse both original immutable bridges and their private owner in one TU.
#include "model_probe_collision.cpp"
#include "mjwarp_sparse_probe.h"

static_assert(sizeof(mjwarp_sparse_mass_info) == 32);
static_assert(alignof(mjwarp_sparse_mass_info) == 8);
static_assert(offsetof(mjwarp_sparse_mass_info, nv) == 8);
static_assert(offsetof(mjwarp_sparse_mass_info, ntree) == 24);
static_assert(sizeof(mjwarp_sparse_mass_targets) == 80);
static_assert(alignof(mjwarp_sparse_mass_targets) == 8);
static_assert(offsetof(mjwarp_sparse_mass_targets, dof_parentid) == 32);
static_assert(offsetof(mjwarp_sparse_mass_targets, M_colind) == 56);
static_assert(offsetof(mjwarp_sparse_mass_targets, tree_dofnum) == 72);
#define WIDTH(field) static_assert(sizeof(*static_cast<mjModel*>(nullptr)->field) == sizeof(int32_t))
WIDTH(dof_parentid); WIDTH(M_rownnz); WIDTH(M_rowadr); WIDTH(M_colind);
WIDTH(tree_dofadr); WIDTH(tree_dofnum);
#undef WIDTH

static bool sparse_counts(const mjModel* m, mjwarp_sparse_mass_info& out) {
  if (m->nv < 0 || m->nC < 0 || m->ntree < 0) return false;
  const uint64_t limit = static_cast<uint64_t>(INT32_MAX) / 4;
  const uint64_t nv = static_cast<uint64_t>(m->nv);
  const uint64_t nc = static_cast<uint64_t>(m->nC);
  const uint64_t ntree = static_cast<uint64_t>(m->ntree);
  if (nv > limit || nc > limit || ntree > limit) return false;
  out = {1, 0, nv, nc, ntree};
  return true;
}
extern "C" int32_t mjwarp_native_sparse_mass_info(
    const mjwarp_native_owner* owner, mjwarp_sparse_mass_info* info) {
  if (!owner || !owner->model || !info) return 8;
  mjwarp_sparse_mass_info staged{};
  if (!sparse_counts(owner->model, staged)) return 8;
  *info = staged;
  return 0;
}
extern "C" int32_t mjwarp_native_copy_sparse_mass(
    const mjwarp_native_owner* owner, const mjwarp_sparse_mass_targets* t) {
  if (!owner || !owner->model || !t || t->info.schema != 1 || t->info.reserved != 0) return 8;
  const mjModel* m = owner->model;
  mjwarp_sparse_mass_info info{};
  if (!sparse_counts(m, info) || t->info.nv != info.nv || t->info.nC != info.nC || t->info.ntree != info.ntree) return 8;
#define REQUIRED(field, count) if ((count) && (!t->field || !m->field)) return 8
  REQUIRED(dof_parentid, info.nv); REQUIRED(M_rownnz, info.nv); REQUIRED(M_rowadr, info.nv);
  REQUIRED(M_colind, info.nC); REQUIRED(tree_dofadr, info.ntree); REQUIRED(tree_dofnum, info.ntree);
#undef REQUIRED
  // Every source/count/target has passed before any destination is changed.
#define COPY(field, count) if (count) memcpy(t->field, m->field, static_cast<size_t>(count) * sizeof(int32_t))
  COPY(dof_parentid, info.nv); COPY(M_rownnz, info.nv); COPY(M_rowadr, info.nv);
  COPY(M_colind, info.nC); COPY(tree_dofadr, info.ntree); COPY(tree_dofnum, info.ntree);
#undef COPY
  return 0;
}
