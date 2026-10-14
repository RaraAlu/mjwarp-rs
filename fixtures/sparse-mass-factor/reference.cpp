// Independent offline MuJoCo author. Product tests never execute this program.
#include <mujoco/mujoco.h>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <map>
#include <memory>
#include <sstream>
#include <vector>
#include <array>

template<class T> void array(std::ostream& o, const char* name, const T* p, size_t n) {
  o << '"' << name << "\":[";
  for (size_t i=0;i<n;++i) { if(i)o<<','; o<<p[i]; }
  o << ']';
}
void open(std::ostream& o, int i) {
  o << "<body name='b"<<i<<"' pos='.08 .01 .02'><joint axis='0 0 1' armature='2'/>"
    << "<inertial pos='.03 .01 0' mass='.01' diaginertia='.001 .001 .001'/>";
}
void chain(std::ostream& o, int count, int base) {
  for(int i=0;i<count;++i)open(o,base+i);
  for(int i=0;i<count;++i)o<<"</body>";
}
int main(int argc,char** argv) {
  if(argc!=2)return 2;
  std::filesystem::path dir(argv[1]);
  std::filesystem::create_directories(dir);
  std::ofstream out(dir/"native.json",std::ios::binary);
  out<<std::setprecision(17)<<"{\"native_version\":"<<mj_version()<<",\"matrix_input\":\"native CRB rounded to f32 before native factorM\",\"models\":[";
  const char* names[]={"chain65","branch67","sparse-compact","sparse-tile"};
  for(int fixture=0;fixture<4;++fixture) {
    if(fixture)out<<',';
    std::ostringstream xml;
    xml<<"<mujoco><option jacobian='sparse'/><worldbody>";
    if(fixture==1) {open(xml,0);chain(xml,33,1);chain(xml,33,34);xml<<"</body>";}
    else {chain(xml,65,0);if(fixture>=2)chain(xml,fixture==2?1:7,65);}
    xml<<"</worldbody></mujoco>";
    auto path=dir/(std::string(names[fixture])+".xml");
    {std::ofstream f(path,std::ios::binary);f<<xml.str();}
    char error[2048]={};
    std::unique_ptr<mjModel,decltype(&mj_deleteModel)> m(mj_loadXML(path.string().c_str(),nullptr,error,sizeof(error)),mj_deleteModel);
    if(!m){fprintf(stderr,"%s\n",error);return 3;}
    std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
    if(!d)return 4;
    const int n=static_cast<int>(m->nv),nc=static_cast<int>(m->nC);
    out<<"{\"name\":\""<<names[fixture]<<"\",\"nv\":"<<n<<",\"nc\":"<<nc<<',';
    array(out,"dof_parentid",m->dof_parentid,n);out<<',';
    array(out,"m_rownnz",m->M_rownnz,n);out<<',';
    array(out,"m_rowadr",m->M_rowadr,n);out<<',';
    array(out,"m_colind",m->M_colind,nc);out<<',';
    array(out,"tree_dofadr",m->tree_dofadr,m->ntree);out<<',';
    array(out,"tree_dofnum",m->tree_dofnum,m->ntree);out<<',';
    // Independent transcription of frozen io.py metadata, never candidate code.
    std::vector<int> adr(n,-1),depth(n,-1),offsets{0};int total=0;
    for(int t=0;t<m->ntree;++t) {
      int start=m->tree_dofadr[t],size=m->tree_dofnum[t];if(!size)continue;
      int last=start+size-1,nnz=m->M_rowadr[last]+m->M_rownnz[last]-m->M_rowadr[start];
      bool compact=nnz==size,triangular=nnz==size*(size+1)/2;
      int address=-1;
      if(size<=6&&(compact||triangular)){address=compact?-2:total;if(!compact)total+=size*size;}
      else if(size<=64){address=total;total+=size*size;}
      for(int i=start;i<start+size;++i)adr[i]=address;
    }
    std::map<int,std::vector<std::array<int,3>>> levels;
    for(int k=0;k<n;++k) {
      if(m->M_rownnz[k]==1)continue;
      depth[k]=depth[m->dof_parentid[k]]+1;
      if(adr[k]!=-1)continue;
      int a=m->M_rowadr[k]+m->M_rownnz[k]-2;
      for(int i=m->dof_parentid[k];i>=0;i=m->dof_parentid[i])levels[depth[i]].push_back({i,k,a--});
    }
    std::vector<int> flat;
    for(auto& [level,updates]:levels){for(auto u:updates)flat.insert(flat.end(),u.begin(),u.end());offsets.push_back(static_cast<int>(flat.size()/3));}
    array(out,"qld_block_adr",adr.data(),adr.size());out<<",\"qld_block_total\":"<<total<<',';
    array(out,"qld_updates_flat",flat.data(),flat.size());out<<',';
    array(out,"qld_level_offsets",offsets.data(),offsets.size());out<<",\"cases\":[";
    for(int c=0;c<2;++c) {
      if(c)out<<',';
      mj_resetData(m.get(),d.get());
      for(int i=0;i<n;++i)d->qpos[i]=static_cast<float>(c?((i*17)%19-9)*.025:0);
      mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_crb(m.get(),d.get());
      for(int a=0;a<nc;++a)d->M[a]=static_cast<float>(d->M[a]);
      mj_factorM(m.get(),d.get());
      out<<'{';array(out,"qpos",d->qpos,m->nq);out<<',';
      array(out,"matrix",d->M,nc);out<<',';
      array(out,"factor",d->qLD,nc);out<<',';
      array(out,"diagonal_inverse",d->qLDiagInv,n);out<<'}';
    }
    out<<"]}";
  }
  out<<"]}\n";
  return out.good()?0:5;
}
