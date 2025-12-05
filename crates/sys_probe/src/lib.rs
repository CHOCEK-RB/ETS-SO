use std::{
    collections::{HashMap, HashSet},
    fs,
};
use sysinfo::{ProcessStatus, System};

const NICE_COL: usize = 18;
const PRIO_COL: usize = 17;
const RT_PRIO_COL: usize = 39;

#[derive(Clone, Debug)]
pub struct Process {
    pub name: String,
    pub pid: u32,
    pub run_time: u64,
    pub nice: i16,
    pub priority: i16,
    pub rt_priority: u16,
    pub ram: u64,

    stat: Vec<String>,
    // TODO: add more
}

#[derive(Clone)]
pub struct ProcessBuilder {
    name: Option<String>,
    pid: Option<u32>,
    run_time: Option<u64>,
    ram: Option<u64>,
}

impl ProcessBuilder {
    fn new() -> ProcessBuilder {
        ProcessBuilder {
            name: None,
            pid: None,
            run_time: None,
            ram: None,
        }
    }

    fn name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    fn pid(&mut self, pid: u32) -> &mut Self {
        self.pid = Some(pid);
        self
    }

    fn run_time(&mut self, run_time: u64) -> &mut Self {
        self.run_time = Some(run_time);
        self
    }

    fn ram(&mut self, ram: u64) -> &mut Self {
        self.ram = Some(ram);
        self
    }

    fn build(&self) -> Process {
        Process {
            name: self.name.clone().unwrap(),
            pid: self.pid.unwrap(),
            run_time: self.run_time.unwrap(),
            nice: 0,
            priority: 0,
            rt_priority: 0,
            ram: self.ram.unwrap(),
            stat: Vec::new(),
        }
    }
}

impl Process {
    pub fn builder() -> ProcessBuilder {
        ProcessBuilder::new()
    }

    fn read_stat(&mut self) {
        if let Ok(stat) = fs::read_to_string(format!("/proc/{}/stat", self.pid)) {
            self.stat = stat.split_whitespace().map(|s| s.to_string()).collect();
        }
    }

    fn set_nice(&mut self) {
        self.nice = self.stat[NICE_COL].parse::<i16>().unwrap();
    }

    fn set_priority(&mut self) {
        self.priority = self.stat[PRIO_COL].parse::<i16>().unwrap();
    }

    fn set_rt_priority(&mut self) {
        self.rt_priority = self.stat[RT_PRIO_COL].parse::<u16>().unwrap_or(0);
    }
}

pub struct SysProbe {
    sys: System,
    pub quantum: u32,
    pub processes: HashMap<u32, Process>,
}

impl SysProbe {
    pub fn new() -> SysProbe {
        SysProbe {
            sys: System::new(),
            processes: HashMap::new(),
            quantum: 0,
        }
    }

    pub fn init(&mut self) {
        self.set_quantum();
        self.refresh_processes();
    }

    pub fn set_quantum(&mut self) {
        let timeslice = fs::read_to_string("/proc/sys/kernel/sched_rr_timeslice_ms").unwrap();
        self.quantum = timeslice.trim().parse::<u32>().unwrap();
    }

    pub fn refresh_processes(&mut self) {
        self.sys.refresh_all();

        let mut current_pids = HashSet::new();

        for (pid, process) in self.sys.processes() {
            if process.status() == ProcessStatus::Dead || process.status() == ProcessStatus::Zombie
            {
                continue;
            }

            current_pids.insert(pid.as_u32());

            let mut _process = Process::builder()
                .name(process.name().to_str().unwrap().to_string())
                .pid(pid.as_u32())
                .run_time(process.run_time())
                .ram(process.memory())
                .build();

            _process.read_stat();
            _process.set_nice();
            _process.set_priority();
            _process.set_rt_priority();

            self.processes.insert(pid.as_u32(), _process);
        }

        self.processes.retain(|&pid, _| current_pids.contains(&pid));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process() {
        let mut sysinfo = SysProbe::new();
        sysinfo.init();

        assert!(sysinfo.processes.len() > 0);
        assert!(sysinfo.quantum == 100);
    }

    #[test]
    fn process_builder() {
        let process = Process::builder()
            .name("test".to_string())
            .pid(1)
            .run_time(0)
            .ram(0)
            .build();

        assert!(process.name == "test");
        assert!(process.pid == 1);
        assert!(process.run_time == 0);
        assert!(process.ram == 0);
    }
}
