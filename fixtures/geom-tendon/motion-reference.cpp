// Development-only reference authoring. Product tests only read frozen JSON.
#include <mujoco/mujoco.h>
#include <cmath>
#include <fstream>
#include <iomanip>
#include <memory>
#include <string>

template<class T>
void array(std::ostream& out,const char* name,const T* values,mjtSize count,bool comma=true) {
    out << '"' << name << "\":[";
    for(mjtSize i=0;i<count;++i) {if(i)out << ',';out << values[i];}
    out << ']' << (comma?",\n":"\n");
}
bool read_values(std::istream& in,mjtNum* values,mjtSize count) {
    for(mjtSize i=0;i<count;++i)if(!(in>>values[i])||!std::isfinite(values[i]))return false;
    return true;
}
int main(int argc,char** argv) {
    if(argc!=4||mj_version()!=3012000)return 2;
    std::unique_ptr<mjModel,decltype(&mj_deleteModel)> m(mj_loadModel(argv[1],nullptr),mj_deleteModel);
    if(!m||m->nq!=6||m->nv!=5||m->ngeom!=4||m->nsite!=20||m->ntendon!=10||
       m->nJten!=10||m->nwrap!=37||m->nmocap!=1)return 3;
    std::unique_ptr<mjData,decltype(&mj_deleteData)> d(mj_makeData(m.get()),mj_deleteData);
    std::ifstream in(argv[2]);std::ofstream out(argv[3],std::ios::binary);
    if(!in||!out||!d)return 4;
    std::string magic;int count=0;
    if(!(in>>magic>>count)||magic!="MJWARP_INSIDE_MOTION_V1"||count!=3)return 5;
    out << std::setprecision(17) << "{\"native_version\":3012000,\"absolute_tolerance\":0.00002,"
        << "\"relative_tolerance\":0.00002,\"source_world\":2,\"cases\":[\n";
    for(int i=0;i<count;++i) {
        int frame=0;
        if(!(in>>frame)||frame!=(i==0?310:(i==1?344:393)))return 6;
        mj_resetData(m.get(),d.get());
        if(!read_values(in,d->qpos,m->nq)||!read_values(in,d->mocap_pos,3*m->nmocap)||
           !read_values(in,d->mocap_quat,4*m->nmocap)||!read_values(in,m->geom_size,3*m->ngeom))return 7;
        mj_kinematics(m.get(),d.get());mj_comPos(m.get(),d.get());mj_tendon(m.get(),d.get());
        if(i)out << ",\n";
        out << "{\"source_frame\":" << frame << ",\n";
        array(out,"qpos",d->qpos,m->nq);array(out,"mocap_pos",d->mocap_pos,3*m->nmocap);
        array(out,"mocap_quat",d->mocap_quat,4*m->nmocap);array(out,"geom_size",m->geom_size,3*m->ngeom);
        array(out,"ten_length",d->ten_length,m->ntendon);array(out,"ten_J",d->ten_J,m->nJten);
        array(out,"ten_wrapadr",d->ten_wrapadr,m->ntendon);array(out,"ten_wrapnum",d->ten_wrapnum,m->ntendon);
        array(out,"wrap_obj",d->wrap_obj,2*m->nwrap);array(out,"wrap_xpos",d->wrap_xpos,6*m->nwrap,false);
        out << '}';
    }
    std::string extra;if(in>>extra)return 8;
    out << "]}\n";out.flush();return out.good()?0:9;
}
