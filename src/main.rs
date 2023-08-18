use cgroups_rs::{cpuset::CpuSetController, hierarchies, Cgroup, CgroupPid};
use hwloc::Topology;

fn main() {
    let _cpuset = std::env::var("CPUSET").ok().map(|cpuset| {
        println!("try set setcpu {}", cpuset);
        let mut cg = CgroupHandler::new(cpuset);
        let tgid = libc::pid_t::from(nix::unistd::getpid()) as u64;
        cg.add_task_by_tgid(tgid.into());
        cg
    });

    let topo = Topology::new();
    let allowed_cores = topo
        .get_cpubind(hwloc::CpuBindFlags::empty())
        .unwrap_or_else(|| {
            topo.object_at_root()
                .allowed_cpuset()
                .unwrap_or_else(hwloc::CpuSet::full)
        });

    dbg!(allowed_cores);
}

struct CgroupHandler {
    cg: Cgroup,
}

impl CgroupHandler {
    fn new(cpus: impl AsRef<str>) -> Self {
        let cg = Cgroup::new(hierarchies::auto(), "test_topo").unwrap();
        let cpuset = cg
            .controller_of::<CpuSetController>()
            .expect("No cpu controller attached!");

        cpuset.set_cpus(cpus.as_ref()).unwrap();

        Self { cg }
    }

    fn add_task_by_tgid(&mut self, tgid: CgroupPid) {
        self.cg.add_task_by_tgid(tgid).expect("add task to cgroup")
    }
}

impl Drop for CgroupHandler {
    fn drop(&mut self) {
        for task in self.cg.tasks() {
            let _ = self.cg.remove_task_by_tgid(task);
        }
        let _ = self.cg.delete();
    }
}
