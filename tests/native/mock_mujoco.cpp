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
