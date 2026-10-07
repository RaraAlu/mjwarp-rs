// Fixture authoring only. Product builds/tests never execute this tool.
#include <mujoco/mujoco.h>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <utility>

template<class T>
void array(std::ostream& out, const char* name, const T* p, size_t n, bool comma = true) {
  out << '"' << name << "\":[";
  for (size_t i = 0; i < n; ++i) { if (i) out << ','; out << p[i]; }
  out << ']' << (comma ? ",\n" : "\n");
}

int main(int argc, char** argv) {
  if (argc != 4) return 2;
  char error[1024]{};
  std::unique_ptr<mjModel, decltype(&mj_deleteModel)> m(
      mj_loadXML(argv[1], nullptr, error, sizeof(error)), mj_deleteModel);
  if (!m) { std::cerr << error; return 3; }
  if (m->nmocap != 2) return 4;
  // Reverse the mapping to test ID order independently from body order.
  for (int b = 0; b < m->nbody; ++b)
    if (m->body_mocapid[b] >= 0) m->body_mocapid[b] = 1 - m->body_mocapid[b];
  mj_saveModel(m.get(), argv[3], nullptr, 0);
  std::unique_ptr<mjData, decltype(&mj_deleteData)> d(mj_makeData(m.get()), mj_deleteData);
  if (!d) return 5;
  std::ofstream out(argv[2], std::ios::binary);
  if (!out) return 6;
  out << std::setprecision(17);
  out << "{\"native_version\":" << mj_version()
      << ",\"seed\":1789,\"absolute_tolerance\":0.00002,\"relative_tolerance\":0.00002,\n"
      << "\"nq\":" << m->nq << ",\"nv\":" << m->nv << ",\"nbody\":" << m->nbody
      << ",\"njnt\":" << m->njnt << ",\"ngeom\":" << m->ngeom << ",\"nsite\":" << m->nsite
      << ",\"nmocap\":" << m->nmocap << ",\"model\":{\n";
  array(out, "qpos0", m->qpos0, m->nq);
  array(out, "body_parentid", m->body_parentid, m->nbody);
  array(out, "body_jntadr", m->body_jntadr, m->nbody);
  array(out, "body_jntnum", m->body_jntnum, m->nbody);
  array(out, "body_pos", m->body_pos, 3*m->nbody);
  array(out, "body_quat", m->body_quat, 4*m->nbody);
  array(out, "body_mocapid", m->body_mocapid, m->nbody);
  array(out, "body_weldid", m->body_weldid, m->nbody);
  array(out, "body_rootid", m->body_rootid, m->nbody);
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
    if (c) {
      for (int q = 0; q < m->nq; ++q) d->qpos[q] = mjtNum(float(3*sample()));
      for (int i = 0; i < 3*m->nmocap; ++i) d->mocap_pos[i] = mjtNum(float(3*sample()));
      for (int i = 0; i < 4*m->nmocap; ++i) d->mocap_quat[i] = mjtNum(float(2*sample()));
      if (c == 1) for (int i = 0; i < m->nmocap; ++i) {
        d->mocap_quat[4*i] = -2;
        for (int q = 1; q < 4; ++q) d->mocap_quat[4*i+q] = 0;
      }
    }
    // Freeze default and changed inputs alike; references retain f64 outputs.
    for (int q = 0; q < m->nq; ++q) d->qpos[q] = mjtNum(float(d->qpos[q]));
    for (int p = 0; p < 3*m->nmocap; ++p) d->mocap_pos[p] = mjtNum(float(d->mocap_pos[p]));
    for (int q = 0; q < 4*m->nmocap; ++q) d->mocap_quat[q] = mjtNum(float(d->mocap_quat[q]));
    mj_kinematics(m.get(), d.get());
    mj_comPos(m.get(), d.get());
    out << "{\"id\":" << c << ',';
    array(out, "qpos", d->qpos, m->nq);
    array(out, "mocap_pos", d->mocap_pos, 3*m->nmocap);
    array(out, "mocap_quat", d->mocap_quat, 4*m->nmocap);
    array(out, "xpos", d->xpos, 3*m->nbody);
    array(out, "xquat", d->xquat, 4*m->nbody);
    array(out, "xmat", d->xmat, 9*m->nbody);
    array(out, "xipos", d->xipos, 3*m->nbody);
    array(out, "ximat", d->ximat, 9*m->nbody);
    array(out, "xanchor", d->xanchor, 3*m->njnt);
    array(out, "xaxis", d->xaxis, 3*m->njnt);
    array(out, "subtree_mass", m->body_subtreemass, m->nbody);
    array(out, "subtree_com", d->subtree_com, 3*m->nbody);
    array(out, "cinert", d->cinert, 10*m->nbody);
    array(out, "cdof", d->cdof, 6*m->nv);
    array(out, "geom_xpos", d->geom_xpos, 3*m->ngeom);
    array(out, "geom_xmat", d->geom_xmat, 9*m->ngeom);
    array(out, "site_xpos", d->site_xpos, 3*m->nsite);
    array(out, "site_xmat", d->site_xmat, 9*m->nsite, false);
    out << '}';
  }
  out << "]}\n";
  return out.good() ? 0 : 7;
}
