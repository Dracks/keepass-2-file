use serde::{Deserialize, Serialize};
use serde_yaml::{self};

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfig {
    pub keepass: Option<String>
}


pub struct GlobalConfig<'f>{
    file: &'f str,
    pub config: YamlConfig
}

impl<'f> GlobalConfig<'f> {
    pub fn new(file: &str) -> GlobalConfig {
        if !std::path::Path::new(file).exists() {
            std::fs::write(file, "").unwrap();
        }

        let config = match std::fs::read_to_string(file) {
            Ok(contents) => serde_yaml::from_str(&contents).unwrap(),
            Err(err) => {
                println!("{}", err);
                panic!("Config file cannot be loaded");
            }
        };

        GlobalConfig {
            file,
            config
        }
    }

    pub fn get_file(&self) -> &str {
        return self.file
    }

    pub fn save(&self) {
        let yaml = serde_yaml::to_string(&self.config).unwrap();
        std::fs::write(self.file, yaml).unwrap();
    }
}
