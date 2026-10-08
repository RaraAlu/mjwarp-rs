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
// Independent reference: symmetric eigendecomposition, not Warp's iteration.
bool polar(const mjtNum* F,mjtNum* xyzw) {
  mjtNum C[9]{},eval[3],V[9],unused[4],inverse[9]{},R[9]{},quat[4];
  for(int i=0;i<3;++i) for(int j=0;j<3;++j)
    for(int k=0;k<3;++k) C[3*i+j]+=F[3*k+i]*F[3*k+j];
  mju_eig3(eval,V,unused,C);
  for(int k=0;k<3;++k) {
    if(eval[k]<=1e-12) return false;
    for(int i=0;i<3;++i) for(int j=0;j<3;++j)
      inverse[3*i+j]+=V[3*i+k]*V[3*j+k]/std::sqrt(eval[k]);
  }
  mju_mulMatMat(R,F,inverse,3,3,3);mju_mat2Quat(quat,R);mju_normalize4(quat);
  xyzw[0]=quat[1];xyzw[1]=quat[2];xyzw[2]=quat[3];xyzw[3]=quat[0];
  return true;
}
int main(int argc,char** argv) {
  if(argc!=3 || mj_version()!=3012000) return 2;
  std::unique_ptr<mjModel,decltype(&mj_deleteModel)> m(mj_loadModel(argv[1],nullptr),mj_deleteModel);
  if(!m || m->nflex!=4 || m->nmocap!=1) return 3;
  std::vector<int> map,face,axes;
  for(int f=0;f<m->nflex;++f) {
    if(m->flex_interp[f]!=1) continue;
    m->flex_interp[f]=-1;
    int* c=m->flex_cellnum+3*f;int local=0;
    // Existing grids have no interior nodes: native TFI does not change them.
    if(c[0]>1 && c[1]>1 && c[2]>1) return 4;
    for(int axis=0;axis<3;++axis) for(int side=0;side<2;++side) {
      int a=(axis+1)%3,b=(axis+2)%3;
      for(int u=0;u<c[a];++u) for(int v=0;v<c[b];++v) {
        map.push_back(f);map.push_back(local++);axes.push_back(axis);
        for(int i=0;i<2;++i) for(int j=0;j<2;++j) {
          int g[3]{};g[axis]=side*c[axis];g[a]=u+i;g[b]=v+j;
          face.push_back(m->flex_nodeadr[f]+(g[0]*(c[1]+1)+g[1])*(c[2]+1)+g[2]);
        }
        for(int pad=0;pad<5;++pad) face.push_back(-1);
      }
    }
  }
  const size_t nf=axes.size();if(nf!=16) return 4;
  std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
  std::ofstream out(argv[2],std::ios::binary);if(!d || !out) return 5;
  out << std::setprecision(17) << "{\"native_version\":" << mj_version()
    << ",\"seed\":2401,\"nflexface\":"<<nf<<",\"model\":{\n";
  array(out,"flex_interp",m->flex_interp,m->nflex);
  array(out,"flex_face_map",map.data(),map.size());array(out,"flex_face",face.data(),face.size(),false);
  out << "},\"cases\":[\n";
  std::vector<mjtNum> nodes(3*m->nflexnode),positions(27*nf),quats(4*nf);
  for(int c=0;c<12;++c) {
    if(c) out << ",\n";mj_resetData(m.get(),d.get());
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
    mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_flex(m.get(),d.get());
    for(int n=0;n<m->nflexnode;++n) {
      int b=m->flex_nodebodyid[n];
      mju_mulMatVec3(nodes.data()+3*n,d->xmat+9*b,m->flex_node+3*n);
      mju_addTo3(nodes.data()+3*n,d->xpos+3*b);
    }
    for(size_t f=0;f<nf;++f) {
      for(int slot=0;slot<9;++slot) {
        mjtNum* p=positions.data()+27*f+3*slot;int n=face[9*f+slot];
        if(n<0) mju_zero3(p);else mju_copy3(p,nodes.data()+3*n);
      }
      const mjtNum* p=positions.data()+27*f;
      mjtNum du[3],dv[3],tmp[3],normal[3],F[9];
      mju_sub3(du,p+6,p);mju_sub3(tmp,p+9,p+3);mju_addTo3(du,tmp);mju_scl3(du,du,0.5);
      mju_sub3(dv,p+3,p);mju_sub3(tmp,p+9,p+6);mju_addTo3(dv,tmp);mju_scl3(dv,dv,0.5);
      mju_cross(normal,du,dv);int axis=axes[f];
      for(int k=0;k<3;++k) {
        F[3*k+axis]=normal[k];F[3*k+(axis+1)%3]=du[k];F[3*k+(axis+2)%3]=dv[k];
      }
      if(!polar(F,quats.data()+4*f)) return 6;
    }
    out << "{\"id\":"<<c<<',';
    array(out,"qpos",d->qpos,m->nq);array(out,"mocap_pos",d->mocap_pos,3);array(out,"mocap_quat",d->mocap_quat,4);
    array(out,"flexnode_xpos",nodes.data(),nodes.size());array(out,"flexvert_xpos",d->flexvert_xpos,3*m->nflexvert);
    array(out,"face_xpos",positions.data(),positions.size());array(out,"face_quat",quats.data(),quats.size(),false);
    out << '}';
  }
  out << "]}\n";return out.good()?0:7;
}
