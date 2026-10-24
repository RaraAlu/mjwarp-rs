#include <cstdint>
#include <cstdio>
#include <cstring>
#include <vector>
#include "mujoco/mujoco.h"

static void dump(const mjModel* m, const char* name, mjtObj type, int count, bool last_type = false) {
  std::vector<int> ids = {-2, -1};
  for (int i = 0; i < count; ++i) ids.push_back(i);
  ids.push_back(count);
  ids.push_back(count + 1);
  for (size_t i = 0; i < ids.size(); ++i) {
    int id = ids[i];
    const char* s = mj_id2name(m, type, id);
    std::printf("\"%s:%d\":", name, id);
    if (s) {
      std::printf("\"");
      for (const char* p = s; *p; ++p) {
        unsigned char c = (unsigned char)*p;
        if (c == '"' || c == '\\') std::printf("\\%c", c);
        else if (c >= 32 && c < 127) std::printf("%c", c);
        else std::printf("\\u%04x", c);
      }
      std::printf("\"");
    } else {
      std::printf("null");
    }
    bool last = last_type && i + 1 == ids.size();
    std::printf(last ? "\n" : ",\n");
  }
}

int main(int argc, char** argv) {
  if (argc < 2) return 2;
  mjModel* m = mj_loadModel(argv[1], nullptr);
  if (!m) return 3;
  std::printf("{\n");
  dump(m, "body", mjOBJ_BODY, m->nbody);
  dump(m, "joint", mjOBJ_JOINT, m->njnt);
  dump(m, "geom", mjOBJ_GEOM, m->ngeom);
  dump(m, "site", mjOBJ_SITE, m->nsite);
  dump(m, "camera", mjOBJ_CAMERA, m->ncam);
  dump(m, "light", mjOBJ_LIGHT, m->nlight);
  dump(m, "mesh", mjOBJ_MESH, m->nmesh);
  dump(m, "texture", mjOBJ_TEXTURE, m->ntex);
  dump(m, "material", mjOBJ_MATERIAL, m->nmat, true);
  std::printf("}\n");
  mj_deleteModel(m);
  return 0;
}