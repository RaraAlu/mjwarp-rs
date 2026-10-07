// Development-only authoring. Product tests never execute this tool.
#include <mujoco/mujoco.h>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <vector>

// MuJoCo 3.12.0 exports this native update helper.
extern "C" __declspec(dllimport) void mj_updateSleep(const mjModel*, mjData*);
template<class T>
void array(std::ostream& out, const char* name, const T* p, size_t n, bool comma=true) {
  out << '"' << name << "\":[";
  for (size_t i=0; i<n; ++i) { if(i) out << ','; out << p[i]; }
  out << ']' << (comma?",\n":"\n");
}
int main(int argc, char** argv) {
  if (argc!=3 || mj_version()!=3012000) return 2;
  std::unique_ptr<mjModel, decltype(&mj_deleteModel)> m(mj_loadModel(argv[1], nullptr), mj_deleteModel);
  if (!m || m->ntree!=3 || m->ntendon!=14) return 3;
  m->opt.enableflags |= mjENBL_SLEEP;
  m->opt.disableflags &= ~mjDSBL_ISLAND;
  std::unique_ptr<mjData, decltype(&mj_deleteData)> d(mj_makeData(m.get()), mj_deleteData);
  std::ofstream out(argv[2], std::ios::binary);
  if (!d || !out) return 4;
  const int nt=static_cast<int>(m->ntendon);
  std::vector<mjtNum> ranges(6*nt), margins(2*nt);
  for (int t=0;t<nt;++t) {
    m->tendon_limited[t]=(t==0 || t==8 || t==11 || t==13);
    for (int r=0;r<3;++r) { ranges[2*(r*nt+t)]=-100; ranges[2*(r*nt+t)+1]=100; }
    margins[t]=mjtNum(float(0.125)); margins[nt+t]=0;
  }
  ranges[0]=1; ranges[1]=2;
  ranges[2*(nt+8)]=0; ranges[2*(nt+8)+1]=mjtNum(float(0.5));
  ranges[2*(2*nt+11)]=0; ranges[2*(2*nt+11)+1]=mjtNum(float(0.5));
  for(int r=0;r<3;++r) { ranges[2*(r*nt+13)]=mjtNum(float(-0.0625)); ranges[2*(r*nt+13)+1]=mjtNum(float(0.0625)); }
  for (int t: {0,8,11,13}) if(m->tendon_treenum[t]!=2) { std::cerr<<"tendon trees "<<t; return 5; }
  out << std::setprecision(17) << "{\"native_version\":"<<mj_version()<<",\"seed\":2102,\"ntree\":3,\n";
  array(out,"body_treeid",m->body_treeid,m->nbody);
  array(out,"tendon_limited",m->tendon_limited,m->ntendon);
  array(out,"tendon_range",ranges.data(),ranges.size());
  array(out,"tendon_margin",margins.data(),margins.size());
  const int states[6][3]={{-11,2,1},{1,0,-11},{0,-17,2},{0,1,2},{-11,-17,-11},{-17,1,2}};
  out << "\"cases\":[\n";
  for(int round=0;round<4;++round) for(int w=0;w<6;++w) {
    if(round || w) out << ",\n";
    mj_resetData(m.get(),d.get());
    for(int i=0;i<3;++i) d->tree_asleep[i]=-11;
    mj_updateSleep(m.get(),d.get());
    mj_fwdKinematics(m.get(),d.get());
    std::vector<mjtNum> unfiltered(d->ten_length,d->ten_length+nt);
    for(int i=0;i<3;++i) d->tree_asleep[i]=states[(w+round)%6][i];
    mj_updateSleep(m.get(),d.get());
    for(int t=0;t<nt;++t) {
      m->tendon_range[2*t]=ranges[2*((w%3)*nt+t)];
      m->tendon_range[2*t+1]=ranges[2*((w%3)*nt+t)+1];
      m->tendon_margin[t]=margins[(w%2)*nt+t];
    }
    out << "{\"world_period_index\":"<<w<<",\"round\":"<<round<<',';
    array(out,"input_tree_asleep",d->tree_asleep,3);
    out << "\"input_nbody_awake\":"<<d->nbody_awake<<",\"input_nv_awake\":"<<d->nv_awake<<',';
    mj_fwdKinematics(m.get(),d.get());
    array(out,"ten_length",unfiltered.data(),unfiltered.size());
    array(out,"native_filtered_ten_length",d->ten_length,m->ntendon);
    array(out,"tree_asleep",d->tree_asleep,3);
    array(out,"tree_awake",d->tree_awake,3);
    out << "\"ntree_awake\":"<<d->ntree_awake
        << ",\"native_nbody_awake\":"<<d->nbody_awake
        << ",\"native_nv_awake\":"<<d->nv_awake<<'}';
  }
  out << "]}\n";return out.good()?0:6;
}
