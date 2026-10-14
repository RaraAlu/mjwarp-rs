// Independent literal-array ABI/preflight test, without DLL/model/physics calls.
#include "../../native/model_probe_sparse.cpp"
#include <array>
#include <cstdlib>
#include <cstdio>
static unsigned checks = 0;
static void require(bool pass) { ++checks; if (!pass) std::abort(); }
int main() {
  mjModel m{};
  m.nv=2; m.nC=3; m.ntree=1;
  int parent[2]{-1,0}, nnz[2]{1,2}, adr[2]{0,1}, col[3]{0,0,1}, ta[1]{0}, tn[1]{2};
  m.dof_parentid=parent; m.M_rownnz=nnz; m.M_rowadr=adr; m.M_colind=col;
  m.tree_dofadr=ta; m.tree_dofnum=tn;
  mjwarp_native_owner owner{nullptr,&m,nullptr};
  mjwarp_sparse_mass_info info{};
  require(mjwarp_native_sparse_mass_info(&owner,&info)==0);
  require(info.schema==1 && info.reserved==0 && info.nv==2 && info.nC==3 && info.ntree==1);
  const auto targets=std::array{&mjwarp_sparse_mass_targets::dof_parentid,&mjwarp_sparse_mass_targets::M_rownnz,
      &mjwarp_sparse_mass_targets::M_rowadr,&mjwarp_sparse_mass_targets::M_colind,
      &mjwarp_sparse_mass_targets::tree_dofadr,&mjwarp_sparse_mass_targets::tree_dofnum};
  const auto sources=std::array{&mjModel::dof_parentid,&mjModel::M_rownnz,&mjModel::M_rowadr,&mjModel::M_colind,
      &mjModel::tree_dofadr,&mjModel::tree_dofnum};
  const size_t lengths[6]{2,2,2,3,1,1};
  int32_t out[6][7];
  auto reset=[&] {for(auto& row:out)for(auto& value:row)value=99;};
  auto unchanged=[&] {for(auto& row:out)for(auto value:row)require(value==99);};
  mjwarp_sparse_mass_targets t{};t.info=info;
  for(size_t i=0;i<6;++i)t.*targets[i]=out[i]+2;
  reset();require(mjwarp_native_copy_sparse_mass(&owner,&t)==0);
  for(size_t i=0;i<6;++i) {
    for(size_t j=0;j<lengths[i];++j)require(out[i][2+j]==(m.*sources[i])[j]);
    for(size_t j=0;j<7;++j)if(j<2 || j>=2+lengths[i])require(out[i][j]==99);
  }
  // Raw ABI copying must not quantize integer payloads through floating point.
  int original[6][3]{};
  for(size_t i=0;i<6;++i)for(size_t j=0;j<lengths[i];++j) {
    original[i][j]=(m.*sources[i])[j];
    (m.*sources[i])[j]=j==0?INT32_MIN+static_cast<int>(i):
        (j==1?INT32_MAX-static_cast<int>(i):16777217);
  }
  reset();require(mjwarp_native_copy_sparse_mass(&owner,&t)==0);
  for(size_t i=0;i<6;++i) {
    require(memcmp(out[i]+2,m.*sources[i],lengths[i]*sizeof(int32_t))==0);
    for(size_t j=0;j<lengths[i];++j)(m.*sources[i])[j]=original[i][j];
  }
  for(int fault=0;fault<11;++fault) {
    auto bad=t;
    switch(fault) {
      case 0:bad.info.schema=2;break;case 1:bad.info.reserved=1;break;
      case 2:++bad.info.nv;break;case 3:++bad.info.nC;break;case 4:++bad.info.ntree;break;
      default:bad.*targets[static_cast<size_t>(fault-5)]=nullptr;break;
    }
    reset();require(mjwarp_native_copy_sparse_mass(&owner,&bad)==8);unchanged();
  }
  for(auto field:sources) {
    int* saved=m.*field;m.*field=nullptr;
    reset();require(mjwarp_native_copy_sparse_mass(&owner,&t)==8);unchanged();m.*field=saved;
  }
  reset();require(mjwarp_native_copy_sparse_mass(nullptr,&t)==8);unchanged();
  require(mjwarp_native_copy_sparse_mass(&owner,nullptr)==8);unchanged();
  auto missing=owner;missing.model=nullptr;
  require(mjwarp_native_copy_sparse_mass(&missing,&t)==8);unchanged();
  require(mjwarp_native_sparse_mass_info(nullptr,&info)==8);
  require(mjwarp_native_sparse_mass_info(&missing,&info)==8);
  require(mjwarp_native_sparse_mass_info(&owner,nullptr)==8);
  for(auto count:std::array{&mjModel::nv,&mjModel::nC,&mjModel::ntree}) {
    mjtSize saved=m.*count;
    for(mjtSize bad:{mjtSize(-1),mjtSize(INT32_MAX)}) {
      m.*count=bad;auto before=info;
      require(mjwarp_native_sparse_mass_info(&owner,&info)==8);
      require(memcmp(&before,&info,sizeof(info))==0);
      reset();require(mjwarp_native_copy_sparse_mass(&owner,&t)==8);unchanged();
    }
    m.*count=saved;
  }
  mjModel zero{};owner.model=&zero;
  require(mjwarp_native_sparse_mass_info(&owner,&info)==0);
  mjwarp_sparse_mass_targets empty{};empty.info=info;
  require(mjwarp_native_copy_sparse_mass(&owner,&empty)==0);
  // Zero DOF/nC does not imply zero tree count; still copy both tree fields.
  int empty_adr[1]{-1},empty_num[1]{0};zero.ntree=1;zero.tree_dofadr=empty_adr;zero.tree_dofnum=empty_num;
  require(mjwarp_native_sparse_mass_info(&owner,&info)==0);empty.info=info;
  int32_t a=99,b=99;empty.tree_dofadr=&a;empty.tree_dofnum=&b;
  require(mjwarp_native_copy_sparse_mass(&owner,&empty)==0);require(a==-1 && b==0);
  std::printf("SPARSE_ABI checks=%u; physics=0; model_compilation=0\n",checks);
}
