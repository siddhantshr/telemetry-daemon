use std::collections::HashMap;
use std::sync::OnceLock;

static USER_MAP: OnceLock<HashMap<u32, String>> = OnceLock::new();

pub struct Sys {
    pub total_memory: u64,
    pub num_cpus: usize,
    pub memory_usage: f32,
    pub cpu_usage: f32,
}

impl Sys {
    pub fn new() -> Self {
        let (total_memory, _) = Self::get_meminfo().unwrap_or((0, 0));
        let (_, num_cpus) = Self::get_cpu_ticks().unwrap_or((0, 1));
        Self {
            total_memory,
            num_cpus,
            memory_usage: 0.0,
            cpu_usage: 0.0,
        }
    }

    pub fn get_meminfo() -> Result<(u64, u64), std::io::Error> {
        let meminfo_content = std::fs::read_to_string("/proc/meminfo")?;
        let mut free_memory = 0;
        let mut total_memory = 0;

        for line in meminfo_content.lines() {
            if line.starts_with("MemAvailable") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    free_memory = parts[1].parse::<u64>().unwrap_or(0);
                }
            }
            if line.starts_with("MemTotal") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    total_memory = parts[1].parse::<u64>().unwrap_or(0);
                }
            }
        }

        Ok((total_memory, free_memory))
    }

    pub fn get_cpu_ticks() -> Option<(u64, usize)> {
        let stat_content = std::fs::read_to_string("/proc/stat").ok()?;
        let mut total_ticks: u64 = 0;
        let first_line = stat_content.lines().next()?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            for part in &parts[1..] {
                total_ticks += part.parse::<u64>().unwrap_or(0);
            }
        }
        let num_cpus = std::thread::available_parallelism().unwrap().get();
        Some((total_ticks, num_cpus))
    }


    pub fn get_username(uid: u32) -> Option<String> {
        let usermap = USER_MAP.get_or_init(|| {
            let mut map = HashMap::new();
            if let Ok(passwd_content) = std::fs::read_to_string("/etc/passwd") {
                for line in passwd_content.lines() {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 3 {
                        if let Ok(uid) = parts[2].parse::<u32>() {
                            map.insert(uid, parts[0].to_string());
                        }
                    }
                }
            }
            map
        });
        usermap.get(&uid).cloned()
    }
}