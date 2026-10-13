// Offline reference author only. Product builds and tests never execute it.
#include <mujoco/mujoco.h>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <memory>

template<class T> void array(std::ostream& out, const char* key, const T* p, int n) {
  out << '"' << key << "\":[";
  for (int i = 0; i < n; ++i) { if (i) out << ','; out << p[i]; }
  out << "],\n";
}
int main(int argc, char** argv) {
  if (argc != 3 || mj_version() != 3012000) return 2;
  std::unique_ptr<mjModel, decltype(&mj_deleteModel)> m(mj_loadModel(argv[1], nullptr), mj_deleteModel);
  if (!m) return 3;
  std::unique_ptr<mjData, decltype(&mj_deleteData)> d(mj_makeData(m.get()), mj_deleteData);
  if (!d) return 4;
  std::ofstream out(argv[2], std::ios::binary);
  if (!out) return 5;
  out << std::setprecision(17) << "{\"native_version\":" << mj_version()
      << ",\"seed\":1789,\"absolute_tolerance\":0.00002,\"relative_tolerance\":0.00002,"
      << "\"nq\":" << m->nq << ",\"nv\":" << m->nv << ",\"nbody\":" << m->nbody
      << ",\"gravities\":[[0,0,-9.8100004196166992],[0,0,0],[1.25,-2,3.5]],\"cases\":[\n";
  uint32_t seed = 1789;
  auto sample = [&] { seed = 1664525u*seed + 1013904223u;
    return mjtNum((int(seed >> 8) - 8388608) / 8388608.0); };
  for (int c = 0; c < 8; ++c) {
    if (c) out << ",\n";
    mj_resetData(m.get(), d.get());
    for (int j = 0; j < m->njnt; ++j) {
      int a = m->jnt_qposadr[j], type = m->jnt_type[j];
      if (c && type == mjJNT_FREE) for (int k = 0; k < 3; ++k) d->qpos[a+k] = 3*sample();
      if (c && (type == mjJNT_FREE || type == mjJNT_BALL)) {
        int q = a + (type == mjJNT_FREE ? 3 : 0);
        for (int k = 0; k < 4; ++k) d->qpos[q+k] = 2*sample();
        if (c == 1) { d->qpos[q] = 0; d->qpos[q+1] = -2; d->qpos[q+2] = 0; d->qpos[q+3] = 0; }
        if (c == 2) { d->qpos[q] = -1; d->qpos[q+1] = 0; d->qpos[q+2] = 0; d->qpos[q+3] = 0; }
      }
      if (c && type >= mjJNT_SLIDE) d->qpos[a] = 3*sample();
    }
    for (int q = 0; q < m->nq; ++q) d->qpos[q] = mjtNum(float(d->qpos[q]));
    for (int v = 0; v < m->nv; ++v) d->qvel[v] = c ? mjtNum(float(3*sample())) : 0;
    for (int v = 0; v < m->nv; ++v) d->qacc[v] = mjtNum(float(c ? 2*sample() : (v+1)*0.125));
    if (c == 3 || c == 4) {
      for (int j = 0; j < m->njnt; ++j) if (m->jnt_type[j] == mjJNT_FREE) {
        int a = m->jnt_dofadr[j] + (c == 3 ? 0 : 3);
        for (int k = 0; k < 3; ++k) d->qvel[a+k] = 0;
      }
    }
    mj_kinematics(m.get(), d.get()); mj_comPos(m.get(), d.get()); mj_comVel(m.get(), d.get());
    out << "{\"id\":" << c << ',';
    array(out, "qpos", d->qpos, m->nq); array(out, "qvel", d->qvel, m->nv); array(out, "qacc", d->qacc, m->nv);
    out << "\"qfrc\":[";
    const float gravity[3][3] = {{0,0,-9.81f},{0,0,0},{1.25f,-2,3.5f}};
    for (int flag = 0; flag < 2; ++flag) {
      if (flag) out << ',';
      out << '[';
      for (int g = 0; g < 3; ++g) {
        if (g) out << ',';
        for (int k = 0; k < 3; ++k) m->opt.gravity[k] = gravity[g][k];
        m->opt.disableflags &= ~mjDSBL_GRAVITY;
        mj_rne(m.get(), d.get(), flag, d->qfrc_bias);
        out << '[';
        for (int v = 0; v < m->nv; ++v) { if (v) out << ','; out << d->qfrc_bias[v]; }
        out << ']';
      }
      out << ']';
    }
    out << "]}";
  }
  out << "]}\n";
  return out.good() ? 0 : 6;
}
