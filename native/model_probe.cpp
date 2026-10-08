#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <stddef.h>
#include <string.h>
#include <new>
#include <type_traits>
#include <mujoco/mujoco.h>
#include "mjwarp_native_probe.h"

static_assert(mjVERSION_HEADER == 3012000, "MuJoCo 3.12.0 headers required");
static_assert(sizeof(void*) == 8 && sizeof(wchar_t) == 2);
static_assert(std::is_same_v<mjtNum, double> && sizeof(mjtNum) == 8);
static_assert(std::is_same_v<mjtSize, int64_t> && sizeof(int) == 4);
static_assert(sizeof(mjwarp_native_info) == 96 && alignof(mjwarp_native_info) == 8);
static_assert(offsetof(mjwarp_native_info, native_model_bytes) == 24);
static_assert(offsetof(mjwarp_native_info, nq) == 32);
static_assert(offsetof(mjwarp_native_info, nbody) == 64);
static_assert(sizeof(mjwarp_kinematic_targets) == 128);
static_assert(alignof(mjwarp_kinematic_targets) == 8);
static_assert(offsetof(mjwarp_kinematic_targets, nq) == 8);
static_assert(offsetof(mjwarp_kinematic_targets, qpos0) == 32);
static_assert(offsetof(mjwarp_kinematic_targets, body_quat) == 72);
static_assert(offsetof(mjwarp_kinematic_targets, jnt_axis) == 120);
static_assert(sizeof(mjwarp_inertial_targets) == 96);
static_assert(alignof(mjwarp_inertial_targets) == 8);
static_assert(offsetof(mjwarp_inertial_targets, nbody) == 8);
static_assert(offsetof(mjwarp_inertial_targets, body_ipos) == 24);
static_assert(offsetof(mjwarp_inertial_targets, dof_bodyid) == 56);
static_assert(offsetof(mjwarp_inertial_targets, dof_damping) == 88);
static_assert(sizeof(mjtBool) == 1);
static_assert(sizeof(mjwarp_flex_position_info) == 32);
static_assert(alignof(mjwarp_flex_position_info) == 8);
static_assert(offsetof(mjwarp_flex_position_info, nflex) == 8);
static_assert(sizeof(mjwarp_flex_position_targets) == 128);
static_assert(alignof(mjwarp_flex_position_targets) == 8);
static_assert(offsetof(mjwarp_flex_position_targets, flex_interp) == 32);
static_assert(offsetof(mjwarp_flex_position_targets, flex_centered) == 80);
static_assert(offsetof(mjwarp_flex_position_targets, flex_vert0) == 120);

struct mjwarp_native_owner {
  HMODULE library;
  mjModel* model;
  decltype(&mj_deleteModel) delete_model;
};

extern "C" int32_t mjwarp_native_flex_position_info(
    const mjwarp_native_owner* owner, mjwarp_flex_position_info* info) {
  if (!owner || !owner->model || !info) return 8;
  const mjModel* m = owner->model;
  if (m->nflex < 0 || m->nflexnode < 0 || m->nflexvert < 0) return 8;
  *info = {1, 0, m->nflex, m->nflexnode, m->nflexvert};
  return 0;
}

extern "C" int32_t mjwarp_native_copy_flex_position(
    const mjwarp_native_owner* owner, const mjwarp_flex_position_targets* t) {
  if (!owner || !owner->model || !t || t->schema != 1 || t->reserved != 0) return 8;
  const mjModel* m = owner->model;
  if (m->nflex < 0 || m->nflexnode < 0 || m->nflexvert < 0 ||
      t->nflex != static_cast<uint64_t>(m->nflex) ||
      t->nflexnode != static_cast<uint64_t>(m->nflexnode) ||
      t->nflexvert != static_cast<uint64_t>(m->nflexvert) ||
      t->nflex > SIZE_MAX / (3 * sizeof(int32_t)) ||
      t->nflexnode > SIZE_MAX / (3 * sizeof(double)) ||
      t->nflexvert > SIZE_MAX / (3 * sizeof(double)) ||
      (t->nflex && (!t->flex_interp || !m->flex_interp ||
                   !t->flex_cellnum || !m->flex_cellnum ||
                   !t->flex_nodeadr || !m->flex_nodeadr ||
                   !t->flex_nodenum || !m->flex_nodenum ||
                   !t->flex_vertadr || !m->flex_vertadr ||
                   !t->flex_vertnum || !m->flex_vertnum ||
                   !t->flex_centered || !m->flex_centered)) ||
      (t->nflexnode && (!t->flex_nodebodyid || !m->flex_nodebodyid ||
                       !t->flex_node || !m->flex_node)) ||
      (t->nflexvert && (!t->flex_vertbodyid || !m->flex_vertbodyid ||
                       !t->flex_vert || !m->flex_vert ||
                       !t->flex_vert0 || !m->flex_vert0))) return 8;
  if (t->nflex) {
    memcpy(t->flex_interp, m->flex_interp, t->nflex * sizeof(int32_t));
    memcpy(t->flex_cellnum, m->flex_cellnum, t->nflex * 3 * sizeof(int32_t));
    memcpy(t->flex_nodeadr, m->flex_nodeadr, t->nflex * sizeof(int32_t));
    memcpy(t->flex_nodenum, m->flex_nodenum, t->nflex * sizeof(int32_t));
    memcpy(t->flex_vertadr, m->flex_vertadr, t->nflex * sizeof(int32_t));
    memcpy(t->flex_vertnum, m->flex_vertnum, t->nflex * sizeof(int32_t));
    memcpy(t->flex_centered, m->flex_centered, t->nflex * sizeof(uint8_t));
  }
  if (t->nflexnode) {
    memcpy(t->flex_nodebodyid, m->flex_nodebodyid, t->nflexnode * sizeof(int32_t));
    memcpy(t->flex_node, m->flex_node, t->nflexnode * 3 * sizeof(double));
  }
  if (t->nflexvert) {
    memcpy(t->flex_vertbodyid, m->flex_vertbodyid, t->nflexvert * sizeof(int32_t));
    memcpy(t->flex_vert, m->flex_vert, t->nflexvert * 3 * sizeof(double));
    memcpy(t->flex_vert0, m->flex_vert0, t->nflexvert * 3 * sizeof(double));
  }
  return 0;
}

