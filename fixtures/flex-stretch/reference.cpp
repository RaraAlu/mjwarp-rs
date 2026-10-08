// Development-only native authoring. Product tests read static files.
// Independent dense derivative: H = 0.5 D^T K D + clamped geometric term.
#include <mujoco/mujoco.h>
#include <algorithm>
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
  if(!m) {std::cerr << error;return 3;}
  // Public model floats and state round to f32 before independent f64 math.
  for(int i=0;i<m->nflexstiffness;++i) m->flex_stiffness[i]=float(m->flex_stiffness[i]);
  for(int i=0;i<m->nflexedge;++i) m->flexedge_length0[i]=float(m->flexedge_length0[i]);
  mj_saveModel(m.get(),argv[3],nullptr,0);
  std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
  std::ofstream out(argv[2],std::ios::binary);
  if(!d || !out) return 4;
  out << std::setprecision(17) << "{\"native_version\":" << mj_version()
      << ",\"seed\":2601,\"absolute_tolerance\":0.00002,\"relative_tolerance\":0.00002,\n"
      << "\"nq\":" << m->nq << ",\"nv\":" << m->nv << ",\"nbody\":" << m->nbody
      << ",\"njnt\":" << m->njnt << ",\"model\":{\n";
#define FIELD(name,n) array(out,#name,m->name,n)
  FIELD(qpos0,m->nq);FIELD(body_parentid,m->nbody);FIELD(body_jntadr,m->nbody);FIELD(body_jntnum,m->nbody);
  FIELD(body_pos,3*m->nbody);FIELD(body_quat,4*m->nbody);FIELD(body_mocapid,m->nbody);
  FIELD(jnt_type,m->njnt);FIELD(jnt_bodyid,m->njnt);FIELD(jnt_qposadr,m->njnt);FIELD(jnt_dofadr,m->njnt);
  FIELD(jnt_pos,3*m->njnt);FIELD(jnt_axis,3*m->njnt);
  FIELD(body_ipos,3*m->nbody);FIELD(body_iquat,4*m->nbody);FIELD(body_mass,m->nbody);FIELD(body_inertia,3*m->nbody);
  FIELD(dof_bodyid,m->nv);FIELD(dof_jntid,m->nv);FIELD(dof_parentid,m->nv);FIELD(dof_armature,m->nv);FIELD(dof_damping,m->nv);
  FIELD(geom_bodyid,m->ngeom);FIELD(geom_pos,3*m->ngeom);FIELD(geom_quat,4*m->ngeom);
  FIELD(site_bodyid,m->nsite);FIELD(site_pos,3*m->nsite);FIELD(site_quat,4*m->nsite);
  FIELD(flex_interp,m->nflex);FIELD(flex_cellnum,3*m->nflex);FIELD(flex_nodeadr,m->nflex);FIELD(flex_nodenum,m->nflex);
  FIELD(flex_vertadr,m->nflex);FIELD(flex_vertnum,m->nflex);FIELD(flex_centered,m->nflex);
  FIELD(flex_nodebodyid,m->nflexnode);FIELD(flex_vertbodyid,m->nflexvert);FIELD(flex_node,3*m->nflexnode);
  FIELD(flex_vert,3*m->nflexvert);FIELD(flex_vert0,3*m->nflexvert);
  FIELD(flex_edgeadr,m->nflex);FIELD(flex_edgenum,m->nflex);FIELD(flex_edge,2*m->nflexedge);
  FIELD(flexedge_J_rowadr,m->nflexedge);FIELD(flexedge_J_rownnz,m->nflexedge);FIELD(flexedge_J_colind,m->nJfe);
  FIELD(flex_dim,m->nflex);FIELD(flex_rigid,m->nflex);FIELD(flex_elemadr,m->nflex);FIELD(flex_elemnum,m->nflex);
  FIELD(flex_elemdataadr,m->nflex);FIELD(flex_elemedgeadr,m->nflex);FIELD(flex_stiffnessadr,m->nflex);
  FIELD(flex_elem,m->nflexelemdata);FIELD(flex_elemedge,m->nflexelemedge);FIELD(flexedge_length0,m->nflexedge);
  array(out,"flex_stiffness",m->flex_stiffness,m->nflexstiffness,false);
