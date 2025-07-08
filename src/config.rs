use crate::{ prelude::*, Bind };
use std::fs;

/// The application config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing, skip_deserializing)]
    path: PathBuf,

    pub com_port: u32,
    pub baud_rate: u32,
    pub binds: HashMap<String, Bind>,

    pub steel_series_filter: bool
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            path: path!("/config.json"),
            com_port: 8,
            baud_rate: 9600,
            binds: map![],

            steel_series_filter: true,
        }
    }
}

impl Config {
    /// Reads/writes config file
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Arc<Mutex<Self>>> {
        let path = path!(path.as_ref());
        
        // reading config file:
        let config = if path.exists() {
            Config::read(path)?
        }
        // or writing default config file:
        else {
            let mut cfg = Config::default();
            cfg.save_to(path)?;

            cfg
        };

        Ok(Arc::new(Mutex::new( config )))
    }
    
    /// Reads config from file
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path!(path.as_ref());

        // read file:
        let json_str = fs::read_to_string(&path)?;

        let mut cfg: Config = serde_json::from_str(&json_str)?;
        cfg.path = path;

        // parse json:
        Ok(cfg)
    }
    
    /// Updates a config file
    pub fn save(&mut self) -> Result<()> {
        self.save_to(self.path.clone())
    }

    /// Saves config to file
    pub fn save_to<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.path = path!(path.as_ref());

        // to json string:
        let json_str = serde_json::to_string_pretty(self)?;

        // create dir:
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        
        // write file:
        fs::write(&self.path, json_str)?;
        
        Ok(())
    }
}
