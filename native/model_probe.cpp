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

struct mjwarp_native_owner {
  HMODULE library;
  mjModel* model;
  decltype(&mj_deleteModel) delete_model;
};

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
