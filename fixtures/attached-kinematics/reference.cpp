// Fixture authoring only. Product builds/tests never compile or execute this file.
#include <mujoco/mujoco.h>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>

template<class T>
void array(std::ostream& out, const char* name, const T* p, size_t n, bool comma = true) {
  out << '"' << name << "\":[";
  for (size_t i = 0; i < n; ++i) { if (i) out << ','; out << p[i]; }
  out << ']' << (comma ? ",\n" : "\n");
}

int main(int argc, char** argv) {
  if (argc < 4 || argc > 5) return 2;
  const bool xml = !std::strcmp(argv[1], "--xml");
  if ((!xml && std::strcmp(argv[1], "--mjb")) || (!xml && argc != 4)) return 2;
  char error[1024]{};
  auto* raw = xml ? mj_loadXML(argv[2], nullptr, error, sizeof(error))
                  : mj_loadModel(argv[2], nullptr);
  if (!raw) { std::cerr << error; return 3; }
  std::unique_ptr<mjModel, decltype(&mj_deleteModel)> m(raw, mj_deleteModel);
  if (xml && argc == 5) mj_saveModel(m.get(), argv[4], nullptr, 0);
  std::unique_ptr<mjData, decltype(&mj_deleteData)> d(mj_makeData(m.get()), mj_deleteData);
  if (!d) return 4;
  std::ofstream out(argv[3], std::ios::binary);
  if (!out) return 5;
  out << std::setprecision(17);
  out << "{\"native_version\":" << mj_version()
      << ",\"seed\":1789,\"absolute_tolerance\":0.00002,\"relative_tolerance\":0.00002,\n"
      << "\"nq\":" << m->nq << ",\"nv\":" << m->nv << ",\"nbody\":" << m->nbody
      << ",\"njnt\":" << m->njnt << ",\"ngeom\":" << m->ngeom << ",\"nsite\":" << m->nsite
      << ",\"model\":{\n";
  array(out, "qpos0", m->qpos0, m->nq);
  array(out, "body_parentid", m->body_parentid, m->nbody);
  array(out, "body_jntadr", m->body_jntadr, m->nbody);
  array(out, "body_jntnum", m->body_jntnum, m->nbody);
  array(out, "body_pos", m->body_pos, 3*m->nbody);
  array(out, "body_quat", m->body_quat, 4*m->nbody);
  array(out, "jnt_type", m->jnt_type, m->njnt);
  array(out, "jnt_bodyid", m->jnt_bodyid, m->njnt);
  array(out, "jnt_qposadr", m->jnt_qposadr, m->njnt);
  array(out, "jnt_dofadr", m->jnt_dofadr, m->njnt);
  array(out, "jnt_pos", m->jnt_pos, 3*m->njnt);
  array(out, "jnt_axis", m->jnt_axis, 3*m->njnt);
  array(out, "body_ipos", m->body_ipos, 3*m->nbody);
  array(out, "body_iquat", m->body_iquat, 4*m->nbody);
  array(out, "body_mass", m->body_mass, m->nbody);
  array(out, "body_inertia", m->body_inertia, 3*m->nbody);
  array(out, "dof_bodyid", m->dof_bodyid, m->nv);
  array(out, "dof_jntid", m->dof_jntid, m->nv);
  array(out, "dof_parentid", m->dof_parentid, m->nv);
  array(out, "dof_armature", m->dof_armature, m->nv);
  array(out, "dof_damping", m->dof_damping, m->nv);
  array(out, "geom_bodyid", m->geom_bodyid, m->ngeom);
  array(out, "geom_pos", m->geom_pos, 3*m->ngeom);
  array(out, "geom_quat", m->geom_quat, 4*m->ngeom);
  array(out, "site_bodyid", m->site_bodyid, m->nsite);
  array(out, "site_pos", m->site_pos, 3*m->nsite);
  array(out, "site_quat", m->site_quat, 4*m->nsite, false);
  out << "},\"cases\":[\n";
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
    // Freeze the exact f32 state sent to the GPU; keep the native outputs in f64.
    for (int q = 0; q < m->nq; ++q) d->qpos[q] = mjtNum(float(d->qpos[q]));
    mj_kinematics(m.get(), d.get());
    out << "{\"id\":" << c << ',';
    array(out, "qpos", d->qpos, m->nq);
    array(out, "geom_xpos", d->geom_xpos, 3*m->ngeom);
    array(out, "geom_xmat", d->geom_xmat, 9*m->ngeom);
    array(out, "site_xpos", d->site_xpos, 3*m->nsite);
    array(out, "site_xmat", d->site_xmat, 9*m->nsite, false);
    out << '}';
  }
  out << "]}\n";
  return out.good() ? 0 : 6;
}
