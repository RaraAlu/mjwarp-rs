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
  if(argc!=4) return 2;
  char error[1024]{};
  std::unique_ptr<mjModel,decltype(&mj_deleteModel)> m(
    mj_loadXML(argv[1],nullptr,error,sizeof(error)),mj_deleteModel);
  if(!m) {std::cerr << error;return 3;}
  if(m->ncam!=8 || m->nlight!=8 || m->nmocap!=1) return 4;
  // Compile fixed cameras/lights first; then exercise absent target fallback.
  m->cam_mode[6]=m->light_mode[6]=mjCAMLIGHT_TARGETBODY;
  m->cam_mode[7]=m->light_mode[7]=mjCAMLIGHT_TARGETBODYCOM;
  m->cam_targetbodyid[6]=m->light_targetbodyid[6]=-2;
  mj_saveModel(m.get(),argv[3],nullptr,0);
  std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
  if(!d) return 5;
  std::ofstream out(argv[2],std::ios::binary);
  if(!out) return 6;
  out << std::setprecision(17) << "{\"native_version\":" << mj_version()
      << ",\"seed\":1789,\"absolute_tolerance\":0.00002,\"relative_tolerance\":0.00002,\n"
      << "\"nq\":" << m->nq << ",\"nv\":" << m->nv << ",\"nbody\":" << m->nbody
      << ",\"njnt\":" << m->njnt << ",\"ngeom\":" << m->ngeom << ",\"nsite\":" << m->nsite
      << ",\"nmocap\":" << m->nmocap << ",\"ncam\":" << m->ncam << ",\"nlight\":" << m->nlight
      << ",\"model\":{\n";
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
  FIELD(cam_mode,m->ncam);FIELD(cam_bodyid,m->ncam);FIELD(cam_targetbodyid,m->ncam);
  FIELD(cam_pos,3*m->ncam);FIELD(cam_quat,4*m->ncam);
  FIELD(cam_poscom0,3*m->ncam);FIELD(cam_pos0,3*m->ncam);FIELD(cam_mat0,9*m->ncam);
  FIELD(light_mode,m->nlight);FIELD(light_bodyid,m->nlight);FIELD(light_targetbodyid,m->nlight);
  FIELD(light_pos,3*m->nlight);FIELD(light_dir,3*m->nlight);
  FIELD(light_poscom0,3*m->nlight);FIELD(light_pos0,3*m->nlight);
  array(out,"light_dir0",m->light_dir0,3*m->nlight,false);
#undef FIELD
  out << "},\"cases\":[\n";
  uint32_t seed=1789;
  auto sample=[&] {seed=1664525u*seed+1013904223u;
    return mjtNum((int(seed>>8)-8388608)/8388608.0);};
  for(int c=0;c<8;++c) {
    if(c) out << ",\n";
    mj_resetData(m.get(),d.get());
    if(c) {
      for(int q=0;q<m->nq;++q) d->qpos[q]=mjtNum(float(3*sample()));
      for(int p=0;p<3*m->nmocap;++p) d->mocap_pos[p]=mjtNum(float(3*sample()));
      for(int q=0;q<4*m->nmocap;++q) d->mocap_quat[q]=mjtNum(float(2*sample()));
    }
    for(int q=0;q<m->nq;++q) d->qpos[q]=mjtNum(float(d->qpos[q]));
    for(int p=0;p<3*m->nmocap;++p) d->mocap_pos[p]=mjtNum(float(d->mocap_pos[p]));
    for(int q=0;q<4*m->nmocap;++q) d->mocap_quat[q]=mjtNum(float(d->mocap_quat[q]));
    mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_camlight(m.get(),d.get());
    out << "{\"id\":" << c << ',';
    array(out,"qpos",d->qpos,m->nq);
    array(out,"mocap_pos",d->mocap_pos,3*m->nmocap);
    array(out,"mocap_quat",d->mocap_quat,4*m->nmocap);
    array(out,"cam_xpos",d->cam_xpos,3*m->ncam);array(out,"cam_xmat",d->cam_xmat,9*m->ncam);
    array(out,"light_xpos",d->light_xpos,3*m->nlight);array(out,"light_xdir",d->light_xdir,3*m->nlight,false);
    out << '}';
  }
  out << "]}\n";
  return out.good() ? 0 : 7;
}
