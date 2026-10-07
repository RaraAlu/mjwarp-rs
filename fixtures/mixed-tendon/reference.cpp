// Development-only authoring. Product tests never execute this tool.
#include <mujoco/mujoco.h>
#include <cmath>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <vector>
template<class T>
void array(std::ostream& out,const char* name,const T* p,size_t n,bool comma=true) {
  out << '"' << name << "\":[";
  for(size_t i=0;i<n;++i) {if(i) out << ',';out << p[i];}
  out << ']' << (comma?",\n":"\n");
}
int main(int argc,char** argv) {
  if(argc!=4 || mj_version()!=3012000) return 2;
  char error[1024]{};
  std::unique_ptr<mjModel,decltype(&mj_deleteModel)> m(mj_loadXML(argv[1],nullptr,error,sizeof(error)),mj_deleteModel);
  if(!m) {std::cerr<<error;return 3;}
  if(m->ntendon!=14 || m->nmocap!=1) return 4;
  mj_saveModel(m.get(),argv[3],nullptr,0);
  std::vector<mjtNum> sizes(m->geom_size,m->geom_size+3*m->ngeom);
  std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
  std::ofstream out(argv[2],std::ios::binary);
  if(!d || !out) return 5;
  out << std::setprecision(17) << "{\"native_version\":" << mj_version()
    << ",\"seed\":2101,\"absolute_tolerance\":0.00002,\"relative_tolerance\":0.00002,\n"
    << "\"nq\":"<<m->nq<<",\"nv\":"<<m->nv<<",\"nbody\":"<<m->nbody
    << ",\"njnt\":"<<m->njnt<<",\"ngeom\":"<<m->ngeom<<",\"nsite\":"<<m->nsite
    << ",\"ntendon\":"<<m->ntendon<<",\"nJten\":"<<m->nJten<<",\"nwrap\":"<<m->nwrap<<",\"model\":{\n";
#define FIELD(name,n) array(out,#name,m->name,n)
  FIELD(qpos0,m->nq);FIELD(body_parentid,m->nbody);FIELD(body_jntadr,m->nbody);FIELD(body_jntnum,m->nbody);
  FIELD(body_pos,3*m->nbody);FIELD(body_quat,4*m->nbody);FIELD(body_mocapid,m->nbody);
  FIELD(jnt_type,m->njnt);FIELD(jnt_bodyid,m->njnt);FIELD(jnt_qposadr,m->njnt);FIELD(jnt_dofadr,m->njnt);
  FIELD(jnt_pos,3*m->njnt);FIELD(jnt_axis,3*m->njnt);
  FIELD(body_ipos,3*m->nbody);FIELD(body_iquat,4*m->nbody);FIELD(body_mass,m->nbody);FIELD(body_inertia,3*m->nbody);
  FIELD(dof_bodyid,m->nv);FIELD(dof_jntid,m->nv);FIELD(dof_parentid,m->nv);
  FIELD(dof_armature,m->nv);FIELD(dof_damping,m->nv);
  FIELD(geom_bodyid,m->ngeom);FIELD(geom_pos,3*m->ngeom);FIELD(geom_quat,4*m->ngeom);
  FIELD(geom_type,m->ngeom);FIELD(geom_size,3*m->ngeom);
  FIELD(site_bodyid,m->nsite);FIELD(site_pos,3*m->nsite);FIELD(site_quat,4*m->nsite);
  FIELD(tendon_adr,m->ntendon);FIELD(tendon_num,m->ntendon);FIELD(wrap_type,m->nwrap);
  FIELD(wrap_objid,m->nwrap);FIELD(wrap_prm,m->nwrap);FIELD(ten_J_rowadr,m->ntendon);FIELD(ten_J_rownnz,m->ntendon);
  array(out,"ten_J_colind",m->ten_J_colind,m->nJten,false);
#undef FIELD
  out << "},\"cases\":[\n";
  for(int c=0;c<12;++c) {
    if(c) out << ",\n";
    int row=c%3,round=c/3;
    mj_resetData(m.get(),d.get());
    for(int g=0;g<m->ngeom;++g) for(int k=0;k<3;++k)
      m->geom_size[3*g+k]=mjtNum(float(sizes[3*g+k]*(k==0?(row==1?0.8:(row==2?1.2:1.0)):1.0)));
    if(round) {
      d->qpos[0]=0.17*(row-1)*round;
      d->qpos[1]=round==1?1.4:(round==2?-0.6:0.1);
      double a=0.09*round*(row+1);d->qpos[2]=std::cos(a);
      d->qpos[3]=std::sin(a)/std::sqrt(3.0);d->qpos[4]=d->qpos[3];d->qpos[5]=d->qpos[3];
      d->mocap_pos[0]+=0.1*(row-1);
      d->mocap_pos[1]+=round==1?1.2:(round==2?-0.5:0.2);
    }
    mju_normalize4(d->qpos+2);
    for(int q=0;q<m->nq;++q) d->qpos[q]=mjtNum(float(d->qpos[q]));
    for(int p=0;p<3;++p) d->mocap_pos[p]=mjtNum(float(d->mocap_pos[p]));
    mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_tendon(m.get(),d.get());
    out << "{\"id\":"<<c<<',';
    array(out,"qpos",d->qpos,m->nq);array(out,"geom_size",m->geom_size,3*m->ngeom);
    array(out,"mocap_pos",d->mocap_pos,3);array(out,"mocap_quat",d->mocap_quat,4);
    array(out,"ten_length",d->ten_length,m->ntendon);array(out,"ten_J",d->ten_J,m->nJten);
    array(out,"ten_wrapadr",d->ten_wrapadr,m->ntendon);array(out,"ten_wrapnum",d->ten_wrapnum,m->ntendon);
    array(out,"wrap_obj",d->wrap_obj,2*m->nwrap);array(out,"wrap_xpos",d->wrap_xpos,6*m->nwrap,false);
    out << '}';
  }
  out << "]}\n";return out.good()?0:6;
}
