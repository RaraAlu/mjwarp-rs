// G0b：用 MuJoCo 3.12.0 C API 编译 G10@P5 模型，生成产品侧 MJB 并 dump
// 碰撞相关字段（hfield / octree / SDF options）的原始小端字节。
//
// OFFLINE ONLY：不进产品构建/测试/门禁；不运行产品代码；不占 GPU。
// 3.12 实测支持 SDF/octree：mjGEOM_SDF、opt.sdf_*、noct/oct_depth/oct_child/
// oct_aabb/oct_coeff、hfield_* 均在 mjmodel.h 中。
//
// 用法：
//   dump_collision_fields_312 <scene.xml> <out.mjb> <out_dir>
//
// 输出：
//   <out.mjb>                 3.12 编译产物（同版本 mj_loadModel 重载校验）
//   <out_dir>/<field>.bin     各字段原始字节（i32/f32/f64/i64 小端；与 3.15 dump 同格式）
//   <out_dir>/fields.json     schema/version/mjb/counts/fields 清单（sha 由 parity 脚本算）
#include <mujoco/mujoco.h>

#include <direct.h>
#include <cerrno>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

namespace {

struct FieldEntry {
  std::string name;
  std::string dtype;
  long long count;
  std::string file;
};

std::vector<FieldEntry> g_fields;

int element_bytes(const char* dtype) {
  if (std::strcmp(dtype, "f64") == 0 || std::strcmp(dtype, "i64") == 0) {
    return 8;
  }
  return 4;
}

bool write_bin(const std::string& path, const void* data, long long count, int bytes) {
  FILE* file = std::fopen(path.c_str(), "wb");
  if (!file) {
    std::fprintf(stderr, "cannot open %s\n", path.c_str());
    return false;
  }
  const std::size_t total = static_cast<std::size_t>(count) * static_cast<std::size_t>(bytes);
  const bool ok = std::fwrite(data, 1, total, file) == total;
  std::fclose(file);
  if (!ok) {
    std::fprintf(stderr, "short write %s\n", path.c_str());
  }
  return ok;
}

void dump_field(const std::string& dir, const char* name, const char* dtype, const void* data,
                long long count) {
  const int bytes = element_bytes(dtype);
  const std::string path = dir + "/" + name + ".bin";
  if (!write_bin(path, data, count, bytes)) {
    std::exit(5);
  }
  g_fields.push_back(FieldEntry{name, dtype, count, std::string(name) + ".bin"});
}

void write_json_string(FILE* out, const char* text) {
  std::fputc('"', out);
  for (const char* p = text; *p; ++p) {
    const unsigned char c = static_cast<unsigned char>(*p);
    if (c == '"' || c == '\\') {
      std::fputc('\\', out);
      std::fputc(c, out);
    } else {
      std::fputc(c, out);
    }
  }
  std::fputc('"', out);
}

bool ensure_dir(const char* path) {
  if (_mkdir(path) == 0) {
    return true;
  }
  // 已存在也视为成功（errno == EEXIST = 17）
  return errno == 17;
}

}  // namespace

