// Development-only authoring. Product tests read frozen native outputs.
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
  if(argc!=3 || mj_version()!=3012000) return 2;
  std::unique_ptr<mjModel,decltype(&mj_deleteModel)> m(mj_loadModel(argv[1],nullptr),mj_deleteModel);
  if(!m || m->nflex!=4 || m->nmocap!=1) return 3;
  std::vector<int> inactive_adr(m->flexedge_J_rowadr,m->flexedge_J_rowadr+m->nflexedge);
  std::vector<int> inactive_nnz(m->flexedge_J_rownnz,m->flexedge_J_rownnz+m->nflexedge);
  std::vector<int> inactive_col(m->flexedge_J_colind,m->flexedge_J_colind+m->nJfe);
  // Enable native Jacobians without changing topology or the saved base model.
  for(int f=0;f<m->nflex;++f) if(!m->flex_interp[f]) m->flex_edgedamping[f]=0.1;
  std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
  std::ofstream out(argv[2],std::ios::binary);
  if(!d || !out) return 4;
  // Rebuild native sparsity after enabling direct-edge damping.
  mj_setConst(m.get(),d.get());
  out << std::setprecision(17) << "{\"native_version\":" << mj_version()
      << ",\"seed\":2301,\"nflexedge\":" << m->nflexedge << ",\"nJfe\":" << m->nJfe
      << ",\"inactive_model\":{\n";
  array(out,"flex_edgeadr",m->flex_edgeadr,m->nflex);
  array(out,"flex_edgenum",m->flex_edgenum,m->nflex);
  array(out,"flex_edge",m->flex_edge,2*m->nflexedge);
  array(out,"flexedge_J_rowadr",inactive_adr.data(),inactive_adr.size());
  array(out,"flexedge_J_rownnz",inactive_nnz.data(),inactive_nnz.size());
  array(out,"flexedge_J_colind",inactive_col.data(),inactive_col.size(),false);
  out << "},\"model\":{\n";
  array(out,"flex_edgeadr",m->flex_edgeadr,m->nflex);
  array(out,"flex_edgenum",m->flex_edgenum,m->nflex);
  array(out,"flex_edge",m->flex_edge,2*m->nflexedge);
  array(out,"flexedge_J_rowadr",m->flexedge_J_rowadr,m->nflexedge);
  array(out,"flexedge_J_rownnz",m->flexedge_J_rownnz,m->nflexedge);
  array(out,"flexedge_J_colind",m->flexedge_J_colind,m->nJfe,false);
  out << "},\"cases\":[\n";
  std::vector<mjtNum> length(m->nflexedge),velocity(m->nflexedge);
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
    for(int q=0;q<m->nv;++q) d->qvel[q]=mjtNum(float(0.3*std::sin(0.21*(q+1)*(c+1))));
    for(int k=0;k<3;++k) d->mocap_pos[k]=mjtNum(float(d->mocap_pos[k]));
    for(int k=0;k<4;++k) d->mocap_quat[k]=mjtNum(float(d->mocap_quat[k]));
    mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_flex(m.get(),d.get());
    // Native mj_flex skips interpolated edge lengths. Its vertex geometry remains
    // native; measure those lengths with mju_sub3 and mju_norm3, not a GPU copy.
    for(int f=0;f<m->nflex;++f) for(int i=0;i<m->flex_edgenum[f];++i) {
      int e=m->flex_edgeadr[f]+i,b=m->flex_vertadr[f];
      mjtNum delta[3];mju_sub3(delta,d->flexvert_xpos+3*(b+m->flex_edge[2*e+1]),
                                    d->flexvert_xpos+3*(b+m->flex_edge[2*e]));
      length[e]=mju_norm3(delta);
      velocity[e]=0;
      for(int k=0;k<m->flexedge_J_rownnz[e];++k) {
        int slot=m->flexedge_J_rowadr[e]+k;
        velocity[e]+=d->flexedge_J[slot]*d->qvel[m->flexedge_J_colind[slot]];
      }
    }
    out << "{\"id\":" << c << ',';
    array(out,"qpos",d->qpos,m->nq);array(out,"qvel",d->qvel,m->nv);
    array(out,"mocap_pos",d->mocap_pos,3);array(out,"mocap_quat",d->mocap_quat,4);
    array(out,"flexedge_length",length.data(),length.size());
    array(out,"native_flexedge_length",d->flexedge_length,m->nflexedge);
    array(out,"flexedge_J",d->flexedge_J,m->nJfe);
    array(out,"flexedge_velocity",velocity.data(),velocity.size(),false);
    out << '}';
  }
  out << "]}\n";
  std::cout << "edges=" << m->nflexedge << " nnz=" << m->nJfe << '\n';
  return out.good()?0:5;
}
