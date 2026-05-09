use std::collections::HashMap;
use crate::process::Process;
use crate::system::Sys;

pub struct Collector {
    pub processes: Vec<Process>,
    pub system: Sys,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            system: Sys::new()
        }
    }

    pub fn collect(&mut self) -> Result<(), std::io::Error> {
        let pids = Self::get_pids()?;
        self.processes = pids.into_iter().filter_map(
            |pid| self.get_pid_info(pid).ok()
        ).collect();
        self.compute_cpu_usage();
        Ok(())
    }

    #[allow(unused)]
    pub fn display(&self) {
        // compute dynamic column widths (handle long PIDs)
        let pid_w: usize = self.processes.iter().map(|p| p.pid.to_string().len()).max().unwrap_or(3).max("PID".len()).max(5);
        let user_w: usize = self.processes.iter().map(|p| p.user.len()).max().unwrap_or(4).max("USER".len()).min(20);
        let command_w: usize = 40;

        // Table header
        let header = format!("| {:>pid_w$} | {:<user_w$} | {:>6} | {:>6} | {:<command_w$} |",
            "PID", "USER", "CPU%", "MEM%", "COMMAND",
            pid_w = pid_w, user_w = user_w, command_w = command_w
        );
        // cyan bold header
        println!("\x1b[1;36m{}\x1b[0m", header);
        // separator sized to header
        println!("\x1b[1;34m{}\x1b[0m", "-".repeat(header.len()));

        for (i, process) in self.processes.iter().enumerate() {
            // color CPU: green/yellow/red
            let cpu_color = if process.cpu_usage > 80.0 { "\x1b[1;31m" } else if process.cpu_usage > 20.0 { "\x1b[1;33m" } else { "\x1b[1;32m" };
            // color MEM: green/yellow/red
            let mem_color = if process.memory_usage > 50.0 { "\x1b[1;31m" } else if process.memory_usage > 20.0 { "\x1b[1;33m" } else { "\x1b[1;32m" };

            // trim long fields to fit computed widths
            let user = if process.user.len() > user_w { format!("{}…", &process.user[..user_w.saturating_sub(1)]) } else { process.user.clone() };
            let command = if process.command.len() > command_w { format!("{}…", &process.command[..command_w.saturating_sub(1)]) } else { process.command.clone() };

            // dim every other row for subtle zebra striping
            let row_prefix = if i % 2 == 0 { "\x1b[2m" } else { "" };
            let reset = "\x1b[0m";

            println!(
                "{}| {:>pid_w$} | {:<user_w$} | {}{:>6.2}%{} | {}{:>6.2}%{} | {:<command_w$} |{}",
                row_prefix,
                process.pid,
                user,
                cpu_color, process.cpu_usage, reset,
                mem_color, process.memory_usage, reset,
                command,
                reset,
                pid_w = pid_w, user_w = user_w, command_w = command_w
            );
        }
    }

    // Helper functions

    fn get_pid_info(&self, pid: u32) -> Result<Process, std::io::Error> {
        let stat_path = format!("/proc/{}/status", pid);
        let stat_content = std::fs::read_to_string(stat_path)?;

        let mut process = Process::new(pid);
        process.command = std::fs::read_to_string(format!("/proc/{}/comm", pid)).unwrap_or("<unknown>".to_string()).trim().to_string();

        for line in stat_content.lines() {
            if line.starts_with("Uid") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Some(user) = Sys::get_username(parts[1].parse::<u32>().unwrap_or(0)) {
                        process.user = user;
                    }
                }
            }
            if line.starts_with("VmRSS") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    process.memory_usage = parts[1].parse::<f32>().unwrap_or(0.0) / (self.system.total_memory as f32) * 100.0;
                }
            }
        }
        Ok(process)
    }


    fn get_process_cpu_ticks(pid: u32) -> Option<u64> {
        // FIX: process name may contain spaces, so we need to handle that (later)
        let stat_content = std::fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
        let parts: Vec<&str> = stat_content.split_whitespace().collect();
        if parts.len() >= 15 {
            let utime = parts[13].parse::<u64>().unwrap_or(0);
            let stime = parts[14].parse::<u64>().unwrap_or(0);
            return Some(utime + stime);
        }
        None
    }

    fn compute_cpu_usage(&mut self) -> () {
        let pid_to_ticks: HashMap<u32, u64> = self.processes.iter().map(|p| (p.pid, Self::get_process_cpu_ticks(p.pid).unwrap_or(0))).collect();
        let (total_ticks, num_cpus) = Sys::get_cpu_ticks().unwrap_or((0, 1));

        std::thread::sleep(std::time::Duration::from_secs(1));

        let new_pid_to_ticks: HashMap<u32, u64> = self.processes.iter().map(|p| (p.pid, Self::get_process_cpu_ticks(p.pid).unwrap_or(0))).collect();
        let new_total_ticks = Sys::get_cpu_ticks().unwrap_or((0, 1)).0;

        for process in &mut self.processes {
            let delta_process = *new_pid_to_ticks.get(&process.pid).unwrap_or(&0) - *pid_to_ticks.get(&process.pid).unwrap_or(&0);
            let delta_cpu = new_total_ticks - total_ticks;

            if delta_cpu > 0 {
                process.cpu_usage = (delta_process as f32 / delta_cpu as f32) * 100.0 * num_cpus as f32;
            }
        }
    }

    pub fn get_net_cpu_usage(&mut self) -> () {
        // to be called only after compute_cpu_usage, which updates each process's cpu_usage field
        // this will double count shared CPU usage across processes, but it's a simple approximation for now
        let mut percent = 0.0;
        self.processes.iter().for_each(|p| percent += p.cpu_usage);
        self.system.cpu_usage = percent.max(0.0).min(100.0);
    }

    pub fn get_net_memory_usage(&mut self) -> () {
        let (total_memory, free_memory) = Sys::get_meminfo().unwrap_or((0, 0));
        self.system.total_memory = total_memory;    
        self.system.memory_usage = ((total_memory - free_memory) as f32 / total_memory as f32) * 100.0;
    }

    fn get_pids() -> Result<Vec<u32>, std::io::Error> {
        let mut pids = Vec::new();
        for entry in std::fs::read_dir("/proc")? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();
            if let Ok(pid) = filename.parse::<u32>() {
                pids.push(pid);
            }
        }
        Ok(pids)
    }
}