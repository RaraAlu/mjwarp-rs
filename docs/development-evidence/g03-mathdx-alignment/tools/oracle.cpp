// Offline oracle: load a MathDx-linked cubin and run it on frozen inputs.
// Analysis-only tool; never part of the Rust product; no Python involved.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <cuda.h>
#include <cstring>
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
std::string narrow(const wchar_t* value) {
    std::string out;
    for (const wchar_t* p = value; *p; ++p) out += static_cast<char>(*p);
    return out;
}
void checked(CUresult result) {
    if (result != CUDA_SUCCESS) throw std::runtime_error("CUDA driver error " + std::to_string(result));
}
std::vector<float> read_f32(const std::string& path) {    std::ifstream file(path, std::ios::binary);
    require(bool(file), "input open failed");
    std::vector<char> bytes{std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>()};
    require(!bytes.empty() && bytes.size() % 4 == 0, "input size invalid");
    std::vector<float> values(bytes.size() / 4);
    std::memcpy(values.data(), bytes.data(), bytes.size());
    return values;
}
void write_f32(const std::string& path, const std::vector<float>& values, const int* info) {
    std::ofstream file(path, std::ios::binary);
    file.write(reinterpret_cast<const char*>(values.data()), std::streamsize(values.size() * 4));
    if (info) file.write(reinterpret_cast<const char*>(info), 4);
    require(bool(file), "output write failed");
}
int wmain(int argc, wchar_t** argv) {
    if (argc != 6) {
        std::cerr << "usage: oracle <cubin> factor|solve <input> <output> <kernel>\n";
        return 1;
    }
    try {
        const std::string mode = narrow(argv[2]);
        const std::string input_path = narrow(argv[3]);
        const std::string output_path = narrow(argv[4]);
        const std::string kernel_name = narrow(argv[5]);
        require(mode == "factor" || mode == "solve", "unknown mode");

        HMODULE driver_dll = LoadLibraryExW(L"nvcuda.dll", nullptr, LOAD_LIBRARY_SEARCH_SYSTEM32);
        require(driver_dll != nullptr, "system NVIDIA driver unavailable");
        checked(entry<decltype(&cuInit)>(driver_dll, "cuInit")(0));
        CUdevice device = 0;
        checked(entry<decltype(&cuDeviceGet)>(driver_dll, "cuDeviceGet")(&device, 0));
        CUcontext context = nullptr;
        checked(entry<decltype(&cuCtxCreate)>(driver_dll, "cuCtxCreate_v2")(&context, 0, device));
        CUmodule module = nullptr;
        checked(entry<decltype(&cuModuleLoad)>(driver_dll, "cuModuleLoad")(&module, narrow(argv[1]).c_str()));
        CUfunction kernel = nullptr;
        checked(entry<decltype(&cuModuleGetFunction)>(driver_dll, "cuModuleGetFunction")(
            &kernel, module, kernel_name.c_str()));

        const auto input = read_f32(input_path);
        const auto alloc = entry<decltype(&cuMemAlloc_v2)>(driver_dll, "cuMemAlloc_v2");
        const auto upload = entry<decltype(&cuMemcpyHtoD_v2)>(driver_dll, "cuMemcpyHtoD_v2");
        const auto download = entry<decltype(&cuMemcpyDtoH_v2)>(driver_dll, "cuMemcpyDtoH_v2");
        const auto launch = entry<decltype(&cuLaunchKernel)>(driver_dll, "cuLaunchKernel");
        const auto sync = entry<decltype(&cuCtxSynchronize)>(driver_dll, "cuCtxSynchronize");

        if (mode == "factor") {
            require(input.size() == 121, "factor input must be 121 floats");
            CUdeviceptr matrix = 0, upper = 0, info = 0;
            checked(alloc(&matrix, 121 * 4));
            checked(alloc(&upper, 121 * 4));
            checked(alloc(&info, 4));
            checked(upload(matrix, input.data(), 121 * 4));
            int zero = 0;
            checked(upload(info, &zero, 4));
            void* args[] = {&matrix, &upper, &info};
            checked(launch(kernel, 1, 1, 1, 32, 1, 1, 0, nullptr, args, nullptr));
            checked(sync());
            std::vector<float> factor(121);
            int status = -1;
            checked(download(factor.data(), upper, 121 * 4));
            checked(download(&status, info, 4));
            write_f32(output_path, factor, &status);
        } else {
            require(input.size() == 132, "solve input must be 121 factor floats + 11 rhs floats");
            CUdeviceptr factor = 0, rhs = 0, solution = 0;
            checked(alloc(&factor, 121 * 4));
            checked(alloc(&rhs, 11 * 4));
            checked(alloc(&solution, 11 * 4));
            checked(upload(factor, input.data(), 121 * 4));
            checked(upload(rhs, input.data() + 121, 11 * 4));
            void* args[] = {&factor, &rhs, &solution};
            checked(launch(kernel, 1, 1, 1, 64, 1, 1, 0, nullptr, args, nullptr));
            checked(sync());
            std::vector<float> x(11);
            checked(download(x.data(), solution, 11 * 4));
            write_f32(output_path, x, nullptr);
        }
        std::cout << "ORACLE_OK\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 2;
    }
}
