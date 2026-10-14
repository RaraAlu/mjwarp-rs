// Offline compiler/link/device probe. Never linked into the Rust product.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <cuda.h>
#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
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
    // The exact x64 C signatures follow pinned warp.h and CUDA's cuda.h.
    // Copy pointer representation, never guess an export's ABI.
    std::memcpy(&typed, &raw, sizeof(typed));
    return typed;
}
void require(bool value, const char* message) {
    if (!value) throw std::runtime_error(message);
}
void checked(CUresult result) {
    if (result != CUDA_SUCCESS) throw std::runtime_error("CUDA driver error " + std::to_string(result));
}
std::vector<char> read(const std::filesystem::path& path) {
    std::ifstream file(path, std::ios::binary);
    require(bool(file), "input open failed");
    std::vector<char> bytes{std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>()};
    require(!bytes.empty() && !file.bad(), "input read failed");
    return bytes;
}
struct Driver {
    HMODULE library = nullptr;
    CUcontext context = nullptr;
    CUmodule module = nullptr;
    std::vector<CUdeviceptr> allocations;
    decltype(&cuCtxDestroy) destroy = nullptr;
    decltype(&cuModuleUnload) unload = nullptr;
    decltype(&cuMemAlloc) alloc = nullptr;
    decltype(&cuMemFree) free = nullptr;
    decltype(&cuMemcpyHtoD) upload = nullptr;
    decltype(&cuMemcpyDtoH) download = nullptr;
    decltype(&cuLaunchKernel) launch = nullptr;
    decltype(&cuCtxSynchronize) sync = nullptr;
    ~Driver() {
        for (const auto ptr : allocations) if (free) free(ptr);
        if (module && unload) unload(module);
        if (context && destroy) destroy(context);
        if (library) FreeLibrary(library);
    }
    void start(const std::filesystem::path& cubin, std::string& gpu, int& driver_version) {
        // Only the operating system's NVIDIA driver may satisfy this DLL.
        library = LoadLibraryExW(L"nvcuda.dll", nullptr, LOAD_LIBRARY_SEARCH_SYSTEM32);
        require(library != nullptr, "system NVIDIA driver unavailable");
        destroy = entry<decltype(destroy)>(library, "cuCtxDestroy_v2");
        unload = entry<decltype(unload)>(library, "cuModuleUnload");
        alloc = entry<decltype(alloc)>(library, "cuMemAlloc_v2");
        free = entry<decltype(free)>(library, "cuMemFree_v2");
        upload = entry<decltype(upload)>(library, "cuMemcpyHtoD_v2");
        download = entry<decltype(download)>(library, "cuMemcpyDtoH_v2");
        launch = entry<decltype(launch)>(library, "cuLaunchKernel");
        sync = entry<decltype(sync)>(library, "cuCtxSynchronize");
        checked(entry<decltype(&cuInit)>(library, "cuInit")(0));
        checked(entry<decltype(&cuDriverGetVersion)>(library, "cuDriverGetVersion")(&driver_version));
        CUdevice device = 0;
        checked(entry<decltype(&cuDeviceGet)>(library, "cuDeviceGet")(&device, 0));
        std::array<char, 256> name{};
        checked(entry<decltype(&cuDeviceGetName)>(library, "cuDeviceGetName")(name.data(), int(name.size()), device));
        gpu = name.data();
        const auto attribute = entry<decltype(&cuDeviceGetAttribute)>(library, "cuDeviceGetAttribute");
        int major = 0, minor = 0;
        checked(attribute(&major, CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR, device));
        checked(attribute(&minor, CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR, device));
        require(major == 8 && minor == 9, "SM89 device required by frozen LTO");
        checked(entry<decltype(&cuCtxCreate)>(library, "cuCtxCreate_v2")(&context, 0, device));
        checked(entry<decltype(&cuModuleLoad)>(library, "cuModuleLoad")(&module, cubin.u8string().c_str()));
    }
    CUdeviceptr buffer(size_t bytes) {
        CUdeviceptr ptr = 0;
        checked(alloc(&ptr, bytes));
        allocations.push_back(ptr);
        return ptr;
    }
    void release() {
        while (!allocations.empty()) {
            checked(free(allocations.back()));
            allocations.pop_back();
        }
    }
};
struct Result {
    bool compile_called = false, loaded = false;
    size_t compile_error = size_t(-1), cubin_bytes = 0;
    int launches = 0, batches = 0, bad_pivots = 0, driver_version = 0;
    size_t scalars = 0, lower_zeros = 0, max_device_bytes = 0;
    double max_abs = 0.0, max_reconstruction = 0.0, phase_ms = 0.0;
    std::string gpu, error;
};
std::string json_string(const std::string& value) {
    std::string out = "\"";
    for (const unsigned char c : value) {
        if (c == '"' || c == '\\') { out += '\\'; out += static_cast<char>(c); }
        else out += c < 32 ? '?' : static_cast<char>(c);
    }
    return out + '"';
}
void receipt(const std::filesystem::path& dir, const Result& r) {
    std::ofstream file(dir / "receipt.json", std::ios::binary);
    file.precision(17);
    file << "{\"schema\":1,\"compile_called\":" << (r.compile_called ? "true" : "false")
         << ",\"compile_error\":" << r.compile_error << ",\"cubin_bytes\":" << r.cubin_bytes
         << ",\"loaded\":" << (r.loaded ? "true" : "false") << ",\"launches\":" << r.launches
         << ",\"batches\":" << r.batches << ",\"bad_pivots\":" << r.bad_pivots
         << ",\"scalars\":" << r.scalars << ",\"lower_zeros\":" << r.lower_zeros
         << ",\"max_abs\":" << r.max_abs << ",\"max_reconstruction\":" << r.max_reconstruction
         << ",\"phase_ms\":" << r.phase_ms << ",\"max_device_bytes\":" << r.max_device_bytes
         << ",\"gpu\":" << json_string(r.gpu) << ",\"driver_cuda_version\":" << r.driver_version
         << ",\"compile_solver_called\":false,\"capability_queries_called\":false,\"python_called\":false"
         << ",\"error\":" << json_string(r.error) << "}\n";
    require(bool(file), "receipt write failed");
}
using Matrix = std::array<float, 49>;
std::pair<Matrix, Matrix> analytic(int sample) {
    Matrix upper{}, matrix{};
    for (int i = 0; i < 7; ++i) {
        upper[i * 7 + i] = sample == 0 ? float(1 << i) : float(2 * i + 3);
        for (int j = i + 1; j < 7; ++j)
            upper[i * 7 + j] = float(((i + j) % 2 ? -1 : 1) * (i + j + 1)) / (sample == 0 ? 4.0f : 8.0f);
    }
    // Independent known U; only Gram multiplication generates M. No host
    // factorization can observe the candidate's output or define its expected U.
    for (int i = 0; i < 7; ++i) for (int j = 0; j < 7; ++j) {
        double gram = 0.0;
        for (int k = 0; k < 7; ++k) gram += double(upper[k * 7 + i]) * upper[k * 7 + j];
        matrix[i * 7 + j] = float(gram);
        require(double(matrix[i * 7 + j]) == gram, "analytic M is not exact f32");
    }
    return {matrix, upper};
}
void batch(Driver& driver, CUfunction kernel, int worlds, const Matrix& matrix,
           const Matrix& expected, int expected_info, Result& result, std::ofstream& outputs) {
    constexpr float guard = -12345.5f;
    constexpr int status_guard = 0x12345678;
    const size_t count = size_t(worlds) * 49;
    std::vector<float> input(count + 2, guard), output(count + 2, guard);
    std::vector<int> info(size_t(worlds) + 2, status_guard);
    for (int w = 0; w < worlds; ++w) std::copy(matrix.begin(), matrix.end(), input.begin() + 1 + size_t(w) * 49);
    const auto pristine = input;
    const size_t float_bytes = input.size() * sizeof(float), int_bytes = info.size() * sizeof(int);
    result.max_device_bytes = (std::max)(result.max_device_bytes, 2 * float_bytes + int_bytes);
    CUdeviceptr source_base = driver.buffer(float_bytes), output_base = driver.buffer(float_bytes), info_base = driver.buffer(int_bytes);
    checked(driver.upload(source_base, input.data(), float_bytes));
    checked(driver.upload(output_base, output.data(), float_bytes));
    checked(driver.upload(info_base, info.data(), int_bytes));
    CUdeviceptr source = source_base + sizeof(float), target = output_base + sizeof(float), status = info_base + sizeof(int);
    void* args[] = {&source, &target, &status};
    ++result.launches;
    checked(driver.launch(kernel, unsigned(worlds), 1, 1, 32, 1, 1, 0, nullptr, args, nullptr));
    checked(driver.sync());
    checked(driver.download(input.data(), source_base, float_bytes));
    checked(driver.download(output.data(), output_base, float_bytes));
    checked(driver.download(info.data(), info_base, int_bytes));
    require(std::memcmp(input.data(), pristine.data(), float_bytes) == 0, "input or input guards modified");
    require(output.front() == guard && output.back() == guard, "output guard modified");
    require(info.front() == status_guard && info.back() == status_guard, "status guard modified");
    for (int w = 0; w < worlds; ++w) {
        require(info[size_t(w) + 1] == expected_info, "unexpected POTRF info");
        if (expected_info != 0) { ++result.bad_pivots; continue; }
        const float* upper = output.data() + 1 + size_t(w) * 49;
        for (int i = 0; i < 49; ++i) {
            const double error = std::abs(double(upper[i]) - expected[i]);
            require(std::isfinite(upper[i]) && error <= 2e-4 + 2e-4 * std::abs(double(expected[i])), "U tolerance exceeded");
            result.max_abs = (std::max)(result.max_abs, error);
            ++result.scalars;
            if (i / 7 > i % 7) {
                unsigned bits = 1;
                std::memcpy(&bits, upper + i, sizeof(bits));
                require(bits == 0, "lower triangle is not positive zero");
                ++result.lower_zeros;
            }
        }
        for (int i = 0; i < 7; ++i) for (int j = 0; j < 7; ++j) {
            double reconstructed = 0.0;
            for (int k = 0; k < 7; ++k) reconstructed += double(upper[k * 7 + i]) * upper[k * 7 + j];
            const double relative = std::abs(reconstructed - matrix[i * 7 + j]) / (std::max)(1.0, std::abs(double(matrix[i * 7 + j])));
            require(relative <= 2e-6, "reconstruction tolerance exceeded");
            result.max_reconstruction = (std::max)(result.max_reconstruction, relative);
        }
    }
    outputs.write(reinterpret_cast<const char*>(output.data()), std::streamsize(float_bytes));
    outputs.write(reinterpret_cast<const char*>(info.data()), std::streamsize(int_bytes));
    require(bool(outputs), "device output archive failed");
    if (expected_info == 0) ++result.batches;
    driver.release();
}
int wmain(int argc, wchar_t** argv) {
    static_assert(sizeof(float) == 4 && sizeof(int) == 4 && sizeof(bool) == 1 && sizeof(size_t) == 8 && sizeof(void*) == 8);
    if (argc != 7) { std::cerr << "absolute DLL/include/LTO/fatbin/source/output required\n"; return 1; }
    std::array<std::filesystem::path, 6> paths;
    for (int i = 0; i < 6; ++i) { paths[i] = argv[i + 1]; if (!paths[i].is_absolute()) return 1; }
    Result result;
    HMODULE warp = nullptr;
    const auto& dir = paths[5];
    try {
        auto source = read(paths[4]), lto = read(paths[2]), fatbin = read(paths[3]);
        source.push_back('\0');
        const auto cubin = dir / "mathdx7.cubin";
        require(!std::filesystem::exists(cubin), "cubin already exists");
        warp = LoadLibraryExW(paths[0].c_str(), nullptr, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
        require(warp != nullptr, "pinned warp.dll load failed");
        using Compile = size_t (__cdecl*)(const char*, const char*, int, const char*, const char*, int,
            const char**, bool, int, bool, bool, bool, bool, bool, bool, bool, const char*, const char*,
            size_t, char**, size_t*, int*);
        const auto compile = entry<Compile>(warp, "wp_cuda_compile_program");
        char* links[] = {lto.data(), fatbin.data()};
        size_t sizes[] = {lto.size(), fatbin.size()};
        int types[] = {3, 4}; // Published nvJitLink LTOIR and FATBIN enums.
        const auto include_path = paths[1].u8string(), output_path = cubin.u8string();
        result.compile_called = true;
        result.compile_error = compile(source.data(), "native-mathdx7-call.cu", 89, "", include_path.c_str(), 0,
            nullptr, false, 3, true, false, false, true, false, false, false, output_path.c_str(), nullptr, 2, links, sizes, types);
        require(result.compile_error == 0, "native program compile/link failed");
        const auto binary = read(cubin);
        result.cubin_bytes = binary.size();
        require(binary.size() >= 4 && binary[0] == 0x7f && binary[1] == 'E' && binary[2] == 'L' && binary[3] == 'F', "linked output is not CUBIN ELF");
        Driver driver;
        driver.start(cubin, result.gpu, result.driver_version);
        CUfunction kernel = nullptr;
        checked(entry<decltype(&cuModuleGetFunction)>(driver.library, "cuModuleGetFunction")(&kernel, driver.module, "mathdx7_factor"));
        result.loaded = true;
        std::ofstream outputs(dir / "device-output.bin", std::ios::binary);
        const auto start = std::chrono::steady_clock::now();
        for (int sample = 0; sample < 2; ++sample) {
            const auto [matrix, upper] = analytic(sample);
            for (int worlds : {1, 2, 257}) batch(driver, kernel, worlds, matrix, upper, 0, result, outputs);
        }
        for (int pivot : {0, 6}) {
            Matrix matrix{}, unused{};
            for (int i = 0; i < 7; ++i) matrix[i * 7 + i] = i == pivot ? -1.0f : 1.0f;
            batch(driver, kernel, 1, matrix, unused, pivot + 1, result, outputs);
        }
        outputs.flush();
        require(bool(outputs), "device output flush failed");
        result.phase_ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start).count();
        require(result.batches == 6 && result.scalars == 25480 && result.lower_zeros == 10920 && result.bad_pivots == 2, "coverage count mismatch");
    } catch (const std::exception& error) { result.error = error.what(); }
    if (warp) FreeLibrary(warp);
    try { receipt(dir, result); } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
    std::cout << "compile=" << result.compile_error << " loaded=" << result.loaded << " batches=" << result.batches
              << " scalars=" << result.scalars << " max_abs=" << result.max_abs << " max_reconstruction=" << result.max_reconstruction << '\n';
    if (!result.error.empty()) { std::cerr << result.error << '\n'; return 2; }
    return 0;
}
