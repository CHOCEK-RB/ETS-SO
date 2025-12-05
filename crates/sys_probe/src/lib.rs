use std::{collections::HashMap, fs, num::ParseIntError};
pub use sysinfo::ProcessStatus;
use sysinfo::System;

const NICE_COL: usize = 18;
const PRIO_COL: usize = 17;
const RT_PRIO_COL: usize = 39;

#[derive(Clone, Debug)]
pub struct Process {
    pub name: String,
    pub pid: u32,
    pub run_time: u64,
    pub ram: u64,
    pub nice: Option<i16>,
    pub status: Option<ProcessStatus>,
    pub priority: Option<i16>,
    pub rt_priority: Option<u16>,
    stat: Vec<String>,
    // TODO: add more
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

    fn get_nice(&mut self) -> Result<i16, ParseIntError> {
        let nice = self.stat[NICE_COL].parse::<i16>();
        nice
    }
    fn get_rt_priority(&mut self) -> Result<u16, ParseIntError> {
        let rt_prio = self.stat[RT_PRIO_COL].parse::<u16>();
        rt_prio
    }

    fn get_priority(&mut self) -> Result<i16, ParseIntError> {
        let priority = self.stat[PRIO_COL].parse::<i16>();
        priority
    }

    pub fn refresh(&mut self) {
        self.read_stat();
        if self.stat.len() > 15 {
            if self.priority.is_none() {
                self.priority = Some(self.get_priority().unwrap_or(0));
            }
            if self.rt_priority.is_none() {
                self.rt_priority = Some(self.get_rt_priority().unwrap_or(0));
            }
            if self.nice.is_none() {
                self.nice = Some(self.get_nice().unwrap_or(0));
            }
        }
    }
}

#[derive(Clone)]
pub struct ProcessBuilder {
    name: Option<String>,
    pid: Option<u32>,
    status: Option<ProcessStatus>,
    run_time: Option<u64>,
    ram: Option<u64>,
}

impl ProcessBuilder {
    fn new() -> ProcessBuilder {
        ProcessBuilder {
            name: None,
            pid: None,
            status: None,
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

    fn status(&mut self, status: ProcessStatus) -> &mut Self {
        self.status = Some(status);
        self
    }

    fn build(&self) -> Process {
        Process {
            name: self.name.clone().unwrap(),
            pid: self.pid.unwrap(),
            run_time: self.run_time.unwrap(),
            nice: None,
            status: self.status,
            priority: None,
            rt_priority: None,
            ram: self.ram.unwrap(),
            stat: Vec::new(),
        }
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

        let live_pids: std::collections::HashSet<u32> = self
            .sys
            .processes()
            .keys()
            .map(|pid| pid.as_u32())
            .collect();

        self.processes.retain(|pid, _| live_pids.contains(pid));

        for (pid, process) in self.sys.processes() {
            let pid_u32 = pid.as_u32();

            match process.status() {
                ProcessStatus::Dead | ProcessStatus::Zombie => {
                    self.processes.remove(&pid_u32);
                    continue;
                }
                _ => {}
            }

            let mut process_entry = Process::builder()
                .name(process.name().to_str().unwrap().to_string())
                .pid(pid_u32)
                .run_time(process.run_time())
                .status(process.status())
                .ram(process.memory())
                .build();

            process_entry.refresh();
            self.processes.insert(pid_u32, process_entry);
        }
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
