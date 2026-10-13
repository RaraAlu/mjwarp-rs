// Isolated model-reader double; it has no physics entrypoints.
#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#define EXPORT __declspec(dllexport)
#else
#include <stdio.h>
#include <stdlib.h>
#define EXPORT __attribute__((visibility("default")))
#endif
#include <mujoco/mjmodel.h>
#ifndef COLLISION_COUNT
#define COLLISION_COUNT 2
#endif
#ifndef COLLISION_MISSING
#define COLLISION_MISSING -1
#endif
#ifndef COLLISION_VERSION
#define COLLISION_VERSION 3012000
#endif
static void record(char event) {
#ifdef _WIN32
  wchar_t path[32768];
  DWORD n = GetEnvironmentVariableW(L"MJWARP_COLLISION_LIFETIME_LOG", path, 32768);
  if (!n || n >= 32768) return;
  HANDLE file = CreateFileW(path, FILE_APPEND_DATA, FILE_SHARE_READ, nullptr,
      OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
  if (file == INVALID_HANDLE_VALUE) return;
  DWORD written;
  WriteFile(file, &event, 1, &written, nullptr);
  CloseHandle(file);
#else
  const char* path = getenv("MJWARP_COLLISION_LIFETIME_LOG");
  if (!path) return;
  FILE* file = fopen(path, "ab");
  if (file) { fputc(event, file); fclose(file); }
#endif
}
extern "C" EXPORT int mj_version() { return COLLISION_VERSION; }
extern "C" EXPORT mjModel* mj_loadModelBuffer(const void*, int) {
#if COLLISION_VERSION != 3012000
  record('L');
  return nullptr;
#else
  static mjModel m{};
  static int type[2]{0, 2}, body[2]{0, 1}, mask[2]{1, 1}, dim[2]{3, 3}, priority[2]{};
  static double scalar[2]{0.125, 0.125}, ref[4]{0.02, 1, 0.02, 1};
  static double imp[10]{}, size[6]{2, 2, 0.125, 0.25, 0, 0}, friction[6]{}, gap[2]{};
  m.nbody = 2;
  m.ngeom = COLLISION_COUNT;
  m.geom_type = type; m.geom_bodyid = body; m.geom_contype = mask; m.geom_conaffinity = mask;
  m.geom_condim = dim; m.geom_priority = priority; m.geom_solmix = scalar; m.geom_solref = ref;
  m.geom_solimp = imp; m.geom_size = size; m.geom_friction = friction;
  m.geom_margin = scalar; m.geom_gap = gap;
  switch (COLLISION_MISSING) {
    case 0: m.geom_type = nullptr; break;
    case 1: m.geom_bodyid = nullptr; break;
    case 2: m.geom_contype = nullptr; break;
    case 3: m.geom_conaffinity = nullptr; break;
    case 4: m.geom_condim = nullptr; break;
    case 5: m.geom_priority = nullptr; break;
    case 6: m.geom_solmix = nullptr; break;
    case 7: m.geom_solref = nullptr; break;
    case 8: m.geom_solimp = nullptr; break;
    case 9: m.geom_size = nullptr; break;
    case 10: m.geom_friction = nullptr; break;
    case 11: m.geom_margin = nullptr; break;
    case 12: m.geom_gap = nullptr; break;
    default: break;
  }
  return &m;
#endif
}
extern "C" EXPORT void mj_deleteModel(mjModel*) { record('D'); }
#ifdef _WIN32
BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
  if (reason == DLL_PROCESS_DETACH) record('U');
  return TRUE;
}
#else
__attribute__((destructor)) static void unload() { record('U'); }
#endif
