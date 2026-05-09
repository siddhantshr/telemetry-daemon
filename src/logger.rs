use rusqlite::{Connection, params};
use crate::process::Process;
use crate::system::Sys;
use crate::collector::Collector;

pub struct Logger {
    conn: Connection,
}

impl Logger {
    pub fn new(db_path: &str) -> Self {
        let conn = Connection::open(db_path).expect("Failed to open database");
        conn.pragma_update(None, "journal_mode", "WAL")
            .expect("Failed to set journal mode");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS telemetry (
                timestamp INTEGER NOT NULL,
                pid INTEGER NOT NULL,
                user TEXT NOT NULL,
                command TEXT NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_usage REAL NOT NULL
            )",
            [],
        ).expect("Failed to create table");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_metrics (
                timestamp INTEGER NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_usage REAL NOT NULL
            )",
            [],
        ).expect("Failed to create table");

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_telemetry_timestamp ON telemetry (timestamp)",
            []
        ).expect("Failed to create index");
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_system_metrics_timestamp ON system_metrics (timestamp)",
            []
        ).expect("Failed to create index");
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_telemetry_pid ON telemetry (pid)",
            []
        ).expect("Failed to create index");
                conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_telemetry_command ON telemetry (command)",
            []
        ).expect("Failed to create index");


        Self { conn }
    }

    pub fn log(&mut self, collector: &mut Collector) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64;
        collector.get_net_memory_usage();
        collector.get_net_cpu_usage();
        for process in &collector.processes {
            Self::log_process(&tx, process, timestamp)?;
        }

        Self::log_system(&tx, &collector.system, timestamp)?;
        tx.commit()?;
        println!("Logged telemetry at timestamp: {}", timestamp);
        Ok(())
    }

    pub fn cleanup(&mut self) -> rusqlite::Result<()> {
        let cutoff = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64 - 86400; // 24 hours
        
        self.conn.execute(
            "DELETE FROM telemetry WHERE timestamp < ?1",
            params![cutoff],
        )?;
        self.conn.execute(
            "DELETE FROM system_metrics WHERE timestamp < ?1",
            params![cutoff],
        )?;
        self.conn.execute(
            "VACUUM",
            [],
        )?;
        Ok(())
    }

    fn log_process(tx: &rusqlite::Transaction, process: &Process, timestamp: i64) -> rusqlite::Result<()> {        
        tx.execute(
            "INSERT INTO telemetry (timestamp, pid, user, command, cpu_usage, memory_usage) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![timestamp, process.pid as i64, &process.user, &process.command, process.cpu_usage as f64, process.memory_usage as f64],
        )?;
        Ok(())
    }

    fn log_system(tx: &rusqlite::Transaction, system: &Sys, timestamp: i64) -> rusqlite::Result<()> {
        tx.execute(
            "INSERT INTO system_metrics (timestamp, cpu_usage, memory_usage) VALUES (?1, ?2, ?3)",
            params![timestamp, system.cpu_usage as f64, system.memory_usage as f64],
        )?;
        Ok(())
    }
}