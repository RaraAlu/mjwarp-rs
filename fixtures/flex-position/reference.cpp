// Development-only authoring. Product tests only read frozen outputs.
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
  for(size_t i=0;i<n;++i) {if(i) out << ',';out << +p[i];}
  out << ']' << (comma?",\n":"\n");
}
int main(int argc,char** argv) {
  if(argc!=4 || mj_version()!=3012000) return 2;
  char error[1024]{};
  std::unique_ptr<mjModel,decltype(&mj_deleteModel)> m(mj_loadXML(argv[1],nullptr,error,sizeof(error)),mj_deleteModel);
  if(!m) {std::cerr<<error;return 3;}
  if(m->nflex!=4 || m->nmocap!=1) return 4;
  // Exercise nonzero node frames through an explicit compiled-model override.
  int linear=mj_name2id(m.get(),mjOBJ_FLEX,"linear");
  if(linear<0 || m->flex_interp[linear]!=1 || m->flex_nodenum[linear]!=12) return 4;
  m->flex_centered[linear]=0;
  int first=m->flex_nodeadr[linear],last=first+m->flex_nodenum[linear]-1;
  m->flex_node[3*first]=0.025;m->flex_node[3*first+1]=-0.015;
  m->flex_node[3*last+2]=0.035;
  mj_saveModel(m.get(),argv[3],nullptr,0);
  std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
  std::ofstream out(argv[2],std::ios::binary);
  if(!d || !out) return 5;
  out << std::setprecision(17) << "{\"native_version\":" << mj_version()
    << ",\"seed\":2201,\"absolute_tolerance\":0.00002,\"relative_tolerance\":0.00002,\n"
    << "\"nq\":"<<m->nq<<",\"nv\":"<<m->nv<<",\"nbody\":"<<m->nbody
    << ",\"njnt\":"<<m->njnt<<",\"ngeom\":"<<m->ngeom<<",\"nsite\":"<<m->nsite
    << ",\"nflex\":"<<m->nflex<<",\"nflexnode\":"<<m->nflexnode<<",\"nflexvert\":"<<m->nflexvert<<",\"model\":{\n";
#define FIELD(name,n) array(out,#name,m->name,n)
  FIELD(qpos0,m->nq);FIELD(body_parentid,m->nbody);FIELD(body_jntadr,m->nbody);FIELD(body_jntnum,m->nbody);
  FIELD(body_pos,3*m->nbody);FIELD(body_quat,4*m->nbody);FIELD(body_mocapid,m->nbody);
  FIELD(jnt_type,m->njnt);FIELD(jnt_bodyid,m->njnt);FIELD(jnt_qposadr,m->njnt);FIELD(jnt_dofadr,m->njnt);
  FIELD(jnt_pos,3*m->njnt);FIELD(jnt_axis,3*m->njnt);
  FIELD(body_ipos,3*m->nbody);FIELD(body_iquat,4*m->nbody);FIELD(body_mass,m->nbody);FIELD(body_inertia,3*m->nbody);
  FIELD(dof_bodyid,m->nv);FIELD(dof_jntid,m->nv);FIELD(dof_parentid,m->nv);FIELD(dof_armature,m->nv);FIELD(dof_damping,m->nv);
  FIELD(geom_bodyid,m->ngeom);FIELD(geom_pos,3*m->ngeom);FIELD(geom_quat,4*m->ngeom);
  FIELD(site_bodyid,m->nsite);FIELD(site_pos,3*m->nsite);FIELD(site_quat,4*m->nsite);
  FIELD(tendon_adr,m->ntendon);FIELD(tendon_num,m->ntendon);FIELD(wrap_type,m->nwrap);FIELD(wrap_objid,m->nwrap);FIELD(wrap_prm,m->nwrap);
  FIELD(ten_J_rowadr,m->ntendon);FIELD(ten_J_rownnz,m->ntendon);FIELD(ten_J_colind,m->nJten);
  FIELD(flex_interp,m->nflex);FIELD(flex_cellnum,3*m->nflex);FIELD(flex_nodeadr,m->nflex);FIELD(flex_nodenum,m->nflex);
  FIELD(flex_vertadr,m->nflex);FIELD(flex_vertnum,m->nflex);FIELD(flex_centered,m->nflex);
  FIELD(flex_nodebodyid,m->nflexnode);FIELD(flex_vertbodyid,m->nflexvert);FIELD(flex_node,3*m->nflexnode);FIELD(flex_vert,3*m->nflexvert);
  array(out,"flex_vert0",m->flex_vert0,3*m->nflexvert,false);
#undef FIELD
  out << "},\"cases\":[\n";
  std::vector<mjtNum> nodes(3*m->nflexnode);
  for(int c=0;c<12;++c) {
    if(c) out << ",\n";
    mj_resetData(m.get(),d.get());
    for(int j=0;j<m->njnt;++j) {
      int q=m->jnt_qposadr[j];
      if(m->jnt_type[j]==mjJNT_SLIDE || m->jnt_type[j]==mjJNT_HINGE)
        d->qpos[q]+=c?0.025*c*std::sin(0.37*(j+1)*(c+1)):0;
      if(m->jnt_type[j]==mjJNT_BALL) {
        double a=0.07*c;d->qpos[q]=std::cos(a);
        d->qpos[q+1]=std::sin(a)/std::sqrt(3.0);d->qpos[q+2]=d->qpos[q+1];d->qpos[q+3]=d->qpos[q+1];
      }
    }
    d->mocap_pos[0]+=0.03*c;d->mocap_pos[1]-=0.02*c;
    d->mocap_quat[0]=std::cos(0.05*c);d->mocap_quat[3]=std::sin(0.05*c);
    for(int q=0;q<m->nq;++q) d->qpos[q]=mjtNum(float(d->qpos[q]));
    for(int k=0;k<3;++k) d->mocap_pos[k]=mjtNum(float(d->mocap_pos[k]));
    for(int k=0;k<4;++k) d->mocap_quat[k]=mjtNum(float(d->mocap_quat[k]));
    mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_tendon(m.get(),d.get());mj_flex(m.get(),d.get());
    // MuJoCo exposes vertex output, not nodal output. Use native transform helpers.
    for(int n=0;n<m->nflexnode;++n) {
      int b=m->flex_nodebodyid[n];
      mju_mulMatVec3(nodes.data()+3*n,d->xmat+9*b,m->flex_node+3*n);
      mju_addTo3(nodes.data()+3*n,d->xpos+3*b);
    }
    out << "{\"id\":"<<c<<',';
    array(out,"qpos",d->qpos,m->nq);array(out,"mocap_pos",d->mocap_pos,3);array(out,"mocap_quat",d->mocap_quat,4);
    array(out,"xpos",d->xpos,3*m->nbody);array(out,"xmat",d->xmat,9*m->nbody);
    array(out,"ten_length",d->ten_length,m->ntendon);array(out,"ten_J",d->ten_J,m->nJten);
    array(out,"flexnode_xpos",nodes.data(),nodes.size());array(out,"flexvert_xpos",d->flexvert_xpos,3*m->nflexvert,false);
    out << '}';
  }
  out << "]}\n";return out.good()?0:6;
}
