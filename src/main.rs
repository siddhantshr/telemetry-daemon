mod collector;
mod process;
mod system;
mod logger;

fn main() {

    let mut collector =
        collector::Collector::new();
    let mut n = 0;

    // log into db
    std::fs::create_dir_all("/var/lib/teld").expect("Failed to create directory");
    let mut logger = logger::Logger::new("/var/lib/teld/telemetry.db");

    loop {
        n += 15;
        if let Err(e) = collector.collect() {
            eprintln!("Error collecting telemetry: {}", e);
        } else {   
            if let Err(e) = logger.log(&mut collector) {
                eprintln!("Error logging telemetry: {}", e);
            }
        }

        if n == 1800 { // clean every 30 minutes
            n = 0;
            if let Err(e) = logger.cleanup() {
                eprintln!("Error cleaning up old telemetry data: {}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(15));
    }
}