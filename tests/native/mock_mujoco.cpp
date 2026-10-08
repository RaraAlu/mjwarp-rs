// Test DLL only. It never links the real MuJoCo runtime.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <mujoco/mjmodel.h>
#ifndef MOCK_VERSION
#define MOCK_VERSION 3012000
#endif
static void record(char event) {
  wchar_t path[32768];
  DWORD size = GetEnvironmentVariableW(L"MJWARP_NATIVE_LIFETIME_LOG", path, 32768);
  if (!size || size >= 32768) return;
  HANDLE file = CreateFileW(path, FILE_APPEND_DATA, FILE_SHARE_READ, nullptr,
      OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
  if (file == INVALID_HANDLE_VALUE) return;
  DWORD written;
  WriteFile(file, &event, 1, &written, nullptr);
  CloseHandle(file);
}
#ifndef MOCK_MISSING_VERSION
extern "C" __declspec(dllexport) int mj_version() { return MOCK_VERSION; }
#endif
#ifndef MOCK_MISSING_LOAD
extern "C" __declspec(dllexport) mjModel* mj_loadModelBuffer(const void*, int) {
#ifdef MOCK_TRACKING
  static mjModel model{};
  static double mass = 0;
  static int parent = 0;
  model.nbody = 1;
#ifdef MOCK_INVALID_COUNTS
  model.nq = -1;
#endif
  model.body_mass = &mass;
  model.body_parentid = &parent;
#ifdef MOCK_G01
  static int address = -1, count = 0, body = 0, geom_type = 2;
  static double position[3]{}, quaternion[4]{1, 0, 0, 0}, size[3]{0.1, 0.1, 0.1};
  model.body_jntadr = &address; model.body_jntnum = &count;
  model.body_pos = position; model.body_quat = quaternion;
  model.body_ipos = position; model.body_iquat = quaternion; model.body_inertia = position;
  model.body_mocapid = &address; model.body_treeid = &address;
  model.ngeom = 1; model.geom_bodyid = &body; model.geom_pos = position;
  model.geom_quat = quaternion; model.geom_type = &geom_type; model.geom_size = size;
#ifdef MOCK_G01_MISSING_SIZE
  model.geom_size = nullptr;
#endif
#ifdef MOCK_G01_NEGATIVE_SITE
  model.nsite = -1;
#endif
#ifdef MOCK_G01_OVERSIZED_CAMERA
  model.ncam = INT64_MAX;
#endif
#endif
#ifdef MOCK_INVALID_FLEX_COUNTS
  model.nflex = -1;
#endif
#ifdef MOCK_OVERSIZED_FLEX_COUNTS
  model.nflexnode = INT64_C(2147483648);
#endif
#if defined(MOCK_MISSING_FLEX_POINTER) || defined(MOCK_FLEX_DIRECT)
  static int mode = 0, cells[3]{1, 1, 1}, zero = 0, one = 1;
  static mjtBool centered = true;
  static double vertex[3]{};
  model.nflex = 1;
  model.nflexvert = 1;
  model.flex_interp = &mode;
  model.flex_cellnum = cells;
  model.flex_nodeadr = &zero;
  model.flex_nodenum = &zero;
  model.flex_vertadr = &zero;
  model.flex_vertnum = &one;
  model.flex_centered = &centered;
  model.flex_vertbodyid = &zero;
  model.flex_vert = vertex;
#ifdef MOCK_MISSING_FLEX_POINTER
  // Fail at the last required source, after all other checks succeed.
  model.flex_vert0 = nullptr;
#else
  model.flex_vert0 = vertex;
#endif
#endif
#if defined(MOCK_MISSING_KINEMATIC_POINTER) || defined(MOCK_MISSING_INERTIAL_POINTER)
  static int address = -1, count = 0;
  static double position[3]{};
  model.body_jntadr = &address;
  model.body_jntnum = &count;
  model.body_pos = position;
#ifdef MOCK_MISSING_INERTIAL_POINTER
  static double quaternion[4]{1, 0, 0, 0};
  model.body_quat = quaternion;
  model.body_ipos = position;
  model.body_iquat = quaternion;
  // Keep the last nonempty inertial body field null for late preflight failure.
  model.body_inertia = nullptr;
#else
  // Keep the last nonempty kinematic body field null for late preflight failure.
  model.body_quat = nullptr;
#endif
#endif
  return &model;
#else
  ExitProcess(82); // Version/symbol checks must prevent model calls.
  return nullptr;
#endif
}
#endif
#ifndef MOCK_MISSING_DELETE
extern "C" __declspec(dllexport) void mj_deleteModel(mjModel*) {
#ifdef MOCK_TRACKING
  record('D');
#else
  ExitProcess(83);
#endif
}
#endif
BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
#ifdef MOCK_TRACKING
  if (reason == DLL_PROCESS_DETACH) record('U');
#else
  (void)reason;
#endif
  return TRUE;
}
