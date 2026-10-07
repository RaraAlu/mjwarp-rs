// Fixture authoring only. Product builds/tests never execute this tool.
#include <mujoco/mujoco.h>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>

template<class T>
void array(std::ostream& out,const char* name,const T* p,size_t n,bool comma=true) {
  out << '"' << name << "\":[";
  for(size_t i=0;i<n;++i) { if(i) out << ',';out << p[i]; }
  out << ']' << (comma ? ",\n" : "\n");
}
int main(int argc,char** argv) {
  if(argc!=4 || mj_version()!=3012000) return 2;
  char error[1024]{};
  std::unique_ptr<mjModel,decltype(&mj_deleteModel)> m(
    mj_loadXML(argv[1],nullptr,error,sizeof(error)),mj_deleteModel);
  if(!m) {std::cerr << error;return 3;}
  if(m->ntendon!=4 || m->nq!=14 || m->nv!=12) return 4;
  mj_saveModel(m.get(),argv[3],nullptr,0);
  std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
  if(!d) return 5;
  std::ofstream out(argv[2],std::ios::binary);
  if(!out) return 6;
  out << std::setprecision(17) << "{\"native_version\":" << mj_version()
      << ",\"seed\":1789,\"absolute_tolerance\":0.00002,\"relative_tolerance\":0.00002,\n"
      << "\"nq\":" << m->nq << ",\"nv\":" << m->nv << ",\"nbody\":" << m->nbody
      << ",\"njnt\":" << m->njnt << ",\"ngeom\":" << m->ngeom << ",\"nsite\":" << m->nsite
      << ",\"ntendon\":" << m->ntendon << ",\"nJten\":" << m->nJten << ",\"model\":{\n";
#define FIELD(name,n) array(out,#name,m->name,n)
  FIELD(qpos0,m->nq);
  FIELD(body_parentid,m->nbody);FIELD(body_jntadr,m->nbody);FIELD(body_jntnum,m->nbody);
  FIELD(body_pos,3*m->nbody);FIELD(body_quat,4*m->nbody);FIELD(body_mocapid,m->nbody);
  FIELD(jnt_type,m->njnt);FIELD(jnt_bodyid,m->njnt);
  FIELD(jnt_qposadr,m->njnt);FIELD(jnt_dofadr,m->njnt);
  FIELD(jnt_pos,3*m->njnt);FIELD(jnt_axis,3*m->njnt);
  FIELD(body_ipos,3*m->nbody);FIELD(body_iquat,4*m->nbody);
  FIELD(body_mass,m->nbody);FIELD(body_inertia,3*m->nbody);
  FIELD(dof_bodyid,m->nv);FIELD(dof_jntid,m->nv);FIELD(dof_parentid,m->nv);
  FIELD(dof_armature,m->nv);FIELD(dof_damping,m->nv);
  FIELD(geom_bodyid,m->ngeom);FIELD(geom_pos,3*m->ngeom);FIELD(geom_quat,4*m->ngeom);
  FIELD(site_bodyid,m->nsite);FIELD(site_pos,3*m->nsite);FIELD(site_quat,4*m->nsite);
  FIELD(tendon_adr,m->ntendon);FIELD(tendon_num,m->ntendon);
  FIELD(wrap_type,m->nwrap);FIELD(wrap_objid,m->nwrap);FIELD(wrap_prm,m->nwrap);
  FIELD(ten_J_rowadr,m->ntendon);FIELD(ten_J_rownnz,m->ntendon);
  array(out,"ten_J_colind",m->ten_J_colind,m->nJten,false);
#undef FIELD
  out << "},\"cases\":[\n";
  uint32_t seed=1789;
  auto sample=[&] {seed=1664525u*seed+1013904223u;
    return mjtNum((int(seed>>8)-8388608)/8388608.0);};
  for(int c=0;c<8;++c) {
    if(c) out << ",\n";
    mj_resetData(m.get(),d.get());
    if(c) for(int q=0;q<m->nq;++q) d->qpos[q]=3*sample();
    // Normalize unrelated free/ball rotations before rounding input to f32.
    for(int j=0;j<m->njnt;++j) {
      if(m->jnt_type[j]==mjJNT_FREE) mju_normalize4(d->qpos+m->jnt_qposadr[j]+3);
      if(m->jnt_type[j]==mjJNT_BALL) mju_normalize4(d->qpos+m->jnt_qposadr[j]);
    }
    for(int q=0;q<m->nq;++q) d->qpos[q]=mjtNum(float(d->qpos[q]));
    mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_tendon(m.get(),d.get());
    out << "{\"id\":" << c << ',';
    array(out,"qpos",d->qpos,m->nq);
    array(out,"ten_length",d->ten_length,m->ntendon);
    array(out,"ten_J",d->ten_J,m->nJten,false);
    out << '}';
  }
  out << "]}\n";
  return out.good() ? 0 : 7;
}