extern "C" void mjwarp_native_close(mjwarp_native_owner* owner) {
  if (!owner) return;
  // Model deletion must run while its DLL still owns executable code.
  if (owner->model) owner->delete_model(owner->model);
  if (owner->library) FreeLibrary(owner->library);
  owner->~mjwarp_native_owner();
  HeapFree(GetProcessHeap(), 0, owner);
}

extern "C" int32_t mjwarp_native_open(
    const uint16_t* path, const void* mjb, int32_t bytes,
    mjwarp_native_owner** output, mjwarp_native_info* info, uint32_t* detail) {
  *output = nullptr;
  *detail = 0;
  HMODULE library = LoadLibraryExW(reinterpret_cast<const wchar_t*>(path), nullptr,
      LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32);
  if (!library) { *detail = GetLastError(); return 1; }
  auto version = reinterpret_cast<decltype(&mj_version)>(GetProcAddress(library, "mj_version"));
  if (!version) { FreeLibrary(library); return 3; }
  int actual = version();
  if (actual != mjVERSION_HEADER) {
    *detail = static_cast<uint32_t>(actual);
    FreeLibrary(library);
    return 2;
  }
  auto load = reinterpret_cast<decltype(&mj_loadModelBuffer)>(
      GetProcAddress(library, "mj_loadModelBuffer"));
  auto destroy = reinterpret_cast<decltype(&mj_deleteModel)>(
      GetProcAddress(library, "mj_deleteModel"));
  if (!load || !destroy) { FreeLibrary(library); return !load ? 4 : 5; }
  void* storage = HeapAlloc(GetProcessHeap(), 0, sizeof(mjwarp_native_owner));
  if (!storage) { FreeLibrary(library); return 7; }
  // Placement construction starts the C++17 object's lifetime explicitly.
  auto owner = new (storage) mjwarp_native_owner{library, nullptr, destroy};
  owner->model = load(mjb, bytes);
  if (!owner->model) { mjwarp_native_close(owner); return 6; }
  const mjModel* m = owner->model;
  *info = {1, mjVERSION_HEADER, sizeof(void*), sizeof(mjtNum), sizeof(int), sizeof(mjtSize),
           sizeof(mjModel), m->nq, m->nv, m->nu, m->na, m->nbody, m->njnt, m->ngeom,
           m->nsensordata};
  *output = owner;
  return 0;
}

extern "C" int32_t mjwarp_native_copy(
    const mjwarp_native_owner* owner, double* qpos0, uint64_t nq,
    double* body_mass, int32_t* body_parentid, uint64_t nbody,
    int32_t* jnt_type, uint64_t njnt) {
  const mjModel* m = owner->model;
  if (m->nq < 0 || m->nbody < 1 || m->njnt < 0 ||
      nq != static_cast<uint64_t>(m->nq) || nbody != static_cast<uint64_t>(m->nbody) ||
      njnt != static_cast<uint64_t>(m->njnt) ||
      nq > SIZE_MAX / sizeof(double) || nbody > SIZE_MAX / sizeof(double) ||
      njnt > SIZE_MAX / sizeof(int32_t) ||
      (nq && (!qpos0 || !m->qpos0)) || !body_mass || !m->body_mass ||
      !body_parentid || !m->body_parentid || (njnt && (!jnt_type || !m->jnt_type))) return 8;
  if (nq) memcpy(qpos0, m->qpos0, nq * sizeof(double));
  memcpy(body_mass, m->body_mass, nbody * sizeof(double));
  memcpy(body_parentid, m->body_parentid, nbody * sizeof(int32_t));
  if (njnt) memcpy(jnt_type, m->jnt_type, njnt * sizeof(int32_t));
  return 0;
}

