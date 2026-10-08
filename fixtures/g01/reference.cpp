// Offline authoring only. Product tests never execute this program.
#include <mujoco/mujoco.h>
#include <cmath>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <vector>
template<class T> void array(std::ostream& out,const char* name,const T* p,size_t n,bool comma=true) {
  out << '"' << name << "\":[";
  for(size_t i=0;i<n;++i) { if(i) out<<',';out<<+p[i]; }
  out<<']'<<(comma?",\n":"\n");
}
// Independent polar oracle: native symmetric eigendecomposition.
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
  xyzw[0]=quat[1];xyzw[1]=quat[2];xyzw[2]=quat[3];xyzw[3]=quat[0];return true;
}
int main(int argc,char** argv) {
  if(argc!=4 || mj_version()!=3012000) return 2;
  char error[1024]{};
  std::unique_ptr<mjModel,decltype(&mj_deleteModel)> m(mj_loadXML(argv[1],nullptr,error,sizeof(error)),mj_deleteModel);
  if(!m) {std::cerr<<error;return 3;}
  if(m->nflex!=4 || m->nmocap!=1 || m->ncam!=8 || m->nlight!=8) return 4;
  int shell=mj_name2id(m.get(),mjOBJ_FLEX,"shell");
  m->flex_interp[shell]=-1;m->flex_centered[shell]=0;
  m->flex_node[3*m->flex_nodeadr[shell]]=0.025;
  for(int kind : {mjOBJ_CAMERA,mjOBJ_LIGHT}) {
    int a=mj_name2id(m.get(),kind,"missing"),b=mj_name2id(m.get(),kind,"missingcom");
    if(kind==mjOBJ_CAMERA) {m->cam_mode[a]=mjCAMLIGHT_TARGETBODY;m->cam_mode[b]=mjCAMLIGHT_TARGETBODYCOM;}
    else {m->light_mode[a]=mjCAMLIGHT_TARGETBODY;m->light_mode[b]=mjCAMLIGHT_TARGETBODYCOM;}
  }
  for(int f=0;f<m->nflex;++f) if(!m->flex_interp[f]) m->flex_edgedamping[f]=0.1;
  std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
  if(!d) return 5;
  mj_setConst(m.get(),d.get());
  mj_saveModel(m.get(),argv[3],nullptr,0);
  std::ofstream out(argv[2],std::ios::binary);if(!d || !out) return 5;
  out<<std::setprecision(17)<<"{\"native_version\":"<<mj_version()<<",\"seed\":2501,\"model\":{\n";
  array(out,"qpos0",m->qpos0,1*m->nq);
  array(out,"body_parentid",m->body_parentid,1*m->nbody);
  array(out,"body_jntadr",m->body_jntadr,1*m->nbody);
  array(out,"body_jntnum",m->body_jntnum,1*m->nbody);
  array(out,"body_pos",m->body_pos,3*m->nbody);
  array(out,"body_quat",m->body_quat,4*m->nbody);
  array(out,"jnt_type",m->jnt_type,1*m->njnt);
  array(out,"jnt_bodyid",m->jnt_bodyid,1*m->njnt);
  array(out,"jnt_qposadr",m->jnt_qposadr,1*m->njnt);
  array(out,"jnt_dofadr",m->jnt_dofadr,1*m->njnt);
  array(out,"jnt_pos",m->jnt_pos,3*m->njnt);
  array(out,"jnt_axis",m->jnt_axis,3*m->njnt);
  array(out,"body_ipos",m->body_ipos,3*m->nbody);
  array(out,"body_iquat",m->body_iquat,4*m->nbody);
  array(out,"body_mass",m->body_mass,1*m->nbody);
  array(out,"body_inertia",m->body_inertia,3*m->nbody);
  array(out,"dof_bodyid",m->dof_bodyid,1*m->nv);
  array(out,"dof_jntid",m->dof_jntid,1*m->nv);
  array(out,"dof_parentid",m->dof_parentid,1*m->nv);
  array(out,"dof_armature",m->dof_armature,1*m->nv);
  array(out,"dof_damping",m->dof_damping,1*m->nv);
  array(out,"flex_interp",m->flex_interp,1*m->nflex);
  array(out,"flex_cellnum",m->flex_cellnum,3*m->nflex);
  array(out,"flex_nodeadr",m->flex_nodeadr,1*m->nflex);
  array(out,"flex_nodenum",m->flex_nodenum,1*m->nflex);
  array(out,"flex_vertadr",m->flex_vertadr,1*m->nflex);
  array(out,"flex_vertnum",m->flex_vertnum,1*m->nflex);
  array(out,"flex_centered",m->flex_centered,1*m->nflex);
  array(out,"flex_nodebodyid",m->flex_nodebodyid,1*m->nflexnode);
  array(out,"flex_vertbodyid",m->flex_vertbodyid,1*m->nflexvert);
  array(out,"flex_node",m->flex_node,3*m->nflexnode);
  array(out,"flex_vert",m->flex_vert,3*m->nflexvert);
  array(out,"flex_vert0",m->flex_vert0,3*m->nflexvert);
  array(out,"body_mocapid",m->body_mocapid,1*m->nbody);
  array(out,"geom_bodyid",m->geom_bodyid,1*m->ngeom);
  array(out,"site_bodyid",m->site_bodyid,1*m->nsite);
  array(out,"cam_mode",m->cam_mode,1*m->ncam);
  array(out,"cam_bodyid",m->cam_bodyid,1*m->ncam);
  array(out,"cam_targetbodyid",m->cam_targetbodyid,1*m->ncam);
  array(out,"light_mode",m->light_mode,1*m->nlight);
  array(out,"light_bodyid",m->light_bodyid,1*m->nlight);
  array(out,"light_targetbodyid",m->light_targetbodyid,1*m->nlight);
  array(out,"geom_type",m->geom_type,1*m->ngeom);
  array(out,"tendon_adr",m->tendon_adr,1*m->ntendon);
  array(out,"tendon_num",m->tendon_num,1*m->ntendon);
  array(out,"wrap_type",m->wrap_type,1*m->nwrap);
  array(out,"wrap_objid",m->wrap_objid,1*m->nwrap);
  array(out,"ten_J_rowadr",m->ten_J_rowadr,1*m->ntendon);
  array(out,"ten_J_rownnz",m->ten_J_rownnz,1*m->ntendon);
  array(out,"ten_J_colind",m->ten_J_colind,1*m->nJten);
  array(out,"body_treeid",m->body_treeid,1*m->nbody);
  array(out,"flex_edgeadr",m->flex_edgeadr,1*m->nflex);
  array(out,"flex_edgenum",m->flex_edgenum,1*m->nflex);
  array(out,"flex_edge",m->flex_edge,2*m->nflexedge);
  array(out,"flexedge_J_rowadr",m->flexedge_J_rowadr,1*m->nflexedge);
  array(out,"flexedge_J_rownnz",m->flexedge_J_rownnz,1*m->nflexedge);
  array(out,"flexedge_J_colind",m->flexedge_J_colind,1*m->nJfe);
  array(out,"geom_pos",m->geom_pos,3*m->ngeom);
  array(out,"geom_quat",m->geom_quat,4*m->ngeom);
  array(out,"site_pos",m->site_pos,3*m->nsite);
  array(out,"site_quat",m->site_quat,4*m->nsite);
  array(out,"cam_pos",m->cam_pos,3*m->ncam);
  array(out,"cam_quat",m->cam_quat,4*m->ncam);
  array(out,"cam_poscom0",m->cam_poscom0,3*m->ncam);
  array(out,"cam_pos0",m->cam_pos0,3*m->ncam);
  array(out,"cam_mat0",m->cam_mat0,9*m->ncam);
  array(out,"light_pos",m->light_pos,3*m->nlight);
  array(out,"light_dir",m->light_dir,3*m->nlight);
  array(out,"light_poscom0",m->light_poscom0,3*m->nlight);
  array(out,"light_pos0",m->light_pos0,3*m->nlight);
  array(out,"light_dir0",m->light_dir0,3*m->nlight);
  array(out,"geom_size",m->geom_size,3*m->ngeom);
  array(out,"wrap_prm",m->wrap_prm,1*m->nwrap);
  array(out,"tendon_range",m->tendon_range,2*m->ntendon);
  array(out,"tendon_margin",m->tendon_margin,1*m->ntendon);
  array(out,"tendon_limited",m->tendon_limited,1*m->ntendon);
  array(out,"enableflags",&m->opt.enableflags,1);
  array(out,"disableflags",&m->opt.disableflags,1,false);
  out<<"},\"cases\":[\n";
  std::vector<mjtNum> nodes(3*m->nflexnode);
  std::vector<mjtNum> length(m->nflexedge),velocity(m->nflexedge);
  std::vector<int> map,faces,axes;
  for(int f=0;f<m->nflex;++f) {
    if(m->flex_interp[f]!=-1) continue;
    int* c=m->flex_cellnum+3*f;int local=0;
    for(int axis=0;axis<3;++axis) for(int side=0;side<2;++side) {
      int a=(axis+1)%3,b=(axis+2)%3;
      for(int u=0;u<c[a];++u) for(int v=0;v<c[b];++v) {
        map.push_back(f);map.push_back(local++);axes.push_back(axis);
        for(int i=0;i<2;++i) for(int j=0;j<2;++j) {
          int g[3]{};g[axis]=side*c[axis];g[a]=u+i;g[b]=v+j;
          faces.push_back(m->flex_nodeadr[f]+(g[0]*(c[1]+1)+g[1])*(c[2]+1)+g[2]);
        }
        for(int p=0;p<5;++p) faces.push_back(-1);
      }
    }
  }
  std::vector<mjtNum> facepos(27*axes.size()),facequat(4*axes.size());
  for(int c=0;c<16;++c) {
    if(c) out<<",\n";mj_resetData(m.get(),d.get());
    for(int j=0;j<m->njnt;++j) {
      int q=m->jnt_qposadr[j],type=m->jnt_type[j];
      if(type==mjJNT_HINGE || type==mjJNT_SLIDE)
        d->qpos[q]+=0.015*c*std::sin(0.37*(j+1)*(c+1));
      if(type==mjJNT_FREE) {d->qpos[q]+=0.02*c;d->qpos[q+1]-=0.01*c;q+=3;}
      if(type==mjJNT_BALL || type==mjJNT_FREE) {
        double a=0.035*c;d->qpos[q]=2*std::cos(a);
        for(int k=1;k<4;++k) d->qpos[q+k]=2*std::sin(a)/std::sqrt(3.0);
      }
    }
    for(int k=0;k<m->nv;++k) d->qvel[k]=mjtNum(float(0.03*std::sin(0.51*(k+1)*(c+1))));
    d->mocap_pos[0]+=0.02*c;d->mocap_pos[1]-=0.01*c;
    d->mocap_quat[0]=2*std::cos(0.04*c);d->mocap_quat[3]=2*std::sin(0.04*c);
    for(int q=0;q<m->nq;++q) d->qpos[q]=mjtNum(float(d->qpos[q]));
    for(int k=0;k<3;++k) d->mocap_pos[k]=mjtNum(float(d->mocap_pos[k]));
    for(int k=0;k<4;++k) d->mocap_quat[k]=mjtNum(float(d->mocap_quat[k]));
    mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_camlight(m.get(),d.get());
    mj_flex(m.get(),d.get());mj_tendon(m.get(),d.get());
    // Native flex skips interpolated edge lengths. Measure native vertices
    // with native vector helpers; retain native direct-edge sparse Jacobians.
    for(int f=0;f<m->nflex;++f) for(int i=0;i<m->flex_edgenum[f];++i) {
      int e=m->flex_edgeadr[f]+i,b=m->flex_vertadr[f];
      mjtNum delta[3];mju_sub3(delta,d->flexvert_xpos+3*(b+m->flex_edge[2*e+1]),
                                    d->flexvert_xpos+3*(b+m->flex_edge[2*e]));
      length[e]=mju_norm3(delta);velocity[e]=0;
      for(int k=0;k<m->flexedge_J_rownnz[e];++k) {
        int slot=m->flexedge_J_rowadr[e]+k;
        velocity[e]+=d->flexedge_J[slot]*d->qvel[m->flexedge_J_colind[slot]];
      }
    }
    for(int n=0;n<m->nflexnode;++n) {
      int b=m->flex_nodebodyid[n];
      mju_mulMatVec3(nodes.data()+3*n,d->xmat+9*b,m->flex_node+3*n);
      mju_addTo3(nodes.data()+3*n,d->xpos+3*b);
    }
    for(size_t fi=0;fi<axes.size();++fi) {
      mjtNum t1[3]{},t2[3]{},normal[3],F[9]{};
      for(int slot=0;slot<9;++slot) for(int k=0;k<3;++k) {
        int n=faces[9*fi+slot];
        mjtNum val=n<0?0:nodes[3*n+k];facepos[27*fi+3*slot+k]=val;
        if(n>=0) {t1[k]+=val*(slot/2?0.5:-0.5);t2[k]+=val*(slot%2?0.5:-0.5);}
      }
      mju_cross(normal,t1,t2);
      const mjtNum* columns[3];
      if(axes[fi]==0) {columns[0]=normal;columns[1]=t1;columns[2]=t2;}
      else if(axes[fi]==1) {columns[0]=t2;columns[1]=normal;columns[2]=t1;}
      else {columns[0]=t1;columns[1]=t2;columns[2]=normal;}
      for(int a=0;a<3;++a) for(int b=0;b<3;++b) F[3*a+b]=columns[b][a];
      if(!polar(F,facequat.data()+4*fi)) return 6;
    }
    out<<"{\"id\":"<<c<<',';
    array(out,"qpos",d->qpos,m->nq);array(out,"qvel",d->qvel,m->nv);
    array(out,"mocap_pos",d->mocap_pos,3*m->nmocap);array(out,"mocap_quat",d->mocap_quat,4*m->nmocap);
    array(out,"xpos",d->xpos,3*m->nbody);
    array(out,"xquat",d->xquat,4*m->nbody);
    array(out,"xmat",d->xmat,9*m->nbody);
    array(out,"xipos",d->xipos,3*m->nbody);
    array(out,"ximat",d->ximat,9*m->nbody);
    array(out,"xanchor",d->xanchor,3*m->njnt);
    array(out,"xaxis",d->xaxis,3*m->njnt);
    array(out,"subtree_com",d->subtree_com,3*m->nbody);
    array(out,"cinert",d->cinert,10*m->nbody);
    array(out,"cdof",d->cdof,6*m->nv);
    array(out,"geom_xpos",d->geom_xpos,3*m->ngeom);
    array(out,"geom_xmat",d->geom_xmat,9*m->ngeom);
    array(out,"site_xpos",d->site_xpos,3*m->nsite);
    array(out,"site_xmat",d->site_xmat,9*m->nsite);
    array(out,"cam_xpos",d->cam_xpos,3*m->ncam);
    array(out,"cam_xmat",d->cam_xmat,9*m->ncam);
    array(out,"light_xpos",d->light_xpos,3*m->nlight);
    array(out,"light_xdir",d->light_xdir,3*m->nlight);
    array(out,"flexvert_xpos",d->flexvert_xpos,3*m->nflexvert);
    array(out,"native_flexedge_length",d->flexedge_length,1*m->nflexedge);
    array(out,"flexedge_length",length.data(),length.size());
    array(out,"flexedge_velocity",velocity.data(),velocity.size());
    array(out,"flexedge_J",d->flexedge_J,1*m->nJfe);
    array(out,"ten_length",d->ten_length,1*m->ntendon);
    array(out,"ten_J",d->ten_J,1*m->nJten);
    array(out,"ten_wrapadr",d->ten_wrapadr,1*m->ntendon);
    array(out,"ten_wrapnum",d->ten_wrapnum,1*m->ntendon);
    int points=0;for(int t=0;t<m->ntendon;++t) points+=d->ten_wrapnum[t];
    array(out,"wrap_xpos",d->wrap_xpos,3*points);array(out,"wrap_obj",d->wrap_obj,points);
    array(out,"flexnode_xpos",nodes.data(),nodes.size());
    array(out,"face_xpos",facepos.data(),facepos.size());array(out,"face_quat",facequat.data(),facequat.size(),false);
    out<<'}';
  }
  out<<"]}\n";return out.good()?0:7;
}
