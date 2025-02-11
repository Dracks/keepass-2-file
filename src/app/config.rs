use serde::{Deserialize, Serialize};
use serde_yaml::{self};

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfig {
    pub keepass: Option<String>,
}

pub struct GlobalConfig<'f> {
    file: &'f str,
    pub config: YamlConfig,
}

impl GlobalConfig<'_> {
    pub fn new(file: &str) -> Result<GlobalConfig, Box<dyn std::error::Error>> {
        if !std::path::Path::new(file).exists() {
            std::fs::write(file, "")?;
        }

        let contents = std::fs::read_to_string(file)?;
        let config = if contents.is_empty() {
            YamlConfig { keepass: None }
        } else {
            serde_yaml::from_str(&contents)?
        };

        Ok(GlobalConfig { file, config })
    }

    pub fn get_file(&self) -> &str {
        self.file
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(&self.config)?;
        std::fs::write(self.file, yaml)?;
        Ok(())
    }
}
