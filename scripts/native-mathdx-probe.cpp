// Offline native compiler capability probe. Never linked into the Rust product.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>

template <class T> T entry(HMODULE module, const char* name) {
  const FARPROC raw = GetProcAddress(module, name);
  if (!raw) throw std::runtime_error(std::string("missing export: ") + name);
  static_assert(sizeof(T) == sizeof(raw));
  T typed{};
  // Exact C ABI signatures below follow warp.h and context.py; x64 pointers
  // share representation. Do not call an export with a guessed prototype.
  std::memcpy(&typed, &raw, sizeof(typed));
  return typed;
}
std::string json_string(const std::string& value) {
  std::string out = "\"";
  for (const unsigned char c : value) {
    if (c == '"' || c == '\\') { out += '\\'; out += static_cast<char>(c); }
    else if (c < 32) out += '?';
    else out += static_cast<char>(c);
  }
  return out + '"';
}
struct Result {
  std::string version, mathdx_version, error;
  int cuda = -1, mathdx = -1;
  bool compile_called = false, compiled = false;
  uintmax_t lto_bytes = 0, fatbin_bytes = 0;
};
void receipt(const std::filesystem::path& dir, const Result& r) {
  std::ofstream out(dir / "receipt.json", std::ios::binary);
  out << "{\"schema\":1,\"warp_version\":" << json_string(r.version)
      << ",\"cuda_enabled\":" << r.cuda << ",\"mathdx_enabled\":" << r.mathdx
      << ",\"mathdx_version\":" << json_string(r.mathdx_version)
      << ",\"compile_called\":" << (r.compile_called ? "true" : "false")
      << ",\"compiled\":" << (r.compiled ? "true" : "false")
      << ",\"lto_bytes\":" << r.lto_bytes << ",\"fatbin_bytes\":" << r.fatbin_bytes
      << ",\"symbol\":\"potrf_7_7_1_89_32_1_1_5_x_x_0\",\"precision\":5"
      << ",\"M\":7,\"N\":7,\"K\":1,\"arch\":89,\"threads\":32"
      << ",\"function\":2,\"side\":-1,\"diag\":-1,\"fill\":0"
      << ",\"arrangement_A\":1,\"arrangement_B\":1,\"includes\":0"
      << ",\"wp_init_called\":false,\"python_called\":false,\"physical_kernel_called\":false"
      << ",\"error\":" << json_string(r.error) << "}\n";
  if (!out) throw std::runtime_error("receipt write failed");
}
int wmain(int argc, wchar_t** argv) {
  static_assert(sizeof(int) == 4 && sizeof(bool) == 1 && sizeof(void*) == 8);
  if (argc != 3) { std::cerr << "absolute warp.dll and output directory required\n"; return 1; }
  const std::filesystem::path dll(argv[1]), dir(argv[2]);
  if (!dll.is_absolute() || !dir.is_absolute() || !std::filesystem::is_directory(dir)) return 1;
  Result result;
  HMODULE module = nullptr;
  try {
    module = LoadLibraryExW(dll.c_str(), nullptr, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
    if (!module) throw std::runtime_error("LoadLibraryExW failed: " + std::to_string(GetLastError()));
    using GetInt = int (__cdecl*)();
    using GetString = const char* (__cdecl*)();
    const char* version = entry<GetString>(module, "wp_version")();
    const char* mathdx_version = entry<GetString>(module, "wp_libmathdx_version")();
    result.version = version ? version : "";
    result.mathdx_version = mathdx_version ? mathdx_version : "";
    result.cuda = entry<GetInt>(module, "wp_is_cuda_enabled")();
    result.mathdx = entry<GetInt>(module, "wp_is_mathdx_enabled")();
    if (result.version != "1.15.0" || result.cuda != 1 || result.mathdx != 1 || result.mathdx_version.empty())
      throw std::runtime_error("pinned native CUDA/MathDx capability unavailable");
    using CompileSolver = bool (__cdecl*)(const char*, const char*, const char*, int,
        const char**, const char*, int, int, int, int, int, int, int, int, int, int, int, int);
    const auto compile = entry<CompileSolver>(module, "wp_cuda_compile_solver");
    const auto lto = dir / "potrf7.lto", fatbin = dir / "potrf7_fatbin.lto";
    if (std::filesystem::exists(lto) || std::filesystem::exists(fatbin)) throw std::runtime_error("outputs already exist");
    const std::string lto_path = lto.u8string(), fatbin_path = fatbin.u8string();
    result.compile_called = true;
    result.compiled = compile(fatbin_path.c_str(), lto_path.c_str(), "potrf_7_7_1_89_32_1_1_5_x_x_0",
        0, nullptr, nullptr, 89, 7, 7, 1, 2, -1, -1, 5, 1, 1, 0, 32);
    if (std::filesystem::exists(lto)) result.lto_bytes = std::filesystem::file_size(lto);
    if (std::filesystem::exists(fatbin)) result.fatbin_bytes = std::filesystem::file_size(fatbin);
    if (!result.compiled || result.lto_bytes == 0 || result.fatbin_bytes == 0)
      throw std::runtime_error("native solver compilation failed or produced empty output");
  } catch (const std::exception& error) { result.error = error.what(); }
  if (module) FreeLibrary(module);
  try { receipt(dir, result); } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
  std::cout << "Warp=" << result.version << " MathDx=" << result.mathdx_version
      << " enabled=" << result.mathdx << " compile=" << result.compiled
      << " LTO=" << result.lto_bytes << " fatbin=" << result.fatbin_bytes << '\n';
  if (!result.error.empty()) { std::cerr << result.error << '\n'; return 2; }
  return 0;
}