#undef FIELD
  out << "},\"cases\":[\n";
  const int tri[6]={1,2,2,0,0,1},tet[12]={0,1,1,2,2,0,2,3,0,3,1,3};
  for(int c=0;c<12;++c) {
    if(c) out << ",\n";
    mj_resetData(m.get(),d.get());
    for(int q=0;q<m->nq;++q) d->qpos[q]=float(c?0.03*c*std::sin(0.37*(q+1)*(c+1)):0);
    mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_flex(m.get(),d.get());
    std::vector<double> diagonal(6*m->nflexvert),blocks(9*m->nflexedge),length(m->nflexedge);
    for(int f=0;f<m->nflex;++f) {
      int va=m->flex_vertadr[f],ea=m->flex_edgeadr[f],nv=m->flex_vertnum[f],dim=m->flex_dim[f];
      for(int e=0;e<m->flex_edgenum[f];++e) {
        mjtNum delta[3];mju_sub3(delta,d->flexvert_xpos+3*(va+m->flex_edge[2*(ea+e)]),
                                     d->flexvert_xpos+3*(va+m->flex_edge[2*(ea+e)+1]));
        length[ea+e]=mju_norm3(delta);
      }
      if(m->flex_interp[f] || m->flex_rigid[f] || dim<2 || m->flex_stiffnessadr[f]<0 ||
         m->flex_stiffness[m->flex_stiffnessadr[f]]==0) continue;
      int n=dim==2?3:6,nd=3*(dim+1),size=3*nv;
      const int* pairs=dim==2?tri:tet;
      std::vector<double> dense(size*size);
      for(int el=0;el<m->flex_elemnum[f];++el) {
        const int* verts=m->flex_elem+m->flex_elemdataadr[f]+el*(dim+1);
        const int* ids=m->flex_elemedge+m->flex_elemedgeadr[f]+el*n;
        double metric[36]{},derivative[72]{},elong[6]{},tension[6]{};
        int k=0;
        for(int a=0;a<n;++a) for(int b=a;b<n;++b) {
          double x=m->flex_stiffness[m->flex_stiffnessadr[f]+21*el+k++];metric[6*a+b]=metric[6*b+a]=x;
        }
        for(int a=0;a<n;++a) {
          int i=pairs[2*a],j=pairs[2*a+1],e=ea+ids[a];
          for(int r=0;r<3;++r) {
            double x=d->flexvert_xpos[3*(va+verts[i])+r]-d->flexvert_xpos[3*(va+verts[j])+r];
            derivative[a*nd+3*i+r]=2*x;derivative[a*nd+3*j+r]=-2*x;
          }
          elong[a]=length[e]*length[e]-m->flexedge_length0[e]*m->flexedge_length0[e];
        }
        for(int a=0;a<n;++a) {
          for(int b=0;b<n;++b) tension[a]+=metric[6*a+b]*elong[b];
          tension[a]=std::max(tension[a],0.0);
        }
        // Assemble all scalar coordinate derivatives, not sparse edge blocks.
        for(int r=0;r<nd;++r) for(int col=0;col<nd;++col) {
          double h=0;
          for(int a=0;a<n;++a) for(int b=0;b<n;++b)
            h+=0.5*derivative[a*nd+r]*metric[6*a+b]*derivative[b*nd+col];
          if(r%3==col%3) for(int a=0;a<n;++a) {
            int i=pairs[2*a],j=pairs[2*a+1];
            int sr=(r/3==i)-(r/3==j),sc=(col/3==i)-(col/3==j);h+=tension[a]*sr*sc;
          }
          int gr=3*verts[r/3]+r%3,gc=3*verts[col/3]+col%3;dense[gr*size+gc]+=h;
        }
      }
      const int upper[12]={0,0,0,1,0,2,1,1,1,2,2,2};
      for(int v=0;v<nv;++v) for(int k=0;k<6;++k)
        diagonal[6*(va+v)+k]=dense[(3*v+upper[2*k])*size+3*v+upper[2*k+1]];
      for(int e=0;e<m->flex_edgenum[f];++e) {
        int a=m->flex_edge[2*(ea+e)],b=m->flex_edge[2*(ea+e)+1];
        for(int r=0;r<3;++r) for(int col=0;col<3;++col)
          blocks[9*(ea+e)+3*r+col]=dense[(3*a+r)*size+3*b+col];
      }
    }
    out << "{\"id\":" << c << ',';
    array(out,"qpos",d->qpos,m->nq);array(out,"flexvert_xpos",d->flexvert_xpos,3*m->nflexvert);
    array(out,"flexedge_length",length.data(),length.size());
    array(out,"flexvert_hessian",diagonal.data(),diagonal.size());
    array(out,"flexedge_hessian",blocks.data(),blocks.size(),false);out << '}';
  }
  out << "]}\n";
  std::cout << "flex=" << m->nflex << " vertices=" << m->nflexvert << " edges=" << m->nflexedge
            << " elements=" << m->nflexelem << " coefficients=" << m->nflexstiffness << '\n';
  return out.good()?0:5;
}
