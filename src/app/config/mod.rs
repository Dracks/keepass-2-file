pub mod yaml;

use std::fmt::Display;

use yaml::YamlConfig;

#[derive(Debug)]
pub struct ConfigError{
    msg: String
}
impl std::error::Error for ConfigError{}
impl Display for ConfigError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

pub struct ConfigHandler {
    file: String,
    project: String,
    pub global: YamlConfig,
    pub local: Option<YamlConfig>,
}

impl ConfigHandler {
    fn get_project_file(project: &String)-> Option<String>{
        let mut project_path = std::path::Path::new(project).to_path_buf();
        if project_path.is_dir(){
            project_path.push(".keepass-2-file");
            if project_path.exists() && project_path.is_file(){
                return project_path.as_os_str().to_str().map(|string| string.to_string());
            }
        }
        None
    }

    pub fn new(file: &str, project: String) -> Result<ConfigHandler, Box<dyn std::error::Error>> {
        if !std::path::Path::new(file).exists() {
            std::fs::write(file, "")?;
        }

        let contents = std::fs::read_to_string(file)?;

        let config = YamlConfig::new(&contents)?;

        let local_config_file : Option<String> = Self::get_project_file(&project);

        let local : Option<YamlConfig> = match local_config_file {
            Some(config) =>  Some(YamlConfig::new(config.as_str())?),
            None => None
        };


        Ok(ConfigHandler {
            file: file.to_string(),
            global: config,
            local,
            project,
        })
    }

    pub fn get_file(&self) -> &str {
        &self.file
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let global_content=self.global.yaml()?;
        std::fs::write(&self.file, global_content)?;
        if let Some(local) = &self.local {
            let local_content=local.yaml()?;
            let Some(local_file) = Self::get_project_file(&self.project) else {
                return Err(Box::new(ConfigError{msg: "Cannot get the local project file, any changes in the project config won't be saved".into()}));
            };
            std::fs::write(local_file, local_content)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::app::test_helpers::tests::TestConfig;

    #[test]
    fn test_loading_empty_file() {
        let test_config = TestConfig::create_empty_file();
        std::fs::write(test_config.get_file_path(), "{}").expect("Can write test file");

        let config = test_config.get();

        assert_eq!(config.global.get_templates().len(), 0);
    }
}
