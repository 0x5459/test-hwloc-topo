use cgroups_rs::{cpuset::CpuSetController, hierarchies, Cgroup, CgroupPid};
use hwloc::{ObjectType, Topology, TopologyObject};

fn main() {
    let _cpuset = std::env::var("CPUSET").ok().map(|cpuset| {
        println!("try set setcpu {}", cpuset);
        let mut cg = CgroupHandler::new(cpuset);
        let tgid = libc::pid_t::from(nix::unistd::getpid()) as u64;
        cg.add_task_by_tgid(tgid.into());
        cg
    });

    let topo = Topology::new();
    let core = get_core_by_index(&topo, CoreIndex(0));

    let cpuset = core.allowed_cpuset().unwrap();
    dbg!(cpuset);
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// `CoreIndex` is a simple wrapper type for indexes into the set of visible cores. A `CoreIndex`
/// should only ever be created with a value known to be less than the number of visible cores.
pub struct CoreIndex(usize);

fn get_core_by_index(topo: &Topology, index: CoreIndex) -> &TopologyObject {
    let idx = index.0;

    match topo.objects_with_type(&ObjectType::Core) {
        Ok(all_cores) if idx < all_cores.len() => all_cores[idx],
        Ok(all_cores) => {
            panic!("idx ({}) out of range for {} cores", idx, all_cores.len())
        }
        _e => panic!("failed to get core by index {}", idx,),
    }
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
