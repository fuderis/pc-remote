use crate::prelude::*;
use std::fs;
use std::fs::File;
use std::io::Write;
use chrono::Local;

/// Application logger
pub struct Logger {
    pub file: StdMutex<Option<File>>,
    pub dir: PathBuf,
    pub limit: usize,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let dt = Local::now().format("%Y-%m-%dT%H:%M:%S%.6f");
            let mut log = fmt!("[{dt}] [{}] {}", record.level(), record.args());

            // printing to terminal:
            println!("{log}");

            if let Some(file) = self.file.lock().unwrap().as_mut() {
                log.push('\n');
                
                let _ = file.write_all(log.as_bytes());
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {}
}

impl Logger {
    /// Creates a new logger
    pub fn new<P: AsRef<Path>>(dir: P, limit: usize) -> Self {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir).expect("Failed to create a logs directory.");
        let path = dir.join( Local::now().format("%Y-%m-%d_%H-%M-%S.log").to_string() );

        let this = Self {
            file: StdMutex::new(if limit > 0 { Some( File::create(path).map_err(|e| err!("Failed to create log file: {e}")).unwrap() ) }else{ None }),
            dir,
            limit,
        };

        this.clear().map_err(|e| err!("Failed to remove extra logs: {e}")).unwrap();

        this
    }

    /// Initializes a logger
    pub fn init(&'static self) -> Result<()> {
        log::set_logger(self).map_err(Error::from)?;
        log::set_max_level(log::LevelFilter::Info);

        Ok(())
    }

    /// Removes an extra old log files
    fn clear(&self) -> Result<()> {
        if self.limit == 0 { return Ok(()) }

        let (dir, limit) = (&self.dir, self.limit);
        
        let mut log_files: Vec<PathBuf> = fs::read_dir(&dir)?
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
        if log_files.len() > limit {
            for old_file in &log_files[0..log_files.len() - limit] {
                let _ = fs::remove_file(old_file);
            }
        }

        Ok(())
    }
}
