use crate::prelude::*;
use std::fs;
use chrono::Local;

/// The program logger
pub struct Logger {
    pub logs: StdMutex<Vec<String>>,
    pub logs_dir: PathBuf,
    pub files_limit: usize,
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
    /// Initializes a logger
    pub fn init(&'static self) -> Result<()> {
        log::set_logger(self).map_err(Error::from)?;
        log::set_max_level(log::LevelFilter::Info);

        Ok(())
    }
    
    /// Creates a new logger
    pub fn new<P: AsRef<Path>>(logs_dir: P, files_limit: usize) -> Self {
        Self {
            logs: StdMutex::new(vec![]),
            logs_dir: path!(logs_dir.as_ref()),
            files_limit,
        }
    }

    /// Collects logs and clears them
    pub fn take(&self) -> Vec<String> {
        let mut logs_lock = self.logs.lock().unwrap();

        std::mem::take(&mut *logs_lock)
    }

    /// Saves logs to file
    pub fn save(&self) -> Result<()> {
        let path = self.logs_dir.join( Local::now().format("%Y-%m-%d_%H-%M-%S.log").to_string() );

        // create file dir:
        let dir = path.parent().unwrap();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        // removing old log files:
        self.cleanup()?;
        
        // writing logs to file:
        let logs_str = self.logs.lock().unwrap().join("\n");
        fs::write(&path, logs_str)?;

        Ok(())
    }

    /// Removes an extra old log files
    fn cleanup(&self) -> Result<()> {
        let mut log_files: Vec<PathBuf> = fs::read_dir(&self.logs_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "log") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // sort files by time:
        log_files.sort_by_key(|path| fs::metadata(path).and_then(|m| m.created()).ok());

        // remove extra files:
        if log_files.len() > self.files_limit {
            for old_file in &log_files[0..log_files.len() - self.files_limit] {
                let _ = fs::remove_file(old_file);
            }
        }

        Ok(())
    }
}