extern "C" int32_t mjwarp_native_copy_inertial(
    const mjwarp_native_owner* owner, const mjwarp_inertial_targets* t) {
  if (!owner || !owner->model || !t || t->schema != 1 || t->reserved != 0) return 8;
  const mjModel* m = owner->model;
  if (m->nbody < 1 || m->nv < 0 ||
      t->nbody != static_cast<uint64_t>(m->nbody) ||
      t->nv != static_cast<uint64_t>(m->nv) ||
      t->nbody > SIZE_MAX / (4 * sizeof(double)) ||
      t->nv > SIZE_MAX / sizeof(double) ||
      !t->body_ipos || !m->body_ipos ||
      !t->body_iquat || !m->body_iquat ||
      !t->body_mass || !m->body_mass ||
      !t->body_inertia || !m->body_inertia ||
      (t->nv && (!t->dof_bodyid || !m->dof_bodyid ||
                 !t->dof_jntid || !m->dof_jntid ||
                 !t->dof_parentid || !m->dof_parentid ||
                 !t->dof_armature || !m->dof_armature ||
                 !t->dof_damping || !m->dof_damping))) return 8;
  memcpy(t->body_ipos, m->body_ipos, t->nbody * 3 * sizeof(double));
  memcpy(t->body_iquat, m->body_iquat, t->nbody * 4 * sizeof(double));
  memcpy(t->body_mass, m->body_mass, t->nbody * sizeof(double));
  memcpy(t->body_inertia, m->body_inertia, t->nbody * 3 * sizeof(double));
  if (t->nv) {
    memcpy(t->dof_bodyid, m->dof_bodyid, t->nv * sizeof(int32_t));
    memcpy(t->dof_jntid, m->dof_jntid, t->nv * sizeof(int32_t));
    memcpy(t->dof_parentid, m->dof_parentid, t->nv * sizeof(int32_t));
    memcpy(t->dof_armature, m->dof_armature, t->nv * sizeof(double));
    memcpy(t->dof_damping, m->dof_damping, t->nv * sizeof(double));
  }
  return 0;
}

extern "C" int32_t mjwarp_native_copy_kinematic(
    const mjwarp_native_owner* owner, const mjwarp_kinematic_targets* t) {
  if (!owner || !owner->model || !t || t->schema != 1 || t->reserved != 0) return 8;
  const mjModel* m = owner->model;
  if (m->nq < 0 || m->nbody < 1 || m->njnt < 0 ||
      t->nq != static_cast<uint64_t>(m->nq) ||
      t->nbody != static_cast<uint64_t>(m->nbody) ||
      t->njnt != static_cast<uint64_t>(m->njnt) ||
      t->nq > SIZE_MAX / sizeof(double) ||
      t->nbody > SIZE_MAX / (4 * sizeof(double)) ||
      t->njnt > SIZE_MAX / (3 * sizeof(double)) ||
      (t->nq && (!t->qpos0 || !m->qpos0)) ||
      !t->body_parentid || !m->body_parentid ||
      !t->body_jntadr || !m->body_jntadr || !t->body_jntnum || !m->body_jntnum ||
      !t->body_pos || !m->body_pos || !t->body_quat || !m->body_quat ||
      (t->njnt && (!t->jnt_type || !m->jnt_type ||
                  !t->jnt_bodyid || !m->jnt_bodyid ||
                  !t->jnt_qposadr || !m->jnt_qposadr ||
                  !t->jnt_dofadr || !m->jnt_dofadr ||
                  !t->jnt_pos || !m->jnt_pos || !t->jnt_axis || !m->jnt_axis))) return 8;
  if (t->nq) memcpy(t->qpos0, m->qpos0, t->nq * sizeof(double));
  memcpy(t->body_parentid, m->body_parentid, t->nbody * sizeof(int32_t));
  memcpy(t->body_jntadr, m->body_jntadr, t->nbody * sizeof(int32_t));
  memcpy(t->body_jntnum, m->body_jntnum, t->nbody * sizeof(int32_t));
  memcpy(t->body_pos, m->body_pos, t->nbody * 3 * sizeof(double));
  memcpy(t->body_quat, m->body_quat, t->nbody * 4 * sizeof(double));
  if (t->njnt) {
    memcpy(t->jnt_type, m->jnt_type, t->njnt * sizeof(int32_t));
    memcpy(t->jnt_bodyid, m->jnt_bodyid, t->njnt * sizeof(int32_t));
    memcpy(t->jnt_qposadr, m->jnt_qposadr, t->njnt * sizeof(int32_t));
    memcpy(t->jnt_dofadr, m->jnt_dofadr, t->njnt * sizeof(int32_t));
    memcpy(t->jnt_pos, m->jnt_pos, t->njnt * 3 * sizeof(double));
    memcpy(t->jnt_axis, m->jnt_axis, t->njnt * 3 * sizeof(double));
  }
  return 0;
}
