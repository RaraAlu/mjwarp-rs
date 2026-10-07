// Independent fixture authoring only. Product builds/tests never execute this.
#include <mujoco/mujoco.h>
#include <climits>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <memory>
#include <vector>

template<class T>
void array(std::ostream& out, const char* name, const T* p, size_t n, bool comma = true) {
  out << '"' << name << "\":[";
  for (size_t i = 0; i < n; ++i) { if (i) out << ','; out << p[i]; }
  out << ']' << (comma ? ",\n" : "\n");
}
int main(int argc, char** argv) {
  if (argc != 3) return 2;
  std::unique_ptr<mjModel, decltype(&mj_deleteModel)> m(mj_loadModel(argv[1], nullptr), mj_deleteModel);
  if (!m || m->ntendon || m->nu || m->nv > INT_MAX) return 3;
  std::unique_ptr<mjData, decltype(&mj_deleteData)> d(mj_makeData(m.get()), mj_deleteData);
  if (!d) return 4;
  std::ofstream out(argv[2], std::ios::binary);
  if (!out) return 5;
  const int nr = 3, nv = static_cast<int>(m->nv);
  std::vector<mjtNum> matrix(size_t(nv)*nv), ld(size_t(nv)*nv), rhs(size_t(nr)*nv), solution(size_t(nr)*nv);
  out << std::setprecision(17);
  out << "{\"native_version\":" << mj_version() << ",\"seed\":1789,\"rhs_seed\":9701,\n"
      << "\"absolute_tolerance\":0.0002,\"relative_tolerance\":0.0002,\"residual_tolerance\":0.00002,"
      << "\"reconstruction_tolerance\":0.000002,\"ntendon\":" << m->ntendon << ",\"nu\":" << m->nu
      << ",\"nq\":" << m->nq << ",\"nv\":" << nv << ",\"nbody\":" << m->nbody << ",\"rhs_count\":" << nr << ",\"cases\":[\n";
  uint32_t seed = 1789, rhs_seed = 9701;
  auto sample = [&] { seed = 1664525u*seed + 1013904223u;
                     return mjtNum((int(seed >> 8) - 8388608) / 8388608.0); };
  auto force = [&] { rhs_seed = 1664525u*rhs_seed + 1013904223u;
                    return mjtNum(float(4.0*(int(rhs_seed >> 8) - 8388608)/8388608.0)); };
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
    for (int i = 0; i < nv; ++i) {
      rhs[i] = 0;
      rhs[nv+i] = mjtNum(float((i % 2 ? -1.0 : 1.0)*(i+1)*0.125));
      rhs[2*nv+i] = force();
    }
    mj_kinematics(m.get(), d.get()); mj_comPos(m.get(), d.get()); mj_crb(m.get(), d.get());
    if (nv) {
      mj_fullM(m.get(), d.get(), matrix.data());
      mj_factorM(m.get(), d.get());
      mj_solveM(m.get(), d.get(), solution.data(), rhs.data(), nr);
      // Copy native sparse factors into a dense lower triangle. No candidate
      // factorization or solve formulas participate in this reference.
      for (int i = 0; i < nv; ++i)
        for (int j = 0; j < m->M_rownnz[i]; ++j) {
          int a = m->M_rowadr[i] + j;
          ld[size_t(i)*nv + m->M_colind[a]] = d->qLD[a];
        }
    }
    out << "{\"id\":" << c << ',';
    array(out,"qpos",d->qpos,m->nq);
    array(out,"matrix",matrix.data(),matrix.size());
    array(out,"ld",ld.data(),ld.size());
    array(out,"diagonal_inverse",d->qLDiagInv,nv);
    array(out,"rhs",rhs.data(),rhs.size());
    array(out,"solution",solution.data(),solution.size(),false);
    out << '}';
  }
  out << "]}\n";
  return out.good() ? 0 : 6;
}
