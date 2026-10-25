// QA 独立复核用：MuJoCo 3.12 C++ 侧 MJB 生成/重载 + 碰撞相关字段原始字节 dump。
// 与交付 scripts/authoring/g10-p5/* 无代码复用：本文件由 QA 独立编写。
// 用法：
//   qa_mjb_dump dump <xml> <mjb_out> <dump_dir>      编译 XML -> 保存 MJB -> 同版本重载 -> dump 字段
//   qa_mjb_dump dumpmjb <mjb_in> <dump_dir>          直接加载既有 MJB -> dump 字段
//   qa_mjb_dump probe <mjb_in>                        仅尝试加载，打印 ngeom/noct 或错误串
#include <mujoco/mujoco.h>

#include <cstdint>
#include <cstdio>
#include <cstring>
#include <string>
#include <sys/stat.h>
#include <vector>

#ifdef _WIN32
#include <direct.h>
#define QA_MKDIR(p) _mkdir(p)
#else
#define QA_MKDIR(p) mkdir(p, 0777)
#endif

static int ensure_dir(const std::string& path) {
  if (path.empty()) return 0;
  std::string cur;
  for (size_t i = 0; i < path.size(); ++i) {
    cur.push_back(path[i]);
    if (path[i] == '/' || path[i] == '\\' || i + 1 == path.size()) {
      if (cur.back() == '/' || cur.back() == '\\') continue;
      QA_MKDIR(cur.c_str());
    }
  }
  struct stat st;
  return stat(path.c_str(), &st) == 0 && (st.st_mode & S_IFDIR) ? 0 : -1;
}

template <typename T>
static void write_raw(const std::string& dir, const char* name, const T* data, int count) {
  std::string path = dir + "/" + name + ".bin";
  FILE* f = fopen(path.c_str(), "wb");
  if (!f) {
    fprintf(stderr, "write failed: %s\n", path.c_str());
    exit(10);
  }
  if (count > 0 && data) fwrite(data, sizeof(T), (size_t)count, f);
  fclose(f);
  printf("field %s count=%d\n", name, count);
}

static void dump_fields(const mjModel* m, const std::string& dir) {
  ensure_dir(dir);
  const int64_t counts[6] = {(int64_t)m->ngeom,      (int64_t)m->nhfield, (int64_t)m->nhfielddata,
                             (int64_t)m->nmesh,     (int64_t)m->noct,    (int64_t)m->nbody};
  write_raw<int64_t>(dir, "counts", counts, 6);
  const int64_t nq_nv[2] = {(int64_t)m->nq, (int64_t)m->nv};
  write_raw<int64_t>(dir, "nq_nv", nq_nv, 2);
  write_raw<int>(dir, "hfield_nrow", m->hfield_nrow, m->nhfield);
  write_raw<int>(dir, "hfield_ncol", m->hfield_ncol, m->nhfield);
  write_raw<int>(dir, "hfield_adr", m->hfield_adr, m->nhfield);
  write_raw<double>(dir, "hfield_size", m->hfield_size, m->nhfield * 4);
  write_raw<float>(dir, "hfield_data", m->hfield_data, m->nhfielddata);
  write_raw<int>(dir, "oct_depth", m->oct_depth, m->noct);
  write_raw<int>(dir, "oct_child", m->oct_child, m->noct * 8);
  write_raw<double>(dir, "oct_aabb", m->oct_aabb, m->noct * 6);
  write_raw<double>(dir, "oct_coeff", m->oct_coeff, m->noct * 8);
  write_raw<int>(dir, "mesh_octadr", m->mesh_octadr, m->nmesh);
  write_raw<int>(dir, "mesh_octnum", m->mesh_octnum, m->nmesh);
  const int opt_sdf[2] = {m->opt.sdf_iterations, m->opt.sdf_initpoints};
  write_raw<int>(dir, "opt_sdf", opt_sdf, 2);
  write_raw<int>(dir, "geom_type", m->geom_type, m->ngeom);
  write_raw<int>(dir, "geom_dataid", m->geom_dataid, m->ngeom);
  write_raw<double>(dir, "geom_size", m->geom_size, m->ngeom * 3);
  write_raw<double>(dir, "geom_margin", m->geom_margin, m->ngeom);
  printf("counts ngeom=%d nhfield=%d nhfielddata=%d nmesh=%d noct=%d nbody=%d nq=%d nv=%d\n",
         m->ngeom, m->nhfield, m->nhfielddata, m->nmesh, m->noct, m->nbody, m->nq, m->nv);
}

int main(int argc, char** argv) {
  const char* version = mj_versionString();
  printf("mujoco_c=%s\n", version);
  if (argc < 3) {
    fprintf(stderr, "usage: %s dump|dumpmjb|probe ...\n", argv[0]);
    return 1;
  }
  const std::string mode = argv[1];

  if (mode == "probe") {
    const char* error = nullptr;
    mjModel* m = mj_loadModel(argv[2], nullptr);
    if (!m) {
      printf("PROBE_FAIL %s\n", argv[2]);
      return 2;
    }
    printf("PROBE_OK %s ngeom=%d noct=%d nq=%d nv=%d\n", argv[2], m->ngeom, m->noct, m->nq, m->nv);
    mj_deleteModel(m);
    return 0;
  }

  if (mode == "dumpmjb") {
    mjModel* m = mj_loadModel(argv[2], nullptr);
    if (!m) {
      fprintf(stderr, "loadModel failed: %s\n", argv[2]);
      return 2;
    }
    dump_fields(m, argv[3]);
    mj_deleteModel(m);
    return 0;
  }

  if (mode == "dump") {
    char err[2000] = {0};
    mjModel* model = mj_loadXML(argv[2], nullptr, err, (int)sizeof(err));
    if (!model) {
      fprintf(stderr, "loadXML failed: %s\n", err);
      return 2;
    }
    mj_saveModel(model, argv[3], nullptr, 0);
    printf("saved %s\n", argv[3]);
    mjModel* reloaded = mj_loadModel(argv[3], nullptr);
    if (!reloaded) {
      fprintf(stderr, "same-version reload failed: %s\n", argv[3]);
      return 3;
    }
    if (reloaded->ngeom != model->ngeom || reloaded->noct != model->noct ||
        reloaded->nhfielddata != model->nhfielddata || reloaded->nq != model->nq ||
        reloaded->nv != model->nv) {
      fprintf(stderr, "reload count mismatch\n");
      return 4;
    }
    printf("reload_ok ngeom=%d noct=%d nhfielddata=%d nq=%d nv=%d\n", reloaded->ngeom,
           reloaded->noct, reloaded->nhfielddata, reloaded->nq, reloaded->nv);
    dump_fields(reloaded, argv[4]);
    mj_deleteModel(reloaded);
    mj_deleteModel(model);
    return 0;
  }

  fprintf(stderr, "unknown mode %s\n", mode.c_str());
  return 1;
}