int main(int argc, char** argv) {
  if (argc != 4) {
    std::fprintf(stderr, "usage: %s <scene.xml> <out.mjb> <out_dir>\n", argv[0]);
    return 1;
  }
  char error[1000] = {0};
  mjModel* compiled = mj_loadXML(argv[1], nullptr, error, sizeof(error));
  if (!compiled) {
    std::fprintf(stderr, "mj_loadXML failed: %s\n", error);
    return 2;
  }
  mj_saveModel(compiled, argv[2], nullptr, 0);
  mj_deleteModel(compiled);
  // 同版本重载校验（双版本互斥：3.15 不能加载 3.12 MJB，故各自校验）。
  mjModel* m = mj_loadModel(argv[2], nullptr);
  if (!m) {
    std::fprintf(stderr, "mj_loadModel failed: %s\n", argv[2]);
    return 3;
  }
  if (!ensure_dir(argv[3])) {
    std::fprintf(stderr, "cannot create %s\n", argv[3]);
    mj_deleteModel(m);
    return 4;
  }

  const std::string dir = argv[3];
  const long long noct = static_cast<long long>(m->noct);
  const long long ngeom = static_cast<long long>(m->ngeom);
  const long long nhfield = static_cast<long long>(m->nhfield);
  const long long nhfielddata = static_cast<long long>(m->nhfielddata);
  const long long nmesh = static_cast<long long>(m->nmesh);

  // 标量计数
  long long counts[6] = {ngeom, nhfield, nhfielddata, nmesh, noct,
                         static_cast<long long>(m->nbody)};
  dump_field(dir, "counts", "i64", counts, 6);
  long long qv[2] = {static_cast<long long>(m->nq), static_cast<long long>(m->nv)};
  dump_field(dir, "nq_nv", "i64", qv, 2);

  // hfield
  dump_field(dir, "hfield_nrow", "i32", m->hfield_nrow, nhfield);
  dump_field(dir, "hfield_ncol", "i32", m->hfield_ncol, nhfield);
  dump_field(dir, "hfield_adr", "i32", m->hfield_adr, nhfield);
  dump_field(dir, "hfield_size", "f64", m->hfield_size, nhfield * 4);
  dump_field(dir, "hfield_data", "f32", m->hfield_data, nhfielddata);

  // octree（volume SDF）
  dump_field(dir, "oct_depth", "i32", m->oct_depth, noct);
  dump_field(dir, "oct_child", "i32", m->oct_child, noct * 8);
  dump_field(dir, "oct_aabb", "f64", m->oct_aabb, noct * 6);
  dump_field(dir, "oct_coeff", "f64", m->oct_coeff, noct * 8);
  dump_field(dir, "mesh_octadr", "i32", m->mesh_octadr, nmesh);
  dump_field(dir, "mesh_octnum", "i32", m->mesh_octnum, nmesh);

  // SDF 碰撞选项（m.opt）
  int sdf_options[2] = {m->opt.sdf_iterations, m->opt.sdf_initpoints};
  dump_field(dir, "opt_sdf", "i32", sdf_options, 2);

  // geom 基础字段（确认模型与 margin 路径）
  dump_field(dir, "geom_type", "i32", m->geom_type, ngeom);
  dump_field(dir, "geom_dataid", "i32", m->geom_dataid, ngeom);
  dump_field(dir, "geom_size", "f64", m->geom_size, ngeom * 3);
  dump_field(dir, "geom_margin", "f64", m->geom_margin, ngeom);

  // 清单
  const std::string index_path = dir + "/fields.json";
  FILE* out = std::fopen(index_path.c_str(), "wb");
  if (!out) {
    std::fprintf(stderr, "cannot open %s\n", index_path.c_str());
    mj_deleteModel(m);
    return 6;
  }
  std::fprintf(out, "{\n  \"schema\": 1,\n  \"tool\": \"dump_collision_fields_312\",\n");
  std::fprintf(out, "  \"mujoco_version\": \"%s\",\n  \"mujoco_version_int\": %d,\n",
               mj_versionString(), mj_version());
  std::fprintf(out, "  \"source_xml\": ");
  write_json_string(out, argv[1]);
  std::fprintf(out, ",\n  \"model_file\": ");
  write_json_string(out, argv[2]);
  std::fprintf(out, ",\n  \"loaded_mjb\": true,\n");
  std::fprintf(out, "  \"counts\": {\"ngeom\": %lld, \"nhfield\": %lld, \"nhfielddata\": %lld, "
                    "\"nmesh\": %lld, \"noct\": %lld},\n",
               ngeom, nhfield, nhfielddata, nmesh, noct);
  std::fprintf(out, "  \"fields\": [\n");
  for (std::size_t i = 0; i < g_fields.size(); ++i) {
    const FieldEntry& f = g_fields[i];
    std::fprintf(out, "    {\"name\": \"%s\", \"dtype\": \"%s\", \"count\": %lld, \"file\": \"%s\"}%s\n",
                 f.name.c_str(), f.dtype.c_str(), f.count, f.file.c_str(),
                 (i + 1) < g_fields.size() ? "," : "");
  }
  std::fprintf(out, "  ]\n}\n");
  std::fclose(out);
  mj_deleteModel(m);
  std::printf("ok: %s -> %s, %s\n", argv[1], argv[2], argv[3]);
  return 0;
}
