#![allow(dead_code)]

use std::{collections::HashMap, fs};
use sysinfo::{ProcessStatus, System};

pub struct Process {
    pub name: String,
    pub pid: u32,
    pub run_time: u64,
    pub nice: i16,
    pub priority: i16,
    pub memory: u64,
    pub ram: u64,

    stat: Vec<String>,
    // TODO: add more
}

#[derive(Clone)]
pub struct ProcessBuilder {
    name: Option<String>,
    pid: Option<u32>,
    run_time: Option<u64>,
    memory: Option<u64>,
    ram: Option<u64>,
}

impl ProcessBuilder {
    fn new() -> ProcessBuilder {
        ProcessBuilder {
            name: None,
            pid: None,
            run_time: None,
            memory: None,
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

    fn memory(&mut self, memory: u64) -> &mut Self {
        self.memory = Some(memory);
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
            memory: self.memory.unwrap(),
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
        let stat = fs::read_to_string(format!("/proc/{}/stat", self.pid as i32)).unwrap();
        self.stat = stat.split_whitespace().map(|s| s.to_string()).collect();
    }

    fn set_nice(&mut self) {
        self.nice = self.stat[20].parse::<i16>().unwrap();
    }

    fn set_priority(&mut self) {
        self.priority = self.stat[19].parse::<i16>().unwrap();
    }
}

pub struct SysProbe {
    sys: System,
    pub quantum: u32,
    processes: HashMap<u32, Process>,
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
        for (pid, process) in self.sys.processes() {
            if process.status() == ProcessStatus::Dead
                || process.status() == ProcessStatus::Zombie
                || process.status() == ProcessStatus::Sleep
            {
                continue;
            }

            let mut _process = Process::builder()
                .name(process.name().to_str().unwrap().to_string())
                .pid(pid.as_u32())
                .run_time(process.run_time())
                .memory(process.memory())
                .ram(process.memory())
                .build();

            _process.read_stat();
            _process.set_nice();
            _process.set_priority();

            self.processes.insert(pid.as_u32(), _process);
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
}
