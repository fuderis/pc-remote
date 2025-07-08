use crate::prelude::*;
use std::fs;
use chrono::Local;

/// The program logger
pub struct Logger {
    pub logs: StdMutex<Vec<String>>,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let dt = Local::now().format("%Y-%m-%dT%H:%M:%S%.6f");
            let log = fmt!("[{dt}] [{}] {}", record.level(), record.args());

            // printing to terminal:
            println!("{log}");

            self.logs.lock().unwrap().push(log);
        }
    }

    fn flush(&self) {}
}

impl Logger {
    /// Initializes logger in program
    pub fn init(&'static self) -> Result<()> {
        log::set_logger(self).map_err(Error::from)?;
        log::set_max_level(log::LevelFilter::Info);

        Ok(())
    }
    
    /// Creates a new logger
    pub fn new() -> Self {
        Self {
            logs: StdMutex::new(vec![])
        }
    }

    /// Collects logs and clears them
    pub fn take(&self) -> Vec<String> {
        let mut logs_lock = self.logs.lock().unwrap();

        std::mem::take(&mut *logs_lock)
    }

    /// Saves logs to file
    pub fn save(&self) -> Result<()> {
        let path = path!(Local::now().format("/logs/%Y-%m-%d_%H-%M-%S.log").to_string());

        // create file dir:
        let dir = path.parent().unwrap();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        // writing logs to file:
        let logs_str = self.logs.lock().unwrap().join("\n");
        fs::write(path, logs_str)?;

        Ok(())
    }
}
