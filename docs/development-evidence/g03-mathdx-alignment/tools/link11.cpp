// Offline compiler/link probe for the pinned 11x11 MathDx POTRF/POTRS LTO.
// Analysis-only tool, never linked into the Rust product; no Python involved.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <cuda.h>
#include <array>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <stdexcept>
#include <string>
#include <vector>

template <class T> T entry(HMODULE module, const char* name) {
    const FARPROC raw = GetProcAddress(module, name);
    if (!raw) throw std::runtime_error(std::string("missing export: ") + name);
    static_assert(sizeof(T) == sizeof(raw));
    T typed{};
    std::memcpy(&typed, &raw, sizeof(typed));
    return typed;
}
void require(bool value, const char* message) {
    if (!value) throw std::runtime_error(message);
}
std::vector<char> read(const std::filesystem::path& path) {
    std::ifstream file(path, std::ios::binary);
    require(bool(file), "input open failed");
    std::vector<char> bytes{std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>()};
    require(!bytes.empty() && !file.bad(), "input read failed");
    return bytes;
}
int wmain(int argc, wchar_t** argv) {
    if (argc != 8) {
        std::cerr << "absolute DLL/include/LTO/fatbin/source/output/symbol required\n";
        return 1;
    }
    std::array<std::filesystem::path, 6> paths;
    for (int i = 0; i < 6; ++i) {
        paths[i] = argv[i + 1];
        if (!paths[i].is_absolute()) return 1;
    }
    const std::wstring symbol = argv[7];
    try {
        auto source = read(paths[4]), lto = read(paths[2]), fatbin = read(paths[3]);
        source.push_back('\0');
        const auto dir = paths[5];
        std::filesystem::create_directories(dir);
        const auto cubin = dir / (std::wstring(symbol.begin(), symbol.end()) + L".cubin");
        require(!std::filesystem::exists(cubin), "cubin already exists");
        HMODULE warp = LoadLibraryExW(paths[0].c_str(), nullptr,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
        require(warp != nullptr, "pinned warp.dll load failed");
        using Compile = size_t(__cdecl*)(const char*, const char*, int, const char*, const char*, int,
            const char**, bool, int, bool, bool, bool, bool, bool, bool, bool, const char*, const char*,
            size_t, char**, size_t*, int*);
        const auto compile = entry<Compile>(warp, "wp_cuda_compile_program");
        char* links[] = {lto.data(), fatbin.data()};
        size_t sizes[] = {lto.size(), fatbin.size()};
        int types[] = {3, 4}; // nvJitLink LTOIR and FATBIN enums.
        const auto include_path = paths[1].u8string(), output_path = cubin.u8string();
        const auto source_name = paths[4].filename().u8string();
        const size_t error = compile(source.data(), source_name.c_str(), 89, "", include_path.c_str(), 0,
            nullptr, false, 3, true, false, false, true, false, false, false, output_path.c_str(), nullptr,
            2, links, sizes, types);
        FreeLibrary(warp);
        require(error == 0, "native program compile/link failed");
        const auto binary = read(cubin);
        require(binary.size() >= 4 && binary[0] == 0x7f && binary[1] == 'E' && binary[2] == 'L' && binary[3] == 'F',
            "linked output is not CUBIN ELF");
        std::cout << "cubin=" << cubin.u8string() << " bytes=" << binary.size() << '\n';
        return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 2;
    }
}
