// Development-only native CPU diagnostic authoring; never a product backend.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <mujoco/mujoco.h>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <iterator>
#include <limits>
#include <locale>
#include <memory>
#include <stdexcept>
#include <vector>

static_assert(mjVERSION_HEADER == 3012000);
static_assert(sizeof(void*) == 8 && sizeof(float) == 4 && sizeof(mjtNum) == 8);
static_assert(sizeof(int) == 4);

struct Library {
  HMODULE value;
  explicit Library(const wchar_t* path)
      : value(LoadLibraryExW(path, nullptr, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32)) {
    if (!value) throw std::runtime_error("native library load failed");
  }
  ~Library() { FreeLibrary(value); }
  Library(const Library&) = delete;
  Library& operator=(const Library&) = delete;
  template <class T> T symbol(const char* name) const {
    auto p = GetProcAddress(value, name);
    if (!p) throw std::runtime_error("native symbol missing");
    return reinterpret_cast<T>(p);
  }
};

using States = std::array<std::array<float, 13>, 7>;
States read_states(const std::filesystem::path& path) {
  constexpr std::uintmax_t bytes = 8 + 3 * 4 + 7 * 13 * 4;
  if (std::filesystem::file_size(path) != bytes) throw std::runtime_error("state size mismatch");
  std::ifstream in(path, std::ios::binary);
  char magic[8]{};
  std::uint32_t header[3]{};
  if (!in.read(magic, sizeof(magic))) throw std::runtime_error("state read failed");
  if (!in.read(reinterpret_cast<char*>(header), sizeof(header))) throw std::runtime_error("state read failed");
  if (std::memcmp(magic, "G08IN01", 8) != 0 || header[0] != 7 || header[1] != 7 || header[2] != 6)
    throw std::runtime_error("state header mismatch");
  States states{};
  for (auto& state : states) {
    for (float& value : state) {
      std::uint32_t bits{};
      if (!in.read(reinterpret_cast<char*>(&bits), sizeof(bits))) throw std::runtime_error("state read failed");
      std::memcpy(&value, &bits, sizeof(value));
      if (!std::isfinite(value)) throw std::runtime_error("nonfinite state");
    }
  }
  if (!in) throw std::runtime_error("state read failed");
  return states;
}

template <class T> void array(std::ostream& out, const T* values, int count) {
  out << '[';
  for (int i = 0; i < count; ++i) {
    if (!std::isfinite(static_cast<double>(values[i]))) throw std::runtime_error("nonfinite contact");
    if (i) out << ',';
    out << values[i];
  }
  out << ']';
}

int wmain(int argc, wchar_t** argv) {
  if (argc != 5) return 2;
  try {
    // Validate the complete bit stream before any native physics call.
    const auto states = read_states(argv[3]);
    Library library(argv[1]);
    const auto version = library.symbol<decltype(&mj_version)>("mj_version");
    if (version() != 3012000) throw std::runtime_error("native version mismatch");
    const auto load = library.symbol<decltype(&mj_loadModelBuffer)>("mj_loadModelBuffer");
    const auto delete_model = library.symbol<decltype(&mj_deleteModel)>("mj_deleteModel");
    const auto make_data = library.symbol<decltype(&mj_makeData)>("mj_makeData");
    const auto delete_data = library.symbol<decltype(&mj_deleteData)>("mj_deleteData");
    const auto forward = library.symbol<decltype(&mj_forward)>("mj_forward");
    const auto name_id = library.symbol<decltype(&mj_name2id)>("mj_name2id");
    const auto model_path = std::filesystem::path(argv[2]);
    if (std::filesystem::file_size(model_path) > 1024 * 1024) throw std::runtime_error("input model too large");
    std::ifstream input(model_path, std::ios::binary);
    std::vector<char> bytes((std::istreambuf_iterator<char>(input)), std::istreambuf_iterator<char>());
    if (!input || bytes.empty()) throw std::runtime_error("model read failed");
    std::unique_ptr<mjModel, decltype(delete_model)> model(load(bytes.data(), static_cast<int>(bytes.size())), delete_model);
    if (!model || model->nq != 7 || model->nv != 6 || model->nbody != 2 || model->ngeom != 2)
      throw std::runtime_error("model layout mismatch");
    const int plane = name_id(model.get(), mjOBJ_GEOM, "plane");
    const int sphere = name_id(model.get(), mjOBJ_GEOM, "sphere");
    if (plane < 0 || sphere < 0 || model->geom_type[plane] != mjGEOM_PLANE || model->geom_type[sphere] != mjGEOM_SPHERE)
      throw std::runtime_error("geometry recipe mismatch");
    std::ofstream out(std::filesystem::path(argv[4]), std::ios::binary);
    out.imbue(std::locale::classic());
    if (!out) throw std::runtime_error("output open failed");
    out << std::setprecision(17)
        << "{\"schema\":1,\"source\":\"MuJoCo native C API CPU diagnostic\",\"native_version\":3012000,"
        << "\"state_scalar\":\"f32 bits converted to native f64\",\"result_scalar\":\"f64\","
        << "\"upstream_gpu\":false,\"physical_acceptance\":false,\"rows\":[";
    for (int index = 0; index < 7; ++index) {
      std::unique_ptr<mjData, decltype(delete_data)> data(make_data(model.get()), delete_data);
      if (!data) throw std::runtime_error("native data allocation failed");
      for (int i = 0; i < 7; ++i) data->qpos[i] = static_cast<mjtNum>(states[index][i]);
      for (int i = 0; i < 6; ++i) data->qvel[i] = static_cast<mjtNum>(states[index][7 + i]);
      forward(model.get(), data.get());
      if (data->ncon < 0 || data->ncon > 1) throw std::runtime_error("unexpected contact count");
      if (index) out << ',';
      out << "{\"state_index\":" << index << ",\"ncon\":" << data->ncon << ",\"contacts\":[";
      for (int i = 0; i < data->ncon; ++i) {
        const auto& contact = data->contact[i];
        if (!((contact.geom[0] == plane && contact.geom[1] == sphere) || (contact.geom[1] == plane && contact.geom[0] == sphere)))
          throw std::runtime_error("unexpected contact pair");
        if (i) out << ',';
        out << "{\"dist\":"; array(out, &contact.dist, 1);
        out << ",\"pos\":"; array(out, contact.pos, 3);
        out << ",\"frame\":"; array(out, contact.frame, 9);
        out << ",\"geom\":"; array(out, contact.geom, 2);
        out << ",\"dim\":" << contact.dim << ",\"includemargin\":";
        array(out, &contact.includemargin, 1);
        out << '}';
      }
      out << "]}";
    }
    out << "]}\n";
    out.flush();
    if (!out) throw std::runtime_error("output write failed");
    return 0;
  } catch (const std::exception& error) {
    std::cerr << "ERROR: " << error.what() << '\n';
    return 1;
  }
}
