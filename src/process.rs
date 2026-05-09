pub struct Process {
    pub pid: u32,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub user: String,
}

impl Process {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            command: String::new(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            user: String::new(),
        }
    }

    #[allow(unused)]
    pub fn display(&self) {
        println!(
            "PID: {}, Command: {}, CPU Usage: {:.2}%, Memory Usage: {:.2}%, User: {}",
            self.pid, self.command, self.cpu_usage, self.memory_usage, self.user
        );
    }
}